//! In-process migration of `crates/vectorizer/tests/api/rest/vector_search_real.rs`
//! (phase39 §1.2) onto the shared harness in `tests/common/mod.rs`.
//!
//! Same assertions as the live-server suite, but dispatched through the
//! real production router via `tower::ServiceExt::oneshot` instead of
//! `reqwest` against `127.0.0.1:15002` — no `#[ignore]`, runs in CI.
//!
//! Covers phase8_fix-search-endpoint-placeholders: both `POST /search`
//! and `POST /collections/{name}/search` previously returned empty
//! `results` regardless of input. This suite seeds a collection through
//! the real insert path, grabs one known vector, runs raw-vector search,
//! and asserts the query vector itself comes back with score ~1.0.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

/// Seed `name` with 5 short texts via the real `/batch_insert` path so the
/// collection has embeddings we can query against.
async fn seed_collection(app: &TestApp, name: &str) {
    let _ = app.delete(&format!("/collections/{name}")).await;

    let (status, _) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create status {status}");

    let body = json!({
        "collection": name,
        "texts": [
            {"text": "alpha doc uno",   "metadata": {"tag": "a"}},
            {"text": "beta doc dos",    "metadata": {"tag": "b"}},
            {"text": "gamma doc tres",  "metadata": {"tag": "c"}},
            {"text": "delta doc cuatro","metadata": {"tag": "d"}},
            {"text": "epsilon doc cinco","metadata": {"tag": "e"}},
        ],
    });
    let (status, resp) = app.post_json("/batch_insert", body).await;
    assert!(status.is_success(), "batch_insert status {status}: {resp}");
    assert_eq!(resp["inserted"].as_u64(), Some(5));
}

/// Fetch the first vector in `name` via `GET /collections/{name}/vectors?limit=1`.
async fn first_vector(app: &TestApp, name: &str) -> (String, Vec<f32>) {
    let (status, resp) = app
        .get(&format!("/collections/{name}/vectors?limit=1"))
        .await;
    assert!(status.is_success(), "list_vectors status {status}: {resp}");
    let vectors = resp["vectors"].as_array().expect("vectors array");
    let first = &vectors[0];
    let id = first["id"].as_str().expect("vector id").to_string();
    let vec = first["vector"]
        .as_array()
        .expect("vector array")
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect();
    (id, vec)
}

#[tokio::test]
async fn search_vectors_returns_target_when_query_is_an_existing_vector() {
    let app = TestApp::new().await;
    seed_collection(&app, "vector_search_real_body").await;
    let (target_id, target_vec) = first_vector(&app, "vector_search_real_body").await;

    let (status, resp) = app
        .post_json(
            "/search",
            json!({
                "collection": "vector_search_real_body",
                "vector": target_vec,
                "limit": 3,
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "POST /search status: {resp}");

    assert_eq!(resp["collection"].as_str(), Some("vector_search_real_body"));
    assert_eq!(resp["query_type"].as_str(), Some("vector"));
    let results = resp["results"].as_array().expect("results array");
    assert!(!results.is_empty(), "/search returned empty results");
    assert_eq!(
        results[0]["id"].as_str(),
        Some(target_id.as_str()),
        "top result should be the query vector itself"
    );
    let top_score = results[0]["score"].as_f64().expect("score field");
    assert!(
        top_score >= 0.999,
        "top score for self-query should be ~1.0, got {}",
        top_score
    );
}

#[tokio::test]
async fn search_by_collection_path_returns_same_results() {
    let app = TestApp::new().await;
    seed_collection(&app, "vector_search_real_path").await;
    let (target_id, target_vec) = first_vector(&app, "vector_search_real_path").await;

    let (status, resp) = app
        .post_json(
            "/collections/vector_search_real_path/search",
            json!({
                "vector": target_vec,
                "limit": 3,
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /collections/{{name}}/search status: {resp}"
    );

    let results = resp["results"].as_array().expect("results array");
    assert!(!results.is_empty());
    assert_eq!(results[0]["id"].as_str(), Some(target_id.as_str()));
}

#[tokio::test]
async fn search_rejects_vector_dimension_mismatch() {
    let app = TestApp::new().await;
    seed_collection(&app, "vector_search_real_dim").await;

    let (status, body) = app
        .post_json(
            "/search",
            json!({
                "collection": "vector_search_real_dim",
                "vector": [0.1, 0.2, 0.3],
                "limit": 1,
            }),
        )
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
    assert!(
        body["message"].as_str().unwrap_or("").contains("dimension"),
        "expected dimension-mismatch message, got: {:?}",
        body["message"]
    );
}

#[tokio::test]
async fn search_rejects_missing_collection_on_bare_route() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json("/search", json!({"vector": vec![0.1_f32; 512], "limit": 1}))
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

#[tokio::test]
async fn search_rejects_missing_vector_parameter() {
    let app = TestApp::new().await;
    seed_collection(&app, "vector_search_real_novec").await;
    let (status, body) = app
        .post_json(
            "/search",
            json!({"collection": "vector_search_real_novec", "limit": 1}),
        )
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}
