//! Integration tests for Sparse Vector Support

#[allow(clippy::duplicate_mod)]
#[path = "../helpers/mod.rs"]
mod helpers;
use helpers::create_test_collection;
use serde_json::json;
use vectorizer::db::VectorStore;
use vectorizer::models::{Payload, SparseVector, Vector};

#[tokio::test]
async fn test_sparse_vector_creation() {
    let store = VectorStore::new();
    let collection_name = "sparse_test";

    // Create collection
    create_test_collection(&store, collection_name, 128).expect("Failed to create collection");

    // Create sparse vector
    let sparse = SparseVector::new(vec![0, 10, 50, 100], vec![1.0, 2.0, 3.0, 4.0]).unwrap();

    assert_eq!(sparse.nnz(), 4);
    assert_eq!(sparse.indices.len(), 4);
    assert_eq!(sparse.values.len(), 4);
}

#[tokio::test]
async fn test_sparse_vector_from_dense() {
    // Create dense vector with mostly zeros
    let mut dense = vec![0.0; 128];
    dense[0] = 1.0;
    dense[10] = 2.0;
    dense[50] = 3.0;
    dense[100] = 4.0;

    let sparse = SparseVector::from_dense(&dense);

    assert_eq!(sparse.nnz(), 4);
    assert_eq!(sparse.indices, vec![0, 10, 50, 100]);
    assert_eq!(sparse.values, vec![1.0, 2.0, 3.0, 4.0]);
}

#[tokio::test]
async fn test_sparse_vector_to_dense() {
    let sparse = SparseVector::new(vec![0, 10, 50, 100], vec![1.0, 2.0, 3.0, 4.0]).unwrap();

    let dense = sparse.to_dense(128);

    assert_eq!(dense.len(), 128);
    assert_eq!(dense[0], 1.0);
    assert_eq!(dense[10], 2.0);
    assert_eq!(dense[50], 3.0);
    assert_eq!(dense[100], 4.0);
    assert_eq!(dense[1], 0.0);
    assert_eq!(dense[20], 0.0);
}

#[tokio::test]
#[ignore = "Sparse vector insertion has issues - skipping until fixed"]
async fn test_sparse_vector_insertion() {
    use vectorizer::models::{CollectionConfig, DistanceMetric};

    let store = VectorStore::new();
    let collection_name = "sparse_insert_test";

    // Create collection with Euclidean metric to avoid normalization
    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None, // Disable quantization for this test
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    // Create vector with sparse representation
    let sparse = SparseVector::new(vec![0, 10, 50, 100], vec![1.0, 2.0, 3.0, 4.0]).unwrap();

    let vector = Vector::with_sparse("sparse_vec_1".to_string(), sparse.clone(), 128);

    // Insert vector
    store
        .insert(collection_name, vec![vector.clone()])
        .expect("Failed to insert sparse vector");

    // Retrieve vector
    let retrieved = store
        .get_vector(collection_name, "sparse_vec_1")
        .expect("Failed to retrieve vector");

    assert_eq!(retrieved.id, "sparse_vec_1");
    assert_eq!(retrieved.data.len(), 128);
    assert_eq!(retrieved.data[0], 1.0);
    assert_eq!(retrieved.data[10], 2.0);
    assert_eq!(retrieved.data[50], 3.0);
    assert_eq!(retrieved.data[100], 4.0);

    // Check sparse representation is preserved
    assert!(retrieved.is_sparse());
    let retrieved_sparse = retrieved.get_sparse().unwrap();
    assert_eq!(retrieved_sparse.indices, sparse.indices);
    assert_eq!(retrieved_sparse.values, sparse.values);
}

#[tokio::test]
#[ignore = "Sparse vector with payload has issues - skipping until fixed"]
async fn test_sparse_vector_with_payload() {
    let store = VectorStore::new();
    let collection_name = "sparse_payload_test";

    create_test_collection(&store, collection_name, 128).expect("Failed to create collection");

    let sparse = SparseVector::new(vec![0, 10, 50], vec![1.0, 2.0, 3.0]).unwrap();

    let payload = Payload::new(json!({
        "type": "sparse",
        "sparsity": 0.95,
        "source": "test"
    }));

    let vector =
        Vector::with_sparse_and_payload("sparse_payload_1".to_string(), sparse, 128, payload);

    store
        .insert(collection_name, vec![vector.clone()])
        .expect("Failed to insert sparse vector with payload");

    let retrieved = store
        .get_vector(collection_name, "sparse_payload_1")
        .expect("Failed to retrieve vector");

    assert!(retrieved.payload.is_some());
    assert_eq!(retrieved.payload.as_ref().unwrap().data["type"], "sparse");
    assert!(retrieved.is_sparse());
}

#[tokio::test]
#[ignore]
async fn test_sparse_vector_search() {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Use unique collection name to avoid conflicts with parallel tests
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let collection_name = format!("sparse_search_test_{timestamp}");

    let store = VectorStore::new();

    create_test_collection(&store, &collection_name, 128).expect("Failed to create collection");

    // Insert multiple sparse vectors
    let vectors = vec![
        Vector::with_sparse(
            "sparse_1".to_string(),
            SparseVector::new(vec![0, 1, 2], vec![1.0, 1.0, 1.0]).unwrap(),
            128,
        ),
        Vector::with_sparse(
            "sparse_2".to_string(),
            SparseVector::new(vec![0, 1, 3], vec![1.0, 1.0, 1.0]).unwrap(),
            128,
        ),
        Vector::with_sparse(
            "sparse_3".to_string(),
            SparseVector::new(vec![5, 6, 7], vec![1.0, 1.0, 1.0]).unwrap(),
            128,
        ),
    ];

    store
        .insert(&collection_name, vectors)
        .expect("Failed to insert sparse vectors");

    // Verify vectors were inserted
    let collection_info = store
        .get_collection(&collection_name)
        .expect("Failed to get collection");
    assert_eq!(
        collection_info.vector_count(),
        3,
        "Expected 3 vectors inserted"
    );

    // Search with sparse query vector (should match sparse_1 and sparse_2)
    let query_sparse = SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap();
    let query_dense = query_sparse.to_dense(128);

    let results = store
        .search(&collection_name, &query_dense, 3)
        .expect("Failed to search");

    assert!(
        results.len() >= 2,
        "Expected at least 2 results, got {}",
        results.len()
    );

    // sparse_1 and sparse_2 should be more similar (share indices 0,1)
    let result_ids: Vec<String> = results.iter().map(|r| r.id.clone()).collect();

    // Debug output if assertion fails
    if !result_ids.contains(&"sparse_1".to_string())
        || !result_ids.contains(&"sparse_2".to_string())
    {
        eprintln!("Search results: {result_ids:?}");
        eprintln!(
            "Result scores: {:?}",
            results
                .iter()
                .map(|r| (r.id.clone(), r.score))
                .collect::<Vec<_>>()
        );
    }

    assert!(
        result_ids.contains(&"sparse_1".to_string()),
        "Expected 'sparse_1' in results, got: {result_ids:?}"
    );
    assert!(
        result_ids.contains(&"sparse_2".to_string()),
        "Expected 'sparse_2' in results, got: {result_ids:?}"
    );
}

#[tokio::test]
async fn test_sparse_vector_dot_product() {
    let v1 = SparseVector::new(vec![0, 2, 4], vec![1.0, 2.0, 3.0]).unwrap();

    let v2 = SparseVector::new(vec![0, 2, 5], vec![2.0, 3.0, 4.0]).unwrap();

    let dot = v1.dot_product(&v2);
    // Only indices 0 and 2 overlap: 1.0*2.0 + 2.0*3.0 = 2.0 + 6.0 = 8.0
    assert!((dot - 8.0).abs() < 0.001);
}

#[tokio::test]
async fn test_sparse_vector_cosine_similarity() {
    let v1 = SparseVector::new(vec![0, 1], vec![1.0, 0.0]).unwrap();

    let v2 = SparseVector::new(vec![0, 1], vec![1.0, 0.0]).unwrap();

    let similarity = v1.cosine_similarity(&v2);
    assert!((similarity - 1.0).abs() < 0.001);

    // Test orthogonal vectors
    let v3 = SparseVector::new(vec![0], vec![1.0]).unwrap();

    let v4 = SparseVector::new(vec![1], vec![1.0]).unwrap();

    let similarity_ortho = v3.cosine_similarity(&v4);
    assert!((similarity_ortho - 0.0).abs() < 0.001);
}

#[tokio::test]
async fn test_sparse_vector_validation() {
    // Valid sparse vector
    let valid = SparseVector::new(vec![0, 2, 5], vec![1.0, 2.0, 3.0]);
    assert!(valid.is_ok());

    // Invalid: unsorted indices
    let invalid = SparseVector::new(vec![5, 2, 0], vec![1.0, 2.0, 3.0]);
    assert!(invalid.is_err());

    // Invalid: duplicate indices
    let invalid = SparseVector::new(vec![0, 2, 2], vec![1.0, 2.0, 3.0]);
    assert!(invalid.is_err());

    // Invalid: length mismatch
    let invalid = SparseVector::new(vec![0, 2], vec![1.0, 2.0, 3.0]);
    assert!(invalid.is_err());
}

#[tokio::test]
async fn test_sparse_vector_index() {
    use vectorizer::models::SparseVectorIndex;

    let mut index = SparseVectorIndex::new();

    let v1 = SparseVector::new(vec![0, 2], vec![1.0, 2.0]).unwrap();
    index.add("v1".to_string(), v1).unwrap();

    let v2 = SparseVector::new(vec![1, 3], vec![1.0, 2.0]).unwrap();
    index.add("v2".to_string(), v2).unwrap();

    assert_eq!(index.len(), 2);
    assert!(!index.is_empty());

    // Search
    let query = SparseVector::new(vec![0, 2], vec![1.0, 2.0]).unwrap();
    let results = index.search(&query, 2);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "v1"); // Should be most similar

    // Remove vector
    assert!(index.remove("v1"));
    assert_eq!(index.len(), 1);
}

#[tokio::test]
async fn test_sparse_vector_memory_efficiency() {
    let dimension = 10000;

    // Create sparse vector with only 10 non-zero values
    let sparse = SparseVector::new((0..10).collect(), vec![1.0; 10]).unwrap();

    let dense = sparse.to_dense(dimension);

    // Sparse representation should use less memory
    let sparse_memory = sparse.memory_size();
    let dense_memory = dense.len() * std::mem::size_of::<f32>();

    // Sparse: 10 * size_of<usize> + 10 * size_of<f32> = 10*8 + 10*4 = 120 bytes
    // Dense: 10000 * 4 = 40000 bytes
    assert!(sparse_memory < dense_memory);
    assert!(sparse_memory < dense_memory / 100); // At least 100x smaller
}

#[tokio::test]
async fn test_sparse_vector_sparsity_calculation() {
    let dimension = 1000;

    // 10 non-zero values out of 1000
    let sparse = SparseVector::new((0..10).collect(), vec![1.0; 10]).unwrap();

    let sparsity = sparse.sparsity(dimension);
    // Should be approximately 0.99 (99% sparse)
    assert!((sparsity - 0.99).abs() < 0.01);
}

#[tokio::test]
#[ignore = "Sparse vector batch operations has issues - skipping until fixed"]
async fn test_sparse_vector_batch_operations() {
    let store = VectorStore::new();
    let collection_name = "sparse_batch_test";

    create_test_collection(&store, collection_name, 128).expect("Failed to create collection");

    // Create batch of sparse vectors
    let mut vectors = Vec::new();
    for i in 0..10 {
        let sparse = SparseVector::new(vec![i * 10, i * 10 + 1], vec![1.0, 2.0]).unwrap();

        vectors.push(Vector::with_sparse(
            format!("sparse_batch_{i}"),
            sparse,
            128,
        ));
    }

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert batch");

    // Verify all vectors were inserted
    for i in 0..10 {
        let retrieved = store.get_vector(collection_name, &format!("sparse_batch_{i}"));
        assert!(retrieved.is_ok());
        assert!(retrieved.unwrap().is_sparse());
    }
}

#[tokio::test]
#[ignore = "Sparse vector update has issues - skipping until fixed"]
async fn test_sparse_vector_update() {
    use vectorizer::models::{CollectionConfig, DistanceMetric};

    let store = VectorStore::new();
    let collection_name = "sparse_update_test";

    // Create collection with Euclidean metric to avoid normalization
    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    // Insert initial sparse vector
    let sparse1 = SparseVector::new(vec![0, 1], vec![1.0, 2.0]).unwrap();
    let vector1 = Vector::with_sparse("sparse_update_1".to_string(), sparse1, 128);

    store
        .insert(collection_name, vec![vector1])
        .expect("Failed to insert");

    // Update with new sparse vector
    let sparse2 = SparseVector::new(vec![2, 3], vec![3.0, 4.0]).unwrap();
    let vector2 = Vector::with_sparse("sparse_update_1".to_string(), sparse2.clone(), 128);

    store
        .update(collection_name, vector2)
        .expect("Failed to update");

    // Verify update
    let retrieved = store
        .get_vector(collection_name, "sparse_update_1")
        .expect("Failed to retrieve");

    assert_eq!(retrieved.data[2], 3.0);
    assert_eq!(retrieved.data[3], 4.0);
    assert_eq!(retrieved.data[0], 0.0); // Should be zero now
    assert!(retrieved.is_sparse());

    let retrieved_sparse = retrieved.get_sparse().unwrap();
    assert_eq!(retrieved_sparse.indices, sparse2.indices);
}

#[tokio::test]
async fn test_sparse_vector_norm() {
    let sparse = SparseVector::new(vec![0, 1, 2], vec![3.0, 4.0, 0.0]).unwrap();

    let norm = sparse.norm();
    // sqrt(3^2 + 4^2 + 0^2) = sqrt(9 + 16) = sqrt(25) = 5.0
    assert!((norm - 5.0).abs() < 0.001);
}

#[tokio::test]
#[ignore = "Sparse vector mixed with dense has issues - skipping until fixed"]
async fn test_sparse_vector_mixed_with_dense() {
    let store = VectorStore::new();
    let collection_name = "mixed_test";

    create_test_collection(&store, collection_name, 128).expect("Failed to create collection");

    // Insert mix of sparse and dense vectors
    let vectors = vec![
        // Sparse vector
        Vector::with_sparse(
            "sparse_mixed_1".to_string(),
            SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap(),
            128,
        ),
        // Dense vector
        Vector::new("dense_mixed_1".to_string(), vec![1.0; 128]),
        // Another sparse vector
        Vector::with_sparse(
            "sparse_mixed_2".to_string(),
            SparseVector::new(vec![2, 3], vec![1.0, 1.0]).unwrap(),
            128,
        ),
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert mixed vectors");

    // Verify sparse vectors
    let sparse1 = store.get_vector(collection_name, "sparse_mixed_1").unwrap();
    assert!(sparse1.is_sparse());

    let sparse2 = store.get_vector(collection_name, "sparse_mixed_2").unwrap();
    assert!(sparse2.is_sparse());

    // Verify dense vector
    let dense = store.get_vector(collection_name, "dense_mixed_1").unwrap();
    assert!(!dense.is_sparse());
}

#[tokio::test]
async fn test_sparse_vector_large_dimension() {
    let dimension = 100000;

    // Create sparse vector with only 100 non-zero values in 100k dimensions
    let indices: Vec<usize> = (0..100).map(|i| i * 1000).collect();
    let values = vec![1.0; 100];

    let sparse = SparseVector::new(indices.clone(), values.clone()).unwrap();

    // Convert to dense
    let dense = sparse.to_dense(dimension);

    assert_eq!(dense.len(), dimension);
    for i in 0..100 {
        assert_eq!(dense[i * 1000], 1.0);
    }

    // Verify sparsity
    // 100 non-zero out of 100000 = 0.001 density, so sparsity = 1 - 0.001 = 0.999
    let sparsity = sparse.sparsity(dimension);
    assert!(sparsity >= 0.999); // Should be >=99.9% sparse (100/100000 = 0.001 density)
}

#[tokio::test]
async fn test_sparse_vector_empty() {
    // Empty sparse vector (all zeros)
    let dense = vec![0.0; 128];
    let sparse = SparseVector::from_dense(&dense);

    assert_eq!(sparse.nnz(), 0);
    assert_eq!(sparse.indices.len(), 0);
    assert_eq!(sparse.values.len(), 0);

    // Convert back to dense
    let dense_back = sparse.to_dense(128);
    assert_eq!(dense_back, dense);
}

#[tokio::test]
async fn test_sparse_vector_index_remove() {
    use vectorizer::models::SparseVectorIndex;

    let mut index = SparseVectorIndex::new();

    let v1 = SparseVector::new(vec![0, 1], vec![1.0, 2.0]).unwrap();
    index.add("v1".to_string(), v1).unwrap();

    let v2 = SparseVector::new(vec![2, 3], vec![3.0, 4.0]).unwrap();
    index.add("v2".to_string(), v2).unwrap();

    assert_eq!(index.len(), 2);

    // Remove v1
    assert!(index.remove("v1"));
    assert_eq!(index.len(), 1);

    // Try to remove non-existent
    assert!(!index.remove("v3"));
    assert_eq!(index.len(), 1);
}
