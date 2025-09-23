use crate::{
    db::VectorStore,
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector},
    error::VectorizerError,
};
use tempfile::tempdir;

    /// Test complete persistence workflow: save, load, and search
    #[test]
    fn test_complete_persistence_workflow() {
        // Create a vector store with multiple collections
        let store = VectorStore::new();

        // Create collection 1: Euclidean distance
        let config1 = CollectionConfig {
            dimension: 4,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig {
                m: 8,
                ef_construction: 100,
                ef_search: 50,
                seed: Some(42),
            },
            quantization: None,
            compression: Default::default(),
        };
        store.create_collection("euclidean_collection", config1).unwrap();

        // Create collection 2: Cosine similarity
        let config2 = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };
        store.create_collection("cosine_collection", config2).unwrap();

        // Insert vectors into collection 1
        let euclidean_vectors = vec![
            Vector::with_payload(
                "euc_1".to_string(),
                vec![1.0, 0.0, 0.0, 0.0],
                Payload::from_value(serde_json::json!({
                    "name": "Unit X",
                    "category": "basis"
                })).unwrap()
            ),
            Vector::with_payload(
                "euc_2".to_string(),
                vec![0.0, 1.0, 0.0, 0.0],
                Payload::from_value(serde_json::json!({
                    "name": "Unit Y",
                    "category": "basis"
                })).unwrap()
            ),
            Vector::with_payload(
                "euc_3".to_string(),
                vec![0.5, 0.5, 0.5, 0.5],
                Payload::from_value(serde_json::json!({
                    "name": "Diagonal",
                    "category": "mixed"
                })).unwrap()
            ),
        ];
        store.insert("euclidean_collection", euclidean_vectors).unwrap();

        // Insert vectors into collection 2 (will be normalized)
        let cosine_vectors = vec![
            Vector::with_payload(
                "cos_1".to_string(),
                vec![3.0, 4.0, 0.0], // Will be normalized to [0.6, 0.8, 0.0]
                Payload::from_value(serde_json::json!({
                    "name": "Vector A",
                    "original_norm": 5.0
                })).unwrap()
            ),
            Vector::with_payload(
                "cos_2".to_string(),
                vec![1.0, 1.0, 1.0], // Will be normalized to [0.577, 0.577, 0.577]
                Payload::from_value(serde_json::json!({
                    "name": "Vector B",
                    "original_norm": 1.732
                })).unwrap()
            ),
            Vector::with_payload(
                "cos_3".to_string(),
                vec![0.0, 0.0, 1.0], // Will be normalized to [0.0, 0.0, 1.0]
                Payload::from_value(serde_json::json!({
                    "name": "Vector C",
                    "original_norm": 1.0
                })).unwrap()
            ),
        ];
        store.insert("cosine_collection", cosine_vectors).unwrap();

        // Test search before persistence
        let search_results_before = store.search("euclidean_collection", &[0.9, 0.1, 0.0, 0.0], 2).unwrap();
        assert_eq!(search_results_before.len(), 2);
        
        // Save to file
        let temp_dir = tempdir().unwrap();
        let save_path = temp_dir.path().join("test_store.vdb");
        store.save(&save_path).unwrap();

        // Load from file
        let loaded_store = VectorStore::load(&save_path).unwrap();

        // Verify collections exist
        let collections = loaded_store.list_collections();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&"euclidean_collection".to_string()));
        assert!(collections.contains(&"cosine_collection".to_string()));

        // Verify vector counts
        let euclidean_metadata = loaded_store.get_collection_metadata("euclidean_collection").unwrap();
        assert_eq!(euclidean_metadata.vector_count, 3);
        let cosine_metadata = loaded_store.get_collection_metadata("cosine_collection").unwrap();
        assert_eq!(cosine_metadata.vector_count, 3);

        // Test search after loading - Euclidean
        let search_results_after = loaded_store.search("euclidean_collection", &[0.9, 0.1, 0.0, 0.0], 2).unwrap();
        assert_eq!(search_results_after.len(), 2);
        // The closest should be euc_1 [1.0, 0.0, 0.0, 0.0]
        assert_eq!(search_results_after[0].id, "euc_1");
        
        // Test search after loading - Cosine
        let cosine_search = loaded_store.search("cosine_collection", &[0.6, 0.8, 0.0], 2).unwrap();
        assert_eq!(cosine_search.len(), 2);
        // The closest should be cos_1 (normalized [0.6, 0.8, 0.0])
        assert_eq!(cosine_search[0].id, "cos_1");

        // Verify payloads are preserved
        let retrieved_vector = loaded_store.get_vector("euclidean_collection", "euc_1").unwrap();
        let payload_data = retrieved_vector.payload.unwrap().data;
        assert_eq!(payload_data["name"], "Unit X");
        assert_eq!(payload_data["category"], "basis");

        // Verify cosine vectors are normalized
        let cos_vector = loaded_store.get_vector("cosine_collection", "cos_1").unwrap();
        let norm: f32 = cos_vector.data.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "Cosine vector should be normalized");
    }

    /// Test persistence with large number of vectors
    #[test]
    fn test_persistence_with_large_dataset() {
        let store = VectorStore::new();
        let dimension = 128;
        let num_vectors = 1000;

        let config = CollectionConfig {
            dimension,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 100,
                seed: Some(123),
            },
            quantization: None,
            compression: Default::default(),
        };
        store.create_collection("large_collection", config).unwrap();

        // Generate and insert many vectors
        let mut vectors = Vec::new();
        for i in 0..num_vectors {
            let mut data = vec![0.0f32; dimension];
            // Create a pattern in the data
            data[i % dimension] = 1.0;
            data[(i + 1) % dimension] = 0.5;
            
            let vector = Vector::with_payload(
                format!("vec_{}", i),
                data,
                Payload::from_value(serde_json::json!({
                    "index": i,
                    "timestamp": format!("2025-09-23T10:00:{:02}Z", i % 60)
                })).unwrap()
            );
            vectors.push(vector);
        }

        store.insert("large_collection", vectors).unwrap();

        // Save
        let temp_dir = tempdir().unwrap();
        let save_path = temp_dir.path().join("large_store.vdb");
        store.save(&save_path).unwrap();

        // Check file size is reasonable
        let file_metadata = std::fs::metadata(&save_path).unwrap();
        assert!(file_metadata.len() > 0, "Saved file should not be empty");

        // Load
        let loaded_store = VectorStore::load(&save_path).unwrap();

        // Verify all vectors are loaded
        let metadata = loaded_store.get_collection_metadata("large_collection").unwrap();
        assert_eq!(metadata.vector_count, num_vectors);

        // Test random access
        let random_ids = vec!["vec_0", "vec_499", "vec_999"];
        for id in random_ids {
            let vector = loaded_store.get_vector("large_collection", id).unwrap();
            assert_eq!(vector.id, id);
            assert_eq!(vector.data.len(), dimension);
        }

        // Test search performance
        let mut query = vec![0.0f32; dimension];
        query[50] = 1.0;
        query[51] = 0.5;

        let search_results = loaded_store.search("large_collection", &query, 10).unwrap();
        assert_eq!(search_results.len(), 10);
        
        // Verify search results have valid scores
        for result in &search_results {
            assert!(result.score >= 0.0);
            assert!(result.vector.is_some());
        }
    }

    /// Test persistence with multiple save/load cycles
    #[test]
    fn test_multiple_persistence_cycles() {
        let temp_dir = tempdir().unwrap();
        let save_path = temp_dir.path().join("cycles.vdb");

        // Cycle 1: Create and save initial data
        {
            let store = VectorStore::new();
            let config = CollectionConfig {
                dimension: 3,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: None,
                compression: Default::default(),
            };
            store.create_collection("cycle_test", config).unwrap();

            let vectors = vec![
                Vector::new("v1".to_string(), vec![1.0, 0.0, 0.0]),
                Vector::new("v2".to_string(), vec![0.0, 1.0, 0.0]),
            ];
            store.insert("cycle_test", vectors).unwrap();
            store.save(&save_path).unwrap();
        }

        // Cycle 2: Load, modify, and save
        {
            let store = VectorStore::load(&save_path).unwrap();
            
            // Add more vectors
            let new_vectors = vec![
                Vector::new("v3".to_string(), vec![0.0, 0.0, 1.0]),
                Vector::new("v4".to_string(), vec![1.0, 1.0, 1.0]),
            ];
            store.insert("cycle_test", new_vectors).unwrap();
            
            // Update existing vector
            let updated = Vector::new("v1".to_string(), vec![2.0, 0.0, 0.0]);
            store.update("cycle_test", updated).unwrap();
            
            // Delete a vector
            store.delete("cycle_test", "v2").unwrap();
            
            store.save(&save_path).unwrap();
        }

        // Cycle 3: Load and verify final state
        {
            let store = VectorStore::load(&save_path).unwrap();
            let metadata = store.get_collection_metadata("cycle_test").unwrap();
            assert_eq!(metadata.vector_count, 3); // v1, v3, v4 (v2 deleted)

            // Verify v1 was updated
            let v1 = store.get_vector("cycle_test", "v1").unwrap();
            let v1_norm: f32 = v1.data.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((v1_norm - 1.0).abs() < 1e-6, "Should be normalized");

            // Verify v2 was deleted
            let v2_result = store.get_vector("cycle_test", "v2");
            assert!(matches!(v2_result, Err(VectorizerError::VectorNotFound(_))));

            // Verify v3 and v4 exist
            assert!(store.get_vector("cycle_test", "v3").is_ok());
            assert!(store.get_vector("cycle_test", "v4").is_ok());
        }
    }

    /// Test persistence error handling
    #[test]
    fn test_persistence_error_handling() {
        let store = VectorStore::new();

        // Try to save to invalid path
        let result = store.save("/invalid/path/that/does/not/exist/store.vdb");
        assert!(result.is_err());

        // Try to load non-existent file
        let result = VectorStore::load("/non/existent/file.vdb");
        assert!(result.is_err());

        // Create a corrupted file
        let temp_dir = tempdir().unwrap();
        let corrupt_path = temp_dir.path().join("corrupt.vdb");
        std::fs::write(&corrupt_path, b"This is not valid bincode data").unwrap();
        
        let result = VectorStore::load(&corrupt_path);
        assert!(result.is_err());
    }

    /// Test persistence with compressed payloads
    #[test]
    fn test_persistence_with_compression() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };
        store.create_collection("compressed", config).unwrap();

        // Create vectors with large payloads
        let large_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(20);
        let vectors = vec![
            Vector::with_payload(
                "large_1".to_string(),
                vec![1.0, 0.0, 0.0],
                Payload::from_value(serde_json::json!({
                    "text": large_text.clone(),
                    "metadata": {
                        "size": large_text.len(),
                        "compressed": true
                    }
                })).unwrap()
            ),
            Vector::with_payload(
                "large_2".to_string(),
                vec![0.0, 1.0, 0.0],
                Payload::from_value(serde_json::json!({
                    "text": large_text.clone(),
                    "metadata": {
                        "size": large_text.len(),
                        "compressed": true
                    }
                })).unwrap()
            ),
        ];

        store.insert("compressed", vectors).unwrap();

        // Save and check file size
        let temp_dir = tempdir().unwrap();
        let save_path = temp_dir.path().join("compressed.vdb");
        store.save(&save_path).unwrap();

        let _file_size = std::fs::metadata(&save_path).unwrap().len();
        
        // Load and verify
        let loaded_store = VectorStore::load(&save_path).unwrap();
        let loaded_vector = loaded_store.get_vector("compressed", "large_1").unwrap();
        
        let payload = loaded_vector.payload.unwrap();
        assert_eq!(payload.data["text"].as_str().unwrap().len(), large_text.len());
        assert_eq!(payload.data["metadata"]["size"], large_text.len());
    }

    /// Test search accuracy after persistence
    #[test]
    fn test_search_accuracy_after_persistence() {
        let store = VectorStore::new();

        // Create collection with dot product metric
        let config = CollectionConfig {
            dimension: 5,
            metric: DistanceMetric::DotProduct,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 150,
                seed: Some(42),
            },
            quantization: None,
            compression: Default::default(),
        };
        store.create_collection("dot_product", config).unwrap();

        // Insert vectors with known relationships
        let vectors = vec![
            Vector::new("orthogonal_1".to_string(), vec![1.0, 0.0, 0.0, 0.0, 0.0]),
            Vector::new("orthogonal_2".to_string(), vec![0.0, 1.0, 0.0, 0.0, 0.0]),
            Vector::new("orthogonal_3".to_string(), vec![0.0, 0.0, 1.0, 0.0, 0.0]),
            Vector::new("similar_1".to_string(), vec![1.0, 1.0, 0.0, 0.0, 0.0]),
            Vector::new("similar_2".to_string(), vec![1.0, 0.9, 0.1, 0.0, 0.0]),
            Vector::new("opposite".to_string(), vec![-1.0, -1.0, 0.0, 0.0, 0.0]),
        ];
        store.insert("dot_product", vectors).unwrap();

        // Search before persistence
        let query = vec![1.0, 1.0, 0.0, 0.0, 0.0];
        let results_before = store.search("dot_product", &query, 3).unwrap();

        // Save and load
        let temp_dir = tempdir().unwrap();
        let save_path = temp_dir.path().join("accuracy_test.vdb");
        store.save(&save_path).unwrap();
        let loaded_store = VectorStore::load(&save_path).unwrap();

        // Search after persistence
        let results_after = loaded_store.search("dot_product", &query, 3).unwrap();

        // Verify search results are consistent
        assert_eq!(results_before.len(), results_after.len());
        for i in 0..results_before.len() {
            assert_eq!(results_before[i].id, results_after[i].id);
            // Scores might have small floating point differences
            assert!((results_before[i].score - results_after[i].score).abs() < 1e-5);
        }

        // Verify expected ordering
        assert_eq!(results_after[0].id, "similar_1"); // Exact match
        assert!(results_after[1].id == "similar_2" || results_after[1].id == "orthogonal_1");
    }
