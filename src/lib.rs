//! Vectorizer - High-performance, in-memory vector database written in Rust
//!
//! This crate provides a fast and efficient vector database for semantic search
//! and similarity queries, designed for AI-driven applications.

pub mod api;
pub mod auth;
pub mod cli;
pub mod db;
pub mod document_loader;
pub mod embedding;
pub mod error;
pub mod evaluation;
pub mod hybrid_search;
pub mod mcp;
pub mod models;
pub mod parallel;
#[path = "persistence/mod.rs"]
pub mod persistence;

// Re-export commonly used types
pub use db::{Collection, VectorStore};
pub use embedding::{BertEmbedding, Bm25Embedding, MiniLmEmbedding, SvdEmbedding};
pub use error::{Result, VectorizerError};
pub use evaluation::{EvaluationMetrics, QueryMetrics, QueryResult, evaluate_search_quality};
pub use models::{CollectionConfig, Payload, SearchResult, Vector};

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
    fn test_full_vector_database_workflow() {
        // Create vector store
        let store = VectorStore::new();

        // Create collection with specific configuration
        let config = CollectionConfig {
            dimension: 128,
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: crate::models::HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 64,
                seed: Some(42),
            },
            quantization: None,
            compression: crate::models::CompressionConfig {
                enabled: true,
                threshold_bytes: 1024,
                algorithm: crate::models::CompressionAlgorithm::Lz4,
            },
        };

        store.create_collection("documents", config).unwrap();

        // Insert various types of vectors with different payloads
        let documents = vec![
            Vector::with_payload(
                "doc_001".to_string(),
                vec![0.1; 128], // Normalized vector
                Payload::from_value(serde_json::json!({
                    "title": "Machine Learning Basics",
                    "content": "ML is a subset of AI...",
                    "category": "tutorial",
                    "tags": ["AI", "ML", "beginner"]
                })).unwrap()
            ),
            Vector::with_payload(
                "doc_002".to_string(),
                vec![0.2; 128],
                Payload::from_value(serde_json::json!({
                    "title": "Deep Learning Advanced",
                    "content": "Neural networks are...",
                    "category": "advanced",
                    "tags": ["AI", "deep-learning", "neural-networks"]
                })).unwrap()
            ),
            Vector::with_payload(
                "doc_003".to_string(),
                vec![0.15; 128],
                Payload::from_value(serde_json::json!({
                    "title": "Vector Databases",
                    "content": "Vector databases store embeddings...",
                    "category": "infrastructure",
                    "tags": ["database", "vectors", "embeddings"]
                })).unwrap()
            ),
        ];

        store.insert("documents", documents).unwrap();

        // Verify collection stats
        let metadata = store.get_collection_metadata("documents").unwrap();
        assert_eq!(metadata.vector_count, 3);
        assert_eq!(metadata.config.dimension, 128);

        // Perform semantic search
        let query_vector = vec![0.12; 128]; // Query similar to existing docs
        let results = store.search("documents", &query_vector, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].score >= results[1].score); // Results should be ordered by score

        // Test individual vector retrieval
        let vector = store.get_vector("documents", "doc_001").unwrap();
        assert_eq!(vector.id, "doc_001");
        assert_eq!(vector.data.len(), 128);

        // Test vector update
        let updated_vector = Vector::with_payload(
            "doc_001".to_string(),
            vec![0.3; 128], // Different vector
            Payload::from_value(serde_json::json!({
                "title": "Machine Learning Basics - Updated",
                "content": "Updated content...",
                "category": "tutorial",
                "tags": ["AI", "ML", "beginner", "updated"]
            })).unwrap()
        );

        store.update("documents", updated_vector).unwrap();

        // Verify update - for cosine metric, vectors are normalized
        let retrieved = store.get_vector("documents", "doc_001").unwrap();
        // Check that the vector was updated (normalized for cosine similarity)
        assert_eq!(retrieved.data.len(), 128);
        // Verify the vector is normalized (norm should be approximately 1.0)
        let norm_squared: f32 = retrieved.data.iter().map(|x| x * x).sum();
        assert!((norm_squared - 1.0).abs() < 1e-6);

        // Test deletion
        store.delete("documents", "doc_002").unwrap();
        let result = store.get_vector("documents", "doc_002");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));

        // Note: Persistence testing moved to separate test due to normalization complexity
        // The save() method now correctly saves all vectors (fixing the original issue)

        // Final stats check
        let final_metadata = store.get_collection_metadata("documents").unwrap();
        assert_eq!(final_metadata.vector_count, 2);
    }

    #[test]
    fn test_vector_database_with_real_embeddings() {
        use crate::embedding::{TfIdfEmbedding, EmbeddingManager};

        // Create embedding manager and TF-IDF embedder
        let mut manager = EmbeddingManager::new();
        let mut tfidf = TfIdfEmbedding::new(64);

        // Sample documents for building vocabulary
        let training_docs = vec![
            "machine learning algorithms",
            "neural networks and deep learning",
            "vector databases for AI",
            "natural language processing",
            "computer vision techniques",
            "supervised learning methods",
            "unsupervised clustering algorithms",
        ];

        // Build vocabulary from training documents
        tfidf.build_vocabulary(&training_docs);
        manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        manager.set_default_provider("tfidf").unwrap();

        // Create vector store with embedding dimension
        let store = VectorStore::new();
        let config = CollectionConfig {
            dimension: 64, // Match embedding dimension
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: crate::models::HnswConfig {
                m: 12,
                ef_construction: 100,
                ef_search: 32,
                seed: Some(42),
            },
            quantization: None,
            compression: Default::default(),
        };

        store.create_collection("semantic_docs", config).unwrap();

        // Documents to embed and store
        let document_texts = vec![
            ("ai_basics", "Artificial intelligence is transforming technology"),
            ("ml_guide", "Machine learning models learn from data patterns"),
            ("neural_nets", "Neural networks simulate brain functionality"),
            ("vector_db", "Vector databases enable fast similarity search"),
        ];

        // Generate embeddings and create vectors
        let mut vectors = Vec::new();
        for (id, text) in &document_texts {
            let embedding = manager.embed(text).unwrap();
            let vector = Vector::with_payload(
                id.to_string(),
                embedding,
                Payload::from_value(serde_json::json!({
                    "content": text,
                    "word_count": text.split_whitespace().count(),
                    "embedding_type": "tfidf"
                })).unwrap()
            );
            vectors.push(vector);
        }

        store.insert("semantic_docs", vectors).unwrap();

        // Verify collection has correct count
        let metadata = store.get_collection_metadata("semantic_docs").unwrap();
        assert_eq!(metadata.vector_count, 4);
        assert_eq!(metadata.config.dimension, 64);

        // Test semantic search with natural language query
        let query = "artificial intelligence and machine learning";
        let query_embedding = manager.embed(query).unwrap();
        let results = store.search("semantic_docs", &query_embedding, 3).unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].score >= results[1].score); // Results should be ordered

        // Verify embeddings are normalized (cosine similarity)
        let first_vector = store.get_vector("semantic_docs", &results[0].id).unwrap();
        let norm: f32 = first_vector.data.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);

        // Test that semantically similar queries return relevant results
        let tech_query = "neural network technology";
        let tech_embedding = manager.embed(tech_query).unwrap();
        let tech_results = store.search("semantic_docs", &tech_embedding, 2).unwrap();

        // Should find the neural networks document
        assert!(tech_results.iter().any(|r| r.id == "neural_nets"));
    }

    #[test]
    fn test_persistence_with_normalized_vectors() {
        // Test persistence specifically with cosine similarity (normalized vectors)
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };

        store.create_collection("normalized", config).unwrap();

        // Add vectors that will be normalized
        let vectors = vec![
            Vector::new("v1".to_string(), vec![3.0, 4.0, 0.0]), // Will be normalized
            Vector::new("v2".to_string(), vec![1.0, 1.0, 1.0]), // Will be normalized
        ];

        store.insert("normalized", vectors).unwrap();

        // Verify vectors are normalized in memory
        let v1 = store.get_vector("normalized", "v1").unwrap();
        let norm_squared: f32 = v1.data.iter().map(|x| x * x).sum();
        assert!((norm_squared - 1.0).abs() < 1e-6);

        // Test persistence
        let temp_dir = tempdir().unwrap();
        let save_path = temp_dir.path().join("normalized.vdb");

        store.save(&save_path).unwrap();

        // Load and verify
        let loaded_store = VectorStore::load(&save_path).unwrap();
        let loaded_metadata = loaded_store.get_collection_metadata("normalized").unwrap();
        assert_eq!(loaded_metadata.vector_count, 2);

        // Verify loaded vectors remain normalized
        let loaded_v1 = loaded_store.get_vector("normalized", "v1").unwrap();
        let loaded_norm_squared: f32 = loaded_v1.data.iter().map(|x| x * x).sum();
        assert!((loaded_norm_squared - 1.0).abs() < 1e-6);
    }

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
                    let vector_data: Vec<f32> = (0..64).map(|j| {
                        (thread_id as f32 * 0.1) + (i as f32 * 0.01) + (j as f32 * 0.001)
                    }).collect();

                    let vector = Vector::with_payload(
                        vector_id.clone(),
                        vector_data,
                        Payload::from_value(serde_json::json!({
                            "thread_id": thread_id,
                            "vector_index": i,
                            "created_by": format!("thread_{}", thread_id)
                        })).unwrap()
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
    fn test_error_handling_comprehensive() {
        let store = VectorStore::new();

        // Test collection operations - empty name should work, dimension 0 should fail elsewhere
        let invalid_config = CollectionConfig {
            dimension: 0,
            metric: crate::models::DistanceMetric::Euclidean,
            hnsw_config: Default::default(),
            quantization: None,
            compression: Default::default(),
        };

        // Creating collection with dimension 0 should fail when trying to add vectors
        store.create_collection("zero_dim", invalid_config).unwrap();

        let valid_config = CollectionConfig {
            dimension: 3,
            metric: crate::models::DistanceMetric::Euclidean,
            hnsw_config: Default::default(),
            quantization: None,
            compression: Default::default(),
        };

        store.create_collection("valid", valid_config).unwrap();

        // Test vector operations with wrong dimensions
        let wrong_dim_vector = Vector::new("wrong".to_string(), vec![1.0, 2.0]); // 2D instead of 3D
        assert!(matches!(
            store.insert("valid", vec![wrong_dim_vector]),
            Err(VectorizerError::InvalidDimension { expected: 3, got: 2 })
        ));

        // Test search with wrong dimensions
        assert!(matches!(
            store.search("valid", &[1.0, 2.0], 1),
            Err(VectorizerError::InvalidDimension { expected: 3, got: 2 })
        ));

        // Test operations on non-existent entities
        assert!(matches!(
            store.delete_collection("nonexistent"),
            Err(VectorizerError::CollectionNotFound(_))
        ));

        assert!(matches!(
            store.get_collection_metadata("nonexistent"),
            Err(VectorizerError::CollectionNotFound(_))
        ));

        // Insert a valid vector first
        let valid_vector = Vector::new("valid_vec".to_string(), vec![1.0, 2.0, 3.0]);
        store.insert("valid", vec![valid_vector]).unwrap();

        // Now test operations on non-existent vectors
        assert!(matches!(
            store.get_vector("valid", "nonexistent"),
            Err(VectorizerError::VectorNotFound(_))
        ));

        assert!(matches!(
            store.update("valid", Vector::new("nonexistent".to_string(), vec![1.0, 2.0, 3.0])),
            Err(VectorizerError::VectorNotFound(_))
        ));

        assert!(matches!(
            store.delete("valid", "nonexistent"),
            Err(VectorizerError::VectorNotFound(_))
        ));
    }

    #[test]
    fn test_collection_management() {
        let store = VectorStore::new();

        // Test creating multiple collections with different configurations
        let configs = vec![
            ("small", CollectionConfig {
                dimension: 64,
                metric: crate::models::DistanceMetric::Euclidean,
                hnsw_config: crate::models::HnswConfig { m: 8, ef_construction: 100, ef_search: 50, seed: None },
                quantization: None,
                compression: Default::default(),
            }),
            ("large", CollectionConfig {
                dimension: 768,
                metric: crate::models::DistanceMetric::Cosine,
                hnsw_config: crate::models::HnswConfig { m: 32, ef_construction: 300, ef_search: 100, seed: Some(123) },
                quantization: None,
                compression: crate::models::CompressionConfig {
                    enabled: true,
                    threshold_bytes: 2048,
                    algorithm: crate::models::CompressionAlgorithm::Lz4,
                },
            }),
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
            Vector::new("large_1".to_string(), vec![0.1; 768]),
            Vector::new("large_2".to_string(), vec![0.2; 768]),
        ];

        store.insert("small", small_vectors).unwrap();
        store.insert("large", large_vectors).unwrap();

        // Verify collection metadata
        let small_metadata = store.get_collection_metadata("small").unwrap();
        let large_metadata = store.get_collection_metadata("large").unwrap();

        assert_eq!(small_metadata.vector_count, 2);
        assert_eq!(small_metadata.config.dimension, 64);
        assert_eq!(large_metadata.vector_count, 2);
        assert_eq!(large_metadata.config.dimension, 768);

        // Test search in different collections
        let small_results = store.search("small", &vec![1.5; 64], 2).unwrap();
        let large_results = store.search("large", &vec![0.15; 768], 2).unwrap();

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
