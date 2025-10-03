//! Vectorizer - High-performance, in-memory vector database written in Rust
//!
//! This crate provides a fast and efficient vector database for semantic search
//! and similarity queries, designed for AI-driven applications.

#![allow(warnings)]

pub mod api;
pub mod auth;
pub mod batch;
pub mod cache;
pub mod cli;
pub mod config;
pub mod cuda;
pub mod db;
pub mod document_loader;
pub mod embedding;
pub mod error;
pub mod evaluation;
#[cfg(feature = "wgpu-gpu")]
pub mod gpu;
pub mod grpc;
pub mod hybrid_search;
pub mod mcp;
pub mod mcp_service;
pub mod models;
pub mod parallel;
#[path = "persistence/mod.rs"]
pub mod persistence;
pub mod summarization;
pub mod workspace;
pub mod utils;
pub mod file_watcher;
pub mod process_manager;
pub mod logging;

// Re-export commonly used types
pub use db::{Collection, VectorStore};
pub use embedding::{BertEmbedding, Bm25Embedding, MiniLmEmbedding, SvdEmbedding};
pub use error::{Result, VectorizerError};
pub use evaluation::{EvaluationMetrics, QueryMetrics, QueryResult, evaluate_search_quality};
pub use models::{CollectionConfig, Payload, SearchResult, Vector};
pub use batch::{BatchProcessor, BatchConfig, BatchOperation, BatchProcessorBuilder};
pub use summarization::{SummarizationManager, SummarizationConfig, SummarizationMethod, SummarizationResult, SummarizationError};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Include test modules
#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use tempfile::tempdir;

    #[test]
    fn test_concurrent_workload_simulation() {
        let store = Arc::new(VectorStore::new());
        let num_threads = 4;
        let vectors_per_thread = 10;

        // Create collection
        let config = CollectionConfig {
            dimension: 64,
            metric: crate::models::DistanceMetric::Euclidean,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };

        store.create_collection("concurrent", config).unwrap();

        let mut handles = vec![];

        // Spawn worker threads
        for thread_id in 0..num_threads {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                let mut local_results = vec![];

                // Each thread inserts its own set of vectors
                for i in 0..vectors_per_thread {
                    let vector_id = format!("thread_{}_vec_{}", thread_id, i);
                    let vector_data: Vec<f32> = (0..64)
                        .map(|j| (thread_id as f32 * 0.1) + (i as f32 * 0.01) + (j as f32 * 0.001))
                        .collect();

                    let vector = Vector::with_payload(
                        vector_id.clone(),
                        vector_data,
                        Payload::from_value(serde_json::json!({
                            "thread_id": thread_id,
                            "vector_index": i,
                            "created_by": format!("thread_{}", thread_id)
                        }))
                        .unwrap(),
                    );

                    store_clone.insert("concurrent", vec![vector]).unwrap();
                    local_results.push(vector_id);
                }

                local_results
            });

            handles.push(handle);
        }

        // Collect results from all threads
        let mut all_vector_ids = vec![];
        for handle in handles {
            let thread_results = handle.join().unwrap();
            all_vector_ids.extend(thread_results);
        }

        // Verify all vectors were inserted
        let metadata = store.get_collection_metadata("concurrent").unwrap();
        assert_eq!(metadata.vector_count, num_threads * vectors_per_thread);

        // Verify we can retrieve all vectors
        for vector_id in &all_vector_ids {
            let vector = store.get_vector("concurrent", vector_id).unwrap();
            assert_eq!(vector.id, *vector_id);
            assert_eq!(vector.data.len(), 64);
        }

        // Test concurrent search operations
        let search_threads = 3;
        let mut search_handles = vec![];

        for _ in 0..search_threads {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                let query = vec![0.5; 64];
                let results = store_clone.search("concurrent", &query, 5).unwrap();
                results.len()
            });
            search_handles.push(handle);
        }

        // All search operations should complete successfully
        for handle in search_handles {
            let result_count = handle.join().unwrap();
            assert_eq!(result_count, 5);
        }
    }

    #[test]
    fn test_collection_management() {
        let store = VectorStore::new();

        // Test creating multiple collections with different configurations
        let configs = vec![
            (
                "small",
                CollectionConfig {
                    dimension: 64,
                    metric: crate::models::DistanceMetric::Euclidean,
                    hnsw_config: crate::models::HnswConfig {
                        m: 8,
                        ef_construction: 100,
                        ef_search: 50,
                        seed: None,
                    },
                    quantization: None,
                    compression: Default::default(),
                },
            ),
            (
                "large",
                CollectionConfig {
                    dimension: 512,
                    metric: crate::models::DistanceMetric::Cosine,
                    hnsw_config: crate::models::HnswConfig {
                        m: 32,
                        ef_construction: 300,
                        ef_search: 100,
                        seed: Some(123),
                    },
                    quantization: None,
                    compression: crate::models::CompressionConfig {
                        enabled: true,
                        threshold_bytes: 2048,
                        algorithm: crate::models::CompressionAlgorithm::Lz4,
                    },
                },
            ),
        ];

        // Create collections
        for (name, config) in &configs {
            store.create_collection(name, config.clone()).unwrap();
        }

        // Verify collections exist
        let collections = store.list_collections();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&"small".to_string()));
        assert!(collections.contains(&"large".to_string()));

        // Test duplicate collection creation
        assert!(matches!(
            store.create_collection("small", configs[0].1.clone()),
            Err(VectorizerError::CollectionAlreadyExists(_))
        ));

        // Add vectors to different collections
        let small_vectors = vec![
            Vector::new("small_1".to_string(), vec![1.0; 64]),
            Vector::new("small_2".to_string(), vec![2.0; 64]),
        ];

        let large_vectors = vec![
            Vector::new("large_1".to_string(), vec![0.1; 512]),
            Vector::new("large_2".to_string(), vec![0.2; 512]),
        ];

        store.insert("small", small_vectors).unwrap();
        store.insert("large", large_vectors).unwrap();

        // Verify collection metadata
        let small_metadata = store.get_collection_metadata("small").unwrap();
        let large_metadata = store.get_collection_metadata("large").unwrap();

        assert_eq!(small_metadata.vector_count, 2);
        assert_eq!(small_metadata.config.dimension, 64);
        assert_eq!(large_metadata.vector_count, 2);
        assert_eq!(large_metadata.config.dimension, 512);

        // Test search in different collections
        let small_results = store.search("small", &vec![1.5; 64], 2).unwrap();
        let large_results = store.search("large", &vec![0.15; 512], 2).unwrap();

        assert_eq!(small_results.len(), 2);
        assert_eq!(large_results.len(), 2);

        // Test deleting collections
        store.delete_collection("small").unwrap();
        assert_eq!(store.list_collections().len(), 1);
        assert!(store.list_collections().contains(&"large".to_string()));

        store.delete_collection("large").unwrap();
        assert_eq!(store.list_collections().len(), 0);
    }
}
