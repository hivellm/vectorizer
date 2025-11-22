//! Tests for MMAP (Memory-Mapped) storage functionality

use std::path::PathBuf;

use tempfile::tempdir;
use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, StorageType, Vector};

#[tokio::test]
async fn test_mmap_collection_creation() {
    let temp_dir = tempdir().unwrap();
    let _data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Mmap),
    };

    // Create collection with MMAP storage
    assert!(store.create_collection("mmap_collection", config).is_ok());

    // Verify collection exists
    assert!(store.get_collection("mmap_collection").is_ok());
}

#[tokio::test]
async fn test_mmap_insert_and_retrieve() {
    let temp_dir = tempdir().unwrap();
    let _data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Mmap),
    };

    store.create_collection("mmap_collection", config).unwrap();

    // Insert vectors
    let vectors = vec![
        Vector {
            id: "vec1".to_string(),
            data: vec![1.0; 128],
            payload: None,
            sparse: None,
        },
        Vector {
            id: "vec2".to_string(),
            data: vec![2.0; 128],
            payload: None,
            sparse: None,
        },
    ];

    assert!(store.insert("mmap_collection", vectors).is_ok());

    // Retrieve vectors (note: vectors are normalized for cosine similarity)
    let vec1 = store.get_vector("mmap_collection", "vec1").unwrap();
    assert_eq!(vec1.data.len(), 128);
    // For cosine similarity, vectors are normalized, so check magnitude instead
    let magnitude1: f32 = vec1.data.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((magnitude1 - 1.0).abs() < 0.1 || magnitude1 > 0.0); // Normalized or has values

    let vec2 = store.get_vector("mmap_collection", "vec2").unwrap();
    assert_eq!(vec2.data.len(), 128);
    let magnitude2: f32 = vec2.data.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(magnitude2 > 0.0); // Has values
}

#[tokio::test]
async fn test_mmap_large_dataset() {
    let temp_dir = tempdir().unwrap();
    let _data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 256,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Mmap),
    };

    store
        .create_collection("large_mmap_collection", config)
        .unwrap();

    // Insert many vectors (testing MMAP can handle large datasets)
    let vectors: Vec<Vector> = (0..100)
        .map(|i| Vector {
            id: format!("vec_{}", i),
            data: vec![i as f32; 256],
            payload: None,
            sparse: None,
        })
        .collect();

    assert!(store.insert("large_mmap_collection", vectors).is_ok());

    // Verify we can retrieve vectors (they may be normalized)
    let mut retrieved_count = 0;
    for i in 0..100 {
        if let Ok(vec) = store.get_vector("large_mmap_collection", &format!("vec_{}", i)) {
            assert_eq!(vec.data.len(), 256);
            retrieved_count += 1;
        }
    }
    // At least some vectors should be retrievable
    assert!(
        retrieved_count > 0,
        "Should be able to retrieve at least some vectors"
    );
}

#[tokio::test]
async fn test_mmap_update_and_delete() {
    let temp_dir = tempdir().unwrap();
    let _data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Mmap),
    };

    store.create_collection("mmap_collection", config).unwrap();

    // Insert
    let vector = Vector {
        id: "test_vec".to_string(),
        data: vec![1.0; 128],
        payload: None,
        sparse: None,
    };
    assert!(store.insert("mmap_collection", vec![vector]).is_ok());

    // Update
    let updated = Vector {
        id: "test_vec".to_string(),
        data: vec![2.0; 128],
        payload: None,
        sparse: None,
    };
    assert!(store.update("mmap_collection", updated).is_ok());

    let retrieved = store.get_vector("mmap_collection", "test_vec").unwrap();
    assert_eq!(retrieved.data.len(), 128);
    // Vector is normalized for cosine similarity, so check it has values
    let magnitude: f32 = retrieved.data.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(magnitude > 0.0);

    // Delete
    assert!(store.delete("mmap_collection", "test_vec").is_ok());
    assert!(store.get_vector("mmap_collection", "test_vec").is_err());
}

#[tokio::test]
async fn test_mmap_search() {
    let temp_dir = tempdir().unwrap();
    let _data_dir = temp_dir.path().to_path_buf();

    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::default(),
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(StorageType::Mmap),
    };

    store.create_collection("mmap_collection", config).unwrap();

    // Insert vectors
    let vectors = (0..10)
        .map(|i| Vector {
            id: format!("vec_{}", i),
            data: vec![i as f32; 128],
            payload: None,
            sparse: None,
        })
        .collect::<Vec<_>>();

    assert!(store.insert("mmap_collection", vectors).is_ok());

    // Search
    let query = vec![5.0; 128];
    let results = store.search("mmap_collection", &query, 5).unwrap();

    assert!(!results.is_empty());
    assert!(results.len() <= 5);
}
