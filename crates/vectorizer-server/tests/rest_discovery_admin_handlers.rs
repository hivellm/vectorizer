//! In-process router-level coverage (phase39 §2 batch B) for the
//! discovery / file-navigation / admin REST handlers that had no
//! dedicated suite: `get_project_outline`, `get_related_files`,
//! `search_by_file_type`, `get_file_summary`, `list_slow_queries` +
//! `set_slow_query_config`, `get_config`, `update_workspace_config`,
//! `broad_discovery`, `semantic_focus`, `promote_readme`,
//! `build_answer_plan`, `render_llm_prompt`, and `get_backup_directory`.
//!
//! Dispatched through the real production router (see
//! `tests/common/mod.rs`) via `tower::ServiceExt::oneshot` — no
//! `#[ignore]`, runs in CI.
//!
//! ## Seeding strategy
//!
//! The file/discovery handlers below read vectors whose payload carries
//! `file_path` / `chunk_index` / `file_extension` / `content` at the
//! flat payload root (see `FileOperations::metadata_view` and
//! `build_chunk_payload` in
//! `crates/vectorizer-server/src/server/rest_handlers/insert.rs`).
//! [`seed_file_collection`] seeds exactly that shape via
//! `POST /insert_vectors` (which passes an explicit `payload` object
//! straight through, unlike `/insert_texts`'s metadata-merge path) so
//! every field is under precise control. Each seeded document's
//! `content` is a verbatim sentence from `common::BM25_SEED_CORPUS` (the
//! corpus the harness's BM25 provider is fitted on) and its embedding is
//! produced by the real `/embed` endpoint for that exact text — so a
//! query using the same verbatim sentence gets a deterministic,
//! non-flaky near-1.0 self-match instead of relying on incidental
//! vocabulary overlap.
//!
//! ## Excluded by design
//!
//! `restart_server` (`POST /admin/restart`) is intentionally NOT covered
//! here: it spawns a background task that (on Unix) sends `SIGHUP` to
//! the current process and (on Windows) schedules `std::process::exit`
//! — real process-level side effects that would kill the test binary
//! itself. No in-process harness can safely exercise it; it is out of
//! scope for this suite by design.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

/// Seed `name` with 3 file-like vectors via `POST /insert_vectors`,
/// each carrying an explicit flat payload (`content`, `file_path`,
/// `chunk_index`, `file_extension`) and a real BM25 embedding of its
/// own verbatim `content` (fetched through `POST /embed`). Returns
/// nothing — callers reference the fixed paths/content below directly:
///
/// | file_path        | extension | content (verbatim BM25 seed sentence)                        |
/// |-------------------|-----------|--------------------------------------------------------------|
/// | `src/main.rs`     | `rs`      | "vector databases store high dimensional embeddings"          |
/// | `docs/guide.md`   | `md`      | "semantic search finds documents by meaning not keywords"      |
/// | `README.md`       | `md`      | "the rust programming language emphasizes safety and speed"    |
async fn seed_file_collection(app: &TestApp, name: &str) {
    let _ = app.delete(&format!("/collections/{name}")).await;

    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(
        status.is_success(),
        "create collection status {status}: {resp}"
    );

    let docs: &[(&str, &str, &str)] = &[
        (
            "src/main.rs",
            "rs",
            "vector databases store high dimensional embeddings",
        ),
        (
            "docs/guide.md",
            "md",
            "semantic search finds documents by meaning not keywords",
        ),
        (
            "README.md",
            "md",
            "the rust programming language emphasizes safety and speed",
        ),
    ];

    for (idx, (path, ext, content)) in docs.iter().enumerate() {
        let (status, embed_resp) = app.post_json("/embed", json!({"text": content})).await;
        assert!(status.is_success(), "embed status {status}: {embed_resp}");
        let embedding = embed_resp["embedding"].clone();

        let (status, resp) = app
            .post_json(
                "/insert_vectors",
                json!({
                    "collection": name,
                    "vectors": [{
                        "id": format!("seed-{idx}"),
                        "embedding": embedding,
                        "payload": {
                            "content": content,
                            "file_path": path,
                            "chunk_index": 0,
                            "file_extension": ext,
                        }
                    }]
                }),
            )
            .await;
        assert!(
            status.is_success(),
            "insert_vectors status {status}: {resp}"
        );
        assert_eq!(
            resp["inserted"].as_u64(),
            Some(1),
            "insert_vectors resp: {resp}"
        );
    }
}

// ---------------------------------------------------------------------
// get_project_outline — POST /file/outline
// ---------------------------------------------------------------------

#[tokio::test]
async fn get_project_outline_returns_structure_and_key_files() {
    let app = TestApp::new().await;
    seed_file_collection(&app, "outline_happy").await;

    let (status, resp) = app
        .post_json("/file/outline", json!({"collection": "outline_happy"}))
        .await;
    assert_eq!(status, StatusCode::OK, "POST /file/outline status: {resp}");

    assert_eq!(resp["collection"].as_str(), Some("outline_happy"));
    assert!(
        resp["structure"].is_object(),
        "structure must be a JSON object: {resp}"
    );
    let key_files = resp["key_files"].as_array().expect("key_files array");
    assert!(
        key_files.iter().any(|v| v.as_str() == Some("README.md")),
        "README.md should be flagged as a key file: {resp}"
    );
    assert_eq!(
        resp["statistics"]["total_files"].as_u64(),
        Some(3),
        "expected all 3 seeded files: {resp}"
    );
}

#[tokio::test]
async fn get_project_outline_rejects_missing_collection() {
    let app = TestApp::new().await;
    let (status, body) = app.post_json("/file/outline", json!({})).await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------
// get_related_files — POST /file/related
// ---------------------------------------------------------------------

#[tokio::test]
async fn get_related_files_returns_related_entries() {
    let app = TestApp::new().await;
    seed_file_collection(&app, "related_happy").await;

    let (status, resp) = app
        .post_json(
            "/file/related",
            json!({
                "collection": "related_happy",
                "file_path": "docs/guide.md",
                // 0.0 admits every other file regardless of BM25 score
                // magnitude, keeping the assertion below deterministic.
                "similarity_threshold": 0.0,
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "POST /file/related status: {resp}");

    assert_eq!(resp["source_file"].as_str(), Some("docs/guide.md"));
    let related = resp["related_files"]
        .as_array()
        .expect("related_files array");
    assert_eq!(
        related.len(),
        2,
        "expected the 2 other seeded files (self excluded): {resp}"
    );
    for entry in related {
        assert!(entry["path"].as_str().is_some());
        assert!(entry["similarity_score"].as_f64().is_some());
    }
}

#[tokio::test]
async fn get_related_files_rejects_nonexistent_collection() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json(
            "/file/related",
            json!({"collection": "does_not_exist_related", "file_path": "a.md"}),
        )
        .await;
    // Unlike `semantic_focus`, the file-operations handlers wrap every
    // `FileOperationError` (including `CollectionNotFound`) as a
    // generic 400 `bad_request` — see `FileOperations::get_related_files`
    // + the `Err(e) => create_bad_request_error(...)` arm in
    // `rest_handlers/files.rs`.
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("bad_request"));
    assert!(
        body["message"]
            .as_str()
            .unwrap_or("")
            .contains("Collection not found"),
        "expected collection-not-found message: {body}"
    );
}

// ---------------------------------------------------------------------
// search_by_file_type — POST /file/search_by_type
// ---------------------------------------------------------------------

#[tokio::test]
async fn search_by_file_type_matches_self_query() {
    let app = TestApp::new().await;
    seed_file_collection(&app, "filetype_happy").await;

    let (status, resp) = app
        .post_json(
            "/file/search_by_type",
            json!({
                "collection": "filetype_happy",
                "query": "vector databases store high dimensional embeddings",
                "file_types": ["rs"],
                "limit": 5,
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /file/search_by_type status: {resp}"
    );

    assert_eq!(
        resp["query"].as_str(),
        Some("vector databases store high dimensional embeddings")
    );
    assert_eq!(resp["file_types"].as_array().unwrap().len(), 1);
    let results = resp["results"].as_array().expect("results array");
    assert!(
        !results.is_empty(),
        "expected the self-matching .rs file: {resp}"
    );
    assert_eq!(results[0]["file_path"].as_str(), Some("src/main.rs"));
    assert_eq!(results[0]["file_type"].as_str(), Some("rs"));
}

#[tokio::test]
async fn search_by_file_type_rejects_missing_file_types() {
    let app = TestApp::new().await;
    seed_file_collection(&app, "filetype_error").await;

    let (status, body) = app
        .post_json(
            "/file/search_by_type",
            json!({"collection": "filetype_error", "query": "vector databases"}),
        )
        .await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------
// get_file_summary — POST /file/summary
// ---------------------------------------------------------------------

#[tokio::test]
async fn get_file_summary_returns_extractive_and_structural() {
    let app = TestApp::new().await;
    seed_file_collection(&app, "summary_happy").await;

    let (status, resp) = app
        .post_json(
            "/file/summary",
            json!({"collection": "summary_happy", "file_path": "docs/guide.md"}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "POST /file/summary status: {resp}");

    assert_eq!(resp["file_path"].as_str(), Some("docs/guide.md"));
    let extractive = resp["extractive_summary"]
        .as_str()
        .expect("extractive_summary must be present (default summary_type is Both)");
    assert!(
        extractive.contains("semantic search"),
        "extractive summary: {extractive}"
    );
    assert!(
        resp["structural_summary"].is_object(),
        "structural_summary must be present: {resp}"
    );
    assert_eq!(resp["metadata"]["file_type"].as_str(), Some("md"));
}

#[tokio::test]
async fn get_file_summary_rejects_missing_file_path() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json("/file/summary", json!({"collection": "summary_error"}))
        .await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------
// list_slow_queries — GET /slow_queries
// set_slow_query_config — POST /slow_queries/config
// ---------------------------------------------------------------------

#[tokio::test]
async fn slow_queries_list_and_configure_round_trip() {
    let app = TestApp::new().await;

    // `list_slow_queries` returns `Json<Value>` directly (not
    // `Result<Json<Value>, ErrorResponse>`) — it takes no `Json`
    // extractor and has no failure branch, so there is no error-branch
    // contract to exercise for it independently of this happy path.
    let (status, initial) = app.get("/slow_queries").await;
    assert_eq!(status, StatusCode::OK, "GET /slow_queries: {initial}");
    assert!(initial["entries"].is_array());
    assert_eq!(initial["total"].as_u64(), Some(0));

    let (status, resp) = app
        .post_json(
            "/slow_queries/config",
            json!({"threshold_ms": 250, "capacity": 42}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "POST /slow_queries/config: {resp}");
    assert_eq!(resp["threshold_ms"].as_u64(), Some(250));
    assert_eq!(resp["capacity"].as_u64(), Some(42));
    assert_eq!(resp["status"].as_str(), Some("ok"));

    let (status, after) = app.get("/slow_queries").await;
    assert_eq!(status, StatusCode::OK, "GET /slow_queries (after): {after}");
    assert_eq!(after["config"]["threshold_ms"].as_u64(), Some(250));
    assert_eq!(after["config"]["capacity"].as_u64(), Some(42));
}

#[tokio::test]
async fn set_slow_query_config_rejects_missing_threshold() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json("/slow_queries/config", json!({"capacity": 10}))
        .await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

#[tokio::test]
async fn set_slow_query_config_rejects_zero_capacity() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json(
            "/slow_queries/config",
            json!({"threshold_ms": 100, "capacity": 0}),
        )
        .await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------
// get_config — GET /config
// ---------------------------------------------------------------------

#[tokio::test]
async fn get_config_returns_json_object() {
    let app = TestApp::new().await;

    // `get_config` takes no `Json` extractor (no request body to
    // malform) and its return type is the bare `Json<Value>` — every
    // internal branch (parsed `config.yml` from one of a few relative
    // candidate paths, or, when none resolve from this test binary's
    // working directory, a hard-coded fallback object carrying an
    // `"error"` key) still returns 200. There is no error-branch
    // contract to assert; this happy-path shape check is the handler's
    // entire externally-observable behavior.
    let (status, body) = app.get("/config").await;
    assert_eq!(status, StatusCode::OK, "GET /config: {body}");
    assert!(
        body.is_object(),
        "config response must be a JSON object either way: {body}"
    );
}

// ---------------------------------------------------------------------
// update_workspace_config — POST /workspace/config
// ---------------------------------------------------------------------

/// Restores `./workspace.yml` (relative to this test binary's working
/// directory, i.e. `crates/vectorizer-server/workspace.yml` — a real,
/// checked-in project file) to its pre-test contents on drop, including
/// during panic unwinding. `update_workspace_config` writes this path
/// unconditionally, so any test exercising it MUST NOT leave the real
/// file mutated after the test run.
struct WorkspaceYamlGuard {
    path: std::path::PathBuf,
    original: Option<Vec<u8>>,
}

impl WorkspaceYamlGuard {
    fn capture() -> Self {
        let path = std::path::PathBuf::from("./workspace.yml");
        let original = std::fs::read(&path).ok();
        Self { path, original }
    }
}

impl Drop for WorkspaceYamlGuard {
    fn drop(&mut self) {
        if let Some(content) = &self.original {
            let _ = std::fs::write(&self.path, content);
        }
    }
}

#[tokio::test]
async fn update_workspace_config_writes_yaml_successfully() {
    let _guard = WorkspaceYamlGuard::capture();
    let app = TestApp::new().await;

    let (status, resp) = app
        .post_json(
            "/workspace/config",
            json!({
                "global_settings": {
                    "file_watcher": {
                        "watch_paths": [],
                        "auto_discovery": true,
                        "enable_auto_update": true,
                        "hot_reload": true,
                        "exclude_patterns": []
                    }
                },
                "projects": [],
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /workspace/config status: {resp}"
    );
    assert_eq!(resp["success"].as_bool(), Some(true));
}

#[tokio::test]
async fn update_workspace_config_rejects_malformed_json_body() {
    // `update_workspace_config` accepts any JSON `Value` and has no
    // field-level validation of its own — any well-formed body (object,
    // array, string, ...) round-trips through `serde_yaml::to_string`
    // successfully, so the handler itself never produces a 4xx. The
    // closest meaningful error-branch contract this route has is axum's
    // `Json` extractor rejecting a syntactically invalid body before the
    // handler ever runs.
    let _guard = WorkspaceYamlGuard::capture();
    let app = TestApp::new().await;

    let (status, _body) = app
        .post_raw("/workspace/config", "application/json", b"{not valid json")
        .await;
    assert!(
        status.is_client_error(),
        "malformed JSON body should be rejected before reaching the handler, got {status}"
    );
}

// ---------------------------------------------------------------------
// broad_discovery — POST /discovery/broad_discovery
// ---------------------------------------------------------------------

#[tokio::test]
async fn broad_discovery_returns_matching_chunk_for_self_query() {
    let app = TestApp::new().await;
    seed_file_collection(&app, "broad_happy").await;

    let (status, resp) = app
        .post_json(
            "/discovery/broad_discovery",
            json!({
                "queries": ["vector databases store high dimensional embeddings"],
                "k": 10,
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /discovery/broad_discovery status: {resp}"
    );

    let chunks = resp["chunks"].as_array().expect("chunks array");
    assert!(
        !chunks.is_empty(),
        "expected at least the self-matching chunk (default similarity_threshold 0.3): {resp}"
    );
    assert_eq!(resp["count"].as_u64(), Some(chunks.len() as u64));
}

#[tokio::test]
async fn broad_discovery_rejects_missing_queries() {
    let app = TestApp::new().await;
    let (status, body) = app.post_json("/discovery/broad_discovery", json!({})).await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------
// semantic_focus — POST /discovery/semantic_focus
// ---------------------------------------------------------------------

#[tokio::test]
async fn semantic_focus_returns_matching_chunk_for_self_query() {
    let app = TestApp::new().await;
    seed_file_collection(&app, "focus_happy").await;

    let (status, resp) = app
        .post_json(
            "/discovery/semantic_focus",
            json!({
                "collection": "focus_happy",
                "queries": ["semantic search finds documents by meaning not keywords"],
                "k": 10,
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /discovery/semantic_focus status: {resp}"
    );

    let chunks = resp["chunks"].as_array().expect("chunks array");
    assert!(
        !chunks.is_empty(),
        "expected at least the self-matching chunk (default similarity_threshold 0.35): {resp}"
    );
}

#[tokio::test]
async fn semantic_focus_rejects_nonexistent_collection() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json(
            "/discovery/semantic_focus",
            json!({"collection": "does_not_exist_focus", "queries": ["x"]}),
        )
        .await;
    // Unlike the file-operations handlers, `semantic_focus` maps
    // `store.get_collection(..)`'s `VectorizerError` straight through
    // `ErrorResponse::from` instead of wrapping it as a generic
    // `bad_request` — so a missing collection here is a real 404.
    assert_eq!(status.as_u16(), 404, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("collection_not_found"));
}

// ---------------------------------------------------------------------
// promote_readme — POST /discovery/promote_readme
// ---------------------------------------------------------------------

#[tokio::test]
async fn promote_readme_boosts_readme_to_top() {
    let app = TestApp::new().await;

    let chunk = |doc_id: &str, file_path: &str, score: f64| {
        json!({
            "collection": "docs",
            "doc_id": doc_id,
            "content": format!("content for {file_path}"),
            "score": score,
            "file_path": file_path,
            "chunk_index": 0,
            "file_extension": "md",
        })
    };

    let (status, resp) = app
        .post_json(
            "/discovery/promote_readme",
            json!({
                "chunks": [
                    chunk("d1", "other.md", 0.9),
                    chunk("d2", "README.md", 0.5),
                ],
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /discovery/promote_readme status: {resp}"
    );

    let promoted = resp["promoted_chunks"]
        .as_array()
        .expect("promoted_chunks array");
    assert_eq!(promoted.len(), 2);
    assert_eq!(
        promoted[0]["file_path"].as_str(),
        Some("README.md"),
        "README should be promoted to the top despite the lower original score: {resp}"
    );
    assert!(
        promoted[0]["score"].as_f64().unwrap_or(0.0) > 0.5,
        "README score should be boosted above its original 0.5: {resp}"
    );
}

#[tokio::test]
async fn promote_readme_rejects_missing_chunks() {
    let app = TestApp::new().await;
    let (status, body) = app.post_json("/discovery/promote_readme", json!({})).await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------
// build_answer_plan — POST /discovery/build_answer_plan
// ---------------------------------------------------------------------

#[tokio::test]
async fn build_answer_plan_groups_bullets_into_sections() {
    let app = TestApp::new().await;

    let bullet = |text: &str, category: &str, score: f64| {
        json!({
            "text": text,
            "source_id": "doc#0",
            "collection": "docs",
            "file_path": "docs/guide.md",
            "score": score,
            "category": category,
        })
    };

    let (status, resp) = app
        .post_json(
            "/discovery/build_answer_plan",
            json!({
                "bullets": [
                    bullet("A definition", "Definition", 0.9),
                    bullet("A feature", "Feature", 0.8),
                ],
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /discovery/build_answer_plan status: {resp}"
    );

    assert_eq!(resp["total_bullets"].as_u64(), Some(2));
    let sections = resp["sections"].as_array().expect("sections array");
    assert!(
        !sections.is_empty(),
        "expected at least one section: {resp}"
    );
    let sources = resp["sources"].as_array().expect("sources array");
    assert!(!sources.is_empty(), "expected at least one source: {resp}");
}

#[tokio::test]
async fn build_answer_plan_rejects_missing_bullets() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json("/discovery/build_answer_plan", json!({}))
        .await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------
// render_llm_prompt — POST /discovery/render_llm_prompt
// ---------------------------------------------------------------------

#[tokio::test]
async fn render_llm_prompt_renders_sections_and_sources() {
    let app = TestApp::new().await;

    let (status, resp) = app
        .post_json(
            "/discovery/render_llm_prompt",
            json!({
                "plan": {
                    "sections": [{
                        "title": "Definition",
                        "priority": 1,
                        "bullets": [{
                            "text": "Test bullet",
                            "source_id": "test#0",
                            "collection": "test",
                            "file_path": "test.md",
                            "score": 0.9,
                            "category": "Definition",
                        }],
                    }],
                    "total_bullets": 1,
                    "sources": ["[test#0]"],
                },
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /discovery/render_llm_prompt status: {resp}"
    );

    let prompt = resp["prompt"].as_str().expect("prompt string");
    assert!(prompt.contains("Context from Vector Database"));
    assert!(prompt.contains("Test bullet"));
    assert!(prompt.contains("[test#0]"));
    assert_eq!(resp["length"].as_u64(), Some(prompt.len() as u64));
}

#[tokio::test]
async fn render_llm_prompt_rejects_missing_plan() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json("/discovery/render_llm_prompt", json!({}))
        .await;
    assert_eq!(status.as_u16(), 400, "body: {body}");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------
// get_backup_directory — GET /backups/directory
// ---------------------------------------------------------------------

#[tokio::test]
async fn get_backup_directory_returns_path() {
    let app = TestApp::new().await;

    // `get_backup_directory` takes no parameters at all (no `State`, no
    // `Json` extractor) and always returns the same literal
    // `{"path": "./backups"}`. There is no input to validate and no
    // failure mode to exercise, so this happy-path assertion is the
    // handler's entire contract.
    let (status, body) = app.get("/backups/directory").await;
    assert_eq!(status, StatusCode::OK, "GET /backups/directory: {body}");
    assert_eq!(body["path"].as_str(), Some("./backups"));
}
