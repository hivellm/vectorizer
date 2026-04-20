//! Extracted unit tests (phase3 test-extraction).
//!
//! Wired from `src/persistence/dynamic.rs` via the `#[path]` attribute.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use tempfile::tempdir;

use super::*;
use crate::models::QuantizationConfig;

async fn create_test_persistence() -> DynamicCollectionPersistence {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap(); // Create directory

    let config = PersistenceConfig {
        data_dir,
        ..Default::default()
    };

    // Create mock vector store
    let vector_store = Arc::new(VectorStore::new());

    DynamicCollectionPersistence::new(config, vector_store)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_create_dynamic_collection() {
    let persistence = create_test_persistence().await;

    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::default(),
        hnsw_config: crate::models::HnswConfig::default(),
        compression: crate::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        encryption: None,
    };

    let metadata = persistence
        .create_collection(
            "test-collection".to_string(),
            config,
            Some("user123".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(metadata.name, "test-collection");
    assert_eq!(metadata.collection_type, CollectionType::Dynamic);
    assert!(!metadata.is_read_only);
    assert!(metadata.is_dynamic());
}

#[tokio::test]
async fn test_collection_exists() {
    let persistence = create_test_persistence().await;

    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::default(),
        hnsw_config: crate::models::HnswConfig::default(),
        compression: crate::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        encryption: None,
    };

    // Collection doesn't exist yet
    assert!(!persistence.collection_exists("test-collection").await);

    // Create collection
    persistence
        .create_collection("test-collection".to_string(), config, None)
        .await
        .unwrap();

    // Collection should exist now
    assert!(persistence.collection_exists("test-collection").await);
}

#[tokio::test]
async fn test_list_collections() {
    let persistence = create_test_persistence().await;

    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::default(),
        hnsw_config: crate::models::HnswConfig::default(),
        compression: crate::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        encryption: None,
    };

    // Initially empty
    let collections = persistence.list_collections().await.unwrap();
    assert_eq!(collections.len(), 0);

    // Create collections
    persistence
        .create_collection("collection1".to_string(), config.clone(), None)
        .await
        .unwrap();
    persistence
        .create_collection("collection2".to_string(), config, None)
        .await
        .unwrap();

    // Should have 2 collections
    let collections = persistence.list_collections().await.unwrap();
    assert_eq!(collections.len(), 2);
}

#[tokio::test]
async fn test_transaction_lifecycle() {
    let persistence = create_test_persistence().await;

    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::default(),
        hnsw_config: crate::models::HnswConfig::default(),
        compression: crate::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        encryption: None,
    };

    let metadata = persistence
        .create_collection("test-collection".to_string(), config, None)
        .await
        .unwrap();

    // Begin transaction
    let transaction_id = persistence.begin_transaction(&metadata.id).await.unwrap();

    // Add operation
    let operation = Operation::InsertVector {
        collection_name: metadata.id.clone(),
        vector_id: "vec1".to_string(),
        data: vec![1.0, 2.0, 3.0],
        metadata: std::collections::HashMap::new(),
    };
    persistence
        .add_to_transaction(transaction_id, operation)
        .await
        .unwrap();

    // Commit transaction
    persistence
        .commit_transaction(transaction_id)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_delete_collection() {
    let persistence = create_test_persistence().await;

    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::default(),
        hnsw_config: crate::models::HnswConfig::default(),
        compression: crate::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        encryption: None,
    };

    // Create collection
    let metadata = persistence
        .create_collection("test-collection".to_string(), config, None)
        .await
        .unwrap();

    // Verify it exists
    assert!(persistence.collection_exists("test-collection").await);

    // Delete collection
    persistence
        .delete_collection("test-collection")
        .await
        .unwrap();

    // Verify it's gone
    assert!(!persistence.collection_exists("test-collection").await);
}

#[tokio::test]
async fn test_get_stats() {
    let persistence = create_test_persistence().await;

    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::default(),
        hnsw_config: crate::models::HnswConfig::default(),
        compression: crate::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        encryption: None,
    };

    // Create some collections
    persistence
        .create_collection("collection1".to_string(), config.clone(), None)
        .await
        .unwrap();
    persistence
        .create_collection("collection2".to_string(), config, None)
        .await
        .unwrap();

    let stats = persistence.get_stats().await.unwrap();
    assert_eq!(stats.total_collections, 2);
    assert_eq!(stats.total_vectors, 0); // No vectors inserted yet
    assert_eq!(stats.total_documents, 0);
}
