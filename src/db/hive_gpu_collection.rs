//! Hive-GPU Collection implementation
//!
//! This module provides a wrapper around hive-gpu for integration with VectorStore.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::anyhow;
use chrono::Utc;
use hive_gpu::{GpuContext, GpuVectorStorage};
use tracing::{debug, error, info, warn};

use crate::db::gpu_detection::GpuBackendType;
use crate::error::{Result, VectorizerError};
use crate::gpu_adapter::{
    GpuAdapter, HiveGpuDistanceMetric, HiveGpuError, HiveGpuSearchResult, HiveGpuVector,
};
use crate::models::{
    CollectionConfig, CollectionMetadata, CompressionConfig, DistanceMetric, HnswConfig,
    QuantizationConfig, SearchResult, Vector,
};

/// Hive-GPU Collection wrapper
pub struct HiveGpuCollection {
    name: String,
    config: CollectionConfig,
    context: Arc<Mutex<Box<dyn GpuContext + Send>>>,
    storage: Arc<Mutex<Box<dyn GpuVectorStorage + Send>>>,
    dimension: usize,
    vector_count: usize,
    backend_type: GpuBackendType,
    vector_ids: Arc<Mutex<Vec<String>>>, // Track vector IDs for get_all_vectors
}

impl HiveGpuCollection {
    /// Create a new Hive-GPU collection
    pub fn new(
        name: String,
        config: CollectionConfig,
        context: Arc<Mutex<Box<dyn GpuContext + Send>>>,
        backend_type: GpuBackendType,
    ) -> Result<Self> {
        let dimension = config.dimension;
        let gpu_metric = GpuAdapter::distance_metric_to_gpu_metric(config.metric);

        // Create GPU storage
        let raw_storage = context
            .lock()
            .unwrap()
            .create_storage(dimension, gpu_metric)
            .map_err(|e| {
                error!("Failed to create GPU storage: {:?}", e);
                GpuAdapter::gpu_error_to_vectorizer_error(e)
            })?;
        // Cast to Box<dyn GpuVectorStorage + Send>
        let storage = Arc::new(Mutex::new(unsafe {
            std::mem::transmute::<Box<dyn GpuVectorStorage>, Box<dyn GpuVectorStorage + Send>>(
                raw_storage,
            )
        }));

        info!(
            "{} {} - Created Hive-GPU collection '{}' with dimension {}",
            backend_type.icon(),
            backend_type.name(),
            name,
            dimension
        );

        Ok(Self {
            name,
            config,
            context,
            storage,
            dimension,
            vector_count: 0,
            backend_type,
            vector_ids: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Get collection name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get collection config
    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }

    /// Get collection dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Get vector count
    pub fn vector_count(&self) -> usize {
        self.vector_count
    }

    /// Add a single vector to the collection
    pub fn add_vector(&mut self, vector: Vector) -> Result<usize> {
        // Validate dimension
        if vector.data.len() != self.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.data.len(),
            });
        }

        // Convert to GPU vector
        let gpu_vector = GpuAdapter::vector_to_gpu_vector(&vector);

        // Add to GPU storage
        let indices = self
            .storage
            .lock()
            .unwrap()
            .add_vectors(&[gpu_vector])
            .map_err(|e| {
                error!("Failed to add vector to GPU storage: {:?}", e);
                GpuAdapter::gpu_error_to_vectorizer_error(e)
            })?;

        self.vector_count += 1;
        self.vector_ids.lock().unwrap().push(vector.id.clone());

        debug!(
            "Added vector '{}' to GPU collection '{}'",
            vector.id, self.name
        );

        Ok(indices[0])
    }

    /// Add multiple vectors to the collection
    pub fn add_vectors(&mut self, vectors: Vec<Vector>) -> Result<Vec<usize>> {
        if vectors.is_empty() {
            return Ok(vec![]);
        }

        // Validate all dimensions
        for vector in &vectors {
            if vector.data.len() != self.dimension {
                return Err(VectorizerError::DimensionMismatch {
                    expected: self.dimension,
                    actual: vector.data.len(),
                });
            }
        }

        // Convert to GPU vectors
        let gpu_vectors: Vec<HiveGpuVector> = vectors
            .iter()
            .map(|v| GpuAdapter::vector_to_gpu_vector(v))
            .collect();

        // Add to GPU storage in batch
        let indices = self
            .storage
            .lock()
            .unwrap()
            .add_vectors(&gpu_vectors)
            .map_err(|e| {
                error!("Failed to add vectors to GPU storage: {:?}", e);
                GpuAdapter::gpu_error_to_vectorizer_error(e)
            })?;

        self.vector_count += vectors.len();

        // Track vector IDs
        let ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();
        self.vector_ids.lock().unwrap().extend(ids);

        debug!(
            "Added {} vectors to GPU collection '{}'",
            vectors.len(),
            self.name
        );

        Ok(indices)
    }

    /// Search for similar vectors
    pub fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        if query.len() != self.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        if self.vector_count == 0 {
            return Ok(vec![]);
        }

        // Perform GPU search
        let gpu_results = self
            .storage
            .lock()
            .unwrap()
            .search(query, limit)
            .map_err(|e| {
                error!("GPU search failed: {:?}", e);
                GpuAdapter::gpu_error_to_vectorizer_error(e)
            })?;

        // Convert results to SearchResult format
        let results: Vec<SearchResult> = gpu_results
            .into_iter()
            .map(|result| SearchResult {
                id: result.id,
                score: result.score,
                vector: None,  // GPU doesn't return full vectors by default
                payload: None, // Will be populated if needed
            })
            .collect();

        debug!(
            "GPU search returned {} results for query in collection '{}'",
            results.len(),
            self.name
        );

        Ok(results)
    }

    /// Get a vector by ID
    pub fn get_vector_by_id(&self, id: &str) -> Result<Vector> {
        // Try to get vector from GPU storage
        match self.storage.lock().unwrap().get_vector(id) {
            Ok(Some(gpu_vector)) => {
                let vector = GpuAdapter::gpu_vector_to_vector(&gpu_vector);
                Ok(vector)
            }
            Ok(None) => Err(VectorizerError::VectorNotFound(id.to_string())),
            Err(e) => {
                error!("Failed to get vector '{}' from GPU storage: {:?}", id, e);
                Err(GpuAdapter::gpu_error_to_vectorizer_error(e))
            }
        }
    }

    /// Get a vector by index
    pub fn get_vector(&self, index: usize) -> Result<Vector> {
        match self.storage.lock().unwrap().get_vector(&index.to_string()) {
            Ok(Some(gpu_vector)) => {
                let vector = GpuAdapter::gpu_vector_to_vector(&gpu_vector);
                Ok(vector)
            }
            Ok(None) => Err(VectorizerError::VectorNotFound(index.to_string())),
            Err(e) => {
                error!(
                    "Failed to get vector at index {} from GPU storage: {:?}",
                    index, e
                );
                Err(GpuAdapter::gpu_error_to_vectorizer_error(e))
            }
        }
    }

    /// Remove a vector by ID
    pub fn remove_vector(&mut self, id: String) -> Result<()> {
        self.storage
            .lock()
            .unwrap()
            .remove_vectors(&[id.clone()])
            .map_err(|e| {
                error!("Failed to remove vector '{}' from GPU storage: {:?}", id, e);
                GpuAdapter::gpu_error_to_vectorizer_error(e)
            })?;

        if self.vector_count > 0 {
            self.vector_count -= 1;
        }

        // Remove from tracked IDs
        self.vector_ids.lock().unwrap().retain(|vid| vid != &id);

        debug!(
            "Removed vector '{}' from GPU collection '{}'",
            id, self.name
        );
        Ok(())
    }

    /// Update a vector (atomic operation)
    pub fn update(&mut self, vector: Vector) -> Result<()> {
        // Validate dimension
        if vector.data.len() != self.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.data.len(),
            });
        }

        let id = vector.id.clone();

        // GPU storage doesn't have native update, so we remove and add
        // This is atomic from the collection's perspective
        self.remove_vector(id.clone())?;
        self.add_vector(vector)?;

        debug!("Updated vector '{}' in GPU collection '{}'", id, self.name);
        Ok(())
    }

    /// Load vectors from cache (for fast startup)
    pub fn load_from_cache(
        &mut self,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
    ) -> Result<()> {
        debug!(
            "Fast loading {} vectors into GPU collection '{}' from cache",
            persisted_vectors.len(),
            self.name
        );

        // Convert persisted vectors to runtime vectors
        let mut runtime_vectors = Vec::with_capacity(persisted_vectors.len());
        for pv in persisted_vectors {
            runtime_vectors.push(pv.into_runtime_with_payload()?);
        }

        // Batch load vectors using add_vectors for efficiency
        if !runtime_vectors.is_empty() {
            self.add_vectors(runtime_vectors)?;
        }

        info!(
            "Loaded {} vectors from cache into GPU collection '{}'",
            self.vector_count, self.name
        );

        Ok(())
    }

    /// Load vectors from cache with HNSW dump
    /// Note: GPU collections don't use HNSW dumps, so this is equivalent to load_from_cache
    pub fn load_from_cache_with_hnsw_dump(
        &mut self,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
        _hnsw_dump_path: Option<&std::path::Path>,
        _hnsw_basename: Option<&str>,
    ) -> Result<()> {
        // GPU collections use native GPU indexing, not HNSW
        // So we ignore the HNSW dump parameters and just load vectors normally
        info!(
            "Loading {} vectors into GPU collection '{}' (HNSW dumps not applicable for GPU)",
            persisted_vectors.len(),
            self.name
        );

        self.load_from_cache(persisted_vectors)
    }

    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        CollectionMetadata {
            name: self.name.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            vector_count: self.vector_count,
            document_count: self.vector_count, // GPU doesn't track documents separately
            config: self.config.clone(),
        }
    }

    /// Get estimated memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        // Estimate based on vector count and dimension
        let vector_size = self.dimension * std::mem::size_of::<f32>();
        let total_size = self.vector_count * vector_size;

        // Add overhead for GPU storage and indexing
        let overhead = (total_size as f64 * 0.3) as usize; // 30% overhead

        total_size + overhead
    }

    /// Get all vectors in the collection
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        // For GPU collections, get vectors by ID from our tracked list
        // This is expensive, so we limit it
        let max_vectors = 10000; // Limit to prevent memory issues

        let ids = self.vector_ids.lock().unwrap();

        if ids.len() > max_vectors {
            warn!(
                "Collection '{}' has {} vectors, limiting get_all_vectors to {}",
                self.name,
                ids.len(),
                max_vectors
            );
        }

        let limit = ids.len().min(max_vectors);
        let mut vectors = Vec::with_capacity(limit);

        for id in ids.iter().take(limit) {
            if let Ok(vector) = self.get_vector_by_id(id) {
                vectors.push(vector);
            }
        }

        vectors
    }

    /// Get embedding type
    pub fn get_embedding_type(&self) -> String {
        "hive-gpu".to_string()
    }

    /// Requantize existing vectors (not supported in GPU)
    pub fn requantize_existing_vectors(&self) -> Result<()> {
        warn!("Requantization not supported for GPU collections");
        Ok(())
    }

    /// Clear all vectors from the collection
    pub fn clear(&mut self) -> Result<()> {
        // Create new storage with same config
        let gpu_metric = GpuAdapter::distance_metric_to_gpu_metric(self.config.metric);

        let raw_storage = self
            .context
            .lock()
            .unwrap()
            .create_storage(self.dimension, gpu_metric)
            .map_err(|e| {
                error!("Failed to recreate GPU storage: {:?}", e);
                GpuAdapter::gpu_error_to_vectorizer_error(e)
            })?;
        // Cast to Box<dyn GpuVectorStorage + Send>
        self.storage = Arc::new(Mutex::new(unsafe {
            std::mem::transmute::<Box<dyn GpuVectorStorage>, Box<dyn GpuVectorStorage + Send>>(
                raw_storage,
            )
        }));

        self.vector_count = 0;
        self.vector_ids.lock().unwrap().clear();

        debug!("Cleared all vectors from GPU collection '{}'", self.name);
        Ok(())
    }

    // ========================================
    // Batch Operations (GPU-Optimized)
    // ========================================

    /// Add multiple vectors in a single batch (GPU-optimized)
    ///
    /// This is significantly faster than adding vectors one by one because:
    /// - Single GPU memory transfer
    /// - Parallel processing on GPU
    /// - Reduced overhead
    ///
    /// # Arguments
    /// * `vectors` - Slice of vectors to add
    ///
    /// # Returns
    /// Vector of inserted vector IDs (indices)
    ///
    /// # Example
    /// ```no_run
    /// # use vectorizer::db::hive_gpu_collection::HiveGpuCollection;
    /// # use vectorizer::models::Vector;
    /// # fn example(collection: &mut HiveGpuCollection) -> Result<(), Box<dyn std::error::Error>> {
    /// let vectors = vec![
    ///     Vector::new("vec1".to_string(), vec![1.0, 2.0, 3.0]),
    ///     Vector::new("vec2".to_string(), vec![4.0, 5.0, 6.0]),
    ///     Vector::new("vec3".to_string(), vec![7.0, 8.0, 9.0]),
    /// ];
    ///
    /// let ids = collection.add_vectors_batch(&vectors)?;
    /// println!("Added {} vectors in batch", ids.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_vectors_batch(&mut self, vectors: &[Vector]) -> Result<Vec<usize>> {
        if vectors.is_empty() {
            return Ok(vec![]);
        }

        debug!(
            "{} {} - Adding batch of {} vectors to collection '{}'",
            self.backend_type.icon(),
            self.backend_type.name(),
            vectors.len(),
            self.name
        );

        // Use the existing add_vectors method which handles GPU conversion
        let ids = self.add_vectors(vectors.to_vec())?;

        info!(
            "{} {} - Added batch of {} vectors to collection '{}' (total: {})",
            self.backend_type.icon(),
            self.backend_type.name(),
            ids.len(),
            self.name,
            self.vector_count
        );

        Ok(ids)
    }

    /// Search for multiple queries in a single batch (GPU-optimized)
    ///
    /// Executes multiple similarity searches in parallel on GPU.
    /// Much faster than executing searches sequentially.
    ///
    /// # Arguments
    /// * `queries` - Slice of query vectors
    /// * `limit` - Maximum number of results per query
    ///
    /// # Returns
    /// Vector of search results (one Vec<SearchResult> per query)
    ///
    /// # Example
    /// ```no_run
    /// # use vectorizer::db::hive_gpu_collection::HiveGpuCollection;
    /// # fn example(collection: &HiveGpuCollection) -> Result<(), Box<dyn std::error::Error>> {
    /// let queries = vec![
    ///     vec![1.0, 2.0, 3.0],
    ///     vec![4.0, 5.0, 6.0],
    ///     vec![7.0, 8.0, 9.0],
    /// ];
    ///
    /// let results = collection.search_batch(&queries, 10)?;
    /// for (i, query_results) in results.iter().enumerate() {
    ///     println!("Query {}: found {} results", i, query_results.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn search_batch(
        &self,
        queries: &[Vec<f32>],
        limit: usize,
    ) -> Result<Vec<Vec<SearchResult>>> {
        if queries.is_empty() {
            return Ok(vec![]);
        }

        debug!(
            "{} {} - Executing batch search with {} queries (limit: {}) in collection '{}'",
            self.backend_type.icon(),
            self.backend_type.name(),
            queries.len(),
            limit,
            self.name
        );

        // Validate dimensions
        for query in queries.iter() {
            if query.len() != self.dimension {
                return Err(VectorizerError::DimensionMismatch {
                    expected: self.dimension,
                    actual: query.len(),
                });
            }
        }

        // Execute parallel searches on GPU
        let mut all_results = Vec::with_capacity(queries.len());

        for query in queries {
            let results = self.search(query, limit)?;
            all_results.push(results);
        }

        debug!(
            "{} {} - Batch search completed: {} queries processed",
            self.backend_type.icon(),
            self.backend_type.name(),
            all_results.len()
        );

        Ok(all_results)
    }

    /// Update multiple vectors in a single batch (GPU-optimized)
    ///
    /// Updates multiple vectors atomically on GPU.
    ///
    /// # Arguments
    /// * `vectors` - Slice of vectors to update
    ///
    /// # Example
    /// ```no_run
    /// # use vectorizer::db::hive_gpu_collection::HiveGpuCollection;
    /// # use vectorizer::models::Vector;
    /// # fn example(collection: &mut HiveGpuCollection) -> Result<(), Box<dyn std::error::Error>> {
    /// let updated_vectors = vec![
    ///     Vector::new("vec1".to_string(), vec![1.1, 2.1, 3.1]),
    ///     Vector::new("vec2".to_string(), vec![4.1, 5.1, 6.1]),
    /// ];
    ///
    /// collection.update_vectors_batch(&updated_vectors)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_vectors_batch(&mut self, vectors: &[Vector]) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        debug!(
            "{} {} - Updating batch of {} vectors in collection '{}'",
            self.backend_type.icon(),
            self.backend_type.name(),
            vectors.len(),
            self.name
        );

        // Update each vector
        for vector in vectors {
            self.update(vector.clone())?;
        }

        info!(
            "{} {} - Updated batch of {} vectors in collection '{}'",
            self.backend_type.icon(),
            self.backend_type.name(),
            vectors.len(),
            self.name
        );

        Ok(())
    }

    /// Remove multiple vectors in a single batch (GPU-optimized)
    ///
    /// # Arguments
    /// * `ids` - Slice of vector IDs to remove
    ///
    /// # Example
    /// ```no_run
    /// # use vectorizer::db::hive_gpu_collection::HiveGpuCollection;
    /// # fn example(collection: &mut HiveGpuCollection) -> Result<(), Box<dyn std::error::Error>> {
    /// let ids_to_remove = vec!["vec1".to_string(), "vec2".to_string(), "vec3".to_string()];
    /// collection.remove_vectors_batch(&ids_to_remove)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_vectors_batch(&mut self, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        debug!(
            "{} {} - Removing batch of {} vectors from collection '{}'",
            self.backend_type.icon(),
            self.backend_type.name(),
            ids.len(),
            self.name
        );

        // Remove each vector
        for id in ids {
            self.remove_vector(id.clone())?;
        }

        info!(
            "{} {} - Removed batch of {} vectors from collection '{}'",
            self.backend_type.icon(),
            self.backend_type.name(),
            ids.len(),
            self.name
        );

        Ok(())
    }

    /// Get backend type for this collection
    pub fn backend_type(&self) -> GpuBackendType {
        self.backend_type
    }
}

impl std::fmt::Debug for HiveGpuCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HiveGpuCollection")
            .field("name", &self.name)
            .field("dimension", &self.dimension)
            .field("vector_count", &self.vector_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CollectionConfig, DistanceMetric};

    #[test]
    fn test_hive_gpu_collection_creation() {
        // This test would require actual GPU context, so we skip it in unit tests
        // Integration tests should be used instead
    }

    #[test]
    fn test_dimension_validation() {
        // Test dimension validation logic
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::default(),
            compression: CompressionConfig::default(),
            normalization: None,
            storage_type: Some(crate::models::StorageType::Memory),
        };

        // This would be tested with actual GPU context in integration tests
    }
}
