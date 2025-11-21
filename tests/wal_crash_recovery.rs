//! Tests for WAL crash recovery functionality

use std::path::PathBuf;

use tempfile::tempdir;
use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, Vector};
use vectorizer::persistence::wal::WALConfig;

#[tokio::test]
async fn test_wal_crash_recovery_insert() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    // Create vector store
    let store = VectorStore::new();

    // Enable WAL
    let wal_config = WALConfig {
        checkpoint_threshold: 1000,
        max_wal_size_mb: 100,
        checkpoint_interval: std::time::Duration::from_secs(300),
        compression: false,
    };
    store
        .enable_wal(data_dir.clone(), Some(wal_config.clone()))
        .await
        .unwrap();

    // Create collection
    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
    };
    store.create_collection("test_collection", config).unwrap();

    // Insert vectors
    let vectors = vec![
        Vector {
            id: "vec1".to_string(),
            data: vec![1.0; 384],
            payload: None,
            sparse: None,
        },
        Vector {
            id: "vec2".to_string(),
            data: vec![2.0; 384],
            payload: None,
            sparse: None,
        },
    ];
    store.insert("test_collection", vectors).unwrap();

    // Wait a bit for async WAL writes to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Simulate crash (don't checkpoint)
    // Create new store instance (simulating restart)
    let store2 = VectorStore::new();
    store2
        .enable_wal(data_dir, Some(wal_config.clone()))
        .await
        .unwrap();

    // Recover from WAL
    let recovered = store2
        .recover_and_replay_wal("test_collection")
        .await
        .unwrap();
    assert_eq!(recovered, 2, "Should recover 2 insert operations");

    // Verify vectors were recovered
    let vec1 = store2.get_vector("test_collection", "vec1").unwrap();
    assert_eq!(vec1.data[0], 1.0);

    let vec2 = store2.get_vector("test_collection", "vec2").unwrap();
    assert_eq!(vec2.data[0], 2.0);
}

#[tokio::test]
async fn test_wal_crash_recovery_update() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig {
        checkpoint_threshold: 1000,
        max_wal_size_mb: 100,
        checkpoint_interval: std::time::Duration::from_secs(300),
        compression: false,
    };
    store
        .enable_wal(data_dir.clone(), Some(wal_config.clone()))
        .await
        .unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
    };
    store.create_collection("test_collection", config).unwrap();

    // Insert vector
    store
        .insert(
            "test_collection",
            vec![Vector {
                id: "vec1".to_string(),
                data: vec![1.0; 384],
                payload: None,
                sparse: None,
            }],
        )
        .unwrap();

    // Update vector
    store
        .update(
            "test_collection",
            Vector {
                id: "vec1".to_string(),
                data: vec![3.0; 384],
                payload: None,
                sparse: None,
            },
        )
        .unwrap();

    // Wait for async WAL writes
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Simulate crash
    let store2 = VectorStore::new();
    store2
        .enable_wal(data_dir, Some(wal_config.clone()))
        .await
        .unwrap();

    // Recover from WAL
    let recovered = store2
        .recover_and_replay_wal("test_collection")
        .await
        .unwrap();
    assert_eq!(recovered, 2, "Should recover 1 insert + 1 update");

    // Verify vector was updated
    let vec1 = store2.get_vector("test_collection", "vec1").unwrap();
    assert_eq!(vec1.data[0], 3.0);
}

#[tokio::test]
async fn test_wal_crash_recovery_delete() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig {
        checkpoint_threshold: 1000,
        max_wal_size_mb: 100,
        checkpoint_interval: std::time::Duration::from_secs(300),
        compression: false,
    };
    store
        .enable_wal(data_dir.clone(), Some(wal_config.clone()))
        .await
        .unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
    };
    store.create_collection("test_collection", config).unwrap();

    // Insert vector
    store
        .insert(
            "test_collection",
            vec![Vector {
                id: "vec1".to_string(),
                data: vec![1.0; 384],
                payload: None,
                sparse: None,
            }],
        )
        .unwrap();

    // Delete vector
    store.delete("test_collection", "vec1").unwrap();

    // Simulate crash
    let store2 = VectorStore::new();
    store2
        .enable_wal(data_dir, Some(wal_config.clone()))
        .await
        .unwrap();

    // Recover from WAL
    let recovered = store2
        .recover_and_replay_wal("test_collection")
        .await
        .unwrap();
    assert_eq!(recovered, 2, "Should recover 1 insert + 1 delete");

    // Verify vector was deleted
    assert!(store2.get_vector("test_collection", "vec1").is_err());
}

#[tokio::test]
async fn test_wal_recover_all_collections() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let wal_config = WALConfig {
        checkpoint_threshold: 1000,
        max_wal_size_mb: 100,
        checkpoint_interval: std::time::Duration::from_secs(300),
        compression: false,
    };
    store
        .enable_wal(data_dir.clone(), Some(wal_config.clone()))
        .await
        .unwrap();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
    };

    // Create multiple collections
    store
        .create_collection("collection1", config.clone())
        .unwrap();
    store.create_collection("collection2", config).unwrap();

    // Insert vectors in both collections
    store
        .insert(
            "collection1",
            vec![Vector {
                id: "vec1".to_string(),
                data: vec![1.0; 384],
                payload: None,
                sparse: None,
            }],
        )
        .unwrap();

    store
        .insert(
            "collection2",
            vec![Vector {
                id: "vec2".to_string(),
                data: vec![2.0; 384],
                payload: None,
                sparse: None,
            }],
        )
        .unwrap();

    // Wait for async WAL writes
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Simulate crash
    let store2 = VectorStore::new();
    store2
        .enable_wal(data_dir, Some(wal_config.clone()))
        .await
        .unwrap();

    // Recover all collections
    let total_recovered = store2.recover_all_from_wal().await.unwrap();
    assert_eq!(total_recovered, 2, "Should recover 2 operations total");

    // Verify both collections were recovered
    let vec1 = store2.get_vector("collection1", "vec1").unwrap();
    assert_eq!(vec1.data[0], 1.0);

    let vec2 = store2.get_vector("collection2", "vec2").unwrap();
    assert_eq!(vec2.data[0], 2.0);
}
