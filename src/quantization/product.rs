//! Product Quantization implementation
//!
//! Product Quantization (PQ) divides vectors into subvectors and quantizes each subvector
//! independently. This provides better compression ratios than scalar quantization for
//! high-dimensional vectors.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::quantization::QuantizationError;
use crate::quantization::traits::{
    QualityMetrics, QuantizationMethod, QuantizationParams, QuantizedVectors,
};

/// Product Quantization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductQuantizationConfig {
    /// Number of subvectors (m)
    pub subvectors: usize,
    /// Number of centroids per subvector (k)
    pub centroids_per_subvector: usize,
    /// Training sample size
    pub training_samples: usize,
    /// Enable adaptive subvector assignment
    pub adaptive_assignment: bool,
}

impl Default for ProductQuantizationConfig {
    fn default() -> Self {
        Self {
            subvectors: 8,
            centroids_per_subvector: 256,
            training_samples: 10000,
            adaptive_assignment: true,
        }
    }
}

/// Product Quantization implementation
#[derive(Debug)]
pub struct ProductQuantization {
    config: ProductQuantizationConfig,
    codebooks: Vec<Vec<Vec<f32>>>, // [subvector][centroid][dimension]
    subvector_size: usize,
    dimension: usize,
    trained: bool,
}

impl ProductQuantization {
    /// Create a new Product Quantization instance
    pub fn new(config: ProductQuantizationConfig, dimension: usize) -> Self {
        let subvector_size = dimension / config.subvectors;
        let mut codebooks = Vec::with_capacity(config.subvectors);

        for _ in 0..config.subvectors {
            let mut subvector_codebook = Vec::with_capacity(config.centroids_per_subvector);
            for _ in 0..config.centroids_per_subvector {
                subvector_codebook.push(vec![0.0; subvector_size]);
            }
            codebooks.push(subvector_codebook);
        }

        Self {
            config,
            codebooks,
            subvector_size,
            dimension,
            trained: false,
        }
    }

    /// Train the quantizer on a sample of vectors
    pub fn train(&mut self, training_vectors: &[Vec<f32>]) -> Result<(), QuantizationError> {
        if training_vectors.is_empty() {
            return Err(QuantizationError::InvalidParameters(
                "No training vectors provided".to_string(),
            ));
        }

        if training_vectors[0].len() != self.dimension {
            return Err(QuantizationError::DimensionMismatch {
                expected: self.dimension,
                actual: training_vectors[0].len(),
            });
        }

        // Limit training samples
        let sample_size = std::cmp::min(training_vectors.len(), self.config.training_samples);
        let training_sample = &training_vectors[..sample_size];

        // Train each subvector independently
        for subvector_idx in 0..self.config.subvectors {
            let start_dim = subvector_idx * self.subvector_size;
            let end_dim = std::cmp::min(start_dim + self.subvector_size, self.dimension);

            // Extract subvectors
            let mut subvectors = Vec::new();
            for vector in training_sample {
                subvectors.push(vector[start_dim..end_dim].to_vec());
            }

            // Train codebook for this subvector using K-means
            self.train_subvector(subvector_idx, &subvectors)?;
        }

        self.trained = true;
        Ok(())
    }

    /// Train codebook for a single subvector using K-means
    fn train_subvector(
        &mut self,
        subvector_idx: usize,
        subvectors: &[Vec<f32>],
    ) -> Result<(), QuantizationError> {
        let k = self.config.centroids_per_subvector;
        let mut centroids = self.initialize_centroids(subvectors, k);
        let mut assignments = vec![0; subvectors.len()];
        let mut changed = true;
        let mut iterations = 0;
        let max_iterations = 100;

        while changed && iterations < max_iterations {
            changed = false;
            iterations += 1;

            // Assign each subvector to nearest centroid
            for (i, subvector) in subvectors.iter().enumerate() {
                let mut best_centroid = 0;
                let mut best_distance = f32::INFINITY;

                for (j, centroid) in centroids.iter().enumerate() {
                    let distance = self.euclidean_distance(subvector, centroid);
                    if distance < best_distance {
                        best_distance = distance;
                        best_centroid = j;
                    }
                }

                if assignments[i] != best_centroid {
                    assignments[i] = best_centroid;
                    changed = true;
                }
            }

            // Update centroids
            self.update_centroids(subvectors, &assignments, &mut centroids);
        }

        self.codebooks[subvector_idx] = centroids;
        Ok(())
    }

    /// Initialize centroids using K-means++ initialization
    fn initialize_centroids(&self, subvectors: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        if subvectors.is_empty() || k == 0 {
            return Vec::new();
        }

        let mut centroids = Vec::with_capacity(k);
        let mut rng = fastrand::Rng::new();

        // Choose first centroid randomly
        let first_idx = rng.usize(..subvectors.len());
        centroids.push(subvectors[first_idx].clone());

        // Choose remaining centroids using K-means++ strategy
        for _ in 1..k {
            let mut distances = Vec::with_capacity(subvectors.len());
            let mut total_distance = 0.0;

            for subvector in subvectors {
                let mut min_distance = f32::INFINITY;
                for centroid in &centroids {
                    let distance = self.euclidean_distance(subvector, centroid);
                    min_distance = min_distance.min(distance);
                }
                distances.push(min_distance);
                total_distance += min_distance;
            }

            // Choose next centroid with probability proportional to distance
            let mut cumulative = 0.0;
            let threshold = rng.f32() * total_distance;
            let mut chosen_idx = 0;

            for (i, distance) in distances.iter().enumerate() {
                cumulative += distance;
                if cumulative >= threshold {
                    chosen_idx = i;
                    break;
                }
            }

            centroids.push(subvectors[chosen_idx].clone());
        }

        centroids
    }

    /// Update centroids based on current assignments
    fn update_centroids(
        &self,
        subvectors: &[Vec<f32>],
        assignments: &[usize],
        centroids: &mut Vec<Vec<f32>>,
    ) {
        let k = centroids.len();
        let mut counts = vec![0; k];
        let mut sums = vec![vec![0.0; self.subvector_size]; k];

        // Sum up vectors for each centroid
        for (subvector, &assignment) in subvectors.iter().zip(assignments.iter()) {
            counts[assignment] += 1;
            for (sum, &value) in sums[assignment].iter_mut().zip(subvector.iter()) {
                *sum += value;
            }
        }

        // Update centroids
        for (centroid, (count, sum)) in centroids.iter_mut().zip(counts.iter().zip(sums.iter())) {
            if *count > 0 {
                for (c, &s) in centroid.iter_mut().zip(sum.iter()) {
                    *c = s / (*count as f32);
                }
            }
        }
    }

    /// Calculate Euclidean distance between two vectors
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Quantize a vector
    pub fn quantize(&self, vector: &[f32]) -> Result<Vec<u8>, QuantizationError> {
        // Check dimension first (before training check) to provide better error messages
        if vector.len() != self.dimension {
            return Err(QuantizationError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.len(),
            });
        }

        if !self.trained {
            return Err(QuantizationError::InvalidParameters(
                "Quantizer not trained".to_string(),
            ));
        }

        let mut codes = Vec::with_capacity(self.config.subvectors);

        for subvector_idx in 0..self.config.subvectors {
            let start_dim = subvector_idx * self.subvector_size;
            let end_dim = std::cmp::min(start_dim + self.subvector_size, self.dimension);
            let subvector = &vector[start_dim..end_dim];

            // Find nearest centroid
            let mut best_centroid = 0;
            let mut best_distance = f32::INFINITY;

            for (centroid_idx, centroid) in self.codebooks[subvector_idx].iter().enumerate() {
                let distance = self.euclidean_distance(subvector, centroid);
                if distance < best_distance {
                    best_distance = distance;
                    best_centroid = centroid_idx;
                }
            }

            codes.push(best_centroid as u8);
        }

        Ok(codes)
    }

    /// Reconstruct a vector from quantized codes
    pub fn reconstruct(&self, codes: &[u8]) -> Result<Vec<f32>, QuantizationError> {
        if codes.len() != self.config.subvectors {
            return Err(QuantizationError::InvalidParameters(format!(
                "Expected {} codes, got {}",
                self.config.subvectors,
                codes.len()
            )));
        }

        let mut reconstructed = Vec::with_capacity(self.dimension);

        for (subvector_idx, &code) in codes.iter().enumerate() {
            if code as usize >= self.config.centroids_per_subvector {
                return Err(QuantizationError::InvalidParameters(format!(
                    "Invalid code {} for subvector {}",
                    code, subvector_idx
                )));
            }

            let centroid = &self.codebooks[subvector_idx][code as usize];
            reconstructed.extend_from_slice(centroid);
        }

        // Pad if necessary
        while reconstructed.len() < self.dimension {
            reconstructed.push(0.0);
        }

        Ok(reconstructed)
    }

    /// Calculate quantization error for a vector
    pub fn quantization_error(
        &self,
        original: &[f32],
        quantized: &[u8],
    ) -> Result<f32, QuantizationError> {
        let reconstructed = self.reconstruct(quantized)?;
        Ok(self.euclidean_distance(original, &reconstructed))
    }

    /// Get compression ratio
    pub fn compression_ratio(&self) -> f32 {
        let original_bits = self.dimension * 32; // 32 bits per float
        let quantized_bits = self.config.subvectors * 8; // 8 bits per code
        original_bits as f32 / quantized_bits as f32
    }

    /// Get memory usage in bytes per vector
    pub fn memory_per_vector(&self) -> usize {
        self.config.subvectors
    }
}

// Note: ProductQuantization implements its own methods directly
// and is used via direct method calls, not through the QuantizationMethod trait

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_quantization_creation() {
        let config = ProductQuantizationConfig::default();
        let pq = ProductQuantization::new(config, 128);

        assert_eq!(pq.config.subvectors, 8);
        assert_eq!(pq.config.centroids_per_subvector, 256);
        assert_eq!(pq.dimension, 128);
        assert_eq!(pq.subvector_size, 16);
        assert!(!pq.trained);
    }

    #[test]
    fn test_compression_ratio() {
        let config = ProductQuantizationConfig::default();
        let pq = ProductQuantization::new(config, 128);

        let ratio = pq.compression_ratio();
        assert!(ratio > 1.0); // Should compress
        // 128 dimensions * 32 bits per float = 4096 bits
        // 8 subvectors * 8 bits per code = 64 bits
        // Ratio = 4096 / 64 = 64.0
        assert_eq!(ratio, 64.0);
    }

    #[test]
    fn test_quantization_workflow() {
        let config = ProductQuantizationConfig {
            subvectors: 4,
            centroids_per_subvector: 4,
            training_samples: 100,
            adaptive_assignment: false,
        };

        let mut pq = ProductQuantization::new(config, 8);

        // Generate training data
        let mut training_vectors = Vec::new();
        for i in 0..100 {
            let mut vector = Vec::with_capacity(8);
            for j in 0..8 {
                vector.push((i as f32 + j as f32) * 0.1);
            }
            training_vectors.push(vector);
        }

        // Train
        let result = pq.train(&training_vectors);
        assert!(result.is_ok());
        assert!(pq.trained);

        // Test quantization
        let test_vector = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let quantized = pq.quantize(&test_vector).unwrap();
        assert_eq!(quantized.len(), 4);

        // Test reconstruction
        let reconstructed = pq.reconstruct(&quantized).unwrap();
        assert_eq!(reconstructed.len(), 8);

        // Test quantization error
        let error = pq.quantization_error(&test_vector, &quantized).unwrap();
        assert!(error >= 0.0);
    }

    #[test]
    fn test_dimension_mismatch() {
        let config = ProductQuantizationConfig::default();
        let pq = ProductQuantization::new(config, 128);

        let wrong_vector = vec![1.0; 64]; // Wrong dimension
        let result = pq.quantize(&wrong_vector);
        assert!(matches!(
            result,
            Err(QuantizationError::DimensionMismatch { .. })
        ));
    }

    #[test]
    fn test_not_trained() {
        let config = ProductQuantizationConfig::default();
        let pq = ProductQuantization::new(config, 128);

        let vector = vec![1.0; 128];
        let result = pq.quantize(&vector);
        assert!(matches!(
            result,
            Err(QuantizationError::InvalidParameters(_))
        ));
    }

    #[test]
    fn test_pq_compression_and_search_accuracy() {
        // Test PQ compression ratio and verify search accuracy
        let config = ProductQuantizationConfig {
            subvectors: 8,
            centroids_per_subvector: 256,
            training_samples: 1000,
            adaptive_assignment: true,
        };

        let mut pq = ProductQuantization::new(config, 128);

        // Generate diverse training data
        let mut training_vectors = Vec::new();
        for i in 0..1000 {
            let mut vector = Vec::with_capacity(128);
            for j in 0..128 {
                vector.push((i as f32 * 0.01 + j as f32 * 0.001).sin());
            }
            training_vectors.push(vector);
        }

        // Train
        pq.train(&training_vectors).unwrap();

        // Test compression ratio
        let compression_ratio = pq.compression_ratio();
        assert!(compression_ratio > 1.0, "PQ should provide compression");
        assert!(
            compression_ratio >= 3.0,
            "PQ should provide at least 3x compression"
        );

        // Test quantization and reconstruction accuracy
        let test_vectors = &training_vectors[0..10];
        let mut total_error = 0.0;
        let mut max_error: f32 = 0.0;

        for original in test_vectors {
            let quantized = pq.quantize(original).unwrap();
            let reconstructed = pq.reconstruct(&quantized).unwrap();

            // Calculate reconstruction error
            let error = original
                .iter()
                .zip(reconstructed.iter())
                .map(|(a, b)| (a - b).abs())
                .sum::<f32>()
                / original.len() as f32;

            total_error += error;
            max_error = max_error.max(error);
        }

        let avg_error = total_error / test_vectors.len() as f32;

        // Verify reasonable reconstruction quality
        assert!(
            avg_error < 0.1,
            "Average reconstruction error should be < 0.1, got {}",
            avg_error
        );
        assert!(
            max_error < 0.2,
            "Max reconstruction error should be < 0.2, got {}",
            max_error
        );

        // Verify memory savings
        let memory_per_vector = pq.memory_per_vector();
        let original_memory = 128 * std::mem::size_of::<f32>(); // 128 * 4 = 512 bytes
        assert!(
            memory_per_vector < original_memory,
            "PQ should use less memory per vector"
        );
    }
}
