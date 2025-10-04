//! GPU-Accelerated HNSW Search
//!
//! This module provides GPU-accelerated operations for HNSW (Hierarchical Navigable Small World)
//! graph traversal and nearest neighbor search. It significantly improves performance by:
//! - Batch distance computation on GPU
//! - Parallel candidate evaluation
//! - Efficient memory transfers
//!
//! The implementation maintains compatibility with the CPU-based hnsw_rs library
//! while leveraging GPU acceleration for distance calculations.

use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info, warn};
use crate::error::{Result, VectorizerError};
use crate::gpu::{GpuContext, GpuOperations};
use crate::models::DistanceMetric;

/// GPU-accelerated HNSW search parameters
#[derive(Debug, Clone)]
pub struct GpuHnswConfig {
    /// Number of candidates to evaluate in parallel on GPU
    pub batch_size: usize,
    /// Minimum candidates before triggering GPU batch processing
    pub gpu_batch_threshold: usize,
    /// Use GPU for distance calculations
    pub use_gpu_distances: bool,
}

impl Default for GpuHnswConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            gpu_batch_threshold: 50,
            use_gpu_distances: true,
        }
    }
}

/// GPU-accelerated HNSW search result
#[derive(Debug, Clone)]
pub struct GpuSearchCandidate {
    /// Candidate vector ID
    pub id: usize,
    /// Distance to query
    pub distance: f32,
}

/// GPU-accelerated HNSW search engine
pub struct GpuHnswSearch {
    /// GPU context for operations
    gpu_context: Arc<GpuContext>,
    /// Configuration
    config: GpuHnswConfig,
    /// Distance metric
    metric: DistanceMetric,
    /// Statistics
    stats: Arc<RwLock<GpuHnswStats>>,
}

/// Statistics for GPU HNSW operations
#[derive(Debug, Default, Clone)]
pub struct GpuHnswStats {
    /// Total searches performed
    pub total_searches: u64,
    /// GPU batch operations
    pub gpu_batches: u64,
    /// CPU fallback operations
    pub cpu_fallbacks: u64,
    /// Average batch size
    pub avg_batch_size: f64,
}

impl GpuHnswSearch {
    /// Create a new GPU-accelerated HNSW search engine
    pub fn new(
        gpu_context: Arc<GpuContext>,
        config: GpuHnswConfig,
        metric: DistanceMetric,
    ) -> Self {
        info!("Initializing GPU-accelerated HNSW search");
        info!("  Batch size: {}", config.batch_size);
        info!("  GPU threshold: {}", config.gpu_batch_threshold);
        info!("  Metric: {:?}", metric);
        
        Self {
            gpu_context,
            config,
            metric,
            stats: Arc::new(RwLock::new(GpuHnswStats::default())),
        }
    }
    
    /// Compute distances for a batch of candidates using GPU
    ///
    /// # Arguments
    /// * `query` - Query vector
    /// * `candidates` - Candidate vectors to evaluate
    ///
    /// # Returns
    /// Vector of distances in the same order as candidates
    pub async fn batch_distance_gpu(
        &self,
        query: &[f32],
        candidates: &[Vec<f32>],
    ) -> Result<Vec<f32>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        
        debug!(
            "GPU batch distance: {} candidates, metric: {:?}",
            candidates.len(),
            self.metric
        );
        
        // Use GPU for distance calculation based on metric
        let distances = match self.metric {
            DistanceMetric::Cosine => {
                self.gpu_context.cosine_similarity(query, candidates).await?
            }
            DistanceMetric::Euclidean => {
                self.gpu_context.euclidean_distance(query, candidates).await?
            }
            DistanceMetric::DotProduct => {
                self.gpu_context.dot_product(query, candidates).await?
            }
        };
        
        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.gpu_batches += 1;
            stats.avg_batch_size = 
                (stats.avg_batch_size * (stats.gpu_batches - 1) as f64 
                 + candidates.len() as f64) / stats.gpu_batches as f64;
        }
        
        Ok(distances)
    }
    
    /// Evaluate candidates in batches using GPU acceleration
    ///
    /// # Arguments
    /// * `query` - Query vector
    /// * `candidates` - List of candidate vectors
    ///
    /// # Returns
    /// Sorted list of candidates with distances
    pub async fn evaluate_candidates_gpu(
        &self,
        query: &[f32],
        candidates: Vec<(usize, Vec<f32>)>,
    ) -> Result<Vec<GpuSearchCandidate>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        
        debug!("Evaluating {} candidates with GPU", candidates.len());
        
        // Check if we should use GPU
        let use_gpu = self.config.use_gpu_distances 
            && candidates.len() >= self.config.gpu_batch_threshold
            && self.gpu_context.should_use_gpu(candidates.len());
        
        if !use_gpu {
            debug!("Using CPU fallback for {} candidates", candidates.len());
            let mut stats = self.stats.write();
            stats.cpu_fallbacks += 1;
            return self.evaluate_candidates_cpu(query, candidates);
        }
        
        // Extract IDs and vectors
        let ids: Vec<usize> = candidates.iter().map(|(id, _)| *id).collect();
        let vectors: Vec<Vec<f32>> = candidates.into_iter().map(|(_, vec)| vec).collect();
        
        // Process in batches for memory efficiency
        let mut all_results = Vec::new();
        
        for (batch_ids, batch_vectors) in ids.chunks(self.config.batch_size)
            .zip(vectors.chunks(self.config.batch_size))
        {
            let distances = self.batch_distance_gpu(query, batch_vectors).await?;
            
            for (id, distance) in batch_ids.iter().zip(distances.iter()) {
                all_results.push(GpuSearchCandidate {
                    id: *id,
                    distance: *distance,
                });
            }
        }
        
        // Sort by distance (ascending for most metrics, descending for cosine similarity)
        match self.metric {
            DistanceMetric::Cosine | DistanceMetric::DotProduct => {
                // Higher is better (similarity)
                all_results.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
            }
            _ => {
                // Lower is better (distance)
                all_results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
            }
        }
        
        Ok(all_results)
    }
    
    /// CPU fallback for candidate evaluation
    fn evaluate_candidates_cpu(
        &self,
        query: &[f32],
        candidates: Vec<(usize, Vec<f32>)>,
    ) -> Result<Vec<GpuSearchCandidate>> {
        let mut results: Vec<GpuSearchCandidate> = candidates
            .into_iter()
            .map(|(id, vec)| {
                let distance = match self.metric {
                    DistanceMetric::Cosine => Self::cosine_similarity_cpu(query, &vec),
                    DistanceMetric::Euclidean => Self::euclidean_distance_cpu(query, &vec),
                    DistanceMetric::DotProduct => Self::dot_product_cpu(query, &vec),
                };
                GpuSearchCandidate { id, distance }
            })
            .collect();
        
        // Sort by distance
        match self.metric {
            DistanceMetric::Cosine | DistanceMetric::DotProduct => {
                results.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
            }
            _ => {
                results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
            }
        }
        
        Ok(results)
    }
    
    // CPU distance functions (fallback)
    
    fn cosine_similarity_cpu(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm_a * norm_b + 1e-8)
    }
    
    fn euclidean_distance_cpu(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
    
    fn dot_product_cpu(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> GpuHnswStats {
        self.stats.read().clone()
    }
    
    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write();
        *stats = GpuHnswStats::default();
    }
    
    /// Get GPU utilization percentage for HNSW operations
    pub fn get_gpu_utilization(&self) -> f64 {
        let stats = self.stats.read();
        if stats.total_searches == 0 {
            return 0.0;
        }
        
        let gpu_ops = stats.gpu_batches as f64;
        let total_ops = (stats.gpu_batches + stats.cpu_fallbacks) as f64;
        
        if total_ops == 0.0 {
            0.0
        } else {
            (gpu_ops / total_ops) * 100.0
        }
    }
}

impl std::fmt::Debug for GpuHnswSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuHnswSearch")
            .field("config", &self.config)
            .field("metric", &self.metric)
            .field("stats", &self.stats.read().clone())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn generate_random_vector(dim: usize) -> Vec<f32> {
        (0..dim).map(|i| (i as f32 * 0.1) % 1.0).collect()
    }
    
    #[tokio::test]
    async fn test_gpu_hnsw_search_creation() {
        use crate::gpu::GpuConfig;
        
        let config = GpuConfig::default();
        let gpu_ctx = match GpuContext::new(config).await {
            Ok(ctx) => Arc::new(ctx),
            Err(_) => return, // Skip test if GPU not available
        };
        
        let hnsw_config = GpuHnswConfig::default();
        let search = GpuHnswSearch::new(
            gpu_ctx,
            hnsw_config,
            DistanceMetric::Cosine,
        );
        
        assert_eq!(search.get_stats().total_searches, 0);
    }
    
    #[tokio::test]
    async fn test_batch_distance_gpu() {
        use crate::gpu::GpuConfig;
        
        let config = GpuConfig::default();
        let gpu_ctx = match GpuContext::new(config).await {
            Ok(ctx) => Arc::new(ctx),
            Err(_) => return,
        };
        
        let hnsw_config = GpuHnswConfig::default();
        let search = GpuHnswSearch::new(
            gpu_ctx,
            hnsw_config,
            DistanceMetric::Cosine,
        );
        
        let query = generate_random_vector(128);
        let candidates: Vec<Vec<f32>> = (0..10)
            .map(|_| generate_random_vector(128))
            .collect();
        
        let distances = search.batch_distance_gpu(&query, &candidates).await;
        assert!(distances.is_ok());
        
        let distances = distances.unwrap();
        assert_eq!(distances.len(), candidates.len());
    }
    
    #[tokio::test]
    async fn test_evaluate_candidates() {
        use crate::gpu::GpuConfig;
        
        let config = GpuConfig::default();
        let gpu_ctx = match GpuContext::new(config).await {
            Ok(ctx) => Arc::new(ctx),
            Err(_) => return,
        };
        
        let hnsw_config = GpuHnswConfig::default();
        let search = GpuHnswSearch::new(
            gpu_ctx,
            hnsw_config,
            DistanceMetric::Cosine,
        );
        
        let query = generate_random_vector(128);
        let candidates: Vec<(usize, Vec<f32>)> = (0..100)
            .map(|i| (i, generate_random_vector(128)))
            .collect();
        
        let results = search.evaluate_candidates_gpu(&query, candidates).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert_eq!(results.len(), 100);
        
        // Check that results are sorted (descending for cosine similarity)
        for i in 0..results.len()-1 {
            assert!(results[i].distance >= results[i+1].distance);
        }
    }
}

