//! Tests for collection persistence across restarts
//!
//! These tests verify that:
//! - Collections created via API are saved to .vecdb
//! - Collections are correctly loaded after simulated restart
//! - Vectors within collections persist correctly
//! - Empty collections preserve their metadata

use std::sync::Arc;
use tempfile::tempdir;
use vectorizer::db::VectorStore;
use vectorizer::db::auto_save::AutoSaveManager;
use vectorizer::models::{CollectionConfig, DistanceMetric, StorageType, Vector};
use vectorizer::storage::StorageCompactor;

/// Test that a newly created collection can be persisted and reloaded
#[tokio::test]
async fn test_collection_persistence_after_force_save() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    // Set the data directory for the test
    // SAFETY: Test runs in isolation, no concurrent access to env vars
    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.to_str().unwrap()) };

    // Create store and collection
    let store = Arc::new(VectorStore::new());

    let config = CollectionConfig {
        graph: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Memory),
        sharding: None,
    };

    store.create_collection("test_persist", config).unwrap();

    // Verify collection exists in memory
    assert!(store.get_collection("test_persist").is_ok());

    // Force save using compactor
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    let result = compactor.compact_from_memory(&store);
    assert!(result.is_ok(), "Compaction should succeed");

    // Create a new store and load collections
    let store2 = Arc::new(VectorStore::new());

    // Load persisted collections
    let load_result = store2.load_all_persisted_collections();
    assert!(load_result.is_ok(), "Loading should succeed");

    // Verify collection was loaded
    assert!(
        store2.get_collection("test_persist").is_ok(),
        "Collection should exist after reload"
    );

    // Cleanup
    unsafe { std::env::remove_var("VECTORIZER_DATA_DIR") };
}

/// Test that collections with vectors persist correctly
#[tokio::test]
async fn test_collection_with_vectors_persistence() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.to_str().unwrap()) };

    let store = Arc::new(VectorStore::new());

    let config = CollectionConfig {
        graph: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Memory),
        sharding: None,
    };

    store.create_collection("vectors_test", config).unwrap();

    // Insert vectors
    let vectors = vec![
        Vector {
            id: "vec1".to_string(),
            data: vec![1.0; 64],
            payload: None,
            sparse: None,
        },
        Vector {
            id: "vec2".to_string(),
            data: vec![0.5; 64],
            payload: None,
            sparse: None,
        },
    ];

    store.insert("vectors_test", vectors).unwrap();

    // Force save
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    compactor.compact_from_memory(&store).unwrap();

    // Create new store and reload
    let store2 = Arc::new(VectorStore::new());
    store2.load_all_persisted_collections().unwrap();

    // Verify collection and vectors exist
    assert!(store2.get_collection("vectors_test").is_ok());

    let vec1 = store2.get_vector("vectors_test", "vec1");
    assert!(vec1.is_ok(), "Vector vec1 should exist after reload");

    let vec2 = store2.get_vector("vectors_test", "vec2");
    assert!(vec2.is_ok(), "Vector vec2 should exist after reload");

    unsafe { std::env::remove_var("VECTORIZER_DATA_DIR") };
}

/// Test that multiple collections persist correctly
#[tokio::test]
async fn test_multiple_collections_persistence() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.to_str().unwrap()) };

    let store = Arc::new(VectorStore::new());

    // Create multiple collections
    for i in 1..=3 {
        let config = CollectionConfig {
            graph: None,
            dimension: 64,
            metric: DistanceMetric::Cosine,
            quantization: vectorizer::models::QuantizationConfig::default(),
            hnsw_config: vectorizer::models::HnswConfig::default(),
            compression: vectorizer::models::CompressionConfig::default(),
            normalization: None,
            storage_type: Some(StorageType::Memory),
            sharding: None,
        };

        store
            .create_collection(&format!("collection_{i}"), config)
            .unwrap();
    }

    // Force save
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    compactor.compact_from_memory(&store).unwrap();

    // Reload
    let store2 = Arc::new(VectorStore::new());
    store2.load_all_persisted_collections().unwrap();

    // Verify all collections exist
    for i in 1..=3 {
        assert!(
            store2.get_collection(&format!("collection_{i}")).is_ok(),
            "Collection {i} should exist after reload"
        );
    }

    unsafe { std::env::remove_var("VECTORIZER_DATA_DIR") };
}

/// Test that AutoSaveManager mark_changed triggers persistence
#[tokio::test]
async fn test_auto_save_manager_mark_changed() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.to_str().unwrap()) };

    let store = Arc::new(VectorStore::new());

    // Create auto-save manager
    let auto_save = AutoSaveManager::new(store.clone(), 1);

    // Create a collection
    let config = CollectionConfig {
        graph: None,
        dimension: 32,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Memory),
        sharding: None,
    };

    store.create_collection("autosave_test", config).unwrap();

    // Mark changes
    auto_save.mark_changed();

    // Force save
    let result = auto_save.force_save().await;
    assert!(result.is_ok(), "Force save should succeed");

    // Reload and verify
    let store2 = Arc::new(VectorStore::new());
    store2.load_all_persisted_collections().unwrap();

    assert!(
        store2.get_collection("autosave_test").is_ok(),
        "Collection should persist after force_save"
    );

    unsafe { std::env::remove_var("VECTORIZER_DATA_DIR") };
}

/// Test that deleted vectors are persisted correctly
#[tokio::test]
async fn test_vector_deletion_persistence() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.to_str().unwrap()) };

    let store = Arc::new(VectorStore::new());

    let config = CollectionConfig {
        graph: None,
        dimension: 32,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Memory),
        sharding: None,
    };

    store.create_collection("delete_test", config).unwrap();

    // Insert vectors
    let vectors = vec![
        Vector {
            id: "to_keep".to_string(),
            data: vec![1.0; 32],
            payload: None,
            sparse: None,
        },
        Vector {
            id: "to_delete".to_string(),
            data: vec![0.5; 32],
            payload: None,
            sparse: None,
        },
    ];

    store.insert("delete_test", vectors).unwrap();

    // Delete one vector
    store.delete("delete_test", "to_delete").unwrap();

    // Force save
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    compactor.compact_from_memory(&store).unwrap();

    // Reload and verify
    let store2 = Arc::new(VectorStore::new());
    store2.load_all_persisted_collections().unwrap();

    // Kept vector should exist
    assert!(
        store2.get_vector("delete_test", "to_keep").is_ok(),
        "Kept vector should exist after reload"
    );

    // Deleted vector should not exist
    assert!(
        store2.get_vector("delete_test", "to_delete").is_err(),
        "Deleted vector should not exist after reload"
    );

    unsafe { std::env::remove_var("VECTORIZER_DATA_DIR") };
}

/// Test collection metadata persistence
#[tokio::test]
async fn test_collection_metadata_persistence() {
    let temp_dir = tempdir().unwrap();
    let data_dir = temp_dir.path().to_path_buf();

    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.to_str().unwrap()) };

    let store = Arc::new(VectorStore::new());

    let config = CollectionConfig {
        graph: None,
        dimension: 256,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig {
            m: 32,
            ef_construction: 400,
            ..Default::default()
        },
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Memory),
        sharding: None,
    };

    store.create_collection("metadata_test", config).unwrap();

    // Force save
    let compactor = StorageCompactor::new(&data_dir, 6, 1000);
    compactor.compact_from_memory(&store).unwrap();

    // Reload
    let store2 = Arc::new(VectorStore::new());
    store2.load_all_persisted_collections().unwrap();

    // Verify metadata via config
    let meta = store2.get_collection_metadata("metadata_test").unwrap();
    assert_eq!(meta.config.dimension, 256);
    assert_eq!(meta.config.metric, DistanceMetric::Euclidean);

    unsafe { std::env::remove_var("VECTORIZER_DATA_DIR") };
}
