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

// ---------------------------------------------------------------------------
// phase9 — client-id honoring + flat chunk payload + /insert_vectors
// ---------------------------------------------------------------------------

/// Scroll the Qdrant-compat endpoint to fetch every point's payload.
fn scroll_all_payloads(http: &reqwest::blocking::Client, name: &str) -> Vec<Value> {
    let resp: Value = http
        .post(format!(
            "{}/qdrant/collections/{}/points/scroll",
            VECTORIZER_API_URL, name
        ))
        .json(&json!({"limit": 100, "with_payload": true, "with_vector": false}))
        .send()
        .expect("POST /qdrant/.../points/scroll")
        .json()
        .expect("decode scroll response");
    resp["result"]["points"]
        .as_array()
        .cloned()
        .or_else(|| resp["points"].as_array().cloned())
        .unwrap_or_default()
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_texts_honors_client_id_for_short_text() {
    let http = client();
    ensure_clean_collection(&http, "phase9_client_id_short");

    let body = json!({
        "collection": "phase9_client_id_short",
        "texts": [{
            "id": "doc:42",
            "text": "Texto curto preserva o id cliente.",
            "metadata": {"casa": "camara", "ano": "2026"},
        }],
    });

    let resp: Value = http
        .post(format!("{}/insert_texts", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("POST /insert_texts")
        .json()
        .expect("decode response");

    assert_eq!(resp["inserted"].as_u64(), Some(1));
    let result = &resp["results"][0];
    assert_eq!(result["client_id"].as_str(), Some("doc:42"));
    let vector_ids = result["vector_ids"].as_array().expect("vector_ids");
    assert_eq!(vector_ids.len(), 1);
    assert_eq!(
        vector_ids[0].as_str(),
        Some("doc:42"),
        "non-chunked Vector.id MUST equal the client-provided id"
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_texts_chunked_id_uses_parent_hash_index_pattern() {
    let http = client();
    ensure_clean_collection(&http, "phase9_chunked_id");

    // Build a >2048 char text to trigger auto-chunking.
    let long_text: String = "Lorem ipsum dolor sit amet. ".repeat(120);
    assert!(long_text.len() > 2048);

    let body = json!({
        "collection": "phase9_chunked_id",
        "texts": [{
            "id": "discurso:2023-07-06",
            "text": long_text,
            "metadata": {"parlamentar": "Jack Rocha", "casa": "camara"},
        }],
    });

    let resp: Value = http
        .post(format!("{}/insert_texts", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("POST /insert_texts")
        .json()
        .expect("decode response");

    assert_eq!(resp["inserted"].as_u64(), Some(1));
    let vector_ids = resp["results"][0]["vector_ids"]
        .as_array()
        .expect("vector_ids");
    assert!(
        vector_ids.len() > 1,
        "long text should produce more than one chunk"
    );
    for (i, vid) in vector_ids.iter().enumerate() {
        let expected = format!("discurso:2023-07-06#{}", i);
        assert_eq!(
            vid.as_str(),
            Some(expected.as_str()),
            "chunk {} should be ids `<parent>#{}`",
            i,
            i
        );
    }
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_texts_chunked_payload_is_flat_with_user_metadata_at_root() {
    let http = client();
    ensure_clean_collection(&http, "phase9_flat_payload");

    let long_text: String = "Discurso plenario com transcricao longa. ".repeat(80);
    assert!(long_text.len() > 2048);

    let body = json!({
        "collection": "phase9_flat_payload",
        "texts": [{
            "id": "discurso:flat",
            "text": long_text,
            "metadata": {
                "casa": "camara",
                "parlamentar": "Jack Rocha",
                "data": "2023-07-06",
            },
        }],
    });

    let resp: Value = http
        .post(format!("{}/insert_texts", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("POST /insert_texts")
        .json()
        .expect("decode response");
    assert_eq!(resp["inserted"].as_u64(), Some(1));

    let points = scroll_all_payloads(&http, "phase9_flat_payload");
    assert!(!points.is_empty(), "scroll should return chunked points");

    for point in &points {
        let payload = &point["payload"];
        assert_eq!(
            payload["casa"].as_str(),
            Some("camara"),
            "user field `casa` must be at payload root"
        );
        assert_eq!(
            payload["parlamentar"].as_str(),
            Some("Jack Rocha"),
            "user field `parlamentar` must be at payload root"
        );
        assert_eq!(
            payload["data"].as_str(),
            Some("2023-07-06"),
            "user field `data` must be at payload root"
        );
        assert_eq!(
            payload["parent_id"].as_str(),
            Some("discurso:flat"),
            "parent_id must equal the client-provided id"
        );
        assert!(
            payload.get("file_path").is_some(),
            "file_path must be at payload root"
        );
        assert!(
            payload.get("chunk_index").is_some(),
            "chunk_index must be at payload root"
        );
        assert!(
            payload.get("metadata").is_none(),
            "phase9 chunked payloads must NOT nest user fields under `metadata`"
        );
    }
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_texts_idempotent_re_ingest_replaces_in_place() {
    let http = client();
    ensure_clean_collection(&http, "phase9_idempotent");

    let payload = json!({
        "collection": "phase9_idempotent",
        "texts": [{
            "id": "doc:idem",
            "text": "first version of the text",
            "metadata": {"version": "1"},
        }],
    });

    // First insert.
    let _ = http
        .post(format!("{}/insert_texts", VECTORIZER_API_URL))
        .json(&payload)
        .send()
        .expect("POST /insert_texts (1)");
    assert_eq!(collection_vector_count(&http, "phase9_idempotent"), 1);

    // Re-insert the same id with different content/metadata.
    let updated = json!({
        "collection": "phase9_idempotent",
        "texts": [{
            "id": "doc:idem",
            "text": "second version of the text",
            "metadata": {"version": "2"},
        }],
    });
    let _ = http
        .post(format!("{}/insert_texts", VECTORIZER_API_URL))
        .json(&updated)
        .send()
        .expect("POST /insert_texts (2)");

    assert_eq!(
        collection_vector_count(&http, "phase9_idempotent"),
        1,
        "re-ingesting with the same client id MUST upsert in place, not duplicate"
    );

    let points = scroll_all_payloads(&http, "phase9_idempotent");
    let p = &points[0];
    assert_eq!(p["id"].as_str(), Some("doc:idem"));
    assert_eq!(
        p["payload"]["version"].as_str(),
        Some("2"),
        "the second write should be the persisted state"
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_texts_rejects_invalid_client_id() {
    let http = client();
    ensure_clean_collection(&http, "phase9_invalid_id");

    for bad in ["", "has whitespace ", " leading", "with#hash"] {
        let resp: Value = http
            .post(format!("{}/insert_texts", VECTORIZER_API_URL))
            .json(&json!({
                "collection": "phase9_invalid_id",
                "texts": [{"id": bad, "text": "ok"}],
            }))
            .send()
            .expect("POST /insert_texts")
            .json()
            .expect("decode response");

        let result = &resp["results"][0];
        assert_eq!(
            result["status"].as_str(),
            Some("error"),
            "invalid id {bad:?} must be rejected per-entry"
        );
        assert_eq!(
            result["error_type"].as_str(),
            Some("validation_error"),
            "id {bad:?} must surface as validation_error"
        );
    }
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_vectors_round_trips_with_explicit_id() {
    let http = client();
    ensure_clean_collection(&http, "phase9_insert_vectors_ok");

    // Collection above is dimension=512 — embed accordingly.
    let embedding: Vec<f32> = (0..512).map(|i| (i as f32 * 0.001).sin()).collect();

    let body = json!({
        "collection": "phase9_insert_vectors_ok",
        "vectors": [{
            "id": "vec:1",
            "embedding": embedding,
            "payload": {"author": "alice", "year": 2026},
        }],
    });

    let resp: Value = http
        .post(format!("{}/insert_vectors", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("POST /insert_vectors")
        .json()
        .expect("decode response");
    assert_eq!(resp["inserted"].as_u64(), Some(1));
    assert_eq!(resp["failed"].as_u64(), Some(0));
    let result = &resp["results"][0];
    assert_eq!(result["client_id"].as_str(), Some("vec:1"));
    assert_eq!(
        result["vector_ids"][0].as_str(),
        Some("vec:1"),
        "/insert_vectors MUST honor the client id verbatim"
    );

    let points = scroll_all_payloads(&http, "phase9_insert_vectors_ok");
    assert_eq!(points.len(), 1);
    let p = &points[0];
    assert_eq!(p["id"].as_str(), Some("vec:1"));
    assert_eq!(p["payload"]["author"].as_str(), Some("alice"));
    assert_eq!(p["payload"]["year"].as_u64(), Some(2026));
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_vectors_rejects_dimension_mismatch() {
    let http = client();
    ensure_clean_collection(&http, "phase9_insert_vectors_bad_dim");

    // Collection is dim=512 but we send a 384-vector.
    let embedding: Vec<f32> = vec![0.0; 384];

    let resp: Value = http
        .post(format!("{}/insert_vectors", VECTORIZER_API_URL))
        .json(&json!({
            "collection": "phase9_insert_vectors_bad_dim",
            "vectors": [{"embedding": embedding}],
        }))
        .send()
        .expect("POST /insert_vectors")
        .json()
        .expect("decode response");

    assert_eq!(resp["inserted"].as_u64(), Some(0));
    assert_eq!(resp["failed"].as_u64(), Some(1));
    let result = &resp["results"][0];
    assert_eq!(result["status"].as_str(), Some("error"));
    assert_eq!(result["error_type"].as_str(), Some("validation_error"));
    let msg = result["error"].as_str().unwrap_or("");
    assert!(
        msg.contains("384") && msg.contains("512"),
        "error message should mention both lengths, got: {msg:?}"
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn insert_vectors_falls_back_to_metadata_when_payload_absent() {
    let http = client();
    ensure_clean_collection(&http, "phase9_insert_vectors_metadata");

    let embedding: Vec<f32> = vec![0.1; 512];
    let body = json!({
        "collection": "phase9_insert_vectors_metadata",
        "vectors": [{
            "id": "fallback:1",
            "embedding": embedding,
            "metadata": {"k1": "v1", "k2": "v2"},
        }],
    });

    let resp: Value = http
        .post(format!("{}/insert_vectors", VECTORIZER_API_URL))
        .json(&body)
        .send()
        .expect("POST /insert_vectors")
        .json()
        .expect("decode response");
    assert_eq!(resp["inserted"].as_u64(), Some(1));

    let points = scroll_all_payloads(&http, "phase9_insert_vectors_metadata");
    let p = &points[0];
    assert_eq!(p["id"].as_str(), Some("fallback:1"));
    assert_eq!(p["payload"]["k1"].as_str(), Some("v1"));
    assert_eq!(p["payload"]["k2"].as_str(), Some("v2"));
}
