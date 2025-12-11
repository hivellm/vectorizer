//! GraphQL encryption tests
//!
//! Tests for optional ECC-AES payload encryption via GraphQL API

use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use p256::SecretKey;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use vectorizer::api::graphql::create_schema;
use vectorizer::db::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::models::CollectionConfig;

/// Helper to create a test ECC key pair
fn create_test_keypair() -> (SecretKey, String) {
    let secret_key = SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let public_key = secret_key.public_key();
    let public_key_encoded = public_key.to_encoded_point(false);
    let public_key_base64 = BASE64.encode(public_key_encoded.as_bytes());
    (secret_key, public_key_base64)
}

/// Test upsert_vector mutation with encryption
#[tokio::test]
async fn test_graphql_upsert_vector_with_encryption() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());
    let start_time = std::time::Instant::now();

    // Create schema
    let schema = create_schema(store.clone(), embedding_manager.clone(), start_time);

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        ..Default::default()
    };
    store
        .create_collection("test_graphql_encrypted", config)
        .unwrap();

    // Generate test keypair
    let (_secret_key, public_key_base64) = create_test_keypair();

    // GraphQL mutation with encryption
    let query = r"
        mutation($collection: String!, $input: UpsertVectorInput!) {
            upsertVector(collection: $collection, input: $input) {
                id
                payload
            }
        }
    ";

    let variables = serde_json::json!({
        "collection": "test_graphql_encrypted",
        "input": {
            "id": "vec1",
            "data": [1.0, 2.0, 3.0],
            "payload": {
                "content": "sensitive data",
                "category": "confidential"
            },
            "publicKey": public_key_base64
        }
    });

    let request = async_graphql::Request::new(query)
        .variables(async_graphql::Variables::from_json(variables));
    let response = schema.execute(request).await;

    assert!(
        response.errors.is_empty(),
        "GraphQL errors: {:?}",
        response.errors
    );

    // Verify vector was inserted with encrypted payload
    let vector = store.get_vector("test_graphql_encrypted", "vec1").unwrap();
    assert!(vector.payload.is_some());
    let payload = vector.payload.unwrap();
    assert!(payload.is_encrypted(), "Payload should be encrypted");
}

/// Test upsert_vector mutation without encryption (backward compatibility)
#[tokio::test]
async fn test_graphql_upsert_vector_without_encryption() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());
    let start_time = std::time::Instant::now();

    let schema = create_schema(store.clone(), embedding_manager, start_time);

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        ..Default::default()
    };
    store
        .create_collection("test_graphql_unencrypted", config)
        .unwrap();

    let query = r"
        mutation($collection: String!, $input: UpsertVectorInput!) {
            upsertVector(collection: $collection, input: $input) {
                id
                payload
            }
        }
    ";

    let variables = serde_json::json!({
        "collection": "test_graphql_unencrypted",
        "input": {
            "id": "vec1",
            "data": [1.0, 2.0, 3.0],
            "payload": {
                "content": "public data"
            }
        }
    });

    let request = async_graphql::Request::new(query)
        .variables(async_graphql::Variables::from_json(variables));
    let response = schema.execute(request).await;

    assert!(
        response.errors.is_empty(),
        "GraphQL errors: {:?}",
        response.errors
    );

    // Verify vector was inserted WITHOUT encryption
    let vector = store
        .get_vector("test_graphql_unencrypted", "vec1")
        .unwrap();
    assert!(vector.payload.is_some());
    let payload = vector.payload.unwrap();
    assert!(!payload.is_encrypted(), "Payload should NOT be encrypted");
}

/// Test upsert_vectors mutation with encryption
#[tokio::test]
async fn test_graphql_upsert_vectors_with_encryption() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());
    let start_time = std::time::Instant::now();

    let schema = create_schema(store.clone(), embedding_manager, start_time);

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        ..Default::default()
    };
    store
        .create_collection("test_graphql_batch_encrypted", config)
        .unwrap();

    let (_secret_key, public_key_base64) = create_test_keypair();

    let query = r"
        mutation($input: UpsertVectorsInput!) {
            upsertVectors(input: $input) {
                success
                affectedCount
            }
        }
    ";

    let variables = serde_json::json!({
        "input": {
            "collection": "test_graphql_batch_encrypted",
            "publicKey": public_key_base64,
            "vectors": [
                {
                    "id": "vec1",
                    "data": [1.0, 2.0, 3.0],
                    "payload": {"content": "secret 1"}
                },
                {
                    "id": "vec2",
                    "data": [4.0, 5.0, 6.0],
                    "payload": {"content": "secret 2"}
                }
            ]
        }
    });

    let request = async_graphql::Request::new(query)
        .variables(async_graphql::Variables::from_json(variables));
    let response = schema.execute(request).await;

    assert!(
        response.errors.is_empty(),
        "GraphQL errors: {:?}",
        response.errors
    );

    // Verify both vectors are encrypted
    let vec1 = store
        .get_vector("test_graphql_batch_encrypted", "vec1")
        .unwrap();
    assert!(vec1.payload.unwrap().is_encrypted());

    let vec2 = store
        .get_vector("test_graphql_batch_encrypted", "vec2")
        .unwrap();
    assert!(vec2.payload.unwrap().is_encrypted());
}

/// Test upsert_vectors with mixed encryption (per-vector override)
#[tokio::test]
async fn test_graphql_upsert_vectors_mixed_encryption() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());
    let start_time = std::time::Instant::now();

    let schema = create_schema(store.clone(), embedding_manager, start_time);

    // Create collection
    let config = CollectionConfig {
        dimension: 3,
        ..Default::default()
    };
    store
        .create_collection("test_graphql_mixed", config)
        .unwrap();

    let (_secret_key1, public_key1) = create_test_keypair();
    let (_secret_key2, public_key2) = create_test_keypair();

    let query = r"
        mutation($input: UpsertVectorsInput!) {
            upsertVectors(input: $input) {
                success
                affectedCount
            }
        }
    ";

    let variables = serde_json::json!({
        "input": {
            "collection": "test_graphql_mixed",
            "publicKey": public_key1,  // Request-level key
            "vectors": [
                {
                    "id": "vec1",
                    "data": [1.0, 2.0, 3.0],
                    "payload": {"content": "uses request key"}
                },
                {
                    "id": "vec2",
                    "data": [4.0, 5.0, 6.0],
                    "payload": {"content": "uses own key"},
                    "publicKey": public_key2  // Vector-level override
                }
            ]
        }
    });

    let request = async_graphql::Request::new(query)
        .variables(async_graphql::Variables::from_json(variables));
    let response = schema.execute(request).await;

    assert!(
        response.errors.is_empty(),
        "GraphQL errors: {:?}",
        response.errors
    );

    // Both should be encrypted (but with different keys)
    let vec1 = store.get_vector("test_graphql_mixed", "vec1").unwrap();
    assert!(vec1.payload.unwrap().is_encrypted());

    let vec2 = store.get_vector("test_graphql_mixed", "vec2").unwrap();
    assert!(vec2.payload.unwrap().is_encrypted());
}

/// Test update_payload mutation with encryption
#[tokio::test]
async fn test_graphql_update_payload_with_encryption() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());
    let start_time = std::time::Instant::now();

    let schema = create_schema(store.clone(), embedding_manager, start_time);

    // Create collection and insert initial vector
    let config = CollectionConfig {
        dimension: 3,
        ..Default::default()
    };
    store
        .create_collection("test_graphql_update", config)
        .unwrap();

    let vector = vectorizer::models::Vector::new("vec1".to_string(), vec![1.0, 2.0, 3.0]);
    store.insert("test_graphql_update", vec![vector]).unwrap();

    let (_secret_key, public_key) = create_test_keypair();

    let query = r"
        mutation($collection: String!, $id: String!, $payload: JSON!, $publicKey: String) {
            updatePayload(collection: $collection, id: $id, payload: $payload, publicKey: $publicKey) {
                success
                message
            }
        }
    ";

    let variables = serde_json::json!({
        "collection": "test_graphql_update",
        "id": "vec1",
        "payload": {
            "content": "updated encrypted content"
        },
        "publicKey": public_key
    });

    let request = async_graphql::Request::new(query)
        .variables(async_graphql::Variables::from_json(variables));
    let response = schema.execute(request).await;

    assert!(
        response.errors.is_empty(),
        "GraphQL errors: {:?}",
        response.errors
    );

    // Verify payload is now encrypted
    let vector = store.get_vector("test_graphql_update", "vec1").unwrap();
    assert!(vector.payload.unwrap().is_encrypted());
}

/// Test invalid public key handling
#[tokio::test]
async fn test_graphql_invalid_public_key() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());
    let start_time = std::time::Instant::now();

    let schema = create_schema(store.clone(), embedding_manager, start_time);

    let config = CollectionConfig {
        dimension: 3,
        ..Default::default()
    };
    store
        .create_collection("test_graphql_invalid", config)
        .unwrap();

    let query = r"
        mutation($collection: String!, $input: UpsertVectorInput!) {
            upsertVector(collection: $collection, input: $input) {
                id
            }
        }
    ";

    let variables = serde_json::json!({
        "collection": "test_graphql_invalid",
        "input": {
            "id": "vec1",
            "data": [1.0, 2.0, 3.0],
            "payload": {"content": "data"},
            "publicKey": "invalid_key"
        }
    });

    let request = async_graphql::Request::new(query)
        .variables(async_graphql::Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Should have errors due to invalid key
    assert!(
        !response.errors.is_empty(),
        "Expected error for invalid key"
    );
}
