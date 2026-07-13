//! In-process migration of
//! `crates/vectorizer/tests/api/rest/batch_ops_real.rs` (phase39 §1.2)
//! onto the shared harness in `tests/common/mod.rs`.
//!
//! Same assertions as the live-server suite, but dispatched through the
//! real production router via `tower::ServiceExt::oneshot` instead of
//! `reqwest` against `127.0.0.1:15002` — no `#[ignore]`, runs in CI.
//!
//! Covers `POST /embed`, `POST /batch_search`, `POST /batch_update`,
//! `POST /batch_delete` — all four handlers previously returned
//! hard-coded values instead of reading/writing through `state.store` /
//! `state.embedding_manager`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use common::TestApp;
use serde_json::{Value, json};

/// Delete-then-create `name` as a 512-dim cosine collection, then batch
/// insert `n` probe texts and return their vector ids.
async fn seed(app: &TestApp, name: &str, n: usize) -> Vec<String> {
    let _ = app.delete(&format!("/collections/{name}")).await;
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create status {status}: {resp}");

    let texts: Vec<Value> = (0..n)
        .map(|i| json!({"text": format!("batch-ops probe doc {}", i)}))
        .collect();
    let (status, resp) = app
        .post_json("/batch_insert", json!({"collection": name, "texts": texts}))
        .await;
    assert!(status.is_success(), "batch_insert status {status}: {resp}");
    assert_eq!(resp["inserted"].as_u64(), Some(n as u64));
    resp["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["vector_ids"][0].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn embed_returns_real_embedding() {
    let app = TestApp::new().await;
    // Query text drawn from `BM25_SEED_CORPUS` (see `tests/common/mod.rs`)
    // so the fitted BM25 vocabulary actually scores it — an out-of-vocab
    // query like the live suite's "hello world" would score to an
    // all-zero vector here, since this harness's vocabulary is fixed to
    // the small seed corpus rather than the live server's live-fitted
    // production vocabulary.
    let (status, resp) = app
        .post_json("/embed", json!({"text": "quick brown fox jumps"}))
        .await;
    assert!(status.is_success(), "POST /embed status {status}: {resp}");

    let dim = resp["dimension"].as_u64().unwrap_or(0);
    assert_eq!(dim, 512);
    let emb = resp["embedding"].as_array().expect("embedding array");
    assert_eq!(emb.len(), 512);
    // The old handler returned every component as a fixed 0.1. The fixed
    // handler must show real BM25 term-frequency variation instead.
    let uniform_one_tenth = emb
        .iter()
        .all(|v| (v.as_f64().unwrap_or(0.0) - 0.1).abs() < 1e-3);
    assert!(
        !uniform_one_tenth,
        "embedding did not come from the real EmbeddingManager"
    );
    // Delta vs. the live-server suite: that suite asserted `has_pos &&
    // has_neg` because the live default dense embedding model produces
    // components spanning both signs. This harness's default provider is
    // BM25 (sparse, non-negative-by-construction term scores — see
    // `BM25Provider::calculate_bm25_score`), so requiring a negative
    // component is unreachable in-process. Instead assert the response
    // has genuine per-term signal: some non-zero (in-vocabulary terms
    // scored) and some zero (out-of-vocabulary vocabulary slots), which
    // is what the real fix actually restored over the flat 0.1 output.
    let has_signal = emb.iter().any(|v| v.as_f64().unwrap_or(0.0) > 0.01);
    let has_zero = emb.iter().any(|v| v.as_f64().unwrap_or(1.0) == 0.0);
    assert!(
        has_signal && has_zero,
        "embedding lacks real term-frequency variation"
    );
}

#[tokio::test]
async fn batch_search_runs_multiple_queries() {
    let app = TestApp::new().await;
    seed(&app, "batch_ops_real_search", 6).await;

    let (status, resp) = app
        .post_json(
            "/batch_search",
            json!({
                "collection": "batch_ops_real_search",
                "queries": [
                    {"query": "probe doc 1", "limit": 2},
                    {"query": "probe doc 5", "limit": 2},
                ],
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "POST /batch_search status {status}: {resp}"
    );

    assert_eq!(resp["succeeded"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(0));
    let results = resp["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    for r in results {
        assert_eq!(r["status"].as_str(), Some("ok"));
        let hits = r["results"].as_array().unwrap();
        assert!(!hits.is_empty(), "batch_search query returned zero hits");
    }
}

#[tokio::test]
async fn batch_update_overwrites_payload() {
    let app = TestApp::new().await;
    let ids = seed(&app, "batch_ops_real_update", 4).await;

    let (status, resp) = app
        .post_json(
            "/batch_update",
            json!({
                "collection": "batch_ops_real_update",
                "updates": [
                    {"id": &ids[0], "payload": {"tag": "updated-a"}},
                    {"id": &ids[1], "payload": {"tag": "updated-b"}},
                    {"id": "non-existent-id", "payload": {"tag": "missing"}},
                ],
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "POST /batch_update status {status}: {resp}"
    );

    assert_eq!(resp["updated"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(1));
    let results = resp["results"].as_array().unwrap();
    assert_eq!(results[0]["status"].as_str(), Some("ok"));
    assert_eq!(results[1]["status"].as_str(), Some("ok"));
    assert_eq!(results[2]["status"].as_str(), Some("error"));
}

#[tokio::test]
async fn batch_delete_removes_vectors_and_reports_missing() {
    let app = TestApp::new().await;
    let ids = seed(&app, "batch_ops_real_delete", 5).await;

    let (status, resp) = app
        .post_json(
            "/batch_delete",
            json!({
                "collection": "batch_ops_real_delete",
                "ids": [&ids[0], &ids[1], "bogus-id-xyz"],
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "POST /batch_delete status {status}: {resp}"
    );

    assert_eq!(resp["deleted"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(1));

    let (status, meta) = app.get("/collections/batch_ops_real_delete").await;
    assert!(
        status.is_success(),
        "GET collection status {status}: {meta}"
    );
    assert_eq!(meta["vector_count"].as_u64(), Some(3));
}

#[tokio::test]
async fn batch_endpoints_reject_empty_arrays() {
    let app = TestApp::new().await;
    let _ = seed(&app, "batch_ops_real_empty", 1).await;

    for (path, body) in [
        (
            "/batch_search",
            json!({"collection": "batch_ops_real_empty", "queries": []}),
        ),
        (
            "/batch_update",
            json!({"collection": "batch_ops_real_empty", "updates": []}),
        ),
        (
            "/batch_delete",
            json!({"collection": "batch_ops_real_empty", "ids": []}),
        ),
    ] {
        let (status, body) = app.post_json(path, body).await;
        assert_eq!(status.as_u16(), 400, "{} should reject empty", path);
        assert_eq!(body["error_type"].as_str(), Some("validation_error"));
    }
}
