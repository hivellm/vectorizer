//! Integration tests for vectorizer + hive-gpu
//!
//! These tests verify that the adapter layer works correctly
//! and that vectorizer can use hive-gpu for GPU acceleration.

#[cfg(feature = "hive-gpu")]
use hive_gpu::GpuDistanceMetric;
use vectorizer::gpu_adapter::GpuAdapter;
use vectorizer::models::{DistanceMetric, Vector};
use vectorizer::{CollectionConfig, VectorStore};

#[cfg(feature = "hive-gpu")]
mod hive_gpu_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_adapter_conversion() {
        // Test Vector -> GpuVector conversion
        let vector = Vector {
            id: "test_vector".to_string(),
            data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            payload: Some(vectorizer::models::Payload::new(serde_json::json!({
                "category": "test",
                "source": "integration"
            }))),
        };

        let gpu_vector = GpuAdapter::vector_to_gpu_vector(&vector);

        assert_eq!(gpu_vector.id, "test_vector");
        assert_eq!(gpu_vector.data, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(
            gpu_vector.metadata.get("category"),
            Some(&"test".to_string())
        );
        assert_eq!(
            gpu_vector.metadata.get("source"),
            Some(&"integration".to_string())
        );

        // Test GpuVector -> Vector conversion
        let converted_back = GpuAdapter::gpu_vector_to_vector(&gpu_vector);

        assert_eq!(converted_back.id, "test_vector");
        assert_eq!(converted_back.data, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(converted_back.payload.is_some());
        let payload = converted_back.payload.unwrap();
        assert_eq!(
            payload.data.get("category"),
            Some(&serde_json::Value::String("test".to_string()))
        );
        assert_eq!(
            payload.data.get("source"),
            Some(&serde_json::Value::String("integration".to_string()))
        );
    }

    #[tokio::test]
    async fn test_distance_metric_conversion() {
        // Test DistanceMetric -> GpuDistanceMetric
        let cosine_metric = GpuAdapter::distance_metric_to_gpu_metric(DistanceMetric::Cosine);
        assert!(matches!(cosine_metric, GpuDistanceMetric::Cosine));

        let euclidean_metric = GpuAdapter::distance_metric_to_gpu_metric(DistanceMetric::Euclidean);
        assert!(matches!(euclidean_metric, GpuDistanceMetric::Euclidean));

        let dot_product_metric =
            GpuAdapter::distance_metric_to_gpu_metric(DistanceMetric::DotProduct);
        assert!(matches!(dot_product_metric, GpuDistanceMetric::DotProduct));

        // Test GpuDistanceMetric -> DistanceMetric
        let back_to_cosine = GpuAdapter::gpu_metric_to_distance_metric(GpuDistanceMetric::Cosine);
        assert!(matches!(back_to_cosine, DistanceMetric::Cosine));

        let back_to_euclidean =
            GpuAdapter::gpu_metric_to_distance_metric(GpuDistanceMetric::Euclidean);
        assert!(matches!(back_to_euclidean, DistanceMetric::Euclidean));

        let back_to_dot_product =
            GpuAdapter::gpu_metric_to_distance_metric(GpuDistanceMetric::DotProduct);
        assert!(matches!(back_to_dot_product, DistanceMetric::DotProduct));
    }

    #[tokio::test]
    async fn test_hnsw_config_conversion() {
        let vectorizer_config = vectorizer::models::HnswConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            seed: Some(42),
        };

        let gpu_config = GpuAdapter::hnsw_config_to_gpu_config(&vectorizer_config);

        assert_eq!(gpu_config.max_connections, 16);
        assert_eq!(gpu_config.ef_construction, 200);
        assert_eq!(gpu_config.ef_search, 50);
        assert_eq!(gpu_config.seed, Some(42));

        let back_to_vectorizer = GpuAdapter::gpu_config_to_hnsw_config(&gpu_config);

        assert_eq!(back_to_vectorizer.m, 16);
        assert_eq!(back_to_vectorizer.ef_construction, 200);
        assert_eq!(back_to_vectorizer.ef_search, 50);
        assert_eq!(back_to_vectorizer.seed, Some(42));
    }

    #[tokio::test]
    async fn test_error_conversion() {
        use hive_gpu::HiveGpuError;
        use vectorizer::error::VectorizerError;

        // Test HiveGpuError -> VectorizerError
        let gpu_error = HiveGpuError::NoDeviceAvailable;
        let vectorizer_error = GpuAdapter::gpu_error_to_vectorizer_error(gpu_error);
        assert!(matches!(vectorizer_error, VectorizerError::Other(_)));

        let gpu_error = HiveGpuError::DimensionMismatch {
            expected: 128,
            actual: 64,
        };
        let vectorizer_error = GpuAdapter::gpu_error_to_vectorizer_error(gpu_error);
        assert!(matches!(
            vectorizer_error,
            VectorizerError::DimensionMismatch {
                expected: 128,
                actual: 64
            }
        ));

        // Test VectorizerError -> HiveGpuError
        let vectorizer_error = VectorizerError::DimensionMismatch {
            expected: 256,
            actual: 128,
        };
        let gpu_error = GpuAdapter::vectorizer_error_to_gpu_error(vectorizer_error);
        assert!(matches!(
            gpu_error,
            HiveGpuError::DimensionMismatch {
                expected: 256,
                actual: 128
            }
        ));
    }

    #[tokio::test]
    async fn test_vectorizer_with_hive_gpu_metal() {
        #[cfg(all(target_os = "macos", feature = "hive-gpu-metal"))]
        {
            // Test that vectorizer can use hive-gpu for Metal Native
            let store = VectorStore::new_auto();

            let config = CollectionConfig {
                dimension: 512,
                metric: DistanceMetric::Cosine,
                hnsw_config: vectorizer::models::HnswConfig {
                    m: 16,
                    ef_construction: 200,
                    ef_search: 50,
                    max_connections: 32,
                },
                quantization: vectorizer::models::QuantizationConfig::default(),
                compression: vectorizer::models::CompressionConfig::default(),
                normalization: None,
            };

            store
                .create_collection("hive_gpu_test", config)
                .expect("Failed to create collection");

            // Add test vectors
            let vectors = vec![
                Vector {
                    id: "test_vec_1".to_string(),
                    data: vec![1.0; 512],
                    payload: Some(vec![("category".to_string(), "test".to_string())]),
                },
                Vector {
                    id: "test_vec_2".to_string(),
                    data: vec![2.0; 512],
                    payload: Some(vec![("category".to_string(), "test".to_string())]),
                },
            ];

            store
                .insert("hive_gpu_test", vectors)
                .expect("Failed to insert vectors");

            // Search for similar vectors
            let query = vec![1.5; 512];
            let results = store
                .search("hive_gpu_test", &query, 10)
                .expect("Failed to search");

            assert!(!results.is_empty());
            assert!(results.len() <= 10);

            // Verify results are sorted by similarity
            for i in 1..results.len() {
                assert!(results[i - 1].score >= results[i].score);
            }
        }
    }

    #[tokio::test]
    async fn test_vectorizer_with_hive_gpu_cuda() {
        #[cfg(feature = "hive-gpu-cuda")]
        {
            // Test that vectorizer can use hive-gpu for CUDA
            let store = VectorStore::new_auto();

            let config = CollectionConfig {
                dimension: 256,
                metric: DistanceMetric::Euclidean,
                hnsw_config: vectorizer::models::HnswConfig::default(),
                quantization: vectorizer::models::QuantizationConfig::default(),
                compression: vectorizer::models::CompressionConfig::default(),
                normalization: None,
            };

            store
                .create_collection("hive_gpu_cuda_test", config)
                .expect("Failed to create collection");

            // Add test vectors
            let vectors = vec![
                Vector {
                    id: "cuda_vec_1".to_string(),
                    data: vec![1.0; 256],
                    payload: None,
                },
                Vector {
                    id: "cuda_vec_2".to_string(),
                    data: vec![2.0; 256],
                    payload: None,
                },
            ];

            store
                .insert("hive_gpu_cuda_test", vectors)
                .expect("Failed to insert vectors");

            // Search for similar vectors
            let query = vec![1.5; 256];
            let results = store
                .search("hive_gpu_cuda_test", &query, 5)
                .expect("Failed to search");

            assert!(!results.is_empty());
            assert!(results.len() <= 5);
        }
    }

    #[tokio::test]
    async fn test_vectorizer_with_hive_gpu_wgpu() {
        #[cfg(feature = "hive-gpu-wgpu")]
        {
            // Test that vectorizer can use hive-gpu for wgpu
            let store = VectorStore::new_auto();

            let config = CollectionConfig {
                dimension: 128,
                metric: DistanceMetric::DotProduct,
                hnsw_config: vectorizer::models::HnswConfig::default(),
                quantization: vectorizer::models::QuantizationConfig::default(),
                compression: vectorizer::models::CompressionConfig::default(),
                normalization: None,
            };

            store
                .create_collection("hive_gpu_wgpu_test", config)
                .expect("Failed to create collection");

            // Add test vectors
            let vectors = vec![
                Vector {
                    id: "wgpu_vec_1".to_string(),
                    data: vec![1.0; 128],
                    payload: None,
                },
                Vector {
                    id: "wgpu_vec_2".to_string(),
                    data: vec![2.0; 128],
                    payload: None,
                },
            ];

            store
                .insert("hive_gpu_wgpu_test", vectors)
                .expect("Failed to insert vectors");

            // Search for similar vectors
            let query = vec![1.5; 128];
            let results = store
                .search("hive_gpu_wgpu_test", &query, 5)
                .expect("Failed to search");

            assert!(!results.is_empty());
            assert!(results.len() <= 5);
        }
    }

    #[tokio::test]
    #[ignore] // Performance test - requires GPU, skipped on CPU-only systems
    async fn test_performance_comparison() {
        // Test that hive-gpu provides performance benefits
        let store = VectorStore::new_auto();

        let config = CollectionConfig {
            dimension: 512,
            metric: DistanceMetric::Cosine,
            hnsw_config: vectorizer::models::HnswConfig::default(),
            quantization: vectorizer::models::QuantizationConfig::default(),
            compression: vectorizer::models::CompressionConfig::default(),
            normalization: None,
        };

        store
            .create_collection("performance_test", config)
            .expect("Failed to create collection");

        // Create a large number of vectors
        let vectors: Vec<Vector> = (0..1000)
            .map(|i| Vector {
                id: format!("perf_vec_{i}"),
                data: vec![i as f32; 512],
                payload: None,
            })
            .collect();

        let start = std::time::Instant::now();
        store
            .insert("performance_test", vectors)
            .expect("Failed to insert vectors");
        let insert_time = start.elapsed();

        // Search should be fast
        let start = std::time::Instant::now();
        let query = vec![500.0; 512];
        let results = store
            .search("performance_test", &query, 10)
            .expect("Failed to search");
        let search_time = start.elapsed();

        assert!(!results.is_empty());
        assert!(insert_time.as_millis() < 1000); // Should be fast
        assert!(search_time.as_millis() < 100); // Search should be very fast
    }
}

#[cfg(not(feature = "hive-gpu"))]
mod no_hive_gpu_tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_to_cpu() {
        // Test that vectorizer falls back to CPU when hive-gpu is not available
        let store = VectorStore::new_auto();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: vectorizer::models::HnswConfig::default(),
            quantization: vectorizer::models::QuantizationConfig::default(),
            compression: vectorizer::models::CompressionConfig::default(),
            normalization: None,
        };

        store
            .create_collection("cpu_fallback_test", config)
            .expect("Failed to create collection");

        let vectors = vec![Vector {
            id: "cpu_vec_1".to_string(),
            data: vec![1.0; 128],
            payload: None,
        }];

        store
            .insert("cpu_fallback_test", vectors)
            .expect("Failed to insert vectors");

        let query = vec![1.0; 128];
        let results = store
            .search("cpu_fallback_test", &query, 10)
            .expect("Failed to search");

        assert!(!results.is_empty());
    }
}
