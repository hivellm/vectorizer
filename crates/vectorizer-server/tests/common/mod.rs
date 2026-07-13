//! Shared in-process REST test harness (phase39 §1.1-1.2).
//!
//! Builds the REAL production Axum router — `VectorizerServer::build_router`,
//! extracted verbatim from `VectorizerServer::start()` (see
//! `crates/vectorizer-server/src/server/core/routing.rs`) — over real
//! in-memory state: a real `VectorStore::new_cpu_only()` and a real
//! `EmbeddingManager` with a BM25 provider fitted on a small seed corpus.
//! Requests are dispatched via `tower::ServiceExt::oneshot`, so every test
//! using this harness runs in-process with no TCP listener, no live
//! server, and no `#[ignore]`.
//!
//! ## Covered route groups
//!
//! `build_router` is the exact function the production binary calls —
//! nothing here is a duplicate or a mock. Because auth / hub / cluster /
//! Raft are all disabled (`None`) in [`TestApp`], every route group whose
//! merge in `routing.rs` is unconditional is present and fully functional:
//! collections, vectors (single + batch), search (raw-vector, text,
//! hybrid, intelligent/multi-collection/semantic/contextual), discovery,
//! file operations + upload, replication status/stats endpoints (report
//! "no active node" since `master_node`/`replica_node` are `None`),
//! Qdrant-compatible routes, graph routes, GraphQL, the admin router
//! (unauthenticated in this harness — see below), and `/health` /
//! `/metrics`.
//!
//! ## Excluded / degraded route groups
//!
//! - **HiveHub routes** (`/hub/*`): registered by `build_router` but every
//!   handler returns an error because `hub_manager` is `None` — this
//!   harness never constructs a `HubManager` (it requires an external
//!   HiveHub connection). Not exercised by any test built on this harness.
//! - **Cluster-only routes** (`crate::api::cluster`): only merged by
//!   `build_router` when `cluster_manager` + `cluster_client_pool` are
//!   `Some`; both are `None` here, so the group is absent entirely.
//! - **gRPC, the real TCP listener, and the Ctrl+C/SIGTERM shutdown
//!   signal handler**: `build_router` deliberately excludes these (see
//!   its own doc comment); this harness never binds a socket.
//! - **Auth-gated behavior**: `auth_handler_state` is `None` for
//!   [`TestApp::new`], so every request runs in the pre-auth "disabled"
//!   mode — the `require_auth_middleware` / CSRF / router-level admin
//!   gates are never attached, and admin routes are reachable without a
//!   token. [`TestApp::with_auth`] builds the same router with a real
//!   `AuthHandlerState` attached instead, for suites that need auth
//!   enforcement active (see `tests/rest_auth_enforcement.rs`). Auth
//!   behavior also has dedicated in-process coverage in
//!   `csrf_bearer_exemption.rs` and `auth_handlers_tests.rs`.
//!
//! ## Known limitation
//!
//! [`TestApp::new`] calls `std::env::set_var("VECTORIZER_DATA_DIR", ..)`,
//! which is process-global. Most tests built on this harness only
//! exercise in-memory collection/vector operations, so a data-dir race
//! between concurrently-running `TestApp` instances in the same test
//! binary is not a correctness risk for them — but any suite that
//! asserts on on-disk paths (native snapshots, backups) MUST NOT run in
//! parallel with other `TestApp` construction in the same binary without
//! additional serialization. See `tests/rest_lifecycle_handlers.rs`
//! (`ENV_DIR_LOCK`) for the pattern: a file-local
//! `tokio::sync::Mutex` held across the full body of the disk-dependent
//! test(s), with every other test in that binary taking the same lock
//! briefly around its own `TestApp::new()` call.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::Value;
use tower::ServiceExt;
use vectorizer::VectorStore;
use vectorizer::auth::roles::Role;
use vectorizer::auth::{AuthConfig, AuthManager, Secret};
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer_server::server::{AuthHandlerState, UserRecord, VectorizerServer};

/// Small, generic seed corpus used to fit the BM25 vocabulary so
/// `embed()` exercises the real term-frequency path instead of the
/// empty-vocabulary hash fallback. Deliberately unrelated to any
/// individual test's fixtures.
const BM25_SEED_CORPUS: &[&str] = &[
    "the quick brown fox jumps over the lazy dog",
    "vector databases store high dimensional embeddings",
    "semantic search finds documents by meaning not keywords",
    "the rust programming language emphasizes safety and speed",
    "machine learning models transform text into numeric vectors",
];

/// Build a real `EmbeddingManager` with a BM25 provider fitted on
/// [`BM25_SEED_CORPUS`], registered as `"bm25"` and set default. Shared by
/// every [`TestApp`] constructor so each one wires the exact same
/// embedding behavior into the production router.
fn build_embedding_manager() -> Arc<EmbeddingManager> {
    let mut bm25 = Bm25Embedding::new(512);
    bm25.build_vocabulary(
        &BM25_SEED_CORPUS
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>(),
    );
    let mut embedding_manager = EmbeddingManager::new();
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager
        .set_default_provider("bm25")
        .expect("bm25 provider was just registered");
    Arc::new(embedding_manager)
}

/// Point `VECTORIZER_DATA_DIR` at a fresh temp directory and return it.
///
/// # Safety
/// `set_var` is unsafe in edition 2024 because mutating the environment
/// is unsound if another thread concurrently reads it via a non-atomic OS
/// API. Every [`TestApp`] constructor calls this before any reader thread
/// has been spawned for this specific value (readers are handler code
/// invoked later, through the very `TestApp` being constructed), so there
/// is no concurrent read/write on the same key here.
fn point_data_dir_at_temp_dir() -> tempfile::TempDir {
    let data_dir = tempfile::tempdir().expect("create temp data dir for TestApp");
    // SAFETY: see the `# Safety` section above — no reader thread for
    // this key exists before the TestApp under construction spawns one.
    unsafe {
        std::env::set_var("VECTORIZER_DATA_DIR", data_dir.path());
    }
    data_dir
}

/// In-process test harness wrapping the real production Axum router.
pub struct TestApp {
    router: Router,
    /// Keeps the per-`TestApp` temp data directory alive (removed on
    /// drop) and backs the [`TestApp::data_dir`] accessor.
    temp_dir: tempfile::TempDir,
}

/// Plaintext credentials for the single admin user seeded by
/// [`TestApp::with_auth`], so a test can `POST /auth/login` through the
/// real handler and obtain a genuine JWT.
#[allow(dead_code)]
pub struct AuthFixture {
    pub username: String,
    pub password: String,
}

impl TestApp {
    /// Build a fresh [`TestApp`]: a real `VectorStore::new_cpu_only()` +
    /// a real `EmbeddingManager` (see [`build_embedding_manager`])
    /// wired into the production router via `VectorizerServer::build_router`.
    ///
    /// `is_production_bind` is passed as `false` (development mode: CORS +
    /// security headers only, no legacy inline auth middleware) since auth
    /// is disabled in this harness regardless.
    ///
    /// Each integration-test binary compiles its own copy of `mod common`,
    /// so constructors used by only some binaries read as dead code in the
    /// others — hence the per-item allow (repo test-helper convention).
    #[allow(dead_code)]
    pub async fn new() -> Self {
        let data_dir = point_data_dir_at_temp_dir();

        let store = Arc::new(VectorStore::new_cpu_only());
        let embedding_manager = build_embedding_manager();

        let server = VectorizerServer::new_for_test_harness(store, embedding_manager);
        let router = server.build_router(false).await;

        Self {
            router,
            temp_dir: data_dir,
        }
    }

    /// Build a [`TestApp`] with authentication ENABLED, mirroring how
    /// `bootstrap.rs` wires a real `AuthManager` + `AuthHandlerState`
    /// (see `VectorizerServer::new_with_root_config`'s `auth_handler_state`
    /// block) — except here the config is a fixed, in-memory `AuthConfig`
    /// instead of one read from `config.yml`.
    ///
    /// `auth_handler_state` is `pub` on `VectorizerServer`, so this builds
    /// the same harness server as [`TestApp::new`] via
    /// `new_for_test_harness` and then attaches the auth state directly —
    /// no new bootstrap constructor needed. `build_router` gates every
    /// data route behind `require_auth_middleware` whenever
    /// `auth_handler_state` is `Some`, regardless of `is_production_bind`
    /// (see `routing.rs`), so passing `false` here still produces a fully
    /// auth-enforced router.
    ///
    /// Returns the app plus an [`AuthFixture`] with one pre-seeded admin
    /// user's plaintext credentials, hashed with bcrypt cost 4 (vs.
    /// production's `DEFAULT_COST`) to keep every login in a suite built
    /// on this harness fast — the same convention `csrf_bearer_exemption.rs`
    /// uses.
    #[allow(dead_code)]
    pub async fn with_auth() -> (Self, AuthFixture) {
        let data_dir = point_data_dir_at_temp_dir();

        let store = Arc::new(VectorStore::new_cpu_only());
        let embedding_manager = build_embedding_manager();
        let mut server = VectorizerServer::new_for_test_harness(store, embedding_manager);

        let auth_config = AuthConfig {
            jwt_secret: Secret::new("t".repeat(64)),
            enabled: true,
            ..AuthConfig::default()
        };
        let auth_manager = Arc::new(AuthManager::new(auth_config).expect("valid auth config"));
        let auth_state = AuthHandlerState::new(auth_manager);

        let username = "test-admin".to_string();
        let password = "test-admin-password-1234".to_string();
        let password_hash = bcrypt::hash(&password, 4).expect("bcrypt hash");
        let admin = UserRecord {
            user_id: username.clone(),
            username: username.clone(),
            password_hash: Secret::new(password_hash),
            roles: vec![Role::Admin],
        };
        auth_state
            .users
            .write()
            .await
            .insert(username.clone(), admin);

        server.auth_handler_state = Some(auth_state);
        let router = server.build_router(false).await;

        (
            Self {
                router,
                temp_dir: data_dir,
            },
            AuthFixture { username, password },
        )
    }

    /// Dispatch `POST <path>` with a JSON body through the real router.
    /// Returns the status code and the decoded JSON body (`Value::Null`
    /// when the body is empty or not valid JSON).
    ///
    /// Not every test binary that links this shared harness exercises
    /// every dispatch method — `dead_code` is allowed per-method to
    /// match the existing convention for shared test helpers (see
    /// `crates/vectorizer/tests/helpers/mod.rs`).
    #[allow(dead_code)]
    pub async fn post_json(&self, path: &str, body: Value) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&body).expect("serialize request body"),
            ))
            .expect("build POST request");
        self.dispatch(req).await
    }

    /// Path to this instance's temp data directory (the same path
    /// written to `VECTORIZER_DATA_DIR` in [`TestApp::new`]). Read this
    /// instead of `std::env::var("VECTORIZER_DATA_DIR")` when a test
    /// needs to assert on-disk artifacts: the env var is process-global
    /// and races with concurrently-running `TestApp` instances in the
    /// same test binary, but this accessor reads the field captured at
    /// construction time for `self` only.
    #[allow(dead_code)]
    pub fn data_dir(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Dispatch `PUT <path>` with a JSON body through the real router.
    /// Returns the status code and the decoded JSON body (`Value::Null`
    /// when the body is empty or not valid JSON).
    #[allow(dead_code)]
    pub async fn put_json(&self, path: &str, body: Value) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("PUT")
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&body).expect("serialize request body"),
            ))
            .expect("build PUT request");
        self.dispatch(req).await
    }

    /// Dispatch `PATCH <path>` with a JSON body through the real router.
    /// Returns the status code and the decoded JSON body (`Value::Null`
    /// when the body is empty or not valid JSON).
    #[allow(dead_code)]
    pub async fn patch_json(&self, path: &str, body: Value) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("PATCH")
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&body).expect("serialize request body"),
            ))
            .expect("build PATCH request");
        self.dispatch(req).await
    }

    /// Dispatch `DELETE <path>` (no body) through the real router.
    #[allow(dead_code)]
    pub async fn delete(&self, path: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("DELETE")
            .uri(path)
            .body(Body::empty())
            .expect("build DELETE request");
        self.dispatch(req).await
    }

    /// Dispatch `GET <path>` through the real router.
    #[allow(dead_code)]
    pub async fn get(&self, path: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .expect("build GET request");
        self.dispatch(req).await
    }

    /// Dispatch `GET <path>` carrying `Authorization: Bearer <token>`
    /// through the real router. Exists for auth-enforcement suites built
    /// on [`TestApp::with_auth`] that need to assert on both anonymous
    /// (via [`TestApp::get`]) and authenticated requests to the same
    /// route.
    #[allow(dead_code)]
    pub async fn get_with_bearer(&self, path: &str, token: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("GET")
            .uri(path)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .expect("build GET request");
        self.dispatch(req).await
    }

    /// Dispatch `POST <path>` with an arbitrary raw body and explicit
    /// `Content-Type`, bypassing `serde_json` serialization entirely.
    ///
    /// [`TestApp::post_json`] can only ever send well-formed JSON (any
    /// `serde_json::Value` serializes to valid JSON), so it cannot
    /// exercise axum's `Json<T>` extractor rejection path. This method
    /// exists for handlers whose own business logic has no field-level
    /// validation to test (e.g. `update_workspace_config` accepts any
    /// JSON `Value` and never returns its own 4xx) — for those, a
    /// syntactically malformed body rejected by the extractor before
    /// the handler runs is the closest meaningful error-branch contract.
    #[allow(dead_code)]
    pub async fn post_raw(
        &self,
        path: &str,
        content_type: &str,
        body: &[u8],
    ) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(path)
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(body.to_vec()))
            .expect("build POST request");
        self.dispatch(req).await
    }

    /// Dispatch `POST <path>` with a hand-encoded `multipart/form-data`
    /// body built from `fields`, in order. Axum's `Multipart` extractor
    /// only needs a syntactically valid multipart stream — there is no
    /// need for a real HTTP client to produce one, so this method encodes
    /// the fields directly instead of routing through `reqwest`.
    #[allow(dead_code)]
    pub async fn post_multipart(
        &self,
        path: &str,
        fields: &[MultipartField],
    ) -> (StatusCode, Value) {
        const BOUNDARY: &str = "TestAppMultipartBoundary7f3c9a1d4e2b6f80";
        let body = encode_multipart(BOUNDARY, fields);
        self.post_raw(
            path,
            &format!("multipart/form-data; boundary={BOUNDARY}"),
            &body,
        )
        .await
    }

    async fn dispatch(&self, req: Request<Body>) -> (StatusCode, Value) {
        let response = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("router dispatch must complete");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 16 * 1024 * 1024)
            .await
            .expect("collect response body");
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, json)
    }
}

/// One field of a hand-encoded `multipart/form-data` body, for
/// [`TestApp::post_multipart`]. Axum's `Multipart` extractor doesn't care
/// which HTTP client produced the stream, so tests build these directly
/// instead of going through `reqwest::multipart`.
#[allow(dead_code)]
pub enum MultipartField {
    /// A plain `name=value` text field (e.g. `collection_name`).
    Text { name: String, value: String },
    /// A file part with an explicit filename and `Content-Type`
    /// (the `file` field consumed by `POST /files/upload`).
    File {
        name: String,
        filename: String,
        content_type: String,
        bytes: Vec<u8>,
    },
}

#[allow(dead_code)]
impl MultipartField {
    /// Build a [`MultipartField::Text`] field.
    pub fn text(name: &str, value: impl Into<String>) -> Self {
        Self::Text {
            name: name.to_string(),
            value: value.into(),
        }
    }

    /// Build a [`MultipartField::File`] field.
    pub fn file(name: &str, filename: &str, content_type: &str, bytes: impl Into<Vec<u8>>) -> Self {
        Self::File {
            name: name.to_string(),
            filename: filename.to_string(),
            content_type: content_type.to_string(),
            bytes: bytes.into(),
        }
    }
}

/// Encode `fields` as a `multipart/form-data` body delimited by
/// `boundary`, in RFC 7578 form: each part starts with `--{boundary}`, a
/// `Content-Disposition` header (plus `Content-Type` for file parts), a
/// blank line, the raw part body, then a trailing `\r\n`. The stream ends
/// with a `--{boundary}--` closing delimiter.
fn encode_multipart(boundary: &str, fields: &[MultipartField]) -> Vec<u8> {
    let mut body = Vec::new();
    for field in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        match field {
            MultipartField::Text { name, value } => {
                body.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
                );
                body.extend_from_slice(value.as_bytes());
            }
            MultipartField::File {
                name,
                filename,
                content_type,
                bytes,
            } => {
                body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n\
                         Content-Type: {content_type}\r\n\r\n"
                    )
                    .as_bytes(),
                );
                body.extend_from_slice(bytes);
            }
        }
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    body
}
