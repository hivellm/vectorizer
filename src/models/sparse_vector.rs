//! Sparse vector data structures and operations
//!
//! Sparse vectors represent high-dimensional vectors where most dimensions are zero.
//! They are stored efficiently as (index, value) pairs for non-zero elements.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Sparse vector representation
/// Stores only non-zero values as (index, value) pairs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SparseVector {
    /// Non-zero indices and their values
    /// Key: dimension index (0-based), Value: vector value at that index
    pub indices: Vec<usize>,
    /// Values corresponding to indices
    pub values: Vec<f32>,
}

impl SparseVector {
    /// Create a new sparse vector from indices and values
    pub fn new(indices: Vec<usize>, values: Vec<f32>) -> Result<Self, SparseVectorError> {
        if indices.len() != values.len() {
            return Err(SparseVectorError::LengthMismatch {
                indices_len: indices.len(),
                values_len: values.len(),
            });
        }

        // Validate indices are sorted and unique
        if !indices.is_empty() {
            for i in 1..indices.len() {
                if indices[i] <= indices[i - 1] {
                    return Err(SparseVectorError::InvalidIndices(
                        "Indices must be sorted and unique".to_string(),
                    ));
                }
            }
        }

        Ok(Self { indices, values })
    }

    /// Create sparse vector from dense vector (converts zeros to sparse format)
    pub fn from_dense(dense: &[f32]) -> Self {
        let mut indices = Vec::new();
        let mut values = Vec::new();

        for (idx, &value) in dense.iter().enumerate() {
            if value != 0.0 {
                indices.push(idx);
                values.push(value);
            }
        }

        Self { indices, values }
    }

    /// Convert sparse vector to dense vector
    pub fn to_dense(&self, dimension: usize) -> Vec<f32> {
        let mut dense = vec![0.0; dimension];
        for (&idx, &value) in self.indices.iter().zip(self.values.iter()) {
            if idx < dimension {
                dense[idx] = value;
            }
        }
        dense
    }

    /// Get the number of non-zero elements
    pub fn nnz(&self) -> usize {
        self.indices.len()
    }

    /// Get the sparsity ratio (1.0 = fully sparse, 0.0 = fully dense)
    pub fn sparsity(&self, dimension: usize) -> f32 {
        if dimension == 0 {
            return 0.0;
        }
        1.0 - (self.nnz() as f32 / dimension as f32)
    }

    /// Calculate dot product with another sparse vector
    pub fn dot_product(&self, other: &SparseVector) -> f32 {
        let mut result = 0.0;
        let mut i = 0;
        let mut j = 0;

        while i < self.indices.len() && j < other.indices.len() {
            match self.indices[i].cmp(&other.indices[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    result += self.values[i] * other.values[j];
                    i += 1;
                    j += 1;
                }
            }
        }

        result
    }

    /// Calculate cosine similarity with another sparse vector
    pub fn cosine_similarity(&self, other: &SparseVector) -> f32 {
        let dot = self.dot_product(other);
        let norm_self = self.norm();
        let norm_other = other.norm();

        if norm_self == 0.0 || norm_other == 0.0 {
            0.0
        } else {
            dot / (norm_self * norm_other)
        }
    }

    /// Calculate L2 norm
    pub fn norm(&self) -> f32 {
        self.values.iter().map(|v| v * v).sum::<f32>().sqrt()
    }

    /// Get memory usage in bytes
    pub fn memory_size(&self) -> usize {
        self.indices.len() * std::mem::size_of::<usize>()
            + self.values.len() * std::mem::size_of::<f32>()
    }

    /// Validate sparse vector structure
    pub fn validate(&self) -> Result<(), SparseVectorError> {
        if self.indices.len() != self.values.len() {
            return Err(SparseVectorError::LengthMismatch {
                indices_len: self.indices.len(),
                values_len: self.values.len(),
            });
        }

        // Check indices are sorted and unique
        for i in 1..self.indices.len() {
            if self.indices[i] <= self.indices[i - 1] {
                return Err(SparseVectorError::InvalidIndices(
                    "Indices must be sorted and unique".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Error types for sparse vector operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum SparseVectorError {
    #[error("Length mismatch: indices={indices_len}, values={values_len}")]
    LengthMismatch {
        indices_len: usize,
        values_len: usize,
    },
    #[error("Invalid indices: {0}")]
    InvalidIndices(String),
    #[error("Dimension out of bounds: index={index}, max={max}")]
    DimensionOutOfBounds { index: usize, max: usize },
}

/// Sparse vector index for efficient search
#[derive(Debug, Clone)]
pub struct SparseVectorIndex {
    /// Vector ID -> SparseVector
    vectors: HashMap<String, SparseVector>,
    /// Inverted index: dimension index -> set of vector IDs with non-zero at this dimension
    inverted_index: HashMap<usize, Vec<String>>,
}

impl SparseVectorIndex {
    /// Create a new sparse vector index
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            inverted_index: HashMap::new(),
        }
    }

    /// Add a sparse vector to the index
    pub fn add(&mut self, id: String, vector: SparseVector) -> Result<(), SparseVectorError> {
        vector.validate()?;

        // Remove old vector if exists
        if let Some(old_vector) = self.vectors.remove(&id) {
            self.remove_from_inverted_index(&id, &old_vector);
        }

        // Add to vectors
        self.vectors.insert(id.clone(), vector.clone());

        // Add to inverted index
        for &idx in &vector.indices {
            self.inverted_index
                .entry(idx)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        Ok(())
    }

    /// Remove a vector from the index
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(vector) = self.vectors.remove(id) {
            self.remove_from_inverted_index(id, &vector);
            true
        } else {
            false
        }
    }

    /// Remove vector from inverted index
    fn remove_from_inverted_index(&mut self, id: &str, vector: &SparseVector) {
        for &idx in &vector.indices {
            if let Some(ids) = self.inverted_index.get_mut(&idx) {
                ids.retain(|x| x != id);
                if ids.is_empty() {
                    self.inverted_index.remove(&idx);
                }
            }
        }
    }

    /// Search for similar sparse vectors
    pub fn search(&self, query: &SparseVector, k: usize) -> Vec<(String, f32)> {
        let mut results: Vec<(String, f32)> = self
            .vectors
            .iter()
            .map(|(id, vector)| {
                let similarity = query.cosine_similarity(vector);
                (id.clone(), similarity)
            })
            .collect();

        // Sort by similarity descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        results
    }

    /// Get vector count
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }
}

impl Default for SparseVectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_vector_creation() {
        let sparse = SparseVector::new(vec![0, 5, 10], vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(sparse.nnz(), 3);
    }

    #[test]
    fn test_sparse_vector_from_dense() {
        let dense = vec![1.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 3.0];
        let sparse = SparseVector::from_dense(&dense);
        assert_eq!(sparse.nnz(), 3);
        assert_eq!(sparse.indices, vec![0, 5, 10]);
        assert_eq!(sparse.values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_sparse_vector_to_dense() {
        let sparse = SparseVector::new(vec![0, 5, 10], vec![1.0, 2.0, 3.0]).unwrap();
        let dense = sparse.to_dense(11);
        assert_eq!(dense[0], 1.0);
        assert_eq!(dense[5], 2.0);
        assert_eq!(dense[10], 3.0);
        assert_eq!(dense[1], 0.0);
    }

    #[test]
    fn test_sparse_vector_dot_product() {
        let v1 = SparseVector::new(vec![0, 2, 4], vec![1.0, 2.0, 3.0]).unwrap();
        let v2 = SparseVector::new(vec![0, 2, 5], vec![2.0, 3.0, 4.0]).unwrap();
        let dot = v1.dot_product(&v2);
        // Only indices 0 and 2 overlap: 1.0*2.0 + 2.0*3.0 = 2.0 + 6.0 = 8.0
        assert!((dot - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_sparse_vector_cosine_similarity() {
        let v1 = SparseVector::new(vec![0, 1], vec![1.0, 0.0]).unwrap();
        let v2 = SparseVector::new(vec![0, 1], vec![1.0, 0.0]).unwrap();
        let similarity = v1.cosine_similarity(&v2);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_sparse_vector_index() {
        let mut index = SparseVectorIndex::new();

        let v1 = SparseVector::new(vec![0, 2], vec![1.0, 2.0]).unwrap();
        index.add("v1".to_string(), v1).unwrap();

        let v2 = SparseVector::new(vec![1, 3], vec![1.0, 2.0]).unwrap();
        index.add("v2".to_string(), v2).unwrap();

        assert_eq!(index.len(), 2);

        let query = SparseVector::new(vec![0, 2], vec![1.0, 2.0]).unwrap();
        let results = index.search(&query, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "v1"); // Should be most similar
    }

    #[test]
    fn test_sparse_vector_validation() {
        // Valid sparse vector
        let valid = SparseVector::new(vec![0, 2, 5], vec![1.0, 2.0, 3.0]);
        assert!(valid.is_ok());

        // Invalid: unsorted indices
        let invalid = SparseVector::new(vec![5, 2, 0], vec![1.0, 2.0, 3.0]);
        assert!(invalid.is_err());

        // Invalid: duplicate indices
        let invalid = SparseVector::new(vec![0, 2, 2], vec![1.0, 2.0, 3.0]);
        assert!(invalid.is_err());

        // Invalid: length mismatch
        let invalid = SparseVector::new(vec![0, 2], vec![1.0, 2.0, 3.0]);
        assert!(invalid.is_err());
    }
}
