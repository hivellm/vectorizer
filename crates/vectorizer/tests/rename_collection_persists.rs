//! Integration tests for `VectorStore::rename_collection` (phase22).
//!
//! Regression coverage for the no-op bug confirmed against `vectorizer:3.3.0`
//! on 2026-05-04: `POST /collections/{name}/rename` returned 200 OK but the
//! collection name was unchanged everywhere it mattered. Root cause: the
//! `Collection.name` field embedded in the value was never updated when the
//! `VectorStore` HashMap key was swapped, and there was no `RenameCollection`
//! WAL op so replicas never observed the rename.
//!
//! These tests pin the contract from the SDKs' point of view — every assertion
//! mirrors a live-server scenario the SDKs hit against `/collections/{name}/rename`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::db::VectorStore;
use vectorizer::error::VectorizerError;
use vectorizer::models::{CollectionConfig, DistanceMetric, Payload, Vector};

fn cfg() -> CollectionConfig {
    CollectionConfig {
        dimension: 4,
        metric: DistanceMetric::Cosine,
        ..Default::default()
    }
}

fn vector(id: &str) -> Vector {
    Vector {
        id: id.to_string(),
        data: vec![0.1, 0.2, 0.3, 0.4],
        payload: Some(Payload {
            data: serde_json::json!({"k": id}),
        }),
        sparse: None,
        document_id: None,
    }
}

// ---------------------------------------------------------------------------
// Test 1 — happy path: rename mutates the canonical name end-to-end
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rename_persists_in_memory() {
    let store = VectorStore::new();
    store.create_collection("c1", cfg()).expect("create c1");
    store.insert("c1", vec![vector("v1")]).expect("insert v1");

    store
        .rename_collection("c1", "c2")
        .expect("rename c1 -> c2");

    // The new name must be present in list_collections.
    let names = store.list_collections();
    assert!(
        names.contains(&"c2".to_string()),
        "list must contain new name; got {names:?}"
    );

    // The collection looked up by the new name must report the new name on
    // its metadata (this is the field the previous bug failed to update).
    let info = store
        .get_collection_metadata("c2")
        .expect("metadata for c2");
    assert_eq!(info.name, "c2", "Collection.name must reflect the new name");

    // Vectors must be intact under the new name. The Cosine metric
    // normalizes data on insert so we only check id + dimension here.
    let v = store.get_vector("c2", "v1").expect("v1 lookup under c2");
    assert_eq!(v.id, "v1");
    assert_eq!(v.data.len(), 4);
}

// ---------------------------------------------------------------------------
// Test 2 — collision: rename to an existing name returns CollectionAlreadyExists
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rename_collision_returns_already_exists_error() {
    let store = VectorStore::new();
    store.create_collection("c1", cfg()).expect("create c1");
    store.create_collection("c2", cfg()).expect("create c2");

    let err = store
        .rename_collection("c1", "c2")
        .expect_err("must fail when destination already exists");

    match err {
        VectorizerError::CollectionAlreadyExists(name) => assert_eq!(name, "c2"),
        other => panic!("expected CollectionAlreadyExists, got {other:?}"),
    }

    // Both collections still exist under their original names.
    let names = store.list_collections();
    assert!(names.contains(&"c1".to_string()));
    assert!(names.contains(&"c2".to_string()));
}

// ---------------------------------------------------------------------------
// Test 3 — self-rename: rename old to same name returns InvalidConfiguration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rename_self_returns_invalid_configuration() {
    let store = VectorStore::new();
    store.create_collection("c1", cfg()).expect("create c1");

    let err = store
        .rename_collection("c1", "c1")
        .expect_err("source equals destination must be rejected");

    assert!(
        matches!(err, VectorizerError::InvalidConfiguration { .. }),
        "expected InvalidConfiguration, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 4 — invalid name: empty / contains '/' is rejected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rename_invalid_name_returns_invalid_configuration() {
    let store = VectorStore::new();
    store.create_collection("c1", cfg()).expect("create c1");

    // Empty new name.
    let err = store
        .rename_collection("c1", "")
        .expect_err("empty new name");
    assert!(matches!(err, VectorizerError::InvalidConfiguration { .. }));

    // New name contains '/'.
    let err = store
        .rename_collection("c1", "a/b")
        .expect_err("new name with '/' must be rejected");
    assert!(matches!(err, VectorizerError::InvalidConfiguration { .. }));

    // Original collection unchanged.
    let names = store.list_collections();
    assert!(names.contains(&"c1".to_string()));
}

// ---------------------------------------------------------------------------
// Test 5 — old name resolves via grace-window alias post-rename
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rename_keeps_old_name_as_grace_alias() {
    let store = VectorStore::new();
    store.create_collection("c1", cfg()).expect("create c1");
    store.insert("c1", vec![vector("v1")]).expect("insert v1");

    store
        .rename_collection("c1", "c2")
        .expect("rename c1 -> c2");

    // The old name must resolve transparently to the new collection so
    // existing SDK callers don't see a 404 mid-deploy.
    let v = store
        .get_vector("c1", "v1")
        .expect("old name must still resolve via grace-window alias");
    assert_eq!(v.id, "v1");
}
