//! In-process migration of
//! `crates/vectorizer/tests/api/rest/batch_insert_real.rs` (phase39 §1.2)
//! onto the shared harness in `tests/common/mod.rs`.
//!
//! Same assertions as the live-server suite, but dispatched through the
//! real production router via `tower::ServiceExt::oneshot` instead of
//! `reqwest` against `127.0.0.1:15002` — no `#[ignore]`, runs in CI.
//!
//! Covers the phase8_fix-batch-insert-endpoints write path (`POST
//! /batch_insert` and `POST /insert_texts` used to return success without
//! persisting) plus phase9 (client-id honoring, flat chunk payloads, and
//! `POST /insert_vectors`).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use common::TestApp;
use serde_json::{Value, json};

/// Delete-then-create `name` (idempotent) as a 512-dim cosine collection,
/// mirroring the live suite's `ensure_clean_collection`.
async fn ensure_clean_collection(app: &TestApp, name: &str) {
    let _ = app.delete(&format!("/collections/{name}")).await;
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create status {status}: {resp}");
}

/// `GET /collections/{name}` and pull out `vector_count`.
async fn collection_vector_count(app: &TestApp, name: &str) -> u64 {
    let (status, meta) = app.get(&format!("/collections/{name}")).await;
    assert!(
        status.is_success(),
        "get collection status {status}: {meta}"
    );
    meta["vector_count"].as_u64().expect("vector_count field")
}

/// Scroll the Qdrant-compat endpoint to fetch every point's payload.
async fn scroll_all_payloads(app: &TestApp, name: &str) -> Vec<Value> {
    let (status, resp) = app
        .post_json(
            &format!("/qdrant/collections/{name}/points/scroll"),
            json!({"limit": 100, "with_payload": true, "with_vector": false}),
        )
        .await;
    assert!(status.is_success(), "scroll status {status}: {resp}");
    resp["result"]["points"]
        .as_array()
        .cloned()
        .or_else(|| resp["points"].as_array().cloned())
        .unwrap_or_default()
}

#[tokio::test]
async fn batch_insert_persists_all_entries() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "batch_insert_real_ok").await;

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

    let (status, resp) = app.post_json("/batch_insert", body).await;
    assert!(
        status.is_success(),
        "POST /batch_insert status {status}: {resp}"
    );

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
        collection_vector_count(&app, "batch_insert_real_ok").await,
        10,
        "collection should expose all 10 vectors"
    );
}

#[tokio::test]
async fn insert_texts_alias_persists_all_entries() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "insert_texts_alias_ok").await;

    let body = json!({
        "collection": "insert_texts_alias_ok",
        "texts": [
            {"text": "alias alpha"},
            {"text": "alias beta"},
            {"text": "alias gamma"},
        ],
    });

    let (status, resp) = app.post_json("/insert_texts", body).await;
    assert!(
        status.is_success(),
        "POST /insert_texts status {status}: {resp}"
    );

    assert_eq!(resp["inserted"].as_u64(), Some(3));
    assert_eq!(resp["failed"].as_u64(), Some(0));
    assert_eq!(resp["count"].as_u64(), Some(3));
    assert_eq!(
        collection_vector_count(&app, "insert_texts_alias_ok").await,
        3,
        "alias endpoint should persist same as batch_insert"
    );
}

#[tokio::test]
async fn batch_insert_partial_failure_is_reported_per_item() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "batch_insert_real_partial").await;

    let body = json!({
        "collection": "batch_insert_real_partial",
        "texts": [
            {"text": "good one"},
            {"metadata": {"missing_text": "yes"}},
            {"text": "good two"},
        ],
    });

    let (status, resp) = app.post_json("/batch_insert", body).await;
    assert!(
        status.is_success(),
        "POST /batch_insert status {status}: {resp}"
    );

    assert_eq!(resp["inserted"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(1));
    let results = resp["results"].as_array().expect("results array");
    assert_eq!(results[0]["status"].as_str(), Some("ok"));
    assert_eq!(results[1]["status"].as_str(), Some("error"));
    assert_eq!(results[2]["status"].as_str(), Some("ok"));

    assert_eq!(
        collection_vector_count(&app, "batch_insert_real_partial").await,
        2,
        "only the 2 valid entries should be persisted"
    );
}

#[tokio::test]
async fn batch_insert_rejects_missing_collection() {
    let app = TestApp::new().await;

    let (status, body) = app
        .post_json("/batch_insert", json!({"texts": [{"text": "orphan"}]}))
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

#[tokio::test]
async fn batch_insert_rejects_empty_texts_array() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "batch_insert_real_empty").await;

    let (status, body) = app
        .post_json(
            "/batch_insert",
            json!({"collection": "batch_insert_real_empty", "texts": []}),
        )
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ---------------------------------------------------------------------------
// phase9 — client-id honoring + flat chunk payload + /insert_vectors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_texts_honors_client_id_for_short_text() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "phase9_client_id_short").await;

    let body = json!({
        "collection": "phase9_client_id_short",
        "texts": [{
            "id": "doc:42",
            "text": "Texto curto preserva o id cliente.",
            "metadata": {"casa": "camara", "ano": "2026"},
        }],
    });

    let (status, resp) = app.post_json("/insert_texts", body).await;
    assert!(
        status.is_success(),
        "POST /insert_texts status {status}: {resp}"
    );

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

#[tokio::test]
async fn insert_texts_chunked_id_uses_parent_hash_index_pattern() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "phase9_chunked_id").await;

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

    let (status, resp) = app.post_json("/insert_texts", body).await;
    assert!(
        status.is_success(),
        "POST /insert_texts status {status}: {resp}"
    );

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

#[tokio::test]
async fn insert_texts_chunked_payload_is_flat_with_user_metadata_at_root() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "phase9_flat_payload").await;

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

    let (status, resp) = app.post_json("/insert_texts", body).await;
    assert!(
        status.is_success(),
        "POST /insert_texts status {status}: {resp}"
    );
    assert_eq!(resp["inserted"].as_u64(), Some(1));

    let points = scroll_all_payloads(&app, "phase9_flat_payload").await;
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

#[tokio::test]
async fn insert_texts_idempotent_re_ingest_replaces_in_place() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "phase9_idempotent").await;

    let payload = json!({
        "collection": "phase9_idempotent",
        "texts": [{
            "id": "doc:idem",
            "text": "first version of the text",
            "metadata": {"version": "1"},
        }],
    });

    // First insert.
    let (status, _) = app.post_json("/insert_texts", payload).await;
    assert!(
        status.is_success(),
        "POST /insert_texts (1) status {status}"
    );
    assert_eq!(collection_vector_count(&app, "phase9_idempotent").await, 1);

    // Re-insert the same id with different content/metadata.
    let updated = json!({
        "collection": "phase9_idempotent",
        "texts": [{
            "id": "doc:idem",
            "text": "second version of the text",
            "metadata": {"version": "2"},
        }],
    });
    let (status, _) = app.post_json("/insert_texts", updated).await;
    assert!(
        status.is_success(),
        "POST /insert_texts (2) status {status}"
    );

    assert_eq!(
        collection_vector_count(&app, "phase9_idempotent").await,
        1,
        "re-ingesting with the same client id MUST upsert in place, not duplicate"
    );

    let points = scroll_all_payloads(&app, "phase9_idempotent").await;
    let p = &points[0];
    assert_eq!(p["id"].as_str(), Some("doc:idem"));
    assert_eq!(
        p["payload"]["version"].as_str(),
        Some("2"),
        "the second write should be the persisted state"
    );
}

#[tokio::test]
async fn insert_texts_rejects_invalid_client_id() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "phase9_invalid_id").await;

    for bad in ["", "has whitespace ", " leading", "with#hash"] {
        let (status, resp) = app
            .post_json(
                "/insert_texts",
                json!({
                    "collection": "phase9_invalid_id",
                    "texts": [{"id": bad, "text": "ok"}],
                }),
            )
            .await;
        assert!(
            status.is_success(),
            "POST /insert_texts status {status}: {resp}"
        );

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

#[tokio::test]
async fn insert_vectors_round_trips_with_explicit_id() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "phase9_insert_vectors_ok").await;

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

    let (status, resp) = app.post_json("/insert_vectors", body).await;
    assert!(
        status.is_success(),
        "POST /insert_vectors status {status}: {resp}"
    );
    assert_eq!(resp["inserted"].as_u64(), Some(1));
    assert_eq!(resp["failed"].as_u64(), Some(0));
    let result = &resp["results"][0];
    assert_eq!(result["client_id"].as_str(), Some("vec:1"));
    assert_eq!(
        result["vector_ids"][0].as_str(),
        Some("vec:1"),
        "/insert_vectors MUST honor the client id verbatim"
    );

    let points = scroll_all_payloads(&app, "phase9_insert_vectors_ok").await;
    assert_eq!(points.len(), 1);
    let p = &points[0];
    assert_eq!(p["id"].as_str(), Some("vec:1"));
    assert_eq!(p["payload"]["author"].as_str(), Some("alice"));
    assert_eq!(p["payload"]["year"].as_u64(), Some(2026));
}

#[tokio::test]
async fn insert_vectors_rejects_dimension_mismatch() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "phase9_insert_vectors_bad_dim").await;

    // Collection is dim=512 but we send a 384-vector.
    let embedding: Vec<f32> = vec![0.0; 384];

    let (status, resp) = app
        .post_json(
            "/insert_vectors",
            json!({
                "collection": "phase9_insert_vectors_bad_dim",
                "vectors": [{"embedding": embedding}],
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "POST /insert_vectors status {status}: {resp}"
    );

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

#[tokio::test]
async fn insert_vectors_falls_back_to_metadata_when_payload_absent() {
    let app = TestApp::new().await;
    ensure_clean_collection(&app, "phase9_insert_vectors_metadata").await;

    let embedding: Vec<f32> = vec![0.1; 512];
    let body = json!({
        "collection": "phase9_insert_vectors_metadata",
        "vectors": [{
            "id": "fallback:1",
            "embedding": embedding,
            "metadata": {"k1": "v1", "k2": "v2"},
        }],
    });

    let (status, resp) = app.post_json("/insert_vectors", body).await;
    assert!(
        status.is_success(),
        "POST /insert_vectors status {status}: {resp}"
    );
    assert_eq!(resp["inserted"].as_u64(), Some(1));

    let points = scroll_all_payloads(&app, "phase9_insert_vectors_metadata").await;
    let p = &points[0];
    assert_eq!(p["id"].as_str(), Some("fallback:1"));
    assert_eq!(p["payload"]["k1"].as_str(), Some("v1"));
    assert_eq!(p["payload"]["k2"].as_str(), Some("v2"));
}
