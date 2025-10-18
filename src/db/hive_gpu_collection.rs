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
}

impl HiveGpuCollection {
    /// Create a new Hive-GPU collection
    pub fn new(
        name: String,
        config: CollectionConfig,
        context: Arc<Mutex<Box<dyn GpuContext + Send>>>,
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
            "Created Hive-GPU collection '{}' with dimension {}",
            name, dimension
        );

        Ok(Self {
            name,
            config,
            context,
            storage,
            dimension,
            vector_count: 0,
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

        debug!(
            "Removed vector '{}' from GPU collection '{}'",
            id, self.name
        );
        Ok(())
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
        // This is expensive for GPU collections, so we limit it
        let max_vectors = 10000; // Limit to prevent memory issues

        if self.vector_count > max_vectors {
            warn!(
                "Collection '{}' has {} vectors, limiting get_all_vectors to {}",
                self.name, self.vector_count, max_vectors
            );
        }

        let limit = self.vector_count.min(max_vectors);
        let mut vectors = Vec::with_capacity(limit);

        for i in 0..limit {
            if let Ok(vector) = self.get_vector(i) {
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

        debug!("Cleared all vectors from GPU collection '{}'", self.name);
        Ok(())
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
        };

        // This would be tested with actual GPU context in integration tests
    }
}
