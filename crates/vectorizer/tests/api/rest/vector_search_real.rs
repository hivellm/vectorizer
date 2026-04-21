//! Live integration tests for `POST /search` and `POST /collections/{name}/search`.
//!
//! Covers phase8_fix-search-endpoint-placeholders: both routes previously
//! returned empty `results` regardless of input. This test seeds a
//! collection through the real insert path, grabs one known vector, runs
//! raw-vector search, and asserts the query vector itself comes back with
//! score ~1.0.
//!
//! Require a running server at `127.0.0.1:15002`. Run with:
//! `cargo test --test all_tests api::rest::vector_search_real -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

use std::time::Duration;

use serde_json::{Value, json};

const VECTORIZER_API_URL: &str = "http://127.0.0.1:15002";

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("build reqwest client")
}

/// Seed `name` with 5 short texts via the real /batch_insert path so the
/// collection has embeddings we can query against.
fn seed_collection(http: &reqwest::blocking::Client, name: &str) {
    let _ = http
        .delete(format!("{}/collections/{}", VECTORIZER_API_URL, name))
        .send();
    let resp = http
        .post(format!("{}/collections", VECTORIZER_API_URL))
        .json(&json!({"name": name, "dimension": 512, "metric": "cosine"}))
        .send()
        .expect("create collection");
    assert!(
        resp.status().is_success(),
        "create status {}",
        resp.status()
    );

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
    let resp: Value = http
        .post(format!("{}/batch_insert", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("batch_insert")
        .json()
        .expect("decode batch_insert");
    assert_eq!(resp["inserted"].as_u64(), Some(5));
}

fn first_vector(http: &reqwest::blocking::Client, name: &str) -> (String, Vec<f32>) {
    let resp: Value = http
        .get(format!(
            "{}/collections/{}/vectors?limit=1",
            VECTORIZER_API_URL, name
        ))
        .send()
        .expect("list_vectors")
        .json()
        .expect("decode list_vectors");
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

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn search_vectors_returns_target_when_query_is_an_existing_vector() {
    let http = client();
    seed_collection(&http, "vector_search_real_body");
    let (target_id, target_vec) = first_vector(&http, "vector_search_real_body");

    let resp: Value = http
        .post(format!("{}/search", VECTORIZER_API_URL))
        .json(&json!({
            "collection": "vector_search_real_body",
            "vector": target_vec,
            "limit": 3,
        }))
        .send()
        .expect("POST /search")
        .json()
        .expect("decode /search");

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

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn search_by_collection_path_returns_same_results() {
    let http = client();
    seed_collection(&http, "vector_search_real_path");
    let (target_id, target_vec) = first_vector(&http, "vector_search_real_path");

    let resp: Value = http
        .post(format!(
            "{}/collections/vector_search_real_path/search",
            VECTORIZER_API_URL
        ))
        .json(&json!({
            "vector": target_vec,
            "limit": 3,
        }))
        .send()
        .expect("POST /collections/{name}/search")
        .json()
        .expect("decode response");

    let results = resp["results"].as_array().expect("results array");
    assert!(!results.is_empty());
    assert_eq!(results[0]["id"].as_str(), Some(target_id.as_str()));
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn search_rejects_vector_dimension_mismatch() {
    let http = client();
    seed_collection(&http, "vector_search_real_dim");

    let resp = http
        .post(format!("{}/search", VECTORIZER_API_URL))
        .json(&json!({
            "collection": "vector_search_real_dim",
            "vector": [0.1, 0.2, 0.3],
            "limit": 1,
        }))
        .send()
        .expect("POST /search");
    assert_eq!(resp.status().as_u16(), 400);
    let body: Value = resp.json().expect("decode error");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
    assert!(
        body["message"].as_str().unwrap_or("").contains("dimension"),
        "expected dimension-mismatch message, got: {:?}",
        body["message"]
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn search_rejects_missing_collection_on_bare_route() {
    let http = client();
    let resp = http
        .post(format!("{}/search", VECTORIZER_API_URL))
        .json(&json!({"vector": vec![0.1_f32; 512], "limit": 1}))
        .send()
        .expect("POST /search");
    assert_eq!(resp.status().as_u16(), 400);
    let body: Value = resp.json().expect("decode error");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn search_rejects_missing_vector_parameter() {
    let http = client();
    seed_collection(&http, "vector_search_real_novec");
    let resp = http
        .post(format!("{}/search", VECTORIZER_API_URL))
        .json(&json!({"collection": "vector_search_real_novec", "limit": 1}))
        .send()
        .expect("POST /search");
    assert_eq!(resp.status().as_u16(), 400);
    let body: Value = resp.json().expect("decode error");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}
