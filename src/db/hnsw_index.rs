//! HNSW index implementation using hnsw_rs

use crate::{
    error::{Result, VectorizerError},
    models::{DistanceMetric, HnswConfig},
};
use tracing::{debug, error};

/// Statistics about the HNSW index
#[derive(Debug, Clone)]
pub struct HnswIndexStats {
    /// Number of vectors in the index
    pub vector_count: usize,
    /// Whether the index needs rebuilding
    pub needs_rebuild: bool,
    /// Distance metric used
    pub metric: DistanceMetric,
    /// Vector dimension
    pub dimension: usize,
}
use hnsw_rs::prelude::*;
use std::collections::HashMap;
use tracing::info;

/// Wrapper around the hnsw_rs HNSW implementation
pub struct HnswIndex {
    /// The underlying HNSW structure
    hnsw: Hnsw<'static, f32, DistL2>,
    /// Mapping from string IDs to internal HNSW IDs
    id_map: HashMap<String, usize>,
    /// Reverse mapping from internal IDs to string IDs
    reverse_id_map: HashMap<usize, String>,
    /// Next available internal ID
    next_id: usize,
    /// Distance metric
    metric: DistanceMetric,
    /// Vector dimension
    dimension: usize,
    /// Flag indicating if index needs rebuild (for efficient updates)
    needs_rebuild: bool,
}

impl std::fmt::Debug for HnswIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HnswIndex")
            .field("id_map", &self.id_map)
            .field("reverse_id_map", &self.reverse_id_map)
            .field("dimension", &self.dimension)
            .field("metric", &self.metric)
            .field("needs_rebuild", &self.needs_rebuild)
            .finish()
    }
}

impl HnswIndex {
    /// Create a new HNSW index
    pub fn new(config: HnswConfig, metric: DistanceMetric, dimension: usize) -> Self {
        info!("Creating new HNSW index with config: {:?}", config);

        // Create HNSW with L2 distance (we'll handle other metrics in search)
        let hnsw = Hnsw::<f32, DistL2>::new(
            config.m,
            16, // max_nb_connection
            config.ef_construction,
            dimension,
            DistL2,
        );

        Self {
            hnsw,
            id_map: HashMap::new(),
            reverse_id_map: HashMap::new(),
            next_id: 0,
            metric,
            dimension,
            needs_rebuild: false,
        }
    }

    /// Add a vector to the index
    pub fn add(&mut self, id: &str, vector: &[f32]) -> Result<()> {
        debug!("Adding vector '{}' with {} dimensions to HNSW index", id, vector.len());

        if vector.len() != self.dimension {
            error!("Dimension mismatch: expected {}, got {}", self.dimension, vector.len());
            return Err(VectorizerError::InvalidDimension {
                expected: self.dimension,
                got: vector.len(),
            });
        }

        // Check if ID already exists
        if self.id_map.contains_key(id) {
            return Err(VectorizerError::Other(format!(
                "Vector with ID '{}' already exists in index",
                id
            )));
        }

        // Assign internal ID
        let internal_id = self.next_id;
        self.next_id += 1;

        // Store mappings
        self.id_map.insert(id.to_string(), internal_id);
        self.reverse_id_map.insert(internal_id, id.to_string());

        // Add to HNSW
        debug!("Inserting vector into HNSW graph (internal_id: {})", internal_id);
        self.hnsw.insert((vector, internal_id));
        debug!("Successfully inserted vector '{}' into HNSW", id);
        Ok(())
    }

    /// Update a vector in the index
    pub fn update(&mut self, id: &str, vector: &[f32]) -> Result<()> {
        debug!("Updating vector '{}' in HNSW index", id);

        // Check if vector exists
        if !self.id_map.contains_key(id) {
            return Err(VectorizerError::VectorNotFound(id.to_string()));
        }

        // For now, we mark the index as needing rebuild and do remove + add
        // This is more efficient than rebuilding immediately for multiple updates
        self.remove(id)?;
        self.add(id, vector)?;
        self.needs_rebuild = true;

        Ok(())
    }

    /// Remove a vector from the index
    pub fn remove(&mut self, id: &str) -> Result<()> {
        debug!("Removing vector '{}' from HNSW index", id);

        let internal_id = self
            .id_map
            .remove(id)
            .ok_or_else(|| VectorizerError::VectorNotFound(id.to_string()))?;

        self.reverse_id_map.remove(&internal_id);

        // Note: hnsw_rs doesn't support removal, so we just remove from our mappings
        // In production, we'd need to periodically rebuild the index

        Ok(())
    }

    /// Search for k nearest neighbors
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        debug!("Searching for {} nearest neighbors", k);

        if query.len() != self.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.dimension,
                got: query.len(),
            });
        }

        // Adaptive strategy for small indices: try to guarantee min(k, N)
        let vector_count = self.id_map.len();
        let effective_k = std::cmp::min(k, vector_count);

        // Early exit cases
        if effective_k == 0 {
            return Ok(Vec::new());
        }

        let neighbors = if vector_count <= 8 {
            // Escalate ef_search and ask for all points to improve recall determinism
            let mut ef = std::cmp::max(64, effective_k * 4);
            let mut best = Vec::new();
            for _ in 0..5 {
                let got = self.hnsw.search(query, vector_count, ef);
                if got.len() >= effective_k {
                    best = got;
                    break;
                }
                if got.len() > best.len() {
                    best = got;
                }
                ef = std::cmp::min(ef * 2, 2048);
            }
            // Fallback to whatever we have
            if best.is_empty() { self.hnsw.search(query, vector_count, ef) } else { best }
        } else {
            let ef_search = std::cmp::max(k * 2, 64);
            self.hnsw.search(query, k, ef_search)
        };

        // Convert results based on metric
        let mut results = Vec::with_capacity(neighbors.len());
        for neighbor in neighbors {
            if let Some(string_id) = self.reverse_id_map.get(&neighbor.d_id) {
                // Convert L2 distance to appropriate similarity score
                let score = match self.metric {
                    DistanceMetric::Euclidean => {
                        // For Euclidean, distance is already meaningful
                        // Convert to similarity using exponential decay
                        (-neighbor.distance).exp()
                    }
                    DistanceMetric::Cosine => {
                        // For cosine similarity with normalized vectors:
                        // L2 distance between normalized vectors relates to angle
                        // cos(θ) = 1 - (d²/2) where d is L2 distance
                        // But we need to be careful with floating point precision
                        let d_squared = neighbor.distance * neighbor.distance;
                        (1.0 - d_squared / 2.0).clamp(-1.0, 1.0)
                    }
                    DistanceMetric::DotProduct => {
                        // For dot product, we can't directly compute from L2
                        // This is a limitation - we'd need to store original vectors
                        // For now, use a placeholder that indicates this needs improvement
                        -neighbor.distance
                    }
                };
                results.push((string_id.clone(), score));
            }
        }

        // Ensure we don't exceed requested k
        if results.len() > effective_k {
            return Ok(results.into_iter().take(effective_k).collect());
        }

        Ok(results)
    }

    /// Get the number of vectors in the index
    pub fn len(&self) -> usize {
        self.id_map.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.id_map.is_empty()
    }

    /// Check if the index needs to be rebuilt
    pub fn needs_rebuild(&self) -> bool {
        self.needs_rebuild
    }

    /// Get index statistics
    pub fn stats(&self) -> HnswIndexStats {
        HnswIndexStats {
            vector_count: self.len(),
            needs_rebuild: self.needs_rebuild,
            metric: self.metric,
            dimension: self.dimension,
        }
    }

    /// Force rebuild of the index (useful after many updates)
    /// This is a placeholder - in a real implementation, this would rebuild the HNSW index
    /// from scratch with current vectors for optimal performance
    pub fn rebuild(&mut self) -> Result<()> {
        debug!("Rebuilding HNSW index");

        // In a real implementation, we would:
        // 1. Collect all current vectors
        // 2. Clear the HNSW index
        // 3. Rebuild it from scratch with optimized parameters

        // For now, just reset the rebuild flag
        self.needs_rebuild = false;

        info!("HNSW index rebuild completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_index() -> HnswIndex {
        let config = HnswConfig::default();
        HnswIndex::new(config, DistanceMetric::Euclidean, 3)
    }

    #[test]
    fn test_add_and_search() {
        let mut index = create_test_index();

        // Add vectors
        index.add("v1", &[1.0, 0.0, 0.0]).unwrap();
        index.add("v2", &[0.0, 1.0, 0.0]).unwrap();
        index.add("v3", &[0.0, 0.0, 1.0]).unwrap();

        // Search
        let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "v1"); // Should be closest
    }

    #[test]
    fn test_remove() {
        let mut index = create_test_index();

        // Add and remove
        index.add("v1", &[1.0, 0.0, 0.0]).unwrap();
        assert_eq!(index.len(), 1);

        index.remove("v1").unwrap();
        assert_eq!(index.len(), 0);

        // Try to remove non-existent
        let result = index.remove("v1");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }

    #[test]
    fn test_dimension_validation() {
        let mut index = create_test_index();

        // Wrong dimension
        let result = index.add("v1", &[1.0, 0.0]); // 2D instead of 3D
        assert!(matches!(
            result,
            Err(VectorizerError::InvalidDimension {
                expected: 3,
                got: 2
            })
        ));
    }

    #[test]
    fn test_distance_metrics() {
        // Test different distance metrics
        let metrics = vec![
            DistanceMetric::Euclidean,
            DistanceMetric::Cosine,
            DistanceMetric::DotProduct,
        ];

        for metric in metrics {
            let mut index = HnswIndex::new(HnswConfig::default(), metric, 3);

            // Add test vectors
            index.add("v1", &[1.0, 0.0, 0.0]).unwrap();
            index.add("v2", &[0.0, 1.0, 0.0]).unwrap();
            index.add("v3", &[0.0, 0.0, 1.0]).unwrap();
            index.add("v4", &[0.5, 0.5, 0.5]).unwrap();

            // Search should return results
            let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].0, "v1"); // Should be closest
        }
    }

    #[test]
    fn test_index_operations_comprehensive() {
        let mut index = create_test_index();

        // Test empty index
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());

        // Add vectors
        index.add("v1", &[1.0, 0.0, 0.0]).unwrap();
        index.add("v2", &[0.0, 1.0, 0.0]).unwrap();
        index.add("v3", &[0.0, 0.0, 1.0]).unwrap();

        assert_eq!(index.len(), 3);
        assert!(!index.is_empty());

        // Test search accuracy
        let results = index.search(&[1.0, 0.0, 0.0], 3).unwrap();
        // With improved ef_search for small indices, we should get all 3 results
        assert_eq!(results.len(), 3, "Should return all 3 vectors for small index");
        assert_eq!(results[0].0, "v1"); // Should be exact match (closest)

        // Test update operation
        index.update("v1", &[2.0, 0.0, 0.0]).unwrap();
        assert_eq!(index.len(), 3); // Length should remain the same (remove + add = same length)

        // Test remove operation
        index.remove("v2").unwrap();
        assert_eq!(index.len(), 2);

        // Verify removed vector is gone
        let result = index.remove("v2");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }

    #[test]
    fn test_search_edge_cases() {
        let mut index = create_test_index();

        // Add single vector
        index.add("single", &[1.0, 2.0, 3.0]).unwrap();

        // Search with k > number of vectors
        let results = index.search(&[1.0, 2.0, 3.0], 10).unwrap();
        assert_eq!(results.len(), 1);

        // Search with k = 0 (edge case)
        let results = index.search(&[1.0, 2.0, 3.0], 0).unwrap();
        assert_eq!(results.len(), 0);

        // Search with exact match
        let results = index.search(&[1.0, 2.0, 3.0], 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "single");
    }

    #[test]
    fn test_duplicate_vector_ids() {
        let mut index = create_test_index();

        // Add vector
        index.add("duplicate", &[1.0, 2.0, 3.0]).unwrap();

        // Try to add same ID again
        let result = index.add("duplicate", &[4.0, 5.0, 6.0]);
        assert!(matches!(result, Err(VectorizerError::Other(_))));
    }

    #[test]
    fn test_large_scale_index() {
        let mut index = HnswIndex::new(HnswConfig::default(), DistanceMetric::Euclidean, 10);

        // Add many vectors - create a specific pattern where vec_0 should be closest to query
        for i in 0..100 {
            let vector: Vec<f32> = if i == 0 {
                // vec_0 is exactly the query vector
                vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
            } else {
                // Other vectors are different
                (0..10).map(|j| if j == (i % 9) + 1 { 1.0 } else { 0.0 }).collect()
            };
            index.add(&format!("vec_{}", i), &vector).unwrap();
        }

        assert_eq!(index.len(), 100);

        // Search should work - query is exactly vec_0
        let query: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let results = index.search(&query, 5).unwrap();
        assert_eq!(results.len(), 5);
        // vec_0 should be the closest (exact match)
        assert!(results.iter().any(|(id, _)| id == "vec_0"));
    }
}
