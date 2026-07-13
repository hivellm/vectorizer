//! In-process migration of the `#[ignore]`d ECC payload-encryption suites
//! (phase39 §1.3) onto the shared harness in `tests/common/mod.rs`.
//!
//! Sources:
//! - `crates/vectorizer/tests/api/rest/encryption.rs` (3 ignored tests)
//! - `crates/vectorizer/tests/api/rest/encryption_complete.rs` (5 ignored
//!   tests, reason: "Flaky on CI - passes locally but fails on macOS CI")
//!
//! Both source files call `vectorizer::db::VectorStore` and
//! `vectorizer::security::payload_encryption::encrypt_payload` directly —
//! none of them actually dispatched an HTTP request despite several being
//! named `test_rest_*` / `test_qdrant_*`. This suite replaces every one of
//! them with a real request through the production router
//! (`tower::ServiceExt::oneshot`, no `#[ignore]`, no live server), so the
//! REST/Qdrant-compatible encryption paths (`POST /insert`, `POST
//! /insert_vectors`, `PUT /qdrant/collections/{name}/points`, `POST
//! /files/upload`) are what's actually exercised now. Every collection is
//! created at dimension 512 (the source tests used 128 for the raw-vector
//! cases) because `POST /collections` validates the requested dimension
//! against the harness's fixed BM25 provider dimension even when the
//! vectors being inserted are pre-computed and never touch the embedding
//! pipeline — see `rest_handlers/collections.rs`'s
//! `provider_dimension_mismatch` check.
//!
//! ## Excluded (2 of the original 8)
//!
//! `encryption.rs::test_encryption_required_validation` and
//! `encryption_complete.rs::test_encryption_required_enforcement` both
//! construct a collection with `EncryptionConfig { required: true, .. }`
//! (or `allow_mixed: false`) directly via `VectorStore::create_collection`
//! and assert that the core engine's validation in
//! `crates/vectorizer/src/db/collection/data.rs` (`insert`, ~line 30-52)
//! rejects unencrypted payloads.
//!
//! That collection-level `EncryptionConfig` is **not reachable from any
//! client-facing API** in this codebase: every call site that builds a
//! `CollectionConfig` from a request hardcodes `encryption: None` —
//! confirmed by inspection of `rest_handlers/collections.rs`,
//! `rest_handlers/insert.rs`, `rest_handlers/backups.rs`,
//! `qdrant/handlers.rs`, `server/mcp/handlers.rs`,
//! `api/graphql/schema/mutation.rs`, and `protocol/rpc/dispatch.rs`. There
//! is no REST, GraphQL, MCP, or Qdrant-compat request field that sets
//! `required` or `allow_mixed`. This is not a HiveHub dependency (the
//! encryption flow itself is pure local ECC/AES, no hub involved) — it's
//! that the `required`/`allow_mixed` enforcement branch is dead code from
//! every client's perspective, so it cannot be driven through this
//! in-process HTTP harness (or any other client-facing surface) at all.
//! Both tests' *positive* assertion (an encrypted payload is accepted) is
//! still covered by every other test below; only the *required-rejects-
//! plaintext* branch is untested here. See the final task report for this
//! finding — it is flagged as a product gap, not fixed in this test-only
//! change.
//!
//! ## Qdrant upsert is fire-and-forget
//!
//! `qdrant::vector_handlers::upsert_points` always inserts on a spawned
//! background task and returns `Acknowledged` immediately — the
//! `wait: bool` field on `QdrantUpsertPointsRequest` is parsed but never
//! read by the handler, so `wait: true` has no effect. Tests that go
//! through the Qdrant upsert path use [`wait_for_vector_count`] to poll
//! `GET /collections/{name}` (bounded, no unbounded loop) instead of
//! asserting immediately after the `PUT` response, to avoid a real race
//! against that background task.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use common::{MultipartField, TestApp};
use p256::SecretKey;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use serde_json::{Value, json};

/// Build a fresh ECC (P-256) key pair and return its base64-encoded
/// uncompressed public key, in the format `encrypt_payload` /
/// `POST .../public_key` fields expect.
fn create_test_public_key() -> String {
    let secret_key = SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let public_key = secret_key.public_key();
    let encoded_point = public_key.to_encoded_point(false);
    BASE64.encode(encoded_point.as_bytes())
}

/// Create `name` as a cosine collection of `dimension` via the real
/// `POST /collections` handler.
async fn create_collection(app: &TestApp, name: &str, dimension: usize) {
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": dimension, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create {name} status {status}: {resp}");
}

/// `GET /collections/{name}` and pull out `vector_count`.
async fn vector_count(app: &TestApp, name: &str) -> u64 {
    let (status, resp) = app.get(&format!("/collections/{name}")).await;
    assert!(
        status.is_success(),
        "get collection status {status}: {resp}"
    );
    resp["vector_count"].as_u64().expect("vector_count field")
}

/// Poll `GET /collections/{name}` until `vector_count` reaches `expected`
/// or a bounded number of attempts is exhausted. Needed because
/// `PUT /qdrant/collections/{name}/points` inserts on a background task
/// and returns `Acknowledged` before the insert completes (see module
/// doc comment) — this replaces an immediate, potentially racy assertion
/// with a short, bounded wait instead of an unbounded loop.
async fn wait_for_vector_count(app: &TestApp, name: &str, expected: u64) {
    const MAX_ATTEMPTS: u32 = 100;
    const STEP: Duration = Duration::from_millis(20);

    let mut last_seen = 0;
    for _ in 0..MAX_ATTEMPTS {
        last_seen = vector_count(app, name).await;
        if last_seen >= expected {
            return;
        }
        tokio::time::sleep(STEP).await;
    }
    panic!(
        "collection '{name}' never reached vector_count {expected} \
         (last observed: {last_seen}) after {} attempts",
        MAX_ATTEMPTS
    );
}

/// `GET /collections/{name}/vectors?limit=50` and return the `vectors`
/// array. 50 is comfortably above every vector count this suite inserts.
async fn list_vectors(app: &TestApp, name: &str) -> Vec<Value> {
    let (status, resp) = app
        .get(&format!("/collections/{name}/vectors?limit=50"))
        .await;
    assert!(status.is_success(), "list_vectors status {status}: {resp}");
    resp["vectors"].as_array().cloned().expect("vectors array")
}

/// Find the vector with `id` in a `list_vectors` response and return its
/// `payload` field.
fn payload_of<'a>(vectors: &'a [Value], id: &str) -> &'a Value {
    vectors
        .iter()
        .find(|v| v["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("vector '{id}' not found in listing: {vectors:?}"))
        .get("payload")
        .unwrap_or_else(|| panic!("vector '{id}' has no payload field"))
}

/// Assert `payload` has the full `EncryptedPayload` shape produced by
/// `vectorizer::security::payload_encryption::encrypt_payload` (mirrors
/// the structural assertions in the original `encryption.rs` /
/// `encryption_complete.rs` tests).
fn assert_encrypted_payload_shape(payload: &Value) {
    assert_eq!(
        payload["version"].as_u64(),
        Some(1),
        "payload should carry version 1: {payload}"
    );
    assert_eq!(
        payload["algorithm"].as_str(),
        Some("ECC-P256-AES256GCM"),
        "payload should carry the ECC-P256-AES256GCM algorithm tag: {payload}"
    );
    for field in ["nonce", "tag", "encrypted_data", "ephemeral_public_key"] {
        let value = payload[field]
            .as_str()
            .unwrap_or_else(|| panic!("payload.{field} should be a non-empty string: {payload}"));
        assert!(
            !value.is_empty(),
            "payload.{field} should not be empty: {payload}"
        );
        assert!(
            BASE64.decode(value).is_ok(),
            "payload.{field} should be valid base64: {payload}"
        );
    }
}

/// Assert `payload` is a plain, unencrypted JSON object (i.e. it does
/// NOT carry the encrypted-payload marker fields).
fn assert_plaintext_payload(payload: &Value) {
    assert!(
        payload.get("algorithm").is_none(),
        "payload should not carry an encryption algorithm marker: {payload}"
    );
}

// ─── encryption.rs migrations ───────────────────────────────────────────────

/// Migrated from `encryption.rs::test_encrypted_payload_insertion_via_collection`
/// (`#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]`).
/// Original called `VectorStore::insert` directly; this version drives the
/// same encrypt-then-store flow through the real `POST /insert_vectors`
/// handler, which accepts a raw pre-computed embedding plus an explicit
/// `id` and per-entry `public_key` — the closest REST equivalent to
/// inserting a hand-built `Vector` with an encrypted `Payload`.
#[tokio::test]
async fn insert_vectors_with_public_key_stores_encrypted_payload() {
    let app = TestApp::new().await;
    let collection = "rest_encryption_insert_vectors_encrypted";
    // 512 matches the harness's fixed BM25 provider dimension (see
    // `common/mod.rs`) — `POST /collections` rejects any other dimension
    // for a bm25-backed collection even though this test's vectors are
    // pre-computed and never touch the embedding pipeline.
    create_collection(&app, collection, 512).await;

    let public_key = create_test_public_key();
    let embedding: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0).collect();
    let payload = json!({
        "user_id": "12345",
        "sensitive_data": "This is confidential information",
        "metadata": {
            "category": "financial",
            "timestamp": "2024-01-15T10:30:00Z"
        }
    });

    let (status, resp) = app
        .post_json(
            "/insert_vectors",
            json!({
                "collection": collection,
                "vectors": [
                    {
                        "id": "encrypted_vector_1",
                        "embedding": embedding,
                        "payload": payload,
                        "public_key": public_key,
                    }
                ],
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

    let vectors = list_vectors(&app, collection).await;
    let stored_payload = payload_of(&vectors, "encrypted_vector_1");
    assert_encrypted_payload_shape(stored_payload);
}

/// Migrated from `encryption.rs::test_mixed_encrypted_and_unencrypted_payloads`
/// (`#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]`).
/// The original collection set `allow_mixed: true`, but that flag is
/// unreachable via REST (see module doc comment) — REST's hardcoded
/// `encryption: None` already imposes no mixing restriction at all, so
/// the coexistence behavior under test still holds unmodified.
#[tokio::test]
async fn insert_vectors_mixed_encrypted_and_plaintext_payloads_coexist() {
    let app = TestApp::new().await;
    let collection = "rest_encryption_insert_vectors_mixed";
    create_collection(&app, collection, 512).await;

    let public_key = create_test_public_key();
    let (status, resp) = app
        .post_json(
            "/insert_vectors",
            json!({
                "collection": collection,
                "vectors": [
                    {
                        "id": "vec1",
                        "embedding": vec![0.1_f32; 512],
                        "payload": {"data": "encrypted"},
                        "public_key": public_key,
                    },
                    {
                        "id": "vec2",
                        "embedding": vec![0.2_f32; 512],
                        "payload": {"data": "unencrypted"},
                    }
                ],
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "insert_vectors status {status}: {resp}"
    );
    assert_eq!(
        resp["inserted"].as_u64(),
        Some(2),
        "insert_vectors resp: {resp}"
    );

    let vectors = list_vectors(&app, collection).await;
    assert_encrypted_payload_shape(payload_of(&vectors, "vec1"));
    let plaintext = payload_of(&vectors, "vec2");
    assert_plaintext_payload(plaintext);
    assert_eq!(plaintext["data"].as_str(), Some("unencrypted"));
}

// ─── encryption_complete.rs migrations ──────────────────────────────────────

/// Migrated from `encryption_complete.rs::test_rest_insert_text_with_encryption`
/// (`#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]`).
/// The original never issued an HTTP request despite its name (it called
/// `EmbeddingManager::embed` + `VectorStore::insert` directly); this
/// version drives the same flow through the real `POST /insert` handler.
#[tokio::test]
async fn insert_text_with_public_key_encrypts_metadata_payload() {
    let app = TestApp::new().await;
    let collection = "rest_encryption_insert_text";
    create_collection(&app, collection, 512).await;

    let public_key = create_test_public_key();
    let (status, resp) = app
        .post_json(
            "/insert",
            json!({
                "collection": collection,
                "text": "This is sensitive confidential data",
                "metadata": {
                    "category": "financial",
                    "user_id": "user123"
                },
                "public_key": public_key,
                "auto_chunk": false,
            }),
        )
        .await;
    assert!(status.is_success(), "/insert status {status}: {resp}");
    assert_eq!(
        resp["vectors_created"].as_u64(),
        Some(1),
        "/insert resp: {resp}"
    );

    let vectors = list_vectors(&app, collection).await;
    assert_eq!(
        vectors.len(),
        1,
        "expected exactly 1 stored vector: {vectors:?}"
    );
    assert_encrypted_payload_shape(&vectors[0]["payload"]);
}

/// Migrated from `encryption_complete.rs::test_qdrant_upsert_with_encryption`
/// (`#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]`).
/// Drives the real Qdrant-compatible `PUT /qdrant/collections/{name}/points`
/// handler with a `public_key`, then polls for the background insert to
/// land (see [`wait_for_vector_count`]) before checking the stored payload.
#[tokio::test]
async fn qdrant_upsert_with_public_key_encrypts_payload() {
    let app = TestApp::new().await;
    let collection = "rest_encryption_qdrant_upsert";
    create_collection(&app, collection, 512).await;

    let public_key = create_test_public_key();
    let vector_data: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0).collect();
    let payload = json!({
        "document": "sensitive contract",
        "amount": 1000000,
        "classification": "confidential"
    });

    let (status, resp) = app
        .put_json(
            &format!("/qdrant/collections/{collection}/points"),
            json!({
                "points": [
                    {
                        "id": "qdrant_vec_1",
                        "vector": vector_data,
                        "payload": payload,
                        "public_key": public_key,
                    }
                ],
                "wait": true,
            }),
        )
        .await;
    assert!(status.is_success(), "qdrant upsert status {status}: {resp}");

    wait_for_vector_count(&app, collection, 1).await;

    let vectors = list_vectors(&app, collection).await;
    assert_encrypted_payload_shape(payload_of(&vectors, "qdrant_vec_1"));
}

/// Migrated from `encryption_complete.rs::test_qdrant_upsert_mixed_encryption`
/// (`#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]`).
/// The original collection set `allow_mixed: true`, which is unreachable
/// via REST (see module doc comment); REST's hardcoded `encryption: None`
/// imposes no restriction either way, so the mixed-payload coexistence
/// behavior under test is unaffected.
#[tokio::test]
async fn qdrant_upsert_mixed_encrypted_and_plaintext_points_coexist() {
    let app = TestApp::new().await;
    let collection = "rest_encryption_qdrant_mixed";
    create_collection(&app, collection, 512).await;

    let public_key = create_test_public_key();
    let (status, resp) = app
        .put_json(
            &format!("/qdrant/collections/{collection}/points"),
            json!({
                "points": [
                    {
                        "id": "vec_encrypted",
                        "vector": vec![0.1_f32; 512],
                        "payload": {"type": "encrypted", "data": "secret"},
                        "public_key": public_key,
                    },
                    {
                        "id": "vec_unencrypted",
                        "vector": vec![0.2_f32; 512],
                        "payload": {"type": "public", "data": "open"},
                    }
                ],
                "wait": true,
            }),
        )
        .await;
    assert!(status.is_success(), "qdrant upsert status {status}: {resp}");

    wait_for_vector_count(&app, collection, 2).await;

    let vectors = list_vectors(&app, collection).await;
    assert_encrypted_payload_shape(payload_of(&vectors, "vec_encrypted"));
    let plaintext = payload_of(&vectors, "vec_unencrypted");
    assert_plaintext_payload(plaintext);
    assert_eq!(plaintext["type"].as_str(), Some("public"));
}

/// Migrated from `encryption_complete.rs::test_file_upload_simulation_with_encryption`
/// (`#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]`).
/// The original built chunk vectors by hand with dummy embeddings and
/// never touched the upload pipeline; this version drives the real
/// `POST /files/upload` multipart handler (chunking + real BM25 embedding)
/// with a `public_key` field, mirroring `rest_file_upload.rs`'s pattern.
#[tokio::test]
async fn file_upload_with_public_key_encrypts_every_chunk_payload() {
    let app = TestApp::new().await;
    let collection = "rest_encryption_file_upload";
    let public_key = create_test_public_key();

    let mut body = String::new();
    for i in 0..3 {
        body.push_str(&format!(
            "## Section {i}\n\n{}\n\n",
            "Chunk 1: Introduction to cryptography and zero-knowledge architecture. ".repeat(40)
        ));
    }

    let fields = vec![
        MultipartField::file("file", "crypto.md", "text/markdown", body.into_bytes()),
        MultipartField::text("collection_name", collection),
        MultipartField::text("public_key", public_key),
    ];
    let (status, resp) = app.post_multipart("/files/upload", &fields).await;
    assert!(status.is_success(), "upload status {status}: {resp}");
    assert_eq!(resp["success"].as_bool(), Some(true), "upload resp: {resp}");

    let chunks_created = resp["chunks_created"].as_u64().expect("chunks_created");
    assert!(chunks_created >= 1, "expected >=1 chunk, got {resp}");
    let vectors_created = resp["vectors_created"].as_u64().expect("vectors_created");
    assert!(vectors_created >= 1, "expected >=1 vector, got {resp}");

    let vectors = list_vectors(&app, collection).await;
    assert_eq!(
        vectors.len() as u64,
        vectors_created,
        "expected every uploaded chunk vector to be listed: {vectors:?}"
    );
    for vector in &vectors {
        assert_encrypted_payload_shape(&vector["payload"]);
    }
}
