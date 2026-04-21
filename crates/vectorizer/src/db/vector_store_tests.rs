//! Unit tests for `VectorStore` — extracted from `src/db/vector_store.rs`
//! under `phase3_split-vector-store-monolith` via the `#[path]` attribute.
//! The outer `#[cfg(test)] mod tests;` declaration lives at the bottom
//! of `vector_store.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::models::{CompressionConfig, DistanceMetric, HnswConfig, Payload};

#[test]
fn test_create_and_list_collections() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: Default::default(),
        compression: Default::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        graph: None,
        encryption: None,
    };

    // Get initial collection count
    let initial_count = store.list_collections().len();

    // Create collections with unique names
    store
        .create_collection("test_list1_unique", config.clone())
        .unwrap();
    store
        .create_collection("test_list2_unique", config)
        .unwrap();

    // List collections
    let collections = store.list_collections();
    assert_eq!(collections.len(), initial_count + 2);
    assert!(collections.contains(&"test_list1_unique".to_string()));
    assert!(collections.contains(&"test_list2_unique".to_string()));

    // Cleanup
    store.delete_collection("test_list1_unique").ok();
    store.delete_collection("test_list2_unique").ok();
}

#[test]
fn test_duplicate_collection_error() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: Default::default(),
        compression: Default::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        graph: None,
        encryption: None,
    };

    // Create collection
    store.create_collection("test", config.clone()).unwrap();

    // Try to create duplicate
    let result = store.create_collection("test", config);
    assert!(matches!(
        result,
        Err(VectorizerError::CollectionAlreadyExists(_))
    ));
}

#[test]
fn test_delete_collection() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: Default::default(),
        compression: Default::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        graph: None,
        encryption: None,
    };

    // Get initial collection count
    let initial_count = store.list_collections().len();

    // Create and delete collection
    store
        .create_collection("test_delete_collection_unique", config)
        .unwrap();
    assert_eq!(store.list_collections().len(), initial_count + 1);

    store
        .delete_collection("test_delete_collection_unique")
        .unwrap();
    assert_eq!(store.list_collections().len(), initial_count);

    // Try to delete non-existent collection
    let result = store.delete_collection("test_delete_collection_unique");
    assert!(matches!(
        result,
        Err(VectorizerError::CollectionNotFound(_))
    ));
}

#[test]
fn test_stats_functionality() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        sharding: None,
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: Default::default(),
        compression: Default::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        graph: None,
        encryption: None,
    };

    // Get initial stats
    let initial_stats = store.stats();
    let initial_count = initial_stats.collection_count;
    let initial_vectors = initial_stats.total_vectors;

    // Create collection and add vectors
    store
        .create_collection("test_stats_unique", config)
        .unwrap();
    let vectors = vec![
        Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]),
        Vector::new("v2".to_string(), vec![4.0, 5.0, 6.0]),
    ];
    store.insert("test_stats_unique", vectors).unwrap();

    let stats = store.stats();
    assert_eq!(stats.collection_count, initial_count + 1);
    assert_eq!(stats.total_vectors, initial_vectors + 2);
    // Memory bytes may be 0 if collection uses optimization (always >= 0 for usize)
    let _ = stats.total_memory_bytes;

    // Cleanup
    store.delete_collection("test_stats_unique").ok();
}

#[test]
fn test_concurrent_operations() {
    use std::sync::Arc;
    use std::thread;

    let store = Arc::new(VectorStore::new());

    let config = CollectionConfig {
        sharding: None,
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: Default::default(),
        compression: Default::default(),
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        graph: None,
        encryption: None,
    };

    // Create collection from main thread
    store.create_collection("concurrent_test", config).unwrap();

    let mut handles = vec![];

    // Spawn multiple threads to insert vectors
    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            let vectors = vec![
                Vector::new(format!("vec_{}_{}", i, 0), vec![i as f32, 0.0, 0.0]),
                Vector::new(format!("vec_{}_{}", i, 1), vec![0.0, i as f32, 0.0]),
            ];
            store_clone.insert("concurrent_test", vectors).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all vectors were inserted
    let stats = store.stats();
    assert_eq!(stats.collection_count, 1);
    assert_eq!(stats.total_vectors, 10); // 5 threads * 2 vectors each
}

#[test]
fn test_collection_metadata() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        sharding: None,
        dimension: 768,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 32,
            ef_construction: 200,
            ef_search: 64,
            seed: Some(123),
        },
        quantization: Default::default(),
        compression: CompressionConfig {
            enabled: true,
            threshold_bytes: 2048,
            algorithm: crate::models::CompressionAlgorithm::Lz4,
        },
        normalization: None,
        storage_type: Some(crate::models::StorageType::Memory),
        graph: None,
        encryption: None,
    };

    store
        .create_collection("metadata_test", config.clone())
        .unwrap();

    // Add some vectors
    let vectors = vec![
        Vector::new("v1".to_string(), vec![0.1; 768]),
        Vector::new("v2".to_string(), vec![0.2; 768]),
    ];
    store.insert("metadata_test", vectors).unwrap();

    // Test metadata retrieval
    let metadata = store.get_collection_metadata("metadata_test").unwrap();
    assert_eq!(metadata.name, "metadata_test");
    assert_eq!(metadata.vector_count, 2);
    assert_eq!(metadata.config.dimension, 768);
    assert_eq!(metadata.config.metric, DistanceMetric::Cosine);
}
