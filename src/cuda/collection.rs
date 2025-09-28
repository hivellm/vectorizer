//! CUDA-Enhanced Collection
//! 
//! This module provides a CUDA-enhanced version of the Collection that uses
//! CUDA for accelerated vector similarity search operations.

use std::sync::Arc;
use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, SearchResult, Vector, Payload, DistanceMetric};
use crate::cuda::{CudaConfig, CudaVectorOperations};
use crate::db::optimized_hnsw::{OptimizedHnswIndex, OptimizedHnswConfig};
use dashmap::DashMap;
use parking_lot::RwLock;
use chrono::DateTime;
use chrono::Utc;
use tracing::{debug, info};

/// CUDA-enhanced collection with accelerated search
pub struct CudaCollection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// Vector storage
    vectors: Arc<DashMap<String, Vector>>,
    /// Vector IDs in insertion order
    vector_order: Arc<RwLock<Vec<String>>>,
    /// CUDA-enhanced HNSW index
    cuda_index: Arc<RwLock<OptimizedHnswIndex>>,
    /// CUDA vector operations
    cuda_operations: CudaVectorOperations,
    /// Embedding type
    embedding_type: Arc<RwLock<String>>,
    /// Document IDs
    document_ids: Arc<DashMap<String, ()>>,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Last update timestamp
    updated_at: Arc<RwLock<DateTime<Utc>>>,
    /// CUDA configuration
    cuda_config: CudaConfig,
}

impl CudaCollection {
    /// Create a new CUDA-enhanced collection
    pub fn new(name: String, config: CollectionConfig, cuda_config: CudaConfig) -> Self {
        Self::new_with_embedding_type(name, config, "bm25".to_string(), cuda_config)
    }

    /// Create a new CUDA-enhanced collection with specific embedding type
    pub fn new_with_embedding_type(
        name: String,
        config: CollectionConfig,
        embedding_type: String,
        cuda_config: CudaConfig,
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

        let cuda_index = OptimizedHnswIndex::new(config.dimension, optimized_config)
            .expect("Failed to create CUDA HNSW index");
        
        let cuda_operations = CudaVectorOperations::new(cuda_config.clone());
        let now = Utc::now();

        Self {
            name,
            config,
            vectors: Arc::new(DashMap::new()),
            vector_order: Arc::new(RwLock::new(Vec::new())),
            cuda_index: Arc::new(RwLock::new(cuda_index)),
            cuda_operations,
            embedding_type: Arc::new(RwLock::new(embedding_type)),
            document_ids: Arc::new(DashMap::new()),
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
            cuda_config,
        }
    }

    /// Search for similar vectors with CUDA acceleration
    pub async fn search_with_cuda(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
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

        // Get all vectors for CUDA processing
        let vectors = self.cuda_index.read().get_all_vectors()?;
        
        if vectors.is_empty() {
            return Ok(vec![]);
        }

        // Extract vector data and IDs
        let vector_data: Vec<Vec<f32>> = vectors.values().cloned().collect();
        let vector_ids: Vec<String> = vectors.keys().cloned().collect();

        // Use CUDA for parallel similarity search
        let similarities = self.cuda_operations
            .parallel_similarity_search(&search_vector, &vector_data, 0.0, self.config.metric)
            .await?;

        // Combine results with IDs and sort by similarity
        let mut results: Vec<(String, f32)> = vector_ids
            .into_iter()
            .zip(similarities.into_iter())
            .collect();

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k results
        results.truncate(k);

        // Build SearchResult objects
        let mut search_results = Vec::with_capacity(results.len());
        for (id, score) in results {
            if let Some(vector) = self.vectors.get(&id) {
                search_results.push(SearchResult {
                    id: id.clone(),
                    score,
                    vector: Some(vector.data.clone()),
                    payload: vector.payload.clone(),
                });
            }
        }

        Ok(search_results)
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
        let index = self.cuda_index.read();
        let neighbors = index.search(&search_vector, k)?;

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

        // Add to CUDA index
        let index = self.cuda_index.read();
        index.add(id.clone(), vector.data.clone())?;

        // Track insertion order
        let mut vector_order = self.vector_order.write();
        vector_order.push(id);

        // Update timestamp
        *self.updated_at.write() = Utc::now();

        Ok(())
    }

    /// Batch add vectors with CUDA acceleration
    pub async fn batch_add_vectors_with_cuda(&self, vectors: Vec<Vector>) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        debug!("Adding {} vectors to CUDA collection '{}'", vectors.len(), self.name);

        // Prepare vectors for batch insertion
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

            // Add to CUDA index
            let index = self.cuda_index.read();
            index.add(id.clone(), vector.data.clone())?;

            // Track insertion order
            let mut vector_order = self.vector_order.write();
            vector_order.push(id);
        }

        // Update timestamp
        *self.updated_at.write() = Utc::now();

        debug!("Added {} vectors to CUDA collection '{}'", self.vectors.len(), self.name);
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

    /// Get CUDA configuration
    pub fn cuda_config(&self) -> &CudaConfig {
        &self.cuda_config
    }

    /// Check if CUDA is available
    pub fn is_cuda_available(&self) -> bool {
        self.cuda_operations.is_cuda_available()
    }

    /// Get CUDA device info
    pub fn get_cuda_device_info(&self) -> Result<crate::cuda::CudaDeviceInfo> {
        self.cuda_operations.get_device_info()
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

        // Remove from CUDA index
        let index = self.cuda_index.read();
        index.remove(vector_id)?;

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

    /// Initialize CUHNSW with data if available
    pub fn initialize_cuhnsw(&mut self, data: &[f32], num_data: usize, num_dims: usize) -> Result<()> {
        self.cuda_operations.initialize_cuhnsw(data, num_data, num_dims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CollectionConfig, HnswConfig, CompressionConfig};

    fn create_test_config() -> CollectionConfig {
        CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 50,
                seed: Some(42),
            },
            compression: CompressionConfig::default(),
            quantization: None,
        }
    }

    #[tokio::test]
    async fn test_cuda_collection_creation() {
        let config = create_test_config();
        let cuda_config = CudaConfig::default();
        
        let collection = CudaCollection::new("test".to_string(), config, cuda_config);
        assert_eq!(collection.name, "test");
        assert_eq!(collection.vector_count(), 0);
    }

    #[tokio::test]
    async fn test_cuda_collection_search() {
        let config = create_test_config();
        let cuda_config = CudaConfig::default();
        
        let collection = CudaCollection::new("test".to_string(), config, cuda_config);
        
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
        
        // Search with CUDA
        let results = collection.search_with_cuda(&[1.0, 0.0, 0.0], 2).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // First result should be the identical vector
        assert_eq!(results[0].id, "vec1");
        assert!(results[0].score > 0.9);
    }

    #[tokio::test]
    async fn test_cuda_collection_batch_add() {
        let config = create_test_config();
        let cuda_config = CudaConfig::default();
        
        let collection = CudaCollection::new("test".to_string(), config, cuda_config);
        
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
        collection.batch_add_vectors_with_cuda(vectors).await.unwrap();
        
        assert_eq!(collection.vector_count(), 2);
    }
}
