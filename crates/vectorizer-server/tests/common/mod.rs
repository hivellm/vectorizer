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
//! - **Auth-gated behavior**: `auth_handler_state` is always `None`, so
//!   every request runs in the pre-auth "disabled" mode — the
//!   `require_auth_middleware` / CSRF / router-level admin gates are
//!   never attached, and admin routes are reachable without a token.
//!   Auth-specific behavior already has dedicated in-process coverage in
//!   `csrf_bearer_exemption.rs` and `auth_handlers_tests.rs`.
//!
//! ## Known limitation
//!
//! [`TestApp::new`] calls `std::env::set_var("VECTORIZER_DATA_DIR", ..)`,
//! which is process-global. Tests built on this harness only exercise
//! in-memory collection/vector operations, so a data-dir race between
//! concurrently-running `TestApp` instances in the same test binary is
//! not currently a correctness risk — but a future suite that asserts on
//! on-disk paths (snapshots, backups) MUST NOT run in parallel with other
//! `TestApp`-based tests without additional serialization.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::Value;
use tower::ServiceExt;
use vectorizer::VectorStore;
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer_server::server::VectorizerServer;

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

/// In-process test harness wrapping the real production Axum router.
pub struct TestApp {
    router: Router,
    /// Keeps the per-`TestApp` temp data directory alive; removed on drop.
    _data_dir: tempfile::TempDir,
}

impl TestApp {
    /// Build a fresh [`TestApp`]: a real `VectorStore::new_cpu_only()` +
    /// a real `EmbeddingManager` (BM25 provider fitted on
    /// [`BM25_SEED_CORPUS`], registered as `"bm25"` and set default)
    /// wired into the production router via `VectorizerServer::build_router`.
    ///
    /// `is_production_bind` is passed as `false` (development mode: CORS +
    /// security headers only, no legacy inline auth middleware) since auth
    /// is disabled in this harness regardless.
    pub async fn new() -> Self {
        let data_dir = tempfile::tempdir().expect("create temp data dir for TestApp");
        // SAFETY: `set_var` is unsafe in edition 2024 because mutating the
        // environment is unsound if another thread concurrently reads it
        // via a non-atomic OS API. This process only ever writes
        // `VECTORIZER_DATA_DIR` before any reader thread has been spawned
        // for this specific value (readers are handler code invoked later,
        // through the very `TestApp` this call is constructing), so there
        // is no concurrent read/write on the same key here.
        unsafe {
            std::env::set_var("VECTORIZER_DATA_DIR", data_dir.path());
        }

        let store = Arc::new(VectorStore::new_cpu_only());

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
        let embedding_manager = Arc::new(embedding_manager);

        let server = VectorizerServer::new_for_test_harness(store, embedding_manager);
        let router = server.build_router(false).await;

        Self {
            router,
            _data_dir: data_dir,
        }
    }

    /// Dispatch `POST <path>` with a JSON body through the real router.
    /// Returns the status code and the decoded JSON body (`Value::Null`
    /// when the body is empty or not valid JSON).
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

    /// Dispatch `DELETE <path>` (no body) through the real router.
    pub async fn delete(&self, path: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("DELETE")
            .uri(path)
            .body(Body::empty())
            .expect("build DELETE request");
        self.dispatch(req).await
    }

    /// Dispatch `GET <path>` through the real router.
    pub async fn get(&self, path: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .expect("build GET request");
        self.dispatch(req).await
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
