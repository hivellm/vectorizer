//! Extended encryption tests - Edge cases, performance, persistence, and concurrency

use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use p256::SecretKey;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use serde_json::json;
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, EncryptionConfig, HnswConfig, Payload,
    QuantizationConfig,
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
fn create_test_collection(
    store: &VectorStore,
    name: &str,
    dimension: usize,
    encryption: Option<EncryptionConfig>,
) {
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
        encryption,
    };
    store.create_collection(name, config).unwrap();
}

#[test]
fn test_empty_payload_encryption() {
    let (_secret_key, public_key) = create_test_keypair();
    let store = VectorStore::new();
    let collection_name = "test_empty_payload";
    create_test_collection(&store, collection_name, 128, None);

    // Test encrypting empty JSON object
    let empty_payload = json!({});
    let encrypted =
        vectorizer::security::payload_encryption::encrypt_payload(&empty_payload, &public_key)
            .expect("Should encrypt empty payload");

    let payload = Payload::from_encrypted(encrypted);
    let vector = vectorizer::models::Vector {
        id: "empty_vec".to_string(),
        data: vec![0.1; 128],
        sparse: None,
        payload: Some(payload),
    };

    store.insert(collection_name, vec![vector]).unwrap();

    // Verify
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector("empty_vec").unwrap();
    assert!(retrieved.payload.is_some());
    assert!(retrieved.payload.unwrap().is_encrypted());

    println!("✅ Empty payload encryption: PASSED");
}

#[test]
fn test_large_payload_encryption() {
    let (_secret_key, public_key) = create_test_keypair();
    let store = VectorStore::new();
    let collection_name = "test_large_payload";
    create_test_collection(&store, collection_name, 128, None);

    // Create a large payload (10KB of data)
    let large_text = "Lorem ipsum dolor sit amet. ".repeat(400); // ~10KB
    let large_payload = json!({
        "title": "Large Document",
        "content": large_text,
        "metadata": {
            "size": large_text.len(),
            "type": "large_document"
        },
        "tags": vec!["tag1", "tag2", "tag3"],
        "nested": {
            "level1": {
                "level2": {
                    "level3": "deep value"
                }
            }
        }
    });

    let encrypted =
        vectorizer::security::payload_encryption::encrypt_payload(&large_payload, &public_key)
            .expect("Should encrypt large payload");

    let payload = Payload::from_encrypted(encrypted);
    let vector = vectorizer::models::Vector {
        id: "large_vec".to_string(),
        data: vec![0.1; 128],
        sparse: None,
        payload: Some(payload),
    };

    store.insert(collection_name, vec![vector]).unwrap();

    // Verify
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector("large_vec").unwrap();
    assert!(retrieved.payload.is_some());
    assert!(retrieved.payload.unwrap().is_encrypted());

    println!("✅ Large payload encryption (~10KB): PASSED");
}

#[test]
fn test_special_characters_in_payload() {
    let (_secret_key, public_key) = create_test_keypair();
    let store = VectorStore::new();
    let collection_name = "test_special_chars";
    create_test_collection(&store, collection_name, 128, None);

    // Payload with special characters, emojis, unicode
    let special_payload = json!({
        "emoji": "🔐💎✨🚀",
        "chinese": "你好世界",
        "arabic": "مرحبا بالعالم",
        "russian": "Привет мир",
        "symbols": "!@#$%^&*()_+-=[]{}|;':\",./<>?",
        "newlines": "line1\nline2\rline3\r\nline4",
        "tabs": "col1\tcol2\tcol3",
        "quotes": "He said \"Hello\" and she said 'Hi'",
        "backslash": "path\\to\\file",
        "null_char": "before\u{0000}after"
    });

    let encrypted =
        vectorizer::security::payload_encryption::encrypt_payload(&special_payload, &public_key)
            .expect("Should encrypt special characters");

    let payload = Payload::from_encrypted(encrypted);
    let vector = vectorizer::models::Vector {
        id: "special_vec".to_string(),
        data: vec![0.1; 128],
        sparse: None,
        payload: Some(payload),
    };

    store.insert(collection_name, vec![vector]).unwrap();

    // Verify
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector("special_vec").unwrap();
    assert!(retrieved.payload.is_some());
    assert!(retrieved.payload.unwrap().is_encrypted());

    println!("✅ Special characters encryption: PASSED");
}

#[test]
fn test_multiple_vectors_same_key() {
    let (_secret_key, public_key) = create_test_keypair();
    let store = VectorStore::new();
    let collection_name = "test_multiple_same_key";
    create_test_collection(&store, collection_name, 128, None);

    // Insert 100 vectors with same key
    let mut vectors = Vec::new();
    for i in 0..100 {
        let payload_json = json!({
            "index": i,
            "data": format!("Vector number {}", i),
            "category": if i % 2 == 0 { "even" } else { "odd" }
        });

        let encrypted =
            vectorizer::security::payload_encryption::encrypt_payload(&payload_json, &public_key)
                .expect("Encryption should succeed");

        vectors.push(vectorizer::models::Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32 / 100.0; 128],
            sparse: None,
            payload: Some(Payload::from_encrypted(encrypted)),
        });
    }

    store.insert(collection_name, vectors).unwrap();

    // Verify all are encrypted
    let collection = store.get_collection(collection_name).unwrap();
    assert_eq!(collection.vector_count(), 100);

    for i in 0..100 {
        let retrieved = collection.get_vector(&format!("vec_{i}")).unwrap();
        assert!(retrieved.payload.is_some());
        assert!(
            retrieved.payload.unwrap().is_encrypted(),
            "Vector {i} should be encrypted"
        );
    }

    println!("✅ Multiple vectors with same key (100 vectors): PASSED");
}

#[test]
fn test_multiple_vectors_different_keys() {
    let store = VectorStore::new();
    let collection_name = "test_multiple_different_keys";
    create_test_collection(&store, collection_name, 128, None);

    // Insert 10 vectors with different keys
    let mut vectors = Vec::new();
    for i in 0..10 {
        let (_secret, public_key) = create_test_keypair(); // Different key for each

        let payload_json = json!({
            "index": i,
            "data": format!("Vector with unique key {}", i)
        });

        let encrypted =
            vectorizer::security::payload_encryption::encrypt_payload(&payload_json, &public_key)
                .expect("Encryption should succeed");

        vectors.push(vectorizer::models::Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32 / 10.0; 128],
            sparse: None,
            payload: Some(Payload::from_encrypted(encrypted)),
        });
    }

    store.insert(collection_name, vectors).unwrap();

    // Verify all are encrypted with different ephemeral keys
    let collection = store.get_collection(collection_name).unwrap();
    assert_eq!(collection.vector_count(), 10);

    let mut ephemeral_keys = std::collections::HashSet::new();
    for i in 0..10 {
        let retrieved = collection.get_vector(&format!("vec_{i}")).unwrap();
        let payload = retrieved.payload.unwrap();
        assert!(payload.is_encrypted());

        let encrypted_data = payload.as_encrypted().unwrap();
        ephemeral_keys.insert(encrypted_data.ephemeral_public_key.clone());
    }

    // All should have different ephemeral keys
    assert_eq!(
        ephemeral_keys.len(),
        10,
        "All vectors should have unique ephemeral keys"
    );

    println!("✅ Multiple vectors with different keys (10 unique keys): PASSED");
}

#[test]
fn test_encryption_with_all_json_types() {
    let (_secret_key, public_key) = create_test_keypair();
    let store = VectorStore::new();
    let collection_name = "test_all_json_types";
    create_test_collection(&store, collection_name, 128, None);

    // Payload with all JSON types
    let comprehensive_payload = json!({
        "string": "text value",
        "number_int": 42,
        "number_float": 123.45,
        "boolean_true": true,
        "boolean_false": false,
        "null": null,
        "array_empty": [],
        "array_mixed": [1, "two", 3.0, true, null, {"nested": "object"}],
        "object_empty": {},
        "object_nested": {
            "level1": {
                "level2": {
                    "level3": {
                        "value": "deep"
                    }
                }
            }
        },
        "large_number": 9007199254740991i64,
        "negative": -42,
        "scientific": 1.23e-4,
        "unicode": "\u{2764}\u{FE0F}",
        "escaped": "line1\nline2\ttab",
    });

    let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
        &comprehensive_payload,
        &public_key,
    )
    .expect("Should encrypt all JSON types");

    let payload = Payload::from_encrypted(encrypted);
    let vector = vectorizer::models::Vector {
        id: "json_types_vec".to_string(),
        data: vec![0.1; 128],
        sparse: None,
        payload: Some(payload),
    };

    store.insert(collection_name, vec![vector]).unwrap();

    // Verify
    let collection = store.get_collection(collection_name).unwrap();
    let retrieved = collection.get_vector("json_types_vec").unwrap();
    assert!(retrieved.payload.is_some());
    assert!(retrieved.payload.unwrap().is_encrypted());

    println!("✅ All JSON types encryption: PASSED");
}

#[test]
fn test_concurrent_insertions_with_encryption() {
    use std::thread;

    let store = Arc::new(VectorStore::new());
    let collection_name = "test_concurrent_encryption";
    create_test_collection(&store, collection_name, 64, None);

    let (_secret_key, public_key) = create_test_keypair();
    let public_key = Arc::new(public_key);

    // Spawn 10 threads, each inserting 10 vectors
    let mut handles = vec![];
    for thread_id in 0..10 {
        let store_clone = Arc::clone(&store);
        let key_clone = Arc::clone(&public_key);
        let collection = collection_name.to_string();

        let handle = thread::spawn(move || {
            for i in 0..10 {
                let payload_json = json!({
                    "thread": thread_id,
                    "index": i,
                    "data": format!("Thread {} - Item {}", thread_id, i)
                });

                let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
                    &payload_json,
                    &key_clone,
                )
                .expect("Encryption should succeed");

                let vector = vectorizer::models::Vector {
                    id: format!("t{thread_id}_v{i}"),
                    data: vec![(thread_id * 10 + i) as f32 / 100.0; 64],
                    sparse: None,
                    payload: Some(Payload::from_encrypted(encrypted)),
                };

                store_clone.insert(&collection, vec![vector]).unwrap();
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all 100 vectors were inserted and encrypted
    let collection = store.get_collection(collection_name).unwrap();
    assert_eq!(
        collection.vector_count(),
        100,
        "Should have 100 vectors from 10 threads"
    );

    println!("✅ Concurrent insertions (10 threads × 10 vectors): PASSED");
}

#[test]
fn test_encryption_required_reject_unencrypted() {
    let store = VectorStore::new();
    let collection_name = "test_strict_encryption";

    // Create collection with REQUIRED encryption
    create_test_collection(
        &store,
        collection_name,
        64,
        Some(EncryptionConfig {
            required: true,
            allow_mixed: false,
        }),
    );

    // Try to insert unencrypted - should FAIL
    let unencrypted_vector = vectorizer::models::Vector {
        id: "unencrypted".to_string(),
        data: vec![0.1; 64],
        sparse: None,
        payload: Some(Payload::new(json!({"data": "should fail"}))),
    };

    let result = store.insert(collection_name, vec![unencrypted_vector]);
    assert!(
        result.is_err(),
        "Should reject unencrypted when encryption is required"
    );

    // Now insert encrypted - should SUCCEED
    let (_secret, public_key) = create_test_keypair();
    let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
        &json!({"data": "encrypted"}),
        &public_key,
    )
    .unwrap();

    let encrypted_vector = vectorizer::models::Vector {
        id: "encrypted".to_string(),
        data: vec![0.2; 64],
        sparse: None,
        payload: Some(Payload::from_encrypted(encrypted)),
    };

    let result = store.insert(collection_name, vec![encrypted_vector]);
    assert!(
        result.is_ok(),
        "Should accept encrypted when encryption is required"
    );

    println!("✅ Encryption required enforcement: PASSED");
}

#[test]
fn test_multiple_key_rotations() {
    let store = VectorStore::new();
    let collection_name = "test_key_rotation";
    create_test_collection(&store, collection_name, 64, None);

    // Simulate key rotation: insert vectors with different keys over time
    let mut vectors = Vec::new();

    for batch in 0..5 {
        let (_secret, public_key) = create_test_keypair(); // New key for each batch

        for i in 0..10 {
            let payload_json = json!({
                "batch": batch,
                "index": i,
                "data": format!("Batch {} - Item {}", batch, i)
            });

            let encrypted = vectorizer::security::payload_encryption::encrypt_payload(
                &payload_json,
                &public_key,
            )
            .expect("Encryption should succeed");

            vectors.push(vectorizer::models::Vector {
                id: format!("b{batch}_i{i}"),
                data: vec![(batch * 10 + i) as f32 / 50.0; 64],
                sparse: None,
                payload: Some(Payload::from_encrypted(encrypted)),
            });
        }
    }

    // Insert all at once
    store.insert(collection_name, vectors).unwrap();

    // Verify all 50 vectors (5 batches × 10 vectors)
    let collection = store.get_collection(collection_name).unwrap();
    assert_eq!(collection.vector_count(), 50);

    for batch in 0..5 {
        for i in 0..10 {
            let retrieved = collection.get_vector(&format!("b{batch}_i{i}")).unwrap();
            assert!(retrieved.payload.is_some());
            assert!(retrieved.payload.unwrap().is_encrypted());
        }
    }

    println!("✅ Multiple key rotations (5 keys × 10 vectors): PASSED");
}

#[test]
fn test_different_key_formats_interoperability() {
    let secret_key = SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let public_key = secret_key.public_key();
    let public_key_point = public_key.to_encoded_point(false);
    let public_key_bytes = public_key_point.as_bytes();

    let payload = json!({"test": "interoperability"});

    // Encrypt with base64 format
    let base64_key = BASE64.encode(public_key_bytes);
    let encrypted_base64 =
        vectorizer::security::payload_encryption::encrypt_payload(&payload, &base64_key)
            .expect("Base64 should work");

    // Encrypt with hex format
    let hex_key = hex::encode(public_key_bytes);
    let encrypted_hex =
        vectorizer::security::payload_encryption::encrypt_payload(&payload, &hex_key)
            .expect("Hex should work");

    // Encrypt with 0x-prefixed hex
    let hex_0x_key = format!("0x{}", hex::encode(public_key_bytes));
    let encrypted_hex_0x =
        vectorizer::security::payload_encryption::encrypt_payload(&payload, &hex_0x_key)
            .expect("Hex with 0x should work");

    // All should produce valid encrypted payloads
    assert_eq!(encrypted_base64.version, 1);
    assert_eq!(encrypted_hex.version, 1);
    assert_eq!(encrypted_hex_0x.version, 1);

    assert_eq!(encrypted_base64.algorithm, "ECC-P256-AES256GCM");
    assert_eq!(encrypted_hex.algorithm, "ECC-P256-AES256GCM");
    assert_eq!(encrypted_hex_0x.algorithm, "ECC-P256-AES256GCM");

    println!("✅ Different key formats interoperability: PASSED");
}

#[test]
fn test_payload_size_variations() {
    let (_secret, public_key) = create_test_keypair();
    let store = VectorStore::new();
    let collection_name = "test_size_variations";
    create_test_collection(&store, collection_name, 128, None);

    // Test different payload sizes
    let sizes = vec![
        ("tiny", json!({"x": 1})),
        ("small", json!({"data": "a".repeat(100)})),
        ("medium", json!({"data": "b".repeat(1000)})),
        ("large", json!({"data": "c".repeat(10000)})),
    ];

    for (name, payload_json) in sizes {
        let encrypted =
            vectorizer::security::payload_encryption::encrypt_payload(&payload_json, &public_key)
                .unwrap_or_else(|_| panic!("Should encrypt {name} payload"));

        let vector = vectorizer::models::Vector {
            id: format!("vec_{name}"),
            data: vec![0.1; 128],
            sparse: None,
            payload: Some(Payload::from_encrypted(encrypted)),
        };

        store.insert(collection_name, vec![vector]).unwrap();
    }

    // Verify all sizes work
    let collection = store.get_collection(collection_name).unwrap();
    assert_eq!(collection.vector_count(), 4);

    for (name, _) in [("tiny", ()), ("small", ()), ("medium", ()), ("large", ())] {
        let retrieved = collection.get_vector(&format!("vec_{name}")).unwrap();
        assert!(retrieved.payload.is_some());
        assert!(retrieved.payload.unwrap().is_encrypted());
    }

    println!("✅ Payload size variations (tiny to 10KB): PASSED");
}

#[test]
fn test_encrypted_payload_structure_validation() {
    let (_secret, public_key) = create_test_keypair();
    let payload = json!({"test": "validation"});

    let encrypted =
        vectorizer::security::payload_encryption::encrypt_payload(&payload, &public_key)
            .expect("Encryption should succeed");

    // Validate structure
    assert_eq!(encrypted.version, 1, "Version should be 1");
    assert_eq!(
        encrypted.algorithm, "ECC-P256-AES256GCM",
        "Algorithm should be ECC-P256-AES256GCM"
    );

    // Validate all fields are present and non-empty
    assert!(!encrypted.nonce.is_empty(), "Nonce should not be empty");
    assert!(!encrypted.tag.is_empty(), "Tag should not be empty");
    assert!(
        !encrypted.encrypted_data.is_empty(),
        "Encrypted data should not be empty"
    );
    assert!(
        !encrypted.ephemeral_public_key.is_empty(),
        "Ephemeral public key should not be empty"
    );

    // Validate base64 encoding (should decode without error)
    assert!(
        BASE64.decode(&encrypted.nonce).is_ok(),
        "Nonce should be valid base64"
    );
    assert!(
        BASE64.decode(&encrypted.tag).is_ok(),
        "Tag should be valid base64"
    );
    assert!(
        BASE64.decode(&encrypted.encrypted_data).is_ok(),
        "Encrypted data should be valid base64"
    );
    assert!(
        BASE64.decode(&encrypted.ephemeral_public_key).is_ok(),
        "Ephemeral key should be valid base64"
    );

    // Validate expected sizes (approximate, as they can vary slightly)
    let nonce_bytes = BASE64.decode(&encrypted.nonce).unwrap();
    assert_eq!(nonce_bytes.len(), 12, "AES-GCM nonce should be 12 bytes");

    let tag_bytes = BASE64.decode(&encrypted.tag).unwrap();
    assert_eq!(tag_bytes.len(), 16, "AES-GCM tag should be 16 bytes");

    let ephemeral_key_bytes = BASE64.decode(&encrypted.ephemeral_public_key).unwrap();
    assert_eq!(
        ephemeral_key_bytes.len(),
        65,
        "Uncompressed P-256 public key should be 65 bytes"
    );

    println!("✅ Encrypted payload structure validation: PASSED");
}
