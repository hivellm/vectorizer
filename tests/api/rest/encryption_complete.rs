//! Complete integration tests for ECC-AES payload encryption across all API endpoints

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use p256::{SecretKey, elliptic_curve::sec1::ToEncodedPoint};
use serde_json::json;
use std::sync::Arc;

use vectorizer::db::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
};

/// Helper to create a test ECC key pair
fn create_test_keypair() -> (SecretKey, String) {
    let secret_key = SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let public_key = secret_key.public_key();
    let public_key_encoded = public_key.to_encoded_point(false);
    let public_key_base64 = BASE64.encode(public_key_encoded.as_bytes());
    (secret_key, public_key_base64)
}

/// Helper to create a test collection
fn create_test_collection(store: &VectorStore, name: &str, dimension: usize) {
    let config = CollectionConfig {
        dimension,
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
    store.create_collection(name, config).unwrap();
}

#[tokio::test]
async fn test_rest_insert_text_with_encryption() {
    let (_secret_key, public_key_base64) = create_test_keypair();

    // Create store and collection
    let store = Arc::new(VectorStore::new());
    let collection_name = "test_insert_text_encrypted";
    create_test_collection(&store, collection_name, 512);

    // Create embedding manager
    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = vectorizer::embedding::Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25").unwrap();

    // Simulate REST insert_text with encryption
    let text = "This is sensitive confidential data";
    let metadata = json!({
        "category": "financial",
        "user_id": "user123"
    });

    // Generate embedding
    let embedding = embedding_manager.embed(text).unwrap();

    // Create payload and encrypt it
    let payload_json = metadata;
    let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
        &payload_json,
        &public_key_base64,
    )
    .expect("Encryption should succeed");

    let payload = vectorizer::models::Payload::from_encrypted(encrypted);

    // Create and insert vector
    let vector = vectorizer::models::Vector {
        id: uuid::Uuid::new_v4().to_string(),
        data: embedding,
        sparse: None,
        payload: Some(payload),
    };

    store.insert(collection_name, vec![vector.clone()]).unwrap();

    // Verify the vector was inserted with encrypted payload
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector(&vector.id).unwrap();

    assert!(retrieved.payload.is_some());
    let retrieved_payload = retrieved.payload.unwrap();
    assert!(
        retrieved_payload.is_encrypted(),
        "Payload should be encrypted"
    );

    // Verify encrypted payload structure
    let encrypted_data = retrieved_payload.as_encrypted().unwrap();
    assert_eq!(encrypted_data.version, 1);
    assert_eq!(encrypted_data.algorithm, "ECC-P256-AES256GCM");
    assert!(!encrypted_data.nonce.is_empty());
    assert!(!encrypted_data.tag.is_empty());
    assert!(!encrypted_data.encrypted_data.is_empty());
    assert!(!encrypted_data.ephemeral_public_key.is_empty());

    println!("✅ REST insert_text with encryption: PASSED");
}

#[tokio::test]
async fn test_rest_insert_text_without_encryption() {
    // Create store and collection
    let store = Arc::new(VectorStore::new());
    let collection_name = "test_insert_text_unencrypted";
    create_test_collection(&store, collection_name, 512);

    // Create embedding manager
    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = vectorizer::embedding::Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25").unwrap();

    // Simulate REST insert_text WITHOUT encryption
    let text = "This is public data";
    let metadata = json!({
        "category": "public",
        "user_id": "user456"
    });

    // Generate embedding
    let embedding = embedding_manager.embed(text).unwrap();

    // Create payload WITHOUT encryption
    let payload = vectorizer::models::Payload::new(metadata);

    // Create and insert vector
    let vector = vectorizer::models::Vector {
        id: uuid::Uuid::new_v4().to_string(),
        data: embedding,
        sparse: None,
        payload: Some(payload),
    };

    store.insert(collection_name, vec![vector.clone()]).unwrap();

    // Verify the vector was inserted with unencrypted payload
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector(&vector.id).unwrap();

    assert!(retrieved.payload.is_some());
    let retrieved_payload = retrieved.payload.unwrap();
    assert!(
        !retrieved_payload.is_encrypted(),
        "Payload should NOT be encrypted"
    );

    // Verify we can read the plaintext data
    assert_eq!(retrieved_payload.data.get("category").unwrap(), "public");
    assert_eq!(retrieved_payload.data.get("user_id").unwrap(), "user456");

    println!("✅ REST insert_text without encryption: PASSED");
}

#[test]
fn test_qdrant_upsert_with_encryption() {
    let (_secret_key, public_key_base64) = create_test_keypair();

    // Create store and collection
    let store = VectorStore::new();
    let collection_name = "test_qdrant_upsert_encrypted";
    create_test_collection(&store, collection_name, 128);

    // Create vector data
    let vector_data: Vec<f32> = (0..128).map(|i| (i as f32) / 128.0).collect();
    let payload_json = json!({
        "document": "sensitive contract",
        "amount": 1000000,
        "classification": "confidential"
    });

    // Encrypt payload
    let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
        &payload_json,
        &public_key_base64,
    )
    .expect("Encryption should succeed");

    let payload = vectorizer::models::Payload::from_encrypted(encrypted);

    // Create and insert vector (simulating Qdrant upsert)
    let vector = vectorizer::models::Vector {
        id: "qdrant_vec_1".to_string(),
        data: vector_data,
        sparse: None,
        payload: Some(payload),
    };

    store.insert(collection_name, vec![vector]).unwrap();

    // Verify encryption
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector("qdrant_vec_1").unwrap();

    assert!(retrieved.payload.is_some());
    assert!(retrieved.payload.unwrap().is_encrypted());

    println!("✅ Qdrant upsert with encryption: PASSED");
}

#[test]
fn test_qdrant_upsert_mixed_encryption() {
    let (_secret_key, public_key_base64) = create_test_keypair();

    // Create store and collection
    let store = VectorStore::new();
    let collection_name = "test_qdrant_mixed";
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
        encryption: Some(vectorizer::models::EncryptionConfig {
            required: false,
            allow_mixed: true,
        }),
    };
    store.create_collection(collection_name, config).unwrap();

    // Vector 1: Encrypted
    let encrypted_payload = vectorizer::security::payload_encryption::encrypt_payload(
        &json!({"type": "encrypted", "data": "secret"}),
        &public_key_base64,
    )
    .unwrap();

    let vector1 = vectorizer::models::Vector {
        id: "vec_encrypted".to_string(),
        data: vec![0.1; 128],
        sparse: None,
        payload: Some(vectorizer::models::Payload::from_encrypted(
            encrypted_payload,
        )),
    };

    // Vector 2: Unencrypted
    let vector2 = vectorizer::models::Vector {
        id: "vec_unencrypted".to_string(),
        data: vec![0.2; 128],
        sparse: None,
        payload: Some(vectorizer::models::Payload::new(
            json!({"type": "public", "data": "open"}),
        )),
    };

    // Insert both
    store
        .insert(collection_name, vec![vector1, vector2])
        .unwrap();

    // Verify mixed payloads
    let collection = store.get_collection(collection_name).unwrap();

    let retrieved1 = collection.get_vector("vec_encrypted").unwrap();
    assert!(retrieved1.payload.as_ref().unwrap().is_encrypted());

    let retrieved2 = collection.get_vector("vec_unencrypted").unwrap();
    assert!(!retrieved2.payload.as_ref().unwrap().is_encrypted());

    println!("✅ Qdrant upsert with mixed encryption: PASSED");
}

#[test]
fn test_file_upload_simulation_with_encryption() {
    let (_secret_key, public_key_base64) = create_test_keypair();

    // Create store and collection
    let store = VectorStore::new();
    let collection_name = "test_file_upload_encrypted";
    create_test_collection(&store, collection_name, 512);

    // Simulate file chunks with metadata
    let chunks = vec![
        ("Chunk 1: Introduction to cryptography", 0),
        ("Chunk 2: ECC and AES encryption", 1),
        ("Chunk 3: Zero-knowledge architecture", 2),
    ];

    let mut vectors = Vec::new();

    for (content, index) in chunks {
        // Simulate embedding generation (using dummy data for test)
        let embedding = vec![0.1 * (index as f32 + 1.0); 512];

        // Create payload with file metadata
        let payload_json = json!({
            "content": content,
            "file_path": "/docs/crypto.pdf",
            "chunk_index": index,
            "language": "en",
            "source": "file_upload",
            "original_filename": "crypto.pdf",
            "file_extension": "pdf"
        });

        // Encrypt payload
        let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
            &payload_json,
            &public_key_base64,
        )
        .expect("Encryption should succeed");

        let payload = vectorizer::models::Payload::from_encrypted(encrypted);

        vectors.push(vectorizer::models::Vector {
            id: uuid::Uuid::new_v4().to_string(),
            data: embedding,
            sparse: None,
            payload: Some(payload),
        });
    }

    // Insert all chunks
    let vector_ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();
    store.insert(collection_name, vectors).unwrap();

    // Verify all chunks are encrypted
    let collection = store.get_collection(collection_name).unwrap();

    for (idx, vector_id) in vector_ids.iter().enumerate() {
        let retrieved = collection.get_vector(vector_id).unwrap();
        assert!(retrieved.payload.is_some());

        let payload = retrieved.payload.unwrap();
        assert!(payload.is_encrypted(), "Chunk {idx} should be encrypted");

        // Verify encrypted structure
        let encrypted_data = payload.as_encrypted().unwrap();
        assert_eq!(encrypted_data.algorithm, "ECC-P256-AES256GCM");
    }

    println!(
        "✅ File upload simulation with encryption: PASSED ({} chunks)",
        vector_ids.len()
    );
}

#[test]
fn test_encryption_with_invalid_key() {
    let store = VectorStore::new();
    let collection_name = "test_invalid_key";
    create_test_collection(&store, collection_name, 128);

    let payload_json = json!({"data": "test"});

    // Try various invalid key formats
    let invalid_keys = [
        "not_base64_!@#$%",
        "dG9vIHNob3J0", // "too short" in base64
        "",
        "invalid",
    ];

    for (idx, invalid_key) in invalid_keys.iter().enumerate() {
        let result =
            vectorizer::security::payload_encryption::encrypt_payload(&payload_json, invalid_key);
        assert!(
            result.is_err(),
            "Should fail with invalid key {idx}: '{invalid_key}'"
        );
    }

    println!("✅ Invalid key handling: PASSED");
}

#[test]
fn test_encryption_required_enforcement() {
    let store = VectorStore::new();
    let collection_name = "test_encryption_required";

    // Create collection with REQUIRED encryption
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
        encryption: Some(vectorizer::models::EncryptionConfig {
            required: true,
            allow_mixed: false,
        }),
    };
    store.create_collection(collection_name, config).unwrap();

    // Try to insert unencrypted vector - should FAIL
    let unencrypted_vector = vectorizer::models::Vector {
        id: "unencrypted".to_string(),
        data: vec![0.1; 64],
        sparse: None,
        payload: Some(vectorizer::models::Payload::new(json!({"data": "test"}))),
    };

    let result = store.insert(collection_name, vec![unencrypted_vector]);
    assert!(
        result.is_err(),
        "Should reject unencrypted payload when encryption is required"
    );

    // Now try with encrypted payload - should SUCCEED
    let (_secret_key, public_key) = create_test_keypair();
    let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
        &json!({"data": "encrypted"}),
        &public_key,
    )
    .unwrap();

    let encrypted_vector = vectorizer::models::Vector {
        id: "encrypted".to_string(),
        data: vec![0.2; 64],
        sparse: None,
        payload: Some(vectorizer::models::Payload::from_encrypted(encrypted)),
    };

    let result = store.insert(collection_name, vec![encrypted_vector]);
    assert!(
        result.is_ok(),
        "Should accept encrypted payload when encryption is required"
    );

    println!("✅ Encryption required enforcement: PASSED");
}

#[test]
fn test_backward_compatibility_all_routes() {
    // Test that all routes work WITHOUT encryption (backward compatibility)
    let store = VectorStore::new();

    // Test 1: Qdrant upsert without encryption
    let collection1 = "compat_qdrant";
    create_test_collection(&store, collection1, 128);

    let vector1 = vectorizer::models::Vector {
        id: "v1".to_string(),
        data: vec![0.1; 128],
        sparse: None,
        payload: Some(vectorizer::models::Payload::new(json!({"type": "qdrant"}))),
    };
    store.insert(collection1, vec![vector1]).unwrap();

    // Test 2: Insert text without encryption
    let collection2 = "compat_insert";
    create_test_collection(&store, collection2, 512);

    let vector2 = vectorizer::models::Vector {
        id: "v2".to_string(),
        data: vec![0.2; 512],
        sparse: None,
        payload: Some(vectorizer::models::Payload::new(
            json!({"type": "insert_text"}),
        )),
    };
    store.insert(collection2, vec![vector2]).unwrap();

    // Test 3: File upload without encryption
    let collection3 = "compat_file";
    create_test_collection(&store, collection3, 512);

    let vector3 = vectorizer::models::Vector {
        id: "v3".to_string(),
        data: vec![0.3; 512],
        sparse: None,
        payload: Some(vectorizer::models::Payload::new(
            json!({"type": "file_upload"}),
        )),
    };
    store.insert(collection3, vec![vector3]).unwrap();

    // Verify all are NOT encrypted
    let c1 = store.get_collection(collection1).unwrap();
    assert!(
        !c1.get_vector("v1")
            .unwrap()
            .payload
            .as_ref()
            .unwrap()
            .is_encrypted()
    );

    let c2 = store.get_collection(collection2).unwrap();
    assert!(
        !c2.get_vector("v2")
            .unwrap()
            .payload
            .as_ref()
            .unwrap()
            .is_encrypted()
    );

    let c3 = store.get_collection(collection3).unwrap();
    assert!(
        !c3.get_vector("v3")
            .unwrap()
            .payload
            .as_ref()
            .unwrap()
            .is_encrypted()
    );

    println!("✅ Backward compatibility (all routes): PASSED");
}

#[test]
fn test_key_format_support() {
    // Test all supported key formats: PEM, hex, base64
    let secret_key = SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let public_key = secret_key.public_key();
    let public_key_point = public_key.to_encoded_point(false);
    let public_key_bytes = public_key_point.as_bytes();

    let payload = json!({"test": "data"});

    // Test 1: Base64 format
    let base64_key = BASE64.encode(public_key_bytes);
    let result1 = vectorizer::security::payload_encryption::encrypt_payload(&payload, &base64_key);
    assert!(result1.is_ok(), "Base64 format should work");

    // Test 2: Hex format (without 0x)
    let hex_key = hex::encode(public_key_bytes);
    let result2 = vectorizer::security::payload_encryption::encrypt_payload(&payload, &hex_key);
    assert!(result2.is_ok(), "Hex format should work");

    // Test 3: Hex format (with 0x prefix)
    let hex_key_with_prefix = format!("0x{}", hex::encode(public_key_bytes));
    let result3 =
        vectorizer::security::payload_encryption::encrypt_payload(&payload, &hex_key_with_prefix);
    assert!(result3.is_ok(), "Hex with 0x prefix should work");

    println!("✅ Key format support (base64, hex, 0x-hex): PASSED");
}
