//! In-process migration of
//! `crates/vectorizer/tests/api/rest/force_save_real.rs` (phase39 §1.2)
//! onto the shared harness in `tests/common/mod.rs`.
//!
//! Same assertions as the live-server suite, but dispatched through the
//! real production router via `tower::ServiceExt::oneshot` instead of
//! `reqwest` against `127.0.0.1:15002` — no `#[ignore]`, runs in CI.
//!
//! Covers phase8_fix-force-save-endpoint: the handler used to return
//! success without ever writing `vectorizer.vecdb`. The live suite seeds
//! a collection via `/batch_insert`, triggers `/force-save`, and asserts
//! the on-disk `.vecdb` file appears and grows.
//!
//! ## Delta vs the live suite
//!
//! [`common::TestApp`] is built via
//! `VectorizerServer::new_for_test_harness`, which always sets
//! `auto_save_manager: None` (see
//! `crates/vectorizer-server/src/server/core/bootstrap.rs`). Per the doc
//! comment on `force_save_collection`
//! (`crates/vectorizer-server/src/server/rest_handlers/collections.rs`),
//! a server with no `auto_save_manager` falls back to
//! `VectorStore::force_save_all`, which — per its own doc comment
//! (`crates/vectorizer/src/db/vector_store/autosave.rs`) — only clears
//! the in-memory pending-saves marker and **never writes `.vecdb`**;
//! the real flush is owned exclusively by `AutoSaveManager::force_save`.
//! So in this harness:
//! - the response still reports `success: true` (the no-auto-save
//!   fallback path succeeds), but `flushed` is `false`, not `true`;
//! - no `vectorizer.vecdb` file is ever created in the harness's
//!   `VECTORIZER_DATA_DIR` tempdir, so the on-disk size assertions from
//!   the live suite cannot be replicated here.
//!
//! This suite asserts the delta explicitly (`flushed == false`, no
//! `.vecdb` on disk) instead of silently dropping the coverage, and
//! keeps the unknown-collection rejection assertion, which is fully
//! portable.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use common::TestApp;
use serde_json::{Value, json};

async fn ensure_collection(app: &TestApp, name: &str) {
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create status {status}: {resp}");
}

async fn seed_texts(app: &TestApp, name: &str, n: usize) {
    let texts: Vec<Value> = (0..n)
        .map(|i| json!({"text": format!("force-save probe doc {}", i)}))
        .collect();
    let (status, resp) = app
        .post_json("/batch_insert", json!({"collection": name, "texts": texts}))
        .await;
    assert!(status.is_success(), "batch_insert status {status}: {resp}");
    assert_eq!(resp["inserted"].as_u64(), Some(n as u64));
}

#[tokio::test]
async fn force_save_reports_success_without_vecdb_flush_when_no_auto_save_manager() {
    let app = TestApp::new().await;
    ensure_collection(&app, "force_save_in_process_ok").await;
    seed_texts(&app, "force_save_in_process_ok", 20).await;

    let (status, resp) = app
        .post_json(
            "/collections/force_save_in_process_ok/force-save",
            json!({}),
        )
        .await;
    assert!(
        status.is_success(),
        "POST /force-save status {status}: {resp}"
    );

    assert_eq!(resp["success"].as_bool(), Some(true));
    // Delta vs the live suite: no `auto_save_manager` in this harness, so
    // the handler falls back to `VectorStore::force_save_all` (pending-
    // marker clear only) and `flushed` is `false` — see module doc.
    assert_eq!(
        resp["flushed"].as_bool(),
        Some(false),
        "flushed must be false in a harness with no auto_save_manager"
    );

    // No `.vecdb` is ever written in this harness — the real flush is
    // owned exclusively by `AutoSaveManager::force_save`, which this
    // harness never constructs. Read the path from `app.data_dir()`
    // (not the process-global `VECTORIZER_DATA_DIR` env var) so this
    // assertion is safe under parallel test execution — see
    // `TestApp::data_dir` doc comment.
    let vecdb_path = app.data_dir().join("vectorizer.vecdb");
    assert!(
        !vecdb_path.exists(),
        "no .vecdb should be written without an auto_save_manager, found at {vecdb_path:?}"
    );
}

#[tokio::test]
async fn force_save_rejects_unknown_collection() {
    let app = TestApp::new().await;
    let (status, _) = app
        .post_json(
            "/collections/force_save_in_process_missing_xyz/force-save",
            json!({}),
        )
        .await;
    assert!(
        !status.is_success(),
        "unknown collection should not return 2xx (got {status})"
    );
}
