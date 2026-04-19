//! Unit tests for `Collection` — extracted from `src/db/collection.rs`
//! under phase3_split-collection-monolith via the `#[path]` attribute.
//! The module body below is what a `mod tests { ... }` block would
//! contain; the outer `mod tests` declaration lives at the bottom of
//! `collection.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::models::{DistanceMetric, HnswConfig};

fn create_test_collection() -> Collection {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };
    Collection::new("test".to_string(), config)
}

#[test]
fn test_insert_and_get_vector() {
    let collection = create_test_collection();

    let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
    collection.insert(vector.clone()).unwrap();

    let retrieved = collection.get_vector("v1").unwrap();
    assert_eq!(retrieved.id, "v1");
    assert_eq!(retrieved.data, vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_dimension_validation() {
    let collection = create_test_collection();

    // Wrong dimension
    let vector = Vector::new("v1".to_string(), vec![1.0, 2.0]); // 2D instead of 3D
    let result = collection.insert(vector);

    assert!(matches!(
        result,
        Err(VectorizerError::InvalidDimension {
            expected: 3,
            got: 2
        })
    ));
}

#[test]
fn test_update_vector() {
    let collection = create_test_collection();

    // Insert original
    let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
    collection.insert(vector).unwrap();

    // Update
    let updated = Vector::new("v1".to_string(), vec![4.0, 5.0, 6.0]);
    collection.update(updated).unwrap();

    // Verify
    let retrieved = collection.get_vector("v1").unwrap();
    assert_eq!(retrieved.data, vec![4.0, 5.0, 6.0]);
}

#[test]
fn test_delete_vector() {
    let collection = create_test_collection();

    // Insert and delete
    let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
    collection.insert(vector).unwrap();
    assert_eq!(collection.vector_count(), 1);

    collection.delete("v1").unwrap();
    assert_eq!(collection.vector_count(), 0);

    // Try to get deleted vector
    let result = collection.get_vector("v1");
    assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
}

#[test]
fn test_vector_count_with_quantization() {
    // Create collection WITH quantization enabled
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::SQ { bits: 8 }, // QUANTIZED!
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };
    let collection = Collection::new("quantized_test".to_string(), config);

    // Insert vectors
    let vec1 = Vector::new("vec1".to_string(), vec![1.0, 0.0, 0.0]);
    let vec2 = Vector::new("vec2".to_string(), vec![0.0, 1.0, 0.0]);
    let vec3 = Vector::new("vec3".to_string(), vec![0.0, 0.0, 1.0]);

    collection.insert_batch(vec![vec1, vec2, vec3]).unwrap();

    // Vector count MUST be correct even with quantization
    assert_eq!(
        collection.vector_count(),
        3,
        "Vector count should be 3 even with quantization enabled"
    );

    // Delete one vector
    collection.delete("vec2").unwrap();
    assert_eq!(
        collection.vector_count(),
        2,
        "Vector count should be 2 after deleting one quantized vector"
    );
}

#[test]
fn test_vector_count_consistency_quantized_vs_normal() {
    // Test that vector_count() works the same for quantized and non-quantized collections

    // Collection 1: WITH quantization
    let config_quantized = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::SQ { bits: 8 },
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };
    let collection_quantized = Collection::new("quantized".to_string(), config_quantized);

    // Collection 2: WITHOUT quantization
    let config_normal = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 3,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };
    let collection_normal = Collection::new("normal".to_string(), config_normal);

    // Insert same vectors to both
    let vectors = vec![
        Vector::new("v1".to_string(), vec![1.0, 0.0, 0.0]),
        Vector::new("v2".to_string(), vec![0.0, 1.0, 0.0]),
        Vector::new("v3".to_string(), vec![0.0, 0.0, 1.0]),
        Vector::new("v4".to_string(), vec![1.0, 1.0, 0.0]),
        Vector::new("v5".to_string(), vec![0.5, 0.5, 0.5]),
    ];

    collection_quantized.insert_batch(vectors.clone()).unwrap();
    collection_normal.insert_batch(vectors).unwrap();

    // Both should have the same count
    assert_eq!(
        collection_quantized.vector_count(),
        5,
        "Quantized collection should have 5 vectors"
    );
    assert_eq!(
        collection_normal.vector_count(),
        5,
        "Normal collection should have 5 vectors"
    );
    assert_eq!(
        collection_quantized.vector_count(),
        collection_normal.vector_count(),
        "Both collections should have the same vector count"
    );
}

#[test]
fn test_collection_creation() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: None,
    };

    let collection = Collection::new("test_coll".to_string(), config);

    assert_eq!(collection.name(), "test_coll");
    assert_eq!(collection.config().dimension, 128);
    assert_eq!(collection.vector_count(), 0);
}

#[test]
fn test_collection_insert_single() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);
    let vector = Vector::new("v1".to_string(), vec![0.1; 128]);

    let result = collection.insert(vector);
    assert!(result.is_ok());
    assert_eq!(collection.vector_count(), 1);
}

#[test]
fn test_collection_insert_batch() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);
    let vectors = vec![
        Vector::new("v1".to_string(), vec![0.1; 64]),
        Vector::new("v2".to_string(), vec![0.2; 64]),
        Vector::new("v3".to_string(), vec![0.3; 64]),
    ];

    let result = collection.insert_batch(vectors);
    assert!(result.is_ok());
    assert_eq!(collection.vector_count(), 3);
}

#[test]
fn test_collection_get_vector() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);
    let vector = Vector::new("v1".to_string(), vec![0.5; 64]);

    collection.insert(vector.clone()).unwrap();

    let retrieved = collection.get_vector("v1");
    assert!(retrieved.is_ok());

    let retrieved_vec = retrieved.unwrap();
    assert_eq!(retrieved_vec.id, "v1");
    assert_eq!(retrieved_vec.data.len(), 64);
}

#[test]
fn test_collection_get_nonexistent() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);
    let result = collection.get_vector("nonexistent");

    assert!(result.is_err());
}

#[test]
fn test_collection_delete() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    // Insert vectors
    for i in 0..5 {
        let vector = Vector::new(format!("v{}", i), vec![0.1 * (i as f32); 64]);
        collection.insert(vector).unwrap();
    }

    assert_eq!(collection.vector_count(), 5);

    // Delete one
    let result = collection.delete("v2");
    assert!(result.is_ok());
    assert_eq!(collection.vector_count(), 4);

    // Try to get deleted vector
    assert!(collection.get_vector("v2").is_err());
}

#[test]
fn test_collection_update() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);
    let vector = Vector::new("v1".to_string(), vec![0.1; 64]);

    collection.insert(vector).unwrap();

    // Update vector
    let new_vector = Vector::new("v1".to_string(), vec![0.5; 64]);
    let result = collection.update(new_vector);

    assert!(result.is_ok());

    // Verify vector still exists after update
    let updated = collection.get_vector("v1");
    assert!(updated.is_ok());
}

#[test]
fn test_collection_search() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    // Insert vectors
    for i in 0..20 {
        let mut vec_data = vec![0.0; 64];
        vec_data[0] = i as f32 * 0.1;
        let vector = Vector::new(format!("v{}", i), vec_data);
        collection.insert(vector).unwrap();
    }

    // Search
    let query = vec![0.5; 64];
    let results = collection.search(&query, 5);

    assert!(results.is_ok());
    let results = results.unwrap();
    assert!(results.len() <= 5);
}

#[test]
fn test_collection_memory_usage() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    // Insert vectors
    for i in 0..10 {
        let vector = Vector::new(format!("v{}", i), vec![0.1; 128]);
        collection.insert(vector).unwrap();
    }

    let (index_size, payload_size, total_size) = collection.calculate_memory_usage();
    assert!(total_size > 0);
    assert!(index_size > 0);
}

#[test]
fn test_collection_metadata() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 256,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: None,
    };

    let collection = Collection::new("metadata_test".to_string(), config);

    let metadata = collection.metadata();
    assert_eq!(metadata.name, "metadata_test");
    assert_eq!(metadata.config.dimension, 256);
    assert_eq!(metadata.vector_count, 0);
}

#[test]
fn test_collection_different_metrics() {
    // Test Cosine
    let config_cosine = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };
    let coll_cosine = Collection::new("cosine".to_string(), config_cosine);
    assert_eq!(coll_cosine.config().metric, DistanceMetric::Cosine);

    // Test Euclidean
    let config_euclidean = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };
    let coll_euclidean = Collection::new("euclidean".to_string(), config_euclidean);
    assert_eq!(coll_euclidean.config().metric, DistanceMetric::Euclidean);

    // Test DotProduct
    let config_dot = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::DotProduct,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };
    let coll_dot = Collection::new("dot".to_string(), config_dot);
    assert_eq!(coll_dot.config().metric, DistanceMetric::DotProduct);
}

#[test]
fn test_collection_with_quantization_sq() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::SQ { bits: 8 },
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: None,
    };

    let collection = Collection::new("quantized_sq".to_string(), config);

    // Insert vectors
    for i in 0..10 {
        let vector = Vector::new(format!("v{}", i), vec![0.1 * (i as f32); 128]);
        collection.insert(vector).unwrap();
    }

    assert_eq!(collection.vector_count(), 10);

    // Search should still work with quantized vectors
    let query = vec![0.5; 128];
    let results = collection.search(&query, 5);
    assert!(results.is_ok());
}

#[test]
fn test_collection_update_nonexistent() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);
    let vector = Vector::new("nonexistent".to_string(), vec![0.1; 64]);

    let result = collection.update(vector);
    assert!(result.is_err());
}

#[test]
fn test_collection_delete_nonexistent() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);
    let result = collection.delete("nonexistent");

    assert!(result.is_err());
}

#[test]
fn test_collection_dimension_validation() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    // Try to insert vector with wrong dimension
    let wrong_dim = Vector::new("v1".to_string(), vec![0.1; 64]);
    let result = collection.insert(wrong_dim);

    assert!(result.is_err());
}

#[test]
fn test_collection_get_all_vectors_ids() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    // Insert some vectors
    for i in 0..5 {
        let vector = Vector::new(format!("v{}", i), vec![0.1; 64]);
        collection.insert(vector).unwrap();
    }

    let all_vectors = collection.get_all_vectors();
    assert_eq!(all_vectors.len(), 5);

    let ids: Vec<String> = all_vectors.iter().map(|v| v.id.clone()).collect();
    assert!(ids.contains(&"v0".to_string()));
    assert!(ids.contains(&"v4".to_string()));
}

#[test]
fn test_collection_embedding_type() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 512,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: None,
    };

    let collection =
        Collection::new_with_embedding_type("test".to_string(), config, "bert".to_string());

    assert_eq!(collection.get_embedding_type(), "bert");
}

#[test]
fn test_collection_search_empty() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    // Search in empty collection
    let query = vec![0.1; 64];
    let results = collection.search(&query, 10);

    assert!(results.is_ok());
    assert_eq!(results.unwrap().len(), 0);
}

#[test]
fn test_collection_concurrent_inserts() {
    use std::thread;

    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: None,
    };

    let collection = Arc::new(Collection::new("concurrent".to_string(), config));

    let mut handles = vec![];

    for i in 0..10 {
        let coll = Arc::clone(&collection);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let vector = Vector::new(
                    format!("v_{}_{}", i, j),
                    vec![0.1 * ((i * 10 + j) as f32); 64],
                );
                coll.insert(vector).unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(collection.vector_count(), 100);
}

#[test]
fn test_collection_search_with_limit() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 64,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    // Insert 50 vectors
    for i in 0..50 {
        let vector = Vector::new(format!("v{}", i), vec![0.01 * (i as f32); 64]);
        collection.insert(vector).unwrap();
    }

    // Search with limit 10
    let query = vec![0.25; 64];
    let results = collection.search(&query, 10);

    assert!(results.is_ok());
    let results = results.unwrap();
    assert!(results.len() <= 10);
}

#[test]
fn test_collection_get_all_vectors() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 32,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    // Insert vectors
    for i in 0..15 {
        let vector = Vector::new(format!("v{}", i), vec![0.1; 32]);
        collection.insert(vector).unwrap();
    }

    let all_vectors = collection.get_all_vectors();
    assert_eq!(all_vectors.len(), 15);
}

#[test]
fn test_collection_metadata_updates() {
    let config = CollectionConfig {
        graph: None,
        sharding: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: crate::models::QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        encryption: None,
        storage_type: Some(crate::models::StorageType::Memory),
    };

    let collection = Collection::new("test".to_string(), config);

    let metadata1 = collection.metadata();
    let created_at1 = metadata1.created_at;

    // Insert a vector
    let vector = Vector::new("v1".to_string(), vec![0.1; 128]);
    collection.insert(vector).unwrap();

    let metadata2 = collection.metadata();

    // created_at should remain the same
    assert_eq!(metadata1.created_at, created_at1);

    // vector_count should change
    assert_eq!(metadata2.vector_count, 1);
}
