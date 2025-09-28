//! Bend-Enhanced Collection
//! 
//! This module provides a Bend-enhanced version of the Collection that uses
//! Bend for accelerated vector similarity search operations.

use std::sync::Arc;
use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, SearchResult, Vector, DistanceMetric};
use crate::bend::{BendConfig, BendVectorOperations};
use crate::bend::hnsw::BendHnswIndex;
use crate::db::optimized_hnsw::OptimizedHnswConfig;
use dashmap::DashMap;
use parking_lot::RwLock;
use chrono::DateTime;
use chrono::Utc;
use tracing::{debug, info};

/// Bend-enhanced collection with accelerated search
pub struct BendCollection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// Vector storage
    vectors: Arc<DashMap<String, Vector>>,
    /// Vector IDs in insertion order
    vector_order: Arc<RwLock<Vec<String>>>,
    /// Bend-enhanced HNSW index
    bend_index: Arc<RwLock<BendHnswIndex>>,
    /// Embedding type
    embedding_type: Arc<RwLock<String>>,
    /// Document IDs
    document_ids: Arc<DashMap<String, ()>>,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Last update timestamp
    updated_at: Arc<RwLock<DateTime<Utc>>>,
    /// Bend configuration
    bend_config: BendConfig,
}

impl BendCollection {
    /// Create a new Bend-enhanced collection
    pub fn new(name: String, config: CollectionConfig, bend_config: BendConfig) -> Self {
        Self::new_with_embedding_type(name, config, "bm25".to_string(), bend_config)
    }

    /// Create a new Bend-enhanced collection with specific embedding type
    pub fn new_with_embedding_type(
        name: String,
        config: CollectionConfig,
        embedding_type: String,
        bend_config: BendConfig,
    ) -> Self {
        // Convert HnswConfig to OptimizedHnswConfig
        let optimized_config = OptimizedHnswConfig {
            max_connections: config.hnsw_config.m,
            max_connections_0: config.hnsw_config.m * 2,
            ef_construction: config.hnsw_config.ef_construction,
            seed: config.hnsw_config.seed,
            distance_metric: config.metric,
            parallel: true,
            initial_capacity: 100_000,
            batch_size: 1000,
        };

        let bend_index = BendHnswIndex::new(config.dimension, optimized_config, bend_config.clone())
            .expect("Failed to create Bend HNSW index");
        
        let now = Utc::now();

        Self {
            name,
            config,
            vectors: Arc::new(DashMap::new()),
            vector_order: Arc::new(RwLock::new(Vec::new())),
            bend_index: Arc::new(RwLock::new(bend_index)),
            embedding_type: Arc::new(RwLock::new(embedding_type)),
            document_ids: Arc::new(DashMap::new()),
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
            bend_config,
        }
    }

    /// Search for similar vectors with Bend acceleration
    pub async fn search_with_bend(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        // Validate dimension
        if query_vector.len() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: query_vector.len(),
            });
        }

        // Normalize query vector for cosine similarity
        let search_vector = if matches!(self.config.metric, DistanceMetric::Cosine) {
            crate::models::vector_utils::normalize_vector(query_vector)
        } else {
            query_vector.to_vec()
        };

        // Search using Bend-enhanced index
        let index = self.bend_index.read();
        let neighbors = index.search_with_bend(&search_vector, k).await?;

        // Build results
        let mut results = Vec::with_capacity(neighbors.len());
        for (id, score) in neighbors {
            if let Some(vector) = self.vectors.get(&id) {
                results.push(SearchResult {
                    id: id.clone(),
                    score,
                    vector: Some(vector.data.clone()),
                    payload: vector.payload.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Regular search (fallback to HNSW)
    pub fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        // Validate dimension
        if query_vector.len() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: query_vector.len(),
            });
        }

        // Normalize query vector for cosine similarity
        let search_vector = if matches!(self.config.metric, DistanceMetric::Cosine) {
            crate::models::vector_utils::normalize_vector(query_vector)
        } else {
            query_vector.to_vec()
        };

        // Search in HNSW index
        let index = self.bend_index.read();
        let neighbors = index.hnsw_index().search(&search_vector, k)?;

        // Build results
        let mut results = Vec::with_capacity(neighbors.len());
        for (id, score) in neighbors {
            if let Some(vector) = self.vectors.get(&id) {
                results.push(SearchResult {
                    id: id.clone(),
                    score,
                    vector: Some(vector.data.clone()),
                    payload: vector.payload.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Add a vector to the collection
    pub fn add_vector(&self, vector: Vector) -> Result<()> {
        let id = vector.id.clone();

        // Extract document ID from payload
        if let Some(payload) = &vector.payload {
            if let Some(file_path) = payload.data.get("file_path") {
                if let Some(file_path_str) = file_path.as_str() {
                    self.document_ids.insert(file_path_str.to_string(), ());
                }
            }
        }

        // Store vector
        self.vectors.insert(id.clone(), vector.clone());

        // Add to Bend index
        let index = self.bend_index.read();
        index.add(id.clone(), vector.data.clone())?;

        // Track insertion order
        let mut vector_order = self.vector_order.write();
        vector_order.push(id);

        // Update timestamp
        *self.updated_at.write() = Utc::now();

        Ok(())
    }

    /// Batch add vectors with Bend acceleration
    pub async fn batch_add_vectors_with_bend(&self, vectors: Vec<Vector>) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        debug!("Adding {} vectors to Bend collection '{}'", vectors.len(), self.name);

        // Prepare vectors for batch insertion
        let mut batch_vectors = Vec::with_capacity(vectors.len());

        for vector in vectors {
            let id = vector.id.clone();

            // Extract document ID from payload
            if let Some(payload) = &vector.payload {
                if let Some(file_path) = payload.data.get("file_path") {
                    if let Some(file_path_str) = file_path.as_str() {
                        self.document_ids.insert(file_path_str.to_string(), ());
                    }
                }
            }

            // Store vector
            self.vectors.insert(id.clone(), vector.clone());

            // Add to batch
            batch_vectors.push((id.clone(), vector.data.clone()));

            // Track insertion order
            let mut vector_order = self.vector_order.write();
            vector_order.push(id);
        }

        // Batch insert into Bend index
        let index = self.bend_index.read();
        index.batch_add_with_bend(batch_vectors).await?;

        // Update timestamp
        *self.updated_at.write() = Utc::now();

        debug!("Added {} vectors to Bend collection '{}'", self.vectors.len(), self.name);
        Ok(())
    }

    /// Get collection metadata
    pub fn metadata(&self) -> crate::models::CollectionMetadata {
        crate::models::CollectionMetadata {
            name: self.name.clone(),
            created_at: self.created_at,
            updated_at: *self.updated_at.read(),
            vector_count: self.vectors.len(),
            document_count: self.document_ids.len(),
            config: self.config.clone(),
        }
    }

    /// Get Bend statistics
    pub fn get_bend_stats(&self) -> crate::bend::hnsw::BendHnswStats {
        let index = self.bend_index.read();
        index.get_stats()
    }

    /// Get Bend configuration
    pub fn bend_config(&self) -> &BendConfig {
        &self.bend_config
    }

    /// Update Bend configuration
    pub fn update_bend_config(&mut self, config: BendConfig) {
        self.bend_config = config.clone();
        let mut index = self.bend_index.write();
        index.update_bend_config(config);
    }

    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }

    /// Get the embedding type
    pub fn get_embedding_type(&self) -> String {
        self.embedding_type.read().clone()
    }

    /// Set the embedding type
    pub fn set_embedding_type(&self, embedding_type: String) {
        *self.embedding_type.write() = embedding_type;
    }

    /// Remove a vector by ID
    pub fn remove_vector(&self, vector_id: &str) -> Result<()> {
        // Remove from vectors
        if self.vectors.remove(vector_id).is_none() {
            return Err(VectorizerError::VectorNotFound(vector_id.to_string()));
        }

        // Remove from order tracking
        let mut vector_order = self.vector_order.write();
        vector_order.retain(|id| id != vector_id);

        // Remove from Bend index
        let index = self.bend_index.read();
        index.hnsw_index().remove(vector_id)?;

        // Update timestamp
        *self.updated_at.write() = Utc::now();

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        self.vectors
            .get(vector_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))
    }

    /// Estimate memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        let vector_size = std::mem::size_of::<f32>() * self.config.dimension;
        let entry_overhead = std::mem::size_of::<String>() + std::mem::size_of::<Vector>();
        let total_per_vector = vector_size + entry_overhead;

        self.vectors.len() * total_per_vector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CollectionConfig, HnswConfig, Payload};

    fn create_test_config() -> CollectionConfig {
        CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                seed: Some(42),
            },
        }
    }

    #[tokio::test]
    async fn test_bend_collection_creation() {
        let config = create_test_config();
        let bend_config = BendConfig::default();
        
        let collection = BendCollection::new("test".to_string(), config, bend_config);
        assert_eq!(collection.name, "test");
        assert_eq!(collection.vector_count(), 0);
    }

    #[tokio::test]
    async fn test_bend_collection_search() {
        let config = create_test_config();
        let bend_config = BendConfig::default();
        
        let collection = BendCollection::new("test".to_string(), config, bend_config);
        
        // Add test vectors
        let vector1 = Vector {
            id: "vec1".to_string(),
            data: vec![1.0, 0.0, 0.0],
            payload: None,
        };
        
        let vector2 = Vector {
            id: "vec2".to_string(),
            data: vec![0.0, 1.0, 0.0],
            payload: None,
        };
        
        collection.add_vector(vector1).unwrap();
        collection.add_vector(vector2).unwrap();
        
        // Search with Bend
        let results = collection.search_with_bend(&[1.0, 0.0, 0.0], 2).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // First result should be the identical vector
        assert_eq!(results[0].id, "vec1");
        assert!(results[0].score > 0.9);
    }

    #[tokio::test]
    async fn test_bend_collection_batch_add() {
        let config = create_test_config();
        let bend_config = BendConfig::default();
        
        let collection = BendCollection::new("test".to_string(), config, bend_config);
        
        // Create test vectors
        let vectors = vec![
            Vector {
                id: "vec1".to_string(),
                data: vec![1.0, 0.0, 0.0],
                payload: None,
            },
            Vector {
                id: "vec2".to_string(),
                data: vec![0.0, 1.0, 0.0],
                payload: None,
            },
        ];
        
        // Batch add
        collection.batch_add_vectors_with_bend(vectors).await.unwrap();
        
        assert_eq!(collection.vector_count(), 2);
    }
}
