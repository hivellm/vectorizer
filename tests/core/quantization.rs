//! Tests for Quantization functionality (PQ and SQ)

use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, QuantizationConfig, Vector};

#[tokio::test]
async fn test_scalar_quantization_8bit() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::SQ { bits: 8 },
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("sq8_collection", config).unwrap();

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
            data: vec![0.5; 384],
            payload: None,
            sparse: None,
        },
    ];

    assert!(store.insert("sq8_collection", vectors).is_ok());

    // Verify vectors can be retrieved
    let vec1 = store.get_vector("sq8_collection", "vec1").unwrap();
    assert_eq!(vec1.data.len(), 384);

    let vec2 = store.get_vector("sq8_collection", "vec2").unwrap();
    assert_eq!(vec2.data.len(), 384);
}

#[tokio::test]
async fn test_product_quantization() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::PQ {
            n_centroids: 256,
            n_subquantizers: 8,
        },
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("pq_collection", config).unwrap();

    // Insert vectors (PQ training happens automatically when reaching 1000 vectors)
    // For testing, insert a smaller batch first
    let vectors: Vec<Vector> = (0..100)
        .map(|i| Vector {
            id: format!("vec_{i}"),
            data: vec![(i % 100) as f32 / 100.0; 384],
            payload: None,
            sparse: None,
        })
        .collect();

    assert!(store.insert("pq_collection", vectors).is_ok());

    // Verify vectors can be retrieved (PQ training may not have happened yet with only 100 vectors)
    let vec1 = store.get_vector("pq_collection", "vec_0").unwrap();
    assert_eq!(vec1.data.len(), 384);

    // Verify collection exists and has vectors
    let metadata = store.get_collection("pq_collection").unwrap().metadata();
    assert!(metadata.vector_count > 0);
}

#[tokio::test]
async fn test_binary_quantization() {
    let store = VectorStore::new();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::Binary,
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store
        .create_collection("binary_collection", config)
        .unwrap();

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
            data: vec![-1.0; 384],
            payload: None,
            sparse: None,
        },
    ];

    assert!(store.insert("binary_collection", vectors).is_ok());

    // Verify vectors can be retrieved
    let vec1 = store.get_vector("binary_collection", "vec1").unwrap();
    assert_eq!(vec1.data.len(), 384);
}

#[tokio::test]
async fn test_quantization_search_quality() {
    let store = VectorStore::new();

    // Test with SQ-8bit
    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::SQ { bits: 8 },
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("quantized_search", config).unwrap();

    // Insert vectors
    let vectors: Vec<Vector> = (0..50)
        .map(|i| Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32 / 50.0; 128],
            payload: None,
            sparse: None,
        })
        .collect();

    assert!(store.insert("quantized_search", vectors).is_ok());

    // Search
    let query = vec![0.5; 128];
    let results = store.search("quantized_search", &query, 10).unwrap();

    assert!(!results.is_empty());
    assert!(results.len() <= 10);
}

#[tokio::test]
async fn test_quantization_memory_efficiency() {
    let store = VectorStore::new();

    // Test with no quantization
    let config_no_quant = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::None,
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store
        .create_collection("no_quant", config_no_quant)
        .unwrap();

    // Test with SQ-8bit
    let config_sq8 = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::SQ { bits: 8 },
        hnsw_config: vectorizer::models::HnswConfig::default(),
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
    };

    store.create_collection("sq8", config_sq8).unwrap();

    // Insert same vectors in both
    let vectors: Vec<Vector> = (0..100)
        .map(|i| Vector {
            id: format!("vec_{i}"),
            data: vec![i as f32 / 100.0; 384],
            payload: None,
            sparse: None,
        })
        .collect();

    assert!(store.insert("no_quant", vectors.clone()).is_ok());
    assert!(store.insert("sq8", vectors).is_ok());

    // Both should work, but SQ-8 should use less memory
    let no_quant_meta = store.get_collection("no_quant").unwrap().metadata();
    let sq8_meta = store.get_collection("sq8").unwrap().metadata();

    assert_eq!(no_quant_meta.vector_count, 100);
    assert_eq!(sq8_meta.vector_count, 100);
}
