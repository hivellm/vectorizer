//! Bend Integration with HNSW Index
//! 
//! This module integrates Bend with the HNSW index to accelerate vector similarity
//! search operations through automatic parallelization.

use std::sync::Arc;
use std::collections::HashMap;
use crate::error::{Result, VectorizerError};
use crate::models::DistanceMetric;
use crate::bend::{BendVectorOperations, BendConfig};
use crate::db::optimized_hnsw::{OptimizedHnswIndex, OptimizedHnswConfig};
use tracing::{debug, info, warn};

/// Bend-enhanced HNSW index
pub struct BendHnswIndex {
    /// The underlying HNSW index
    hnsw_index: OptimizedHnswIndex,
    /// Bend vector operations
    bend_operations: BendVectorOperations,
    /// Bend configuration
    bend_config: BendConfig,
    /// Statistics
    stats: Arc<parking_lot::RwLock<BendHnswStats>>,
}

/// Statistics for Bend HNSW operations
#[derive(Debug, Default)]
pub struct BendHnswStats {
    /// Total searches performed
    pub total_searches: u64,
    /// Searches that used Bend
    pub bend_searches: u64,
    /// Searches that fell back to HNSW
    pub fallback_searches: u64,
    /// Total time spent in Bend (ms)
    pub total_bend_time_ms: f64,
    /// Total time spent in HNSW (ms)
    pub total_hnsw_time_ms: f64,
    /// Average Bend speedup factor
    pub average_speedup: f64,
}

impl BendHnswIndex {
    /// Create a new Bend-enhanced HNSW index
    pub fn new(dimension: usize, config: OptimizedHnswConfig, bend_config: BendConfig) -> Result<Self> {
        let hnsw_index = OptimizedHnswIndex::new(dimension, config)?;
        let bend_operations = BendVectorOperations::new(bend_config.clone());
        
        Ok(Self {
            hnsw_index,
            bend_operations,
            bend_config,
            stats: Arc::new(parking_lot::RwLock::new(BendHnswStats::default())),
        })
    }

    /// Search with Bend acceleration
    pub async fn search_with_bend(
        &self,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(String, f32)>> {
        let start_time = std::time::Instant::now();
        
        // Update total searches
        {
            let mut stats = self.stats.write();
            stats.total_searches += 1;
        }

        // Check if Bend should be used
        let use_bend = self.should_use_bend(query.len(), k);
        
        if use_bend {
            match self.search_with_bend_parallel(query, k).await {
                Ok(results) => {
                    let bend_time = start_time.elapsed().as_secs_f64() * 1000.0;
                    
                    // Update stats
                    {
                        let mut stats = self.stats.write();
                        stats.bend_searches += 1;
                        stats.total_bend_time_ms += bend_time;
                        
                        // Calculate speedup (simplified)
                        if stats.total_hnsw_time_ms > 0.0 {
                            stats.average_speedup = stats.total_bend_time_ms / stats.total_hnsw_time_ms;
                        }
                    }
                    
                    debug!("Bend search completed in {:.2}ms, found {} results", bend_time, results.len());
                    Ok(results)
                }
                Err(e) => {
                    warn!("Bend search failed, falling back to HNSW: {}", e);
                    self.fallback_to_hnsw(query, k, start_time).await
                }
            }
        } else {
            self.fallback_to_hnsw(query, k, start_time).await
        }
    }

    /// Perform search using Bend parallelization
    async fn search_with_bend_parallel(
        &self,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(String, f32)>> {
        // Get all vectors from the index
        let vectors = self.hnsw_index.get_all_vectors()?;
        
        if vectors.is_empty() {
            return Ok(vec![]);
        }

        // Extract vector data and IDs
        let vector_data: Vec<Vec<f32>> = vectors.values().cloned().collect();
        let vector_ids: Vec<String> = vectors.keys().cloned().collect();

        // Use Bend for parallel similarity search
        let similarities = self.bend_operations
            .parallel_similarity_search(query, &vector_data, 0.0)
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

        Ok(results)
    }

    /// Fallback to HNSW search
    async fn fallback_to_hnsw(
        &self,
        query: &[f32],
        k: usize,
        start_time: std::time::Instant,
    ) -> Result<Vec<(String, f32)>> {
        let results = self.hnsw_index.search(query, k)?;
        let hnsw_time = start_time.elapsed().as_secs_f64() * 1000.0;
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.fallback_searches += 1;
            stats.total_hnsw_time_ms += hnsw_time;
        }
        
        debug!("HNSW search completed in {:.2}ms, found {} results", hnsw_time, results.len());
        Ok(results)
    }

    /// Determine if Bend should be used for this search
    fn should_use_bend(&self, query_dimension: usize, k: usize) -> bool {
        // Don't use Bend if disabled
        if !self.bend_config.enabled {
            return false;
        }

        // Check if Bend is available
        if self.bend_operations.executor.check_bend_availability().is_err() {
            return false;
        }

        // Use Bend for larger searches or when parallelization is beneficial
        let vector_count = self.hnsw_index.len();
        
        // Use Bend if:
        // 1. We have enough vectors to benefit from parallelization
        // 2. The search is complex enough (large k or many vectors)
        // 3. The query dimension is reasonable for Bend
        vector_count >= 100 && 
        (k >= 10 || vector_count >= 1000) &&
        query_dimension <= 1000
    }

    /// Add a vector to the index
    pub fn add(&self, id: String, data: Vec<f32>) -> Result<()> {
        self.hnsw_index.add(id, data)
    }

    /// Batch add vectors with Bend acceleration
    pub async fn batch_add_with_bend(
        &self,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        // For batch operations, we can use Bend to pre-compute similarities
        // and optimize the insertion order
        if self.bend_config.enabled && vectors.len() > 10 {
            self.optimize_batch_insertion_with_bend(vectors).await
        } else {
            // Fallback to regular batch insertion
            for (id, data) in vectors {
                self.hnsw_index.add(id, data)?;
            }
            Ok(())
        }
    }

    /// Optimize batch insertion using Bend
    async fn optimize_batch_insertion_with_bend(
        &self,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        // Extract vector data for Bend processing
        let vector_data: Vec<Vec<f32>> = vectors.iter().map(|(_, data)| data.clone()).collect();
        
        // Use Bend to compute pairwise similarities for optimization
        // This helps determine the best insertion order
        if vector_data.len() > 1 {
            // For now, just insert in order
            // In a more sophisticated implementation, we could:
            // 1. Compute pairwise similarities with Bend
            // 2. Order vectors by similarity clusters
            // 3. Insert in optimized order
        }

        // Insert vectors
        for (id, data) in vectors {
            self.hnsw_index.add(id, data)?;
        }

        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> BendHnswStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = BendHnswStats::default();
    }

    /// Get the underlying HNSW index
    pub fn hnsw_index(&self) -> &OptimizedHnswIndex {
        &self.hnsw_index
    }

    /// Get Bend configuration
    pub fn bend_config(&self) -> &BendConfig {
        &self.bend_config
    }

    /// Update Bend configuration
    pub fn update_bend_config(&mut self, config: BendConfig) {
        self.bend_config = config.clone();
        self.bend_operations = BendVectorOperations::new(config);
    }
}

impl Clone for BendHnswStats {
    fn clone(&self) -> Self {
        Self {
            total_searches: self.total_searches,
            bend_searches: self.bend_searches,
            fallback_searches: self.fallback_searches,
            total_bend_time_ms: self.total_bend_time_ms,
            total_hnsw_time_ms: self.total_hnsw_time_ms,
            average_speedup: self.average_speedup,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bend_hnsw_creation() {
        let config = OptimizedHnswConfig::default();
        let bend_config = BendConfig::default();
        
        let index = BendHnswIndex::new(384, config, bend_config).unwrap();
        assert_eq!(index.hnsw_index().len(), 0);
    }

    #[tokio::test]
    async fn test_bend_hnsw_search() {
        let config = OptimizedHnswConfig::default();
        let bend_config = BendConfig::default();
        
        let index = BendHnswIndex::new(3, config, bend_config).unwrap();
        
        // Add test vectors
        index.add("vec1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
        index.add("vec2".to_string(), vec![0.0, 1.0, 0.0]).unwrap();
        index.add("vec3".to_string(), vec![0.0, 0.0, 1.0]).unwrap();
        
        // Search
        let results = index.search_with_bend(&[1.0, 0.0, 0.0], 2).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // First result should be the identical vector
        assert_eq!(results[0].0, "vec1");
        assert!(results[0].1 > 0.9); // High similarity
    }

    #[tokio::test]
    async fn test_bend_hnsw_stats() {
        let config = OptimizedHnswConfig::default();
        let bend_config = BendConfig::default();
        
        let index = BendHnswIndex::new(3, config, bend_config).unwrap();
        
        // Add test vectors
        index.add("vec1".to_string(), vec![1.0, 0.0, 0.0]).unwrap();
        
        // Perform search
        let _results = index.search_with_bend(&[1.0, 0.0, 0.0], 1).await.unwrap();
        
        // Check stats
        let stats = index.get_stats();
        assert_eq!(stats.total_searches, 1);
    }
}
