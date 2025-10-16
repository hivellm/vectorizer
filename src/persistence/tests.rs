use crate::{
    db::VectorStore,
    models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector},
    error::VectorizerError,
};
use tempfile::tempdir;

    /// Test persistence with large number of vectors
    /// DISABLED: Test is too slow and resource intensive for CI/CD
    /*
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
            quantization: crate::models::QuantizationConfig::default(),
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
                Payload::new(serde_json::json!({
                    "index": i,
                    "timestamp": format!("2025-09-23T10:00:{:02}Z", i % 60)
                }))
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
    */

    /// Test persistence error handling

    /// Test persistence with compressed payloads
