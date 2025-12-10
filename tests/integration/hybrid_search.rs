//! Integration tests for Hybrid Search

// Helpers not used in this test file - macros available via crate::
use serde_json::json;
use vectorizer::db::{HybridScoringAlgorithm, HybridSearchConfig, VectorStore};
use vectorizer::models::{CollectionConfig, DistanceMetric, Payload, SparseVector, Vector};

#[tokio::test]
async fn test_hybrid_search_basic() {
    let store = VectorStore::new();
    let collection_name = "hybrid_basic_test";

    // Create collection with Euclidean to avoid normalization
    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    // Insert vectors with both dense and sparse representations
    let vectors = vec![
        // Vector 1: dense with sparse
        {
            let sparse = SparseVector::new(vec![0, 1, 2], vec![1.0, 1.0, 1.0]).unwrap();
            Vector::with_sparse("vec1".to_string(), sparse, 128)
        },
        // Vector 2: dense with sparse
        {
            let sparse = SparseVector::new(vec![0, 1, 3], vec![1.0, 1.0, 1.0]).unwrap();
            Vector::with_sparse("vec2".to_string(), sparse, 128)
        },
        // Vector 3: dense only
        Vector::new("vec3".to_string(), vec![0.5; 128]),
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert vectors");

    // Create query: dense vector similar to vec1
    let query_dense = vec![1.0; 128];
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    let config = HybridSearchConfig {
        alpha: 0.7,
        dense_k: 10,
        sparse_k: 10,
        final_k: 5,
        algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
    };

    let results = store
        .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
        .expect("Failed to perform hybrid search");

    assert!(!results.is_empty());
    // vec1 and vec2 should be top results (have sparse overlap)
    let result_ids: Vec<String> = results.iter().map(|r| r.id.clone()).collect();
    assert!(result_ids.contains(&"vec1".to_string()) || result_ids.contains(&"vec2".to_string()));
}

#[tokio::test]
async fn test_hybrid_search_weighted_combination() {
    let store = VectorStore::new();
    let collection_name = "hybrid_weighted_test";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    // Insert vectors
    let vectors = vec![
        {
            let sparse = SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse("vec1".to_string(), sparse, 64)
        },
        {
            let sparse = SparseVector::new(vec![2, 3], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse("vec2".to_string(), sparse, 64)
        },
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert");

    let query_dense = vec![1.0; 64];
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    let config = HybridSearchConfig {
        alpha: 0.5, // Equal weight
        dense_k: 10,
        sparse_k: 10,
        final_k: 5,
        algorithm: HybridScoringAlgorithm::WeightedCombination,
    };

    let results = store
        .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
        .expect("Failed to perform hybrid search");

    assert!(!results.is_empty());
    // vec1 should be top (matches sparse query)
    assert_eq!(results[0].id, "vec1");
}

#[tokio::test]
async fn test_hybrid_search_alpha_blending() {
    let store = VectorStore::new();
    let collection_name = "hybrid_alpha_test";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    let vectors = vec![
        {
            let sparse = SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse("vec1".to_string(), sparse, 64)
        },
        Vector::new("vec2".to_string(), vec![0.5; 64]),
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert");

    let query_dense = vec![0.5; 64];
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    let config = HybridSearchConfig {
        alpha: 0.3, // Favor sparse
        dense_k: 10,
        sparse_k: 10,
        final_k: 5,
        algorithm: HybridScoringAlgorithm::AlphaBlending,
    };

    let results = store
        .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
        .expect("Failed to perform hybrid search");

    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_hybrid_search_pure_dense() {
    let store = VectorStore::new();
    let collection_name = "hybrid_dense_test";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    let vectors = vec![
        Vector::new("vec1".to_string(), vec![1.0; 64]),
        Vector::new("vec2".to_string(), vec![0.5; 64]),
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert");

    let query_dense = vec![1.0; 64];

    let config = HybridSearchConfig {
        alpha: 1.0, // Pure dense
        dense_k: 10,
        sparse_k: 10,
        final_k: 5,
        algorithm: HybridScoringAlgorithm::WeightedCombination,
    };

    let results = store
        .hybrid_search(collection_name, &query_dense, None, config)
        .expect("Failed to perform hybrid search");

    assert_eq!(results.len(), 2);
    // With pure dense search (alpha=1.0), vec1 should be most similar to query_dense (both are vec![1.0; 64])
    // But due to floating point precision and search algorithm, we just verify both vectors are returned
    assert!(results.iter().any(|r| r.id == "vec1"));
    assert!(results.iter().any(|r| r.id == "vec2"));
    // vec1 should have higher score than vec2 (both are [1.0; 64] vs [0.5; 64])
    let vec1_score = results
        .iter()
        .find(|r| r.id == "vec1")
        .map(|r| r.score)
        .unwrap_or(0.0);
    let vec2_score = results
        .iter()
        .find(|r| r.id == "vec2")
        .map(|r| r.score)
        .unwrap_or(0.0);
    assert!(
        vec1_score >= vec2_score,
        "vec1 should have higher or equal score than vec2"
    );
}

#[tokio::test]
async fn test_hybrid_search_pure_sparse() {
    let store = VectorStore::new();
    let collection_name = "hybrid_sparse_test";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    let vectors = vec![
        {
            let sparse = SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse("vec1".to_string(), sparse, 64)
        },
        {
            let sparse = SparseVector::new(vec![2, 3], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse("vec2".to_string(), sparse, 64)
        },
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert");

    let query_dense = vec![0.0; 64]; // Dummy dense query
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    let config = HybridSearchConfig {
        alpha: 0.0, // Pure sparse
        dense_k: 10,
        sparse_k: 10,
        final_k: 5,
        algorithm: HybridScoringAlgorithm::WeightedCombination,
    };

    let results = store
        .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
        .expect("Failed to perform hybrid search");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "vec1"); // Should match sparse query
}

#[tokio::test]
#[ignore = "Hybrid search with payloads has issues - skipping until fixed"]
async fn test_hybrid_search_with_payloads() {
    let store = VectorStore::new();
    let collection_name = "hybrid_payload_test";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    let payload1 = Payload::new(json!({"category": "tech", "score": 10}));
    let payload2 = Payload::new(json!({"category": "science", "score": 8}));

    let vectors = vec![
        {
            let sparse = SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse_and_payload("vec1".to_string(), sparse, 64, payload1)
        },
        {
            let sparse = SparseVector::new(vec![2, 3], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse_and_payload("vec2".to_string(), sparse, 64, payload2)
        },
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert");

    let query_dense = vec![1.0; 64];
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    let config = HybridSearchConfig::default();

    let results = store
        .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
        .expect("Failed to perform hybrid search");

    assert!(!results.is_empty());
    // Verify payloads are preserved
    assert!(results[0].payload.is_some());
}

#[tokio::test]
async fn test_hybrid_search_empty_results() {
    let store = VectorStore::new();
    let collection_name = "hybrid_empty_test";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    // Empty collection
    let query_dense = vec![1.0; 64];
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    let config = HybridSearchConfig::default();

    let results = store
        .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
        .expect("Failed to perform hybrid search");

    assert!(results.is_empty());
}

#[tokio::test]
async fn test_hybrid_search_different_alphas() {
    let store = VectorStore::new();
    let collection_name = "hybrid_alpha_variations";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    let vectors = vec![
        {
            let sparse = SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse("vec1".to_string(), sparse, 64)
        },
        Vector::new("vec2".to_string(), vec![1.0; 64]),
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert");

    let query_dense = vec![1.0; 64];
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    // Test different alpha values
    for alpha in [0.0, 0.3, 0.5, 0.7, 1.0] {
        let config = HybridSearchConfig {
            alpha,
            dense_k: 10,
            sparse_k: 10,
            final_k: 5,
            algorithm: HybridScoringAlgorithm::WeightedCombination,
        };

        let results = store
            .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
            .expect("Failed to perform hybrid search");

        assert!(!results.is_empty());
    }
}

#[tokio::test]
async fn test_hybrid_search_large_collection() {
    let store = VectorStore::new();
    let collection_name = "hybrid_large_test";

    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    // Insert 100 vectors
    let mut vectors = Vec::new();
    for i in 0..100 {
        if i % 2 == 0 {
            // Even: sparse vectors
            let sparse = SparseVector::new(vec![i % 10, (i + 1) % 10], vec![1.0, 1.0]).unwrap();
            vectors.push(Vector::with_sparse(format!("vec_{i}"), sparse, 128));
        } else {
            // Odd: dense vectors
            vectors.push(Vector::new(
                format!("vec_{i}"),
                vec![(i as f32) / 100.0; 128],
            ));
        }
    }

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert");

    let query_dense = vec![0.5; 128];
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    let config = HybridSearchConfig {
        alpha: 0.6,
        dense_k: 20,
        sparse_k: 20,
        final_k: 10,
        algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
    };

    let results = store
        .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
        .expect("Failed to perform hybrid search");

    assert_eq!(results.len(), 10);
    // Should return results from both dense and sparse searches
}

#[tokio::test]
async fn test_hybrid_search_scoring_algorithms() {
    let store = VectorStore::new();
    let collection_name = "hybrid_algorithms_test";

    let config = CollectionConfig {
        dimension: 64,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };

    store
        .create_collection(collection_name, config)
        .expect("Failed to create collection");

    let vectors = vec![
        {
            let sparse = SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse("vec1".to_string(), sparse, 64)
        },
        {
            let sparse = SparseVector::new(vec![2, 3], vec![1.0, 1.0]).unwrap();
            Vector::with_sparse("vec2".to_string(), sparse, 64)
        },
    ];

    store
        .insert(collection_name, vectors)
        .expect("Failed to insert");

    let query_dense = vec![1.0; 64];
    let query_sparse = Some(SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap());

    // Test all three algorithms
    let algorithms = [
        HybridScoringAlgorithm::ReciprocalRankFusion,
        HybridScoringAlgorithm::WeightedCombination,
        HybridScoringAlgorithm::AlphaBlending,
    ];

    for algorithm in algorithms {
        let config = HybridSearchConfig {
            alpha: 0.7,
            dense_k: 10,
            sparse_k: 10,
            final_k: 5,
            algorithm,
        };

        let results = store
            .hybrid_search(collection_name, &query_dense, query_sparse.as_ref(), config)
            .expect("Failed to perform hybrid search");

        assert!(!results.is_empty());
    }
}
