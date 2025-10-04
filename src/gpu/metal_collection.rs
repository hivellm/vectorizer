//! Metal-accelerated Collection using GPU
//! 
//! This module provides a collection implementation that uses Metal GPU
//! for vector operations, similar to CudaCollection.

use crate::db::optimized_hnsw::OptimizedHnswIndex;
use crate::error::{Result, VectorizerError};
use crate::models::{
    CollectionConfig, CollectionMetadata, DistanceMetric, SearchResult, Vector,
};
use super::{GpuContext, GpuConfig, GpuOperations};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Metal-accelerated collection for Apple Silicon
pub struct MetalCollection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// GPU context for Metal operations
    gpu_ctx: Arc<GpuContext>,
    /// HNSW index (CPU-based graph structure)
    hnsw_index: Arc<RwLock<OptimizedHnswIndex>>,
    /// Vector storage
    vectors: Arc<DashMap<String, Vector>>,
    /// Embedding type
    embedding_type: Arc<RwLock<String>>,
    /// Creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    updated_at: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl MetalCollection {
    /// Create a new Metal-accelerated collection
    pub async fn new(name: String, config: CollectionConfig, gpu_config: GpuConfig) -> Result<Self> {
        info!("Creating Metal-accelerated collection '{}'", name);
        
        // Initialize GPU context
        let gpu_ctx = GpuContext::new(gpu_config).await?;
        let gpu_info = gpu_ctx.info();
        info!("GPU initialized: {} for collection '{}'", gpu_info.name, name);
        
        // Create HNSW index with optimized config
        let hnsw_config = crate::db::optimized_hnsw::OptimizedHnswConfig {
            max_connections: config.hnsw_config.m,
            max_connections_0: config.hnsw_config.m * 2,
            ef_construction: config.hnsw_config.ef_construction,
            seed: config.hnsw_config.seed,
            distance_metric: config.metric.clone(),
            parallel: true,
            initial_capacity: 100_000,
            batch_size: 1000,
        };
        
        let hnsw_index = OptimizedHnswIndex::new(config.dimension, hnsw_config)?;
        let now = chrono::Utc::now();
        
        Ok(Self {
            name,
            config,
            gpu_ctx: Arc::new(gpu_ctx),
            hnsw_index: Arc::new(RwLock::new(hnsw_index)),
            vectors: Arc::new(DashMap::new()),
            embedding_type: Arc::new(RwLock::new("bm25".to_string())),
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
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
    
    /// Add a vector to the collection with GPU-accelerated distance computation
    pub async fn add_vector(&self, vector: Vector) -> Result<()> {
        // Validate dimension
        if vector.data.len() != self.config.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.config.dimension,
                actual: vector.data.len(),
            });
        }
        
        let vector_id = vector.id.clone();
        
        // Store vector
        self.vectors.insert(vector_id.clone(), vector.clone());
        
        // Add to HNSW index
        // Note: HNSW graph construction is still CPU-based, but we could use GPU
        // for distance computations in the future
        let index = self.hnsw_index.read();
        index.add(vector_id, vector.data)?;
        
        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();
        
        Ok(())
    }
    
    /// Search for similar vectors using GPU-accelerated re-ranking
    pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        // Validate dimension
        if query.len() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: query.len(),
            });
        }
        
        if self.vectors.is_empty() {
            return Ok(vec![]);
        }
        
        // Step 1: HNSW approximate search to get candidates (CPU)
        // Get more candidates than needed for GPU re-ranking
        let candidate_count = std::cmp::min(k * 10, self.vectors.len());
        let index = self.hnsw_index.read();
        let hnsw_candidates = index.search(query, candidate_count)?;
        
        if hnsw_candidates.is_empty() {
            return Ok(vec![]);
        }
        
        debug!(
            "Metal collection '{}': HNSW returned {} candidates, re-ranking with GPU",
            self.name,
            hnsw_candidates.len()
        );
        
        // Step 2: Extract candidate vectors
        let candidate_vectors: Vec<Vec<f32>> = hnsw_candidates
            .iter()
            .filter_map(|(id, _score)| {
                self.vectors.get(id).map(|v| v.data.clone())
            })
            .collect();
        
        if candidate_vectors.is_empty() {
            return Ok(vec![]);
        }
        
        // Step 3: GPU-accelerated exact distance computation
        let exact_scores = match self.config.metric {
            DistanceMetric::Cosine => {
                self.gpu_ctx.cosine_similarity(query, &candidate_vectors).await?
            }
            DistanceMetric::Euclidean => {
                // For Euclidean, we get distances, need to convert to similarity
                let distances = self.gpu_ctx.euclidean_distance(query, &candidate_vectors).await?;
                // Convert distance to similarity (inverse)
                distances.into_iter().map(|d| 1.0 / (1.0 + d)).collect()
            }
            DistanceMetric::DotProduct => {
                self.gpu_ctx.dot_product(query, &candidate_vectors).await?
            }
            _ => {
                // Fallback to HNSW scores for unsupported metrics
                return Ok(hnsw_candidates
                    .into_iter()
                    .take(k)
                    .filter_map(|(id, score)| {
                        self.vectors.get(&id).map(|vec_ref| {
                            let vec = vec_ref.value();
                            SearchResult {
                                id: id.clone(),
                                score,
                                vector: Some(vec.data.clone()),
                                payload: vec.payload.clone(),
                            }
                        })
                    })
                    .collect());
            }
        };
        
        // Step 4: Combine with IDs and sort by score
        let mut results: Vec<SearchResult> = hnsw_candidates
            .into_iter()
            .zip(exact_scores.into_iter())
            .filter_map(|((id, _orig_score), score)| {
                // Get vector from storage
                self.vectors.get(&id).map(|vec_ref| {
                    let vec = vec_ref.value();
                    SearchResult {
                        id: id.clone(),
                        score,
                        vector: Some(vec.data.clone()),
                        payload: vec.payload.clone(),
                    }
                })
            })
            .collect();
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top-k
        Ok(results.into_iter().take(k).collect())
    }
    
    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        CollectionMetadata {
            name: self.name.clone(),
            vector_count: self.vectors.len(),
            document_count: 0, // Metal collections don't track documents separately
            created_at: self.created_at,
            updated_at: *self.updated_at.read(),
            config: self.config.clone(),
        }
    }
    
    /// Remove a vector from the collection
    pub fn remove_vector(&self, id: &str) -> Result<()> {
        if self.vectors.remove(id).is_none() {
            return Err(VectorizerError::VectorNotFound(id.to_string()));
        }
        
        // Note: HNSW doesn't support efficient deletion, so we just remove from storage
        // The HNSW index will still have a reference, but search will filter it out
        
        *self.updated_at.write() = chrono::Utc::now();
        Ok(())
    }
    
    /// Get a vector by ID
    pub fn get_vector(&self, id: &str) -> Result<Vector> {
        self.vectors
            .get(id)
            .map(|v| v.clone())
            .ok_or_else(|| VectorizerError::VectorNotFound(id.to_string()))
    }
    
    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }
    
    /// Get estimated memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        // Vector storage
        let vector_memory = self.vectors.len() * self.config.dimension * std::mem::size_of::<f32>();
        
        // HNSW index memory
        let index = self.hnsw_index.read();
        let index_memory = index.memory_stats().total_memory_bytes;
        
        vector_memory + index_memory
    }
    
    /// Get all vectors in the collection
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        self.vectors
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Get embedding type
    pub fn get_embedding_type(&self) -> String {
        self.embedding_type.read().clone()
    }
    
    /// Set embedding type
    pub fn set_embedding_type(&self, embedding_type: String) {
        *self.embedding_type.write() = embedding_type;
    }
    
    /// Get GPU info
    pub fn gpu_info(&self) -> String {
        let info = self.gpu_ctx.info();
        format!("{} (Metal)", info.name)
    }
    
    /// Batch add vectors (more efficient)
    pub async fn batch_add(&self, vectors: Vec<Vector>) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }
        
        // Validate dimensions
        for vector in &vectors {
            if vector.data.len() != self.config.dimension {
                return Err(VectorizerError::DimensionMismatch {
                    expected: self.config.dimension,
                    actual: vector.data.len(),
                });
            }
        }
        
        // Store vectors
        for vector in &vectors {
            self.vectors.insert(vector.id.clone(), vector.clone());
        }
        
        // Batch add to HNSW
        let batch: Vec<(String, Vec<f32>)> = vectors
            .into_iter()
            .map(|v| (v.id, v.data))
            .collect();
        
        let index = self.hnsw_index.read();
        index.batch_add(batch)?;
        
        *self.updated_at.write() = chrono::Utc::now();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HnswConfig;
    
    #[tokio::test]
    async fn test_metal_collection_creation() {
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: crate::models::CompressionConfig::None,
        };
        
        let gpu_config = GpuConfig::for_metal_silicon();
        let result = MetalCollection::new("test".to_string(), config, gpu_config).await;
        
        // May fail if GPU not available, which is OK
        if result.is_ok() {
            let collection = result.unwrap();
            assert_eq!(collection.name(), "test");
            assert_eq!(collection.vector_count(), 0);
        }
    }
    
    #[tokio::test]
    async fn test_metal_collection_add_and_search() {
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: crate::models::CompressionConfig::None,
        };
        
        let gpu_config = GpuConfig::for_metal_silicon();
        if let Ok(collection) = MetalCollection::new("test".to_string(), config, gpu_config).await {
            // Add vector
            let vector = Vector {
                id: "v1".to_string(),
                data: vec![1.0; 128],
                payload: None,
            };
            
            collection.add_vector(vector).await.unwrap();
            assert_eq!(collection.vector_count(), 1);
            
            // Search
            let query = vec![1.0; 128];
            let results = collection.search(&query, 1).await.unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].id, "v1");
        }
    }
}

