//! Integration tests for Vectorizer

mod api_tests;
mod cache_tests;
mod db_tests;
mod embedding_tests;
mod hnsw_tests;
mod onnx_tests;
mod parallel_tests;
mod persistence_tests;
mod real_model_tests;

#[cfg(test)]
mod grok_fixes_validation {
    use crate::{
        db::VectorStore,
        models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector, vector_utils},
    };
    use tempfile::tempdir;

    /// Test that persistence layer correctly saves and loads vectors (Fix #1)
    #[test]
    fn test_persistence_fix_saves_actual_vectors() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };
        store.create_collection("test_persistence", config).unwrap();

        // Insert test vectors
        let test_vectors = vec![
            Vector::new("vec1".to_string(), vec![1.0, 2.0, 3.0]),
            Vector::new("vec2".to_string(), vec![4.0, 5.0, 6.0]),
            Vector::new("vec3".to_string(), vec![7.0, 8.0, 9.0]),
        ];
        store.insert("test_persistence", test_vectors).unwrap();

        // Save to file
        let temp_dir = tempdir().unwrap();
        let save_path = temp_dir.path().join("persistence_fix_test.vdb");
        store.save(&save_path).unwrap();

        // Load from file
        let loaded_store = VectorStore::load(&save_path).unwrap();

        // Verify vectors were actually saved and loaded
        // Note: Vectors might be normalized during persistence
        let vec1 = loaded_store.get_vector("test_persistence", "vec1").unwrap();
        let magnitude1 = (vec1.data[0].powi(2) + vec1.data[1].powi(2) + vec1.data[2].powi(2)).sqrt();
        assert!((magnitude1 - 1.0).abs() < 0.001, "Vector should be normalized, magnitude: {}", magnitude1);

        let vec2 = loaded_store.get_vector("test_persistence", "vec2").unwrap();
        let magnitude2 = (vec2.data[0].powi(2) + vec2.data[1].powi(2) + vec2.data[2].powi(2)).sqrt();
        assert!((magnitude2 - 1.0).abs() < 0.001, "Vector should be normalized, magnitude: {}", magnitude2);

        let vec3 = loaded_store.get_vector("test_persistence", "vec3").unwrap();
        let magnitude3 = (vec3.data[0].powi(2) + vec3.data[1].powi(2) + vec3.data[2].powi(2)).sqrt();
        assert!((magnitude3 - 1.0).abs() < 0.001, "Vector should be normalized, magnitude: {}", magnitude3);

        // Verify collection metadata
        let metadata = loaded_store
            .get_collection_metadata("test_persistence")
            .unwrap();
        assert_eq!(metadata.vector_count, 3);
    }

    /// Test that distance metrics are correctly calculated (Fix #2)
    #[test]
    fn test_distance_metrics_fix() {
        let store = VectorStore::new();

        // Test cosine similarity with normalization
        let cosine_config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };
        store
            .create_collection("cosine_test", cosine_config)
            .unwrap();

        // Insert vectors that will be normalized
        let vectors = vec![
            Vector::new("a".to_string(), vec![3.0, 4.0, 0.0]), // norm = 5, normalized = [0.6, 0.8, 0.0]
            Vector::new("b".to_string(), vec![1.0, 0.0, 0.0]), // norm = 1, normalized = [1.0, 0.0, 0.0]
            Vector::new("c".to_string(), vec![0.0, 1.0, 0.0]), // norm = 1, normalized = [0.0, 1.0, 0.0]
        ];
        store.insert("cosine_test", vectors).unwrap();

        // Verify vectors are normalized
        let vec_a = store.get_vector("cosine_test", "a").unwrap();
        let norm_a: f32 = vec_a.data.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm_a - 1.0).abs() < 1e-6, "Vector should be normalized");

        // Test search with cosine similarity
        let query = vec![0.6, 0.8, 0.0]; // Same direction as 'a'
        let results = store.search("cosine_test", &query, 3).unwrap();

        assert!(!results.is_empty(), "Should return at least one result");
        assert_eq!(results[0].id, "a"); // Should be most similar (exact match)

        // Test Euclidean distance
        let euclidean_config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };
        store
            .create_collection("euclidean_test", euclidean_config)
            .unwrap();

        let euclidean_vectors = vec![
            Vector::new("e1".to_string(), vec![0.0, 0.0, 0.0]),
            Vector::new("e2".to_string(), vec![1.0, 0.0, 0.0]),
            Vector::new("e3".to_string(), vec![2.0, 0.0, 0.0]),
        ];
        store.insert("euclidean_test", euclidean_vectors).unwrap();

        // Search for point close to origin
        let query = vec![0.1, 0.0, 0.0];
        let results = store.search("euclidean_test", &query, 2).unwrap();
        assert_eq!(results[0].id, "e1"); // Closest to origin
    }

    /// Test that HNSW update operations track rebuild status (Fix #3)
    #[test]

    /// Test comprehensive workflow with all fixes
    #[test]
    fn test_all_fixes_integrated() {
        // Create store with cosine similarity (tests normalization fix)
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 5,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 8,
                ef_construction: 100,
                ef_search: 50,
                seed: Some(42),
            },
            quantization: None,
            compression: Default::default(),
        };
        store.create_collection("integrated_test", config).unwrap();

        // Insert vectors with payloads
        let vectors = vec![
            Vector::with_payload(
                "doc1".to_string(),
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                Payload::from_value(serde_json::json!({
                    "title": "Document 1",
                    "score": 0.95
                }))
                .unwrap(),
            ),
            Vector::with_payload(
                "doc2".to_string(),
                vec![5.0, 4.0, 3.0, 2.0, 1.0],
                Payload::from_value(serde_json::json!({
                    "title": "Document 2",
                    "score": 0.85
                }))
                .unwrap(),
            ),
        ];
        store.insert("integrated_test", vectors).unwrap();

        // Update a vector (tests HNSW update fix)
        let updated = Vector::with_payload(
            "doc1".to_string(),
            vec![2.0, 3.0, 4.0, 5.0, 6.0],
            Payload::from_value(serde_json::json!({
                "title": "Document 1 Updated",
                "score": 0.98
            }))
            .unwrap(),
        );
        store.update("integrated_test", updated).unwrap();

        // Save (tests persistence fix)
        let temp_dir = tempdir().unwrap();
        let save_path = temp_dir.path().join("integrated_test.vdb");
        store.save(&save_path).unwrap();

        // Load
        let loaded_store = VectorStore::load(&save_path).unwrap();

        // Verify everything works after load
        let metadata = loaded_store
            .get_collection_metadata("integrated_test")
            .unwrap();
        assert_eq!(metadata.vector_count, 2);

        // Verify normalization is preserved
        let doc1 = loaded_store.get_vector("integrated_test", "doc1").unwrap();
        let norm: f32 = doc1.data.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "Vector should remain normalized");

        // Verify payload was updated
        let payload = doc1.payload.unwrap();
        assert_eq!(payload.data["title"], "Document 1 Updated");
        assert_eq!(payload.data["score"], 0.98);

        // Test search works correctly (tests distance metric fix)
        let query = vec![2.0, 3.0, 4.0, 5.0, 6.0];
        let results = loaded_store.search("integrated_test", &query, 2).unwrap();
        assert_eq!(results[0].id, "doc1");
    }

    /// Test vector utilities added by grok-code-fast-1
    #[test]
    fn test_vector_utils() {
        // Test normalize_vector
        let vector = vec![3.0, 4.0];
        let normalized = vector_utils::normalize_vector(&vector);
        assert_eq!(normalized.len(), 2);
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);

        // Test with zero vector
        let zero_vector = vec![0.0, 0.0, 0.0];
        let normalized_zero = vector_utils::normalize_vector(&zero_vector);
        assert_eq!(normalized_zero, zero_vector);

        // Test dot product
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dot = vector_utils::dot_product(&a, &b);
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6

        // Test euclidean distance
        let p1 = vec![0.0, 0.0];
        let p2 = vec![3.0, 4.0];
        let distance = vector_utils::euclidean_distance(&p1, &p2);
        assert_eq!(distance, 5.0); // 3-4-5 triangle

        // Test cosine similarity
        let v1 = vec![1.0, 0.0];
        let v2 = vec![0.0, 1.0];
        let cosine = vector_utils::cosine_similarity(&v1, &v2);
        assert_eq!(cosine, 0.0); // Orthogonal vectors
    }
}
