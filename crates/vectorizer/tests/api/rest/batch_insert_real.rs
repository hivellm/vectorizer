//! Live integration tests for `POST /batch_insert` and `POST /insert_texts`.
//!
//! These cover the phase8_fix-batch-insert-endpoints write path: the two
//! handlers used to return success without persisting; this test seeds a
//! collection with a batch and asserts the vectors are actually there.
//!
//! Require a running server at `127.0.0.1:15002`. Run with:
//! `cargo test --test all_tests api::rest::batch_insert_real -- --ignored`

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

fn ensure_clean_collection(http: &reqwest::blocking::Client, name: &str) {
    let _ = http
        .delete(format!("{}/collections/{}", VECTORIZER_API_URL, name))
        .send();
    let resp = http
        .post(format!("{}/collections", VECTORIZER_API_URL))
        .json(&json!({
            "name": name,
            "dimension": 512,
            "metric": "cosine",
        }))
        .send()
        .expect("create collection");
    assert!(
        resp.status().is_success(),
        "unexpected create_collection status {}",
        resp.status()
    );
}

fn collection_vector_count(http: &reqwest::blocking::Client, name: &str) -> u64 {
    let meta: Value = http
        .get(format!("{}/collections/{}", VECTORIZER_API_URL, name))
        .send()
        .expect("get collection")
        .json()
        .expect("decode collection meta");
    meta["vector_count"].as_u64().expect("vector_count field")
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn batch_insert_persists_all_entries() {
    let http = client();
    ensure_clean_collection(&http, "batch_insert_real_ok");

    let body = json!({
        "collection": "batch_insert_real_ok",
        "texts": [
            {"text": "alpha document one",    "metadata": {"idx": "0"}},
            {"text": "beta document two",      "metadata": {"idx": "1"}},
            {"text": "gamma document three",   "metadata": {"idx": "2"}},
            {"text": "delta document four",    "metadata": {"idx": "3"}},
            {"text": "epsilon document five",  "metadata": {"idx": "4"}},
            {"text": "zeta document six",      "metadata": {"idx": "5"}},
            {"text": "eta document seven",     "metadata": {"idx": "6"}},
            {"text": "theta document eight",   "metadata": {"idx": "7"}},
            {"text": "iota document nine",     "metadata": {"idx": "8"}},
            {"text": "kappa document ten",     "metadata": {"idx": "9"}},
        ],
    });

    let resp: Value = http
        .post(format!("{}/batch_insert", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("POST /batch_insert")
        .json()
        .expect("decode response");

    assert_eq!(resp["inserted"].as_u64(), Some(10));
    assert_eq!(resp["failed"].as_u64(), Some(0));
    assert_eq!(resp["count"].as_u64(), Some(10));
    let results = resp["results"].as_array().expect("results array");
    assert_eq!(results.len(), 10);
    for (i, r) in results.iter().enumerate() {
        assert_eq!(r["index"].as_u64(), Some(i as u64));
        assert_eq!(r["status"].as_str(), Some("ok"));
        let ids = r["vector_ids"].as_array().expect("vector_ids");
        assert!(!ids.is_empty(), "entry {i} has no vector ids");
    }

    assert_eq!(
        collection_vector_count(&http, "batch_insert_real_ok"),
        10,
        "collection should expose all 10 vectors"
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_texts_alias_persists_all_entries() {
    let http = client();
    ensure_clean_collection(&http, "insert_texts_alias_ok");

    let body = json!({
        "collection": "insert_texts_alias_ok",
        "texts": [
            {"text": "alias alpha"},
            {"text": "alias beta"},
            {"text": "alias gamma"},
        ],
    });

    let resp: Value = http
        .post(format!("{}/insert_texts", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("POST /insert_texts")
        .json()
        .expect("decode response");

    assert_eq!(resp["inserted"].as_u64(), Some(3));
    assert_eq!(resp["failed"].as_u64(), Some(0));
    assert_eq!(resp["count"].as_u64(), Some(3));
    assert_eq!(
        collection_vector_count(&http, "insert_texts_alias_ok"),
        3,
        "alias endpoint should persist same as batch_insert"
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn batch_insert_partial_failure_is_reported_per_item() {
    let http = client();
    ensure_clean_collection(&http, "batch_insert_real_partial");

    let body = json!({
        "collection": "batch_insert_real_partial",
        "texts": [
            {"text": "good one"},
            {"metadata": {"missing_text": "yes"}},
            {"text": "good two"},
        ],
    });

    let resp: Value = http
        .post(format!("{}/batch_insert", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("POST /batch_insert")
        .json()
        .expect("decode response");

    assert_eq!(resp["inserted"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(1));
    let results = resp["results"].as_array().expect("results array");
    assert_eq!(results[0]["status"].as_str(), Some("ok"));
    assert_eq!(results[1]["status"].as_str(), Some("error"));
    assert_eq!(results[2]["status"].as_str(), Some("ok"));

    assert_eq!(
        collection_vector_count(&http, "batch_insert_real_partial"),
        2,
        "only the 2 valid entries should be persisted"
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn batch_insert_rejects_missing_collection() {
    let http = client();

    let resp = http
        .post(format!("{}/batch_insert", VECTORIZER_API_URL))
        .json(&json!({
            "texts": [{"text": "orphan"}],
        }))
        .send()
        .expect("POST /batch_insert");
    assert_eq!(resp.status().as_u16(), 400);
    let body: Value = resp.json().expect("decode error");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn batch_insert_rejects_empty_texts_array() {
    let http = client();
    ensure_clean_collection(&http, "batch_insert_real_empty");

    let resp = http
        .post(format!("{}/batch_insert", VECTORIZER_API_URL))
        .json(&json!({
            "collection": "batch_insert_real_empty",
            "texts": [],
        }))
        .send()
        .expect("POST /batch_insert");
    assert_eq!(resp.status().as_u16(), 400);
    let body: Value = resp.json().expect("decode error");
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}
