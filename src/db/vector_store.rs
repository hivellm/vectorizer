//! Main VectorStore implementation

use crate::{
    error::{Result, VectorizerError},
    models::{CollectionConfig, CollectionMetadata, SearchResult, Vector},
};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info};

use super::collection::Collection;

/// Thread-safe in-memory vector store
#[derive(Clone)]
pub struct VectorStore {
    /// Collections stored in a concurrent hash map
    collections: Arc<DashMap<String, Collection>>,
}

impl VectorStore {
    /// Create a new empty vector store
    pub fn new() -> Self {
        info!("Creating new VectorStore");
        Self {
            collections: Arc::new(DashMap::new()),
        }
    }

    /// Create a new collection
    pub fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        debug!("Creating collection '{}' with config: {:?}", name, config);

        if self.collections.contains_key(name) {
            return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
        }

        let collection = Collection::new(name.to_string(), config);
        self.collections.insert(name.to_string(), collection);

        info!("Collection '{}' created successfully", name);
        Ok(())
    }

    /// Delete a collection
    pub fn delete_collection(&self, name: &str) -> Result<()> {
        debug!("Deleting collection '{}'", name);

        self.collections
            .remove(name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))?;

        info!("Collection '{}' deleted successfully", name);
        Ok(())
    }

    /// Get a collection by name
    pub fn get_collection(&self, name: &str) -> Result<Collection> {
        self.collections
            .get(name)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))
    }

    /// List all collections
    pub fn list_collections(&self) -> Vec<String> {
        self.collections
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get collection metadata
    pub fn get_collection_metadata(&self, name: &str) -> Result<CollectionMetadata> {
        let collection = self.get_collection(name)?;
        Ok(collection.metadata())
    }

    /// Insert vectors into a collection
    pub fn insert(&self, collection_name: &str, vectors: Vec<Vector>) -> Result<()> {
        debug!(
            "Inserting {} vectors into collection '{}'",
            vectors.len(),
            collection_name
        );

        let collection = self
            .collections
            .get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        collection.insert_batch(vectors)?;

        Ok(())
    }

    /// Update a vector in a collection
    pub fn update(&self, collection_name: &str, vector: Vector) -> Result<()> {
        debug!(
            "Updating vector '{}' in collection '{}'",
            vector.id, collection_name
        );

        let collection = self
            .collections
            .get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        collection.update(vector)?;

        Ok(())
    }

    /// Delete a vector from a collection
    pub fn delete(&self, collection_name: &str, vector_id: &str) -> Result<()> {
        debug!(
            "Deleting vector '{}' from collection '{}'",
            vector_id, collection_name
        );

        let collection = self
            .collections
            .get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        collection.delete(vector_id)?;

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, collection_name: &str, vector_id: &str) -> Result<Vector> {
        let collection = self
            .collections
            .get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        collection.get_vector(vector_id)
    }

    /// Search for similar vectors
    pub fn search(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        k: usize,
    ) -> Result<Vec<SearchResult>> {
        debug!(
            "Searching for {} nearest neighbors in collection '{}'",
            k, collection_name
        );

        let collection = self
            .collections
            .get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        collection.search(query_vector, k)
    }

    /// Get statistics about the vector store
    pub fn stats(&self) -> VectorStoreStats {
        let mut total_vectors = 0;
        let mut total_memory_bytes = 0;
        
        for entry in self.collections.iter() {
            let collection = entry.value();
            total_vectors += collection.vector_count();
            total_memory_bytes += collection.estimated_memory_usage();
        }
        
        VectorStoreStats {
            collection_count: self.collections.len(),
            total_vectors,
            total_memory_bytes,
        }
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the vector store
#[derive(Debug, Default, Clone)]
pub struct VectorStoreStats {
    /// Number of collections
    pub collection_count: usize,
    /// Total number of vectors across all collections
    pub total_vectors: usize,
    /// Estimated memory usage in bytes
    pub total_memory_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DistanceMetric, HnswConfig, CompressionConfig, Payload};

    #[test]
    fn test_create_and_list_collections() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };

        // Create collections
        store.create_collection("test1", config.clone()).unwrap();
        store.create_collection("test2", config).unwrap();

        // List collections
        let collections = store.list_collections();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&"test1".to_string()));
        assert!(collections.contains(&"test2".to_string()));
    }

    #[test]
    fn test_duplicate_collection_error() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };

        // Create collection
        store.create_collection("test", config.clone()).unwrap();

        // Try to create duplicate
        let result = store.create_collection("test", config);
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionAlreadyExists(_))
        ));
    }

    #[test]
    fn test_delete_collection() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };

        // Create and delete collection
        store.create_collection("test", config).unwrap();
        assert_eq!(store.list_collections().len(), 1);

        store.delete_collection("test").unwrap();
        assert_eq!(store.list_collections().len(), 0);

        // Try to delete non-existent collection
        let result = store.delete_collection("test");
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));
    }

    #[test]
    fn test_vector_operations_integration() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig {
                m: 4,
                ef_construction: 100,
                ef_search: 50,
                seed: Some(42),
            },
            quantization: None,
            compression: Default::default(),
        };

        store.create_collection("test", config).unwrap();

        // Test inserting multiple vectors
        let vectors = vec![
            Vector::with_payload(
                "vec1".to_string(),
                vec![1.0, 0.0, 0.0],
                Payload::from_value(serde_json::json!({"type": "test", "id": 1})).unwrap()
            ),
            Vector::with_payload(
                "vec2".to_string(),
                vec![0.0, 1.0, 0.0],
                Payload::from_value(serde_json::json!({"type": "test", "id": 2})).unwrap()
            ),
            Vector::with_payload(
                "vec3".to_string(),
                vec![0.0, 0.0, 1.0],
                Payload::from_value(serde_json::json!({"type": "test", "id": 3})).unwrap()
            ),
        ];

        store.insert("test", vectors).unwrap();

        // Test search
        let results = store.search("test", &[1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "vec1");

        // Test get individual vector
        let vector = store.get_vector("test", "vec1").unwrap();
        assert_eq!(vector.id, "vec1");
        assert_eq!(vector.data, vec![1.0, 0.0, 0.0]);

        // Test update
        let updated = Vector::with_payload(
            "vec1".to_string(),
            vec![2.0, 0.0, 0.0],
            Payload::from_value(serde_json::json!({"type": "updated", "id": 1})).unwrap()
        );
        store.update("test", updated).unwrap();

        let retrieved = store.get_vector("test", "vec1").unwrap();
        assert_eq!(retrieved.data, vec![2.0, 0.0, 0.0]);

        // Test delete
        store.delete("test", "vec2").unwrap();
        let result = store.get_vector("test", "vec2");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }

    #[test]
    fn test_stats_functionality() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };

        // Empty store stats
        let stats = store.stats();
        assert_eq!(stats.collection_count, 0);
        assert_eq!(stats.total_vectors, 0);

        // Create collection and add vectors
        store.create_collection("test", config).unwrap();
        let vectors = vec![
            Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]),
            Vector::new("v2".to_string(), vec![4.0, 5.0, 6.0]),
        ];
        store.insert("test", vectors).unwrap();

        let stats = store.stats();
        assert_eq!(stats.collection_count, 1);
        assert_eq!(stats.total_vectors, 2);
        assert!(stats.total_memory_bytes > 0);
    }

    #[test]
    fn test_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(VectorStore::new());

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };

        // Create collection from main thread
        store.create_collection("concurrent_test", config).unwrap();

        let mut handles = vec![];

        // Spawn multiple threads to insert vectors
        for i in 0..5 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                let vectors = vec![
                    Vector::new(format!("vec_{}_{}", i, 0), vec![i as f32, 0.0, 0.0]),
                    Vector::new(format!("vec_{}_{}", i, 1), vec![0.0, i as f32, 0.0]),
                ];
                store_clone.insert("concurrent_test", vectors).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all vectors were inserted
        let stats = store.stats();
        assert_eq!(stats.collection_count, 1);
        assert_eq!(stats.total_vectors, 10); // 5 threads * 2 vectors each
    }

    #[test]
    fn test_collection_metadata() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 768,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 32,
                ef_construction: 200,
                ef_search: 64,
                seed: Some(123),
            },
            quantization: None,
            compression: CompressionConfig {
                enabled: true,
                threshold_bytes: 2048,
                algorithm: crate::models::CompressionAlgorithm::Lz4,
            },
        };

        store.create_collection("metadata_test", config.clone()).unwrap();

        // Add some vectors
        let vectors = vec![
            Vector::new("v1".to_string(), vec![0.1; 768]),
            Vector::new("v2".to_string(), vec![0.2; 768]),
        ];
        store.insert("metadata_test", vectors).unwrap();

        // Test metadata retrieval
        let metadata = store.get_collection_metadata("metadata_test").unwrap();
        assert_eq!(metadata.name, "metadata_test");
        assert_eq!(metadata.vector_count, 2);
        assert_eq!(metadata.config.dimension, 768);
        assert_eq!(metadata.config.metric, DistanceMetric::Cosine);
    }

    #[test]
    fn test_error_handling_edge_cases() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };

        store.create_collection("error_test", config).unwrap();

        // Test operations on non-existent collection
        let result = store.insert("non_existent", vec![]);
        assert!(matches!(result, Err(VectorizerError::CollectionNotFound(_))));

        let result = store.search("non_existent", &[1.0, 2.0, 3.0], 1);
        assert!(matches!(result, Err(VectorizerError::CollectionNotFound(_))));

        let result = store.get_vector("non_existent", "v1");
        assert!(matches!(result, Err(VectorizerError::CollectionNotFound(_))));

        // Test operations on non-existent vector
        let result = store.get_vector("error_test", "non_existent");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));

        let result = store.update("error_test", Vector::new("non_existent".to_string(), vec![1.0, 2.0, 3.0]));
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));

        let result = store.delete("error_test", "non_existent");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }
}
