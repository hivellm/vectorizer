//! HNSW index implementation using hnsw_rs

use crate::{
    error::{Result, VectorizerError},
    models::{DistanceMetric, HnswConfig},
};
use hnsw_rs::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info};

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
        }
    }

    /// Add a vector to the index
    pub fn add(&mut self, id: &str, vector: &[f32]) -> Result<()> {
        debug!("Adding vector '{}' to HNSW index", id);

        if vector.len() != self.dimension {
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
        self.hnsw.insert((vector, internal_id));

        Ok(())
    }

    /// Update a vector in the index
    pub fn update(&mut self, id: &str, vector: &[f32]) -> Result<()> {
        debug!("Updating vector '{}' in HNSW index", id);

        // For now, we implement update as remove + add
        // This is not optimal but hnsw_rs doesn't support in-place updates
        self.remove(id)?;
        self.add(id, vector)?;

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

        // Search in HNSW
        let neighbors = self.hnsw.search(query, k, 30); // ef_search = 30

        // Convert results
        let mut results = Vec::with_capacity(neighbors.len());
        for neighbor in neighbors {
            if let Some(string_id) = self.reverse_id_map.get(&neighbor.d_id) {
                // Convert distance based on metric
                let score = match self.metric {
                    DistanceMetric::Euclidean => neighbor.distance,
                    DistanceMetric::Cosine => {
                        // Convert L2 distance to cosine similarity
                        // This is an approximation; for exact cosine we'd need normalized vectors
                        1.0 - (neighbor.distance / 2.0).min(1.0)
                    }
                    DistanceMetric::DotProduct => {
                        // For dot product, we'd need the actual vectors
                        // This is a placeholder
                        -neighbor.distance
                    }
                };
                results.push((string_id.clone(), score));
            }
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
}
