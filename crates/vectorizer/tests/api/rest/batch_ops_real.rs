//! Live integration tests for `POST /embed`, `POST /batch_search`,
//! `POST /batch_update`, `POST /batch_delete`.
//!
//! All four handlers previously returned hard-coded values. This test
//! seeds a collection and asserts every endpoint actually reads and
//! writes through `state.store` / `state.embedding_manager`.
//!
//! Require a running server at `127.0.0.1:15002`. Run with:
//! `cargo test --test all_tests api::rest::batch_ops_real -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

use std::time::Duration;

use serde_json::{Value, json};

const BASE: &str = "http://127.0.0.1:15002";

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("build reqwest client")
}

fn seed(http: &reqwest::blocking::Client, name: &str, n: usize) -> Vec<String> {
    let _ = http.delete(format!("{}/collections/{}", BASE, name)).send();
    let resp = http
        .post(format!("{}/collections", BASE))
        .json(&json!({"name": name, "dimension": 512, "metric": "cosine"}))
        .send()
        .expect("create collection");
    assert!(
        resp.status().is_success(),
        "create status {}",
        resp.status()
    );
    let texts: Vec<Value> = (0..n)
        .map(|i| json!({"text": format!("batch-ops probe doc {}", i)}))
        .collect();
    let resp: Value = http
        .post(format!("{}/batch_insert", BASE))
        .json(&json!({"collection": name, "texts": texts}))
        .send()
        .expect("batch_insert")
        .json()
        .expect("decode batch_insert");
    assert_eq!(resp["inserted"].as_u64(), Some(n as u64));
    resp["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["vector_ids"][0].as_str().unwrap().to_string())
        .collect()
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn embed_returns_real_embedding() {
    let http = client();
    let resp: Value = http
        .post(format!("{}/embed", BASE))
        .json(&json!({"text": "hello world"}))
        .send()
        .expect("POST /embed")
        .json()
        .expect("decode /embed");
    let dim = resp["dimension"].as_u64().unwrap_or(0);
    assert_eq!(dim, 512);
    let emb = resp["embedding"].as_array().expect("embedding array");
    assert_eq!(emb.len(), 512);
    // The pre-fix handler returned every component as 0.1. Post-fix
    // components must span both signs with real variation.
    let uniform_one_tenth = emb
        .iter()
        .all(|v| (v.as_f64().unwrap_or(0.0) - 0.1).abs() < 1e-3);
    assert!(
        !uniform_one_tenth,
        "embedding did not come from the real EmbeddingManager"
    );
    let has_pos = emb.iter().any(|v| v.as_f64().unwrap_or(0.0) > 0.01);
    let has_neg = emb.iter().any(|v| v.as_f64().unwrap_or(0.0) < -0.01);
    assert!(has_pos && has_neg, "embedding lacks sign variation");
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn batch_search_runs_multiple_queries() {
    let http = client();
    seed(&http, "batch_ops_real_search", 6);

    let resp: Value = http
        .post(format!("{}/batch_search", BASE))
        .json(&json!({
            "collection": "batch_ops_real_search",
            "queries": [
                {"query": "probe doc 1", "limit": 2},
                {"query": "probe doc 5", "limit": 2},
            ],
        }))
        .send()
        .expect("POST /batch_search")
        .json()
        .expect("decode");

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

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn batch_update_overwrites_payload() {
    let http = client();
    let ids = seed(&http, "batch_ops_real_update", 4);

    let resp: Value = http
        .post(format!("{}/batch_update", BASE))
        .json(&json!({
            "collection": "batch_ops_real_update",
            "updates": [
                {"id": &ids[0], "payload": {"tag": "updated-a"}},
                {"id": &ids[1], "payload": {"tag": "updated-b"}},
                {"id": "non-existent-id", "payload": {"tag": "missing"}},
            ],
        }))
        .send()
        .expect("POST /batch_update")
        .json()
        .expect("decode");

    assert_eq!(resp["updated"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(1));
    let results = resp["results"].as_array().unwrap();
    assert_eq!(results[0]["status"].as_str(), Some("ok"));
    assert_eq!(results[1]["status"].as_str(), Some("ok"));
    assert_eq!(results[2]["status"].as_str(), Some("error"));
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn batch_delete_removes_vectors_and_reports_missing() {
    let http = client();
    let ids = seed(&http, "batch_ops_real_delete", 5);

    let resp: Value = http
        .post(format!("{}/batch_delete", BASE))
        .json(&json!({
            "collection": "batch_ops_real_delete",
            "ids": [&ids[0], &ids[1], "bogus-id-xyz"],
        }))
        .send()
        .expect("POST /batch_delete")
        .json()
        .expect("decode");

    assert_eq!(resp["deleted"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(1));

    let meta: Value = http
        .get(format!("{}/collections/batch_ops_real_delete", BASE))
        .send()
        .expect("GET collection")
        .json()
        .expect("decode meta");
    assert_eq!(meta["vector_count"].as_u64(), Some(3));
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn batch_endpoints_reject_empty_arrays() {
    let http = client();
    let _ = seed(&http, "batch_ops_real_empty", 1);

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
        let resp = http
            .post(format!("{}{}", BASE, path))
            .json(&body)
            .send()
            .expect("POST");
        assert_eq!(resp.status().as_u16(), 400, "{} should reject empty", path);
        let body: Value = resp.json().expect("decode error");
        assert_eq!(body["error_type"].as_str(), Some("validation_error"));
    }
}
