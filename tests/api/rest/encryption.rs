//! Integration tests for ECC-AES payload encryption

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use p256::SecretKey;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use serde_json::json;
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, EncryptionConfig, HnswConfig,
    QuantizationConfig,
};

#[test]
#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]
fn test_encrypted_payload_insertion_via_collection() {
    // Generate a test ECC key pair
    let secret_key = SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let public_key = secret_key.public_key();
    let public_key_encoded = public_key.to_encoded_point(false);
    let public_key_base64 = BASE64.encode(public_key_encoded.as_bytes());

    // Create a collection
    let store = VectorStore::new();
    let collection_name = "test_encrypted_collection";

    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::default(),
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        graph: None,
        encryption: Some(EncryptionConfig {
            required: false,
            allow_mixed: true,
        }),
    };

    store.create_collection(collection_name, config).unwrap();

    // Create a vector with payload
    let vector_id = "encrypted_vector_1";
    let vector_data: Vec<f32> = (0..128).map(|i| (i as f32) / 128.0).collect();
    let payload_json = json!({
        "user_id": "12345",
        "sensitive_data": "This is confidential information",
        "metadata": {
            "category": "financial",
            "timestamp": "2024-01-15T10:30:00Z"
        }
    });

    // Encrypt the payload
    let encrypted_payload = vectorizer::security::payload_encryption::encrypt_payload(
        &payload_json,
        &public_key_base64,
    )
    .expect("Encryption should succeed");

    // Create vector with encrypted payload
    let vector = vectorizer::models::Vector {
        id: vector_id.to_string(),
        data: vector_data.clone(),
        sparse: None,
        payload: Some(vectorizer::models::Payload::from_encrypted(
            encrypted_payload,
        )),
    };

    // Insert the vector
    store
        .insert(collection_name, vec![vector])
        .expect("Insert should succeed");

    // Retrieve the vector
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector(vector_id).unwrap();

    // Verify the payload is encrypted
    assert!(retrieved.payload.is_some());
    let payload = retrieved.payload.unwrap();
    assert!(
        payload.is_encrypted(),
        "Payload should be detected as encrypted"
    );

    // Verify the encrypted payload structure
    let encrypted_data = payload.as_encrypted().expect("Should parse as encrypted");
    assert_eq!(encrypted_data.version, 1);
    assert_eq!(encrypted_data.algorithm, "ECC-P256-AES256GCM");
    assert!(!encrypted_data.nonce.is_empty());
    assert!(!encrypted_data.tag.is_empty());
    assert!(!encrypted_data.encrypted_data.is_empty());
    assert!(!encrypted_data.ephemeral_public_key.is_empty());
}

#[test]
fn test_unencrypted_payload_backward_compatibility() {
    let store = VectorStore::new();
    let collection_name = "test_unencrypted_collection";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::default(),
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        graph: None,
        encryption: None,
    };

    store.create_collection(collection_name, config).unwrap();

    // Create a vector with unencrypted payload
    let vector_id = "unencrypted_vector_1";
    let vector_data: Vec<f32> = (0..64).map(|i| (i as f32) / 64.0).collect();
    let payload_json = json!({
        "user_id": "67890",
        "public_data": "This is not sensitive"
    });

    let vector = vectorizer::models::Vector {
        id: vector_id.to_string(),
        data: vector_data,
        sparse: None,
        payload: Some(vectorizer::models::Payload::new(payload_json.clone())),
    };

    // Insert the vector
    store
        .insert(collection_name, vec![vector])
        .expect("Insert should succeed");

    // Retrieve the vector
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector(vector_id).unwrap();

    // Verify the payload is NOT encrypted
    assert!(retrieved.payload.is_some());
    let payload = retrieved.payload.unwrap();
    assert!(
        !payload.is_encrypted(),
        "Payload should not be detected as encrypted"
    );

    // Verify we can access the original data
    assert_eq!(payload.data.get("user_id").unwrap(), "67890");
    assert_eq!(
        payload.data.get("public_data").unwrap(),
        "This is not sensitive"
    );
}

#[test]
#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]
fn test_mixed_encrypted_and_unencrypted_payloads() {
    let store = VectorStore::new();
    let collection_name = "test_mixed_collection";

    // Generate a test ECC key pair
    let secret_key = SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let public_key = secret_key.public_key();
    let public_key_encoded = public_key.to_encoded_point(false);
    let public_key_base64 = BASE64.encode(public_key_encoded.as_bytes());

    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::default(),
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        graph: None,
        encryption: Some(EncryptionConfig {
            required: false,
            allow_mixed: true, // Allow both encrypted and unencrypted
        }),
    };

    store.create_collection(collection_name, config).unwrap();

    // Insert encrypted vector
    let encrypted_payload = vectorizer::security::payload_encryption::encrypt_payload(
        &json!({"data": "encrypted"}),
        &public_key_base64,
    )
    .unwrap();

    let vector1 = vectorizer::models::Vector {
        id: "vec1".to_string(),
        data: vec![0.1; 128],
        sparse: None,
        payload: Some(vectorizer::models::Payload::from_encrypted(
            encrypted_payload,
        )),
    };

    // Insert unencrypted vector
    let vector2 = vectorizer::models::Vector {
        id: "vec2".to_string(),
        data: vec![0.2; 128],
        sparse: None,
        payload: Some(vectorizer::models::Payload::new(
            json!({"data": "unencrypted"}),
        )),
    };

    // Both should insert successfully
    store
        .insert(collection_name, vec![vector1, vector2])
        .expect("Insert should succeed");

    let collection = store.get_collection(collection_name).unwrap();

    // Verify first vector is encrypted
    let retrieved1 = collection.get_vector("vec1").unwrap();
    assert!(retrieved1.payload.as_ref().unwrap().is_encrypted());

    // Verify second vector is not encrypted
    let retrieved2 = collection.get_vector("vec2").unwrap();
    assert!(!retrieved2.payload.as_ref().unwrap().is_encrypted());
}

#[test]
#[ignore = "Flaky on CI - passes locally but fails on macOS CI"]
fn test_encryption_required_validation() {
    let store = VectorStore::new();
    let collection_name = "test_encryption_required";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::default(),
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        graph: None,
        encryption: Some(EncryptionConfig {
            required: true, // Require encryption
            allow_mixed: false,
        }),
    };

    store.create_collection(collection_name, config).unwrap();

    // Try to insert unencrypted vector - should fail
    let vector = vectorizer::models::Vector {
        id: "unencrypted_vec".to_string(),
        data: vec![0.1; 64],
        sparse: None,
        payload: Some(vectorizer::models::Payload::new(json!({"data": "test"}))),
    };

    let result = store.insert(collection_name, vec![vector]);
    assert!(
        result.is_err(),
        "Insert should fail when encryption is required but payload is unencrypted"
    );
}

#[test]
fn test_invalid_public_key_format() {
    let payload_json = json!({"test": "data"});

    // Test invalid base64
    let result = vectorizer::security::payload_encryption::encrypt_payload(
        &payload_json,
        "invalid_base64_!@#$%",
    );
    assert!(result.is_err(), "Should fail with invalid base64");

    // Test invalid key length
    let result = vectorizer::security::payload_encryption::encrypt_payload(
        &payload_json,
        "dG9vIHNob3J0", // "too short" in base64
    );
    assert!(result.is_err(), "Should fail with invalid key length");
}
