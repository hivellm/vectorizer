//! Comprehensive tests for WAL (Write-Ahead Log) functionality

use serde_json::json;
use tempfile::tempdir;
use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, Payload, Vector};
use vectorizer::persistence::wal::WALConfig;

#[tokio::test]
async fn test_wal_multiple_operations() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig::default();
    store
        .enable_wal(data_dir.clone(), Some(wal_config.clone()))
        .await
        .unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Euclidean, // Use Euclidean to avoid automatic normalization
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("test_collection", config).unwrap();

    // Insert multiple vectors
    let vectors = (0..10)
        .map(|i| Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32; 384],
            payload: None,
            sparse: None,
        })
        .collect::<Vec<_>>();

    assert!(store.insert("test_collection", vectors).is_ok());

    // Wait for async writes
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Verify all vectors were inserted
    for i in 0..10 {
        let vec = store
            .get_vector("test_collection", &format!("vec_{i}"))
            .unwrap();
        assert_eq!(vec.data[0], i as f32);
    }
}

#[tokio::test]
async fn test_wal_with_payload() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig::default();
    store.enable_wal(data_dir, Some(wal_config)).await.unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("test_collection", config).unwrap();

    // Insert vector with payload
    let payload = Payload {
        data: json!({
            "file_path": "/path/to/file.txt",
            "title": "Test Document",
            "author": "Test Author"
        }),
    };

    let vector = Vector {
        id: "vec_with_payload".to_string(),
        data: vec![1.0; 384],
        payload: Some(payload),
        sparse: None,
    };

    assert!(
        store
            .insert("test_collection", vec![vector.clone()])
            .is_ok()
    );

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Verify vector with payload
    let retrieved = store
        .get_vector("test_collection", "vec_with_payload")
        .unwrap();
    assert!(retrieved.payload.is_some());
    assert_eq!(
        retrieved.payload.as_ref().unwrap().data["title"],
        "Test Document"
    );
}

#[tokio::test]
async fn test_wal_update_sequence() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig::default();
    store.enable_wal(data_dir, Some(wal_config)).await.unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Euclidean, // Use Euclidean to avoid automatic normalization
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("test_collection", config).unwrap();

    // Insert
    let vector1 = Vector {
        id: "test_vec".to_string(),
        data: vec![1.0; 384],
        payload: None,
        sparse: None,
    };
    assert!(store.insert("test_collection", vec![vector1]).is_ok());

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update multiple times
    for i in 2..=5 {
        let updated = Vector {
            id: "test_vec".to_string(),
            data: vec![i as f32; 384],
            payload: None,
            sparse: None,
        };
        assert!(store.update("test_collection", updated).is_ok());
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Verify final state
    let final_vec = store.get_vector("test_collection", "test_vec").unwrap();
    assert_eq!(final_vec.data[0], 5.0);
}

#[tokio::test]
async fn test_wal_delete_sequence() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig::default();
    store.enable_wal(data_dir, Some(wal_config)).await.unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("test_collection", config).unwrap();

    // Insert multiple vectors
    let vectors = (0..5)
        .map(|i| Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32; 384],
            payload: None,
            sparse: None,
        })
        .collect::<Vec<_>>();

    assert!(store.insert("test_collection", vectors).is_ok());
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Delete some vectors
    for i in 0..3 {
        assert!(store.delete("test_collection", &format!("vec_{i}")).is_ok());
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Verify deleted vectors are gone
    for i in 0..3 {
        assert!(
            store
                .get_vector("test_collection", &format!("vec_{i}"))
                .is_err()
        );
    }

    // Verify remaining vectors still exist
    for i in 3..5 {
        assert!(
            store
                .get_vector("test_collection", &format!("vec_{i}"))
                .is_ok()
        );
    }
}

#[tokio::test]
async fn test_wal_multiple_collections() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig::default();
    store.enable_wal(data_dir, Some(wal_config)).await.unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Euclidean, // Use Euclidean to avoid automatic normalization
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    // Create multiple collections
    store
        .create_collection("collection1", config.clone())
        .unwrap();
    store
        .create_collection("collection2", config.clone())
        .unwrap();
    store.create_collection("collection3", config).unwrap();

    // Insert vectors in each collection
    for i in 1..=3 {
        let vector = Vector {
            id: format!("vec_col{i}"),
            data: vec![i as f32; 384],
            payload: None,
            sparse: None,
        };
        assert!(
            store
                .insert(&format!("collection{i}"), vec![vector])
                .is_ok()
        );
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Verify all collections have their vectors
    for i in 1..=3 {
        let vec = store
            .get_vector(&format!("collection{i}"), &format!("vec_col{i}"))
            .unwrap();
        assert_eq!(vec.data[0], i as f32);
    }
}

#[tokio::test]
async fn test_wal_checkpoint() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig {
        checkpoint_threshold: 5, // Low threshold for testing
        max_wal_size_mb: 100,
        checkpoint_interval: std::time::Duration::from_secs(300),
        compression: false,
    };
    store.enable_wal(data_dir, Some(wal_config)).await.unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("test_collection", config).unwrap();

    // Insert vectors to trigger checkpoint threshold
    let vectors = (0..10)
        .map(|i| Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32; 384],
            payload: None,
            sparse: None,
        })
        .collect::<Vec<_>>();

    assert!(store.insert("test_collection", vectors).is_ok());
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify vectors still exist after potential checkpoint
    for i in 0..10 {
        assert!(
            store
                .get_vector("test_collection", &format!("vec_{i}"))
                .is_ok()
        );
    }
}

#[tokio::test]
async fn test_wal_error_handling() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig::default();
    store.enable_wal(data_dir, Some(wal_config)).await.unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("test_collection", config).unwrap();

    // Try to recover from non-existent collection (should not panic)
    let entries = store.recover_from_wal("nonexistent").await.unwrap();
    assert_eq!(entries.len(), 0);

    // Try to recover and replay from non-existent collection
    let count = store.recover_and_replay_wal("nonexistent").await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_wal_without_enabling() {
    // Test that operations work normally when WAL is not enabled
    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("test_collection", config).unwrap();

    // All operations should work without WAL
    let vector = Vector {
        id: "test_vec".to_string(),
        data: vec![1.0; 384],
        payload: None,
        sparse: None,
    };

    assert!(
        store
            .insert("test_collection", vec![vector.clone()])
            .is_ok()
    );
    assert!(store.update("test_collection", vector.clone()).is_ok());
    assert!(store.delete("test_collection", "test_vec").is_ok());

    // Recover should return empty when WAL is not enabled
    let entries = store.recover_from_wal("test_collection").await.unwrap();
    assert_eq!(entries.len(), 0);
}
