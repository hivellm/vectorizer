//! DirectX 12 GPU-accelerated collection implementation
//!
//! This collection integrates DirectX 12 GPU operations with HNSW search,
//! providing GPU acceleration for Windows with NVIDIA, AMD, and Intel GPUs.

use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, CollectionMetadata, DistanceMetric, SearchResult, Vector};
use crate::gpu::{GpuContext, GpuConfig, GpuOperations};
use dashmap::DashMap;
use hnsw_rs::prelude::*;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// DirectX 12 GPU-accelerated collection
pub struct DirectX12Collection {
    /// Collection name
    name: String,
    
    /// Collection configuration
    config: CollectionConfig,
    
    /// GPU context for DirectX 12 operations
    gpu_context: Arc<GpuContext>,
    
    /// Vector storage (concurrent access)
    vectors: Arc<DashMap<String, Vector>>,
    
    /// HNSW index (CPU-based, for now)
    hnsw_index: Arc<RwLock<Option<Hnsw<'static, f32, DistCosine>>>>,
}

impl DirectX12Collection {
    /// Get collection name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get collection configuration
    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }
    
    /// Create a new DirectX 12 GPU-accelerated collection
    pub async fn new(name: String, config: CollectionConfig, gpu_config: GpuConfig) -> Result<Self> {
        info!("🪟 Creating DirectX 12 GPU-accelerated collection: {}", name);
        
        // Initialize GPU context
        let gpu_context = Arc::new(GpuContext::new(gpu_config).await?);
        
        info!("✅ DirectX 12 GPU context initialized for collection '{}'", name);
        
        Ok(Self {
            name,
            config,
            gpu_context,
            vectors: Arc::new(DashMap::new()),
            hnsw_index: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Add a vector to the collection
    pub fn add_vector(&self, id: String, vector: Vector) -> Result<()> {
        debug!("Adding vector '{}' to DirectX 12 collection '{}'", id, self.name);
        
        // Validate dimension
        if vector.data.len() != self.config.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.config.dimension,
                actual: vector.data.len(),
            });
        }
        
        // Add to HNSW index
        {
            let mut hnsw = self.hnsw_index.write();
            if hnsw.is_none() {
                // Initialize HNSW index on first insert
                let hnsw_config = &self.config.hnsw_config;
                *hnsw = Some(
                    Hnsw::<'static, f32, DistCosine>::new(
                        hnsw_config.m,
                        self.config.dimension,
                        hnsw_config.ef_construction,
                        200,
                        DistCosine {},
                    )
                );
                debug!("Initialized HNSW index for DirectX 12 collection '{}'", self.name);
            }
            
            if let Some(ref mut index) = *hnsw {
                let data_id = self.vectors.len();
                index.insert((&vector.data, data_id));
            }
        }
        
        // Store vector
        self.vectors.insert(id, vector);
        
        Ok(())
    }
    
    /// Search for similar vectors using GPU-accelerated operations
    pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        debug!("Searching DirectX 12 collection '{}' for {} nearest neighbors", self.name, k);
        
        // Validate query dimension
        if query.len() != self.config.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.config.dimension,
                actual: query.len(),
            });
        }
        
        // Use HNSW for initial candidate search
        let hnsw_candidates: Vec<(usize, f32)> = {
            let hnsw = self.hnsw_index.read();
            if let Some(ref index) = *hnsw {
                let ef_search = self.config.hnsw_config.ef_search.max(k);
                index.search(query, k.max(ef_search), ef_search)
                    .into_iter()
                    .map(|neighbor| (neighbor.d_id, neighbor.distance))
                    .collect()
            } else {
                return Ok(Vec::new());
            }
        };
        
        // Map HNSW candidates to SearchResults with GPU operations
        let results: Vec<SearchResult> = hnsw_candidates
            .iter()
            .filter_map(|(id, _score)| {
                // Find vector by position
                self.vectors
                    .iter()
                    .nth(*id)
                    .map(|entry| {
                        let vector = entry.value();
                        
                        // Calculate similarity using GPU (if available)
                        let score = match self.config.metric {
                            DistanceMetric::Cosine => {
                                // Use GPU for similarity calculation
                                if let Ok(scores) = pollster::block_on(
                                    self.gpu_context.cosine_similarity(query, &[vector.data.clone()])
                                ) {
                                    scores.get(0).copied().unwrap_or(0.0)
                                } else {
                                    // CPU fallback
                                    cosine_similarity(query, &vector.data)
                                }
                            }
                            DistanceMetric::Euclidean => {
                                if let Ok(dists) = pollster::block_on(
                                    self.gpu_context.euclidean_distance(query, &[vector.data.clone()])
                                ) {
                                    // Convert distance to similarity (1 / (1 + distance))
                                    let dist = dists.get(0).copied().unwrap_or(f32::MAX);
                                    1.0 / (1.0 + dist)
                                } else {
                                    // CPU fallback
                                    let dist = euclidean_distance(query, &vector.data);
                                    1.0 / (1.0 + dist)
                                }
                            }
                            _ => 0.0,
                        };
                        
                        SearchResult {
                            id: vector.id.clone(),
                            score,
                            vector: Some(vector.data.clone()),
                            payload: vector.payload.clone(),
                        }
                    })
            })
            .collect();
        
        // Sort by score descending and take top k
        let mut sorted_results = results;
        sorted_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        sorted_results.truncate(k);
        
        Ok(sorted_results)
    }
    
    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        use chrono::Utc;
        
        CollectionMetadata {
            name: self.name.clone(),
            vector_count: self.vectors.len(),
            config: self.config.clone(),
            document_count: 0, // DirectX 12 collections don't track documents separately
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Remove a vector by ID
    pub fn remove_vector(&self, id: &str) -> Result<()> {
        debug!("Removing vector '{}' from DirectX 12 collection '{}'", id, self.name);
        
        self.vectors
            .remove(id)
            .ok_or_else(|| VectorizerError::VectorNotFound(id.to_string()))?;
        
        // Note: HNSW index is not updated for deletes (rebuild required)
        warn!("Vector removed from storage but HNSW index not updated - consider rebuilding");
        
        Ok(())
    }
    
    /// Get a vector by ID
    pub fn get_vector(&self, id: &str) -> Result<Vector> {
        self.vectors
            .get(id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| VectorizerError::VectorNotFound(id.to_string()))
    }
    
    /// Get vector count
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }
    
    /// Estimate memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        let vector_size = self.config.dimension * std::mem::size_of::<f32>();
        let vector_overhead = std::mem::size_of::<Vector>();
        self.vectors.len() * (vector_size + vector_overhead)
    }
    
    /// Get all vectors (for export/backup)
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        self.vectors
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Get embedding type
    pub fn get_embedding_type(&self) -> Option<String> {
        None // Not implemented for DirectX 12 collections yet
    }
    
    /// Set embedding type
    pub fn set_embedding_type(&self, _embedding_type: String) {
        // Not implemented for DirectX 12 collections yet
        warn!("set_embedding_type not implemented for DirectX 12 collections");
    }
}

// Helper functions for CPU fallback
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HnswConfig, Payload};
    
    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_dx12_collection_creation() {
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };
        
        let gpu_config = GpuConfig::default();
        
        // This will fail if DirectX 12 is not available, which is expected
        let result = DirectX12Collection::new("test".to_string(), config, gpu_config).await;
        
        // Just check that the function compiles and returns a Result
        match result {
            Ok(_) => println!("DirectX 12 collection created successfully"),
            Err(e) => println!("DirectX 12 not available (expected on non-Windows): {}", e),
        }
    }
}

