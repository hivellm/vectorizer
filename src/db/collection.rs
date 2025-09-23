//! Collection implementation for storing vectors

use crate::{
    error::{Result, VectorizerError},
    models::{vector_utils, CollectionConfig, CollectionMetadata, DistanceMetric, SearchResult, Vector},
};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;

use super::hnsw_index::HnswIndex;

/// A collection of vectors with an associated HNSW index
#[derive(Clone)]
pub struct Collection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// Vector storage
    vectors: Arc<DashMap<String, Vector>>,
    /// HNSW index for similarity search
    index: Arc<RwLock<HnswIndex>>,
    /// Creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    updated_at: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl Collection {
    /// Create a new collection
    pub fn new(name: String, config: CollectionConfig) -> Self {
        let index = HnswIndex::new(config.hnsw_config.clone(), config.metric, config.dimension);
        let now = chrono::Utc::now();

        Self {
            name,
            config,
            vectors: Arc::new(DashMap::new()),
            index: Arc::new(RwLock::new(index)),
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
        }
    }

    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        CollectionMetadata {
            name: self.name.clone(),
            created_at: self.created_at,
            updated_at: *self.updated_at.read(),
            vector_count: self.vectors.len(),
            config: self.config.clone(),
        }
    }

    /// Insert a batch of vectors
    pub fn insert_batch(&self, vectors: Vec<Vector>) -> Result<()> {
        // Validate dimensions
        for vector in &vectors {
            if vector.dimension() != self.config.dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: self.config.dimension,
                    got: vector.dimension(),
                });
            }
        }

        // Insert vectors and update index
        let mut index = self.index.write();
        for mut vector in vectors {
            let id = vector.id.clone();
            let mut data = vector.data.clone();

            // Normalize vector for cosine similarity
            if matches!(self.config.metric, DistanceMetric::Cosine) {
                data = vector_utils::normalize_vector(&data);
                vector.data = data.clone(); // Update stored vector to normalized version
            }

            // Store vector
            self.vectors.insert(id.clone(), vector);

            // Add to index
            index.add(&id, &data)?;
        }

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Insert a single vector
    pub fn insert(&self, vector: Vector) -> Result<()> {
        self.insert_batch(vec![vector])
    }

    /// Update a vector
    pub fn update(&self, mut vector: Vector) -> Result<()> {
        // Validate dimension
        if vector.dimension() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: vector.dimension(),
            });
        }

        let id = vector.id.clone();
        let mut data = vector.data.clone();

        // Check if vector exists
        if !self.vectors.contains_key(&id) {
            return Err(VectorizerError::VectorNotFound(id));
        }

        // Normalize vector for cosine similarity
        if matches!(self.config.metric, DistanceMetric::Cosine) {
            data = vector_utils::normalize_vector(&data);
            vector.data = data.clone(); // Update stored vector to normalized version
        }

        // Update vector
        self.vectors.insert(id.clone(), vector);

        // Update index
        let mut index = self.index.write();
        index.update(&id, &data)?;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Delete a vector
    pub fn delete(&self, vector_id: &str) -> Result<()> {
        // Remove from storage
        self.vectors
            .remove(vector_id)
            .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))?;

        // Remove from index
        let mut index = self.index.write();
        index.remove(vector_id)?;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        self.vectors
            .get(vector_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))
    }

    /// Search for similar vectors
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
            vector_utils::normalize_vector(query_vector)
        } else {
            query_vector.to_vec()
        };

        // Search in index
        let index = self.index.read();
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

    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }

    /// Estimate memory usage in bytes
    pub fn estimated_memory_usage(&self) -> usize {
        let vector_size = std::mem::size_of::<f32>() * self.config.dimension;
        let entry_overhead = std::mem::size_of::<String>() + std::mem::size_of::<Vector>();
        let total_per_vector = vector_size + entry_overhead;

        self.vectors.len() * total_per_vector
    }

    /// Get all vectors in the collection (for persistence)
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        self.vectors
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DistanceMetric, HnswConfig};

    fn create_test_collection() -> Collection {
        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: None,
            compression: Default::default(),
        };
        Collection::new("test".to_string(), config)
    }

    #[test]
    fn test_insert_and_get_vector() {
        let collection = create_test_collection();

        let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
        collection.insert(vector.clone()).unwrap();

        let retrieved = collection.get_vector("v1").unwrap();
        assert_eq!(retrieved.id, "v1");
        assert_eq!(retrieved.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_dimension_validation() {
        let collection = create_test_collection();

        // Wrong dimension
        let vector = Vector::new("v1".to_string(), vec![1.0, 2.0]); // 2D instead of 3D
        let result = collection.insert(vector);

        assert!(matches!(
            result,
            Err(VectorizerError::InvalidDimension {
                expected: 3,
                got: 2
            })
        ));
    }

    #[test]
    fn test_update_vector() {
        let collection = create_test_collection();

        // Insert original
        let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
        collection.insert(vector).unwrap();

        // Update
        let updated = Vector::new("v1".to_string(), vec![4.0, 5.0, 6.0]);
        collection.update(updated).unwrap();

        // Verify
        let retrieved = collection.get_vector("v1").unwrap();
        assert_eq!(retrieved.data, vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_delete_vector() {
        let collection = create_test_collection();

        // Insert and delete
        let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
        collection.insert(vector).unwrap();
        assert_eq!(collection.vector_count(), 1);

        collection.delete("v1").unwrap();
        assert_eq!(collection.vector_count(), 0);

        // Try to get deleted vector
        let result = collection.get_vector("v1");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }
}
