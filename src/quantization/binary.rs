//! Binary Quantization implementation
//!
//! Binary quantization converts vectors to 1-bit per dimension, providing extreme
//! compression (32x reduction) at the cost of lower quality. Useful for first-stage
//! filtering or when memory is extremely constrained.

use crate::quantization::{
    QuantizationResult, QuantizationError, QuantizationType,
    traits::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Binary Quantization implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryQuantization {
    /// Threshold for binary conversion (values > threshold = 1, else = -1)
    threshold: f32,
    /// Trained flag
    trained: bool,
}

impl BinaryQuantization {
    /// Create a new binary quantization instance
    pub fn new() -> Self {
        Self {
            threshold: 0.0,
            trained: false,
        }
    }

    /// Train the quantizer on a sample of vectors
    /// Uses median value as threshold for optimal binary encoding
    pub fn train(&mut self, training_vectors: &[Vec<f32>]) -> QuantizationResult<()> {
        if training_vectors.is_empty() {
            return Err(QuantizationError::InvalidParameters(
                "Cannot train binary quantization on empty dataset".to_string()
            ));
        }

        // Calculate median as threshold
        let mut all_values: Vec<f32> = training_vectors
            .iter()
            .flat_map(|v| v.iter().copied())
            .collect();
        
        all_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        self.threshold = if all_values.is_empty() {
            0.0
        } else {
            all_values[all_values.len() / 2]
        };
        
        self.trained = true;
        Ok(())
    }

    /// Quantize a single vector to binary representation
    /// Returns packed bits (1 byte per 8 dimensions)
    pub fn quantize_vector(&self, vector: &[f32]) -> QuantizationResult<Vec<u8>> {
        if !self.trained {
            return Err(QuantizationError::InvalidParameters(
                "Binary quantizer not trained. Call train() first.".to_string()
            ));
        }

        // Pack bits into bytes (8 bits per byte)
        let mut bytes = vec![0u8; (vector.len() + 7) / 8];

        for (i, &val) in vector.iter().enumerate() {
            if val > self.threshold {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                bytes[byte_idx] |= 1 << bit_idx;
            }
        }

        Ok(bytes)
    }

    /// Dequantize binary representation back to f32 vector
    /// Returns vector with values -1.0 or 1.0
    pub fn dequantize_vector(&self, codes: &[u8], dimension: usize) -> QuantizationResult<Vec<f32>> {
        let mut vector = vec![0.0; dimension];

        for i in 0..dimension {
            let byte_idx = i / 8;
            let bit_idx = i % 8;

            if byte_idx < codes.len() {
                let bit_set = (codes[byte_idx] & (1 << bit_idx)) != 0;
                vector[i] = if bit_set { 1.0 } else { -1.0 };
            }
        }

        Ok(vector)
    }

    /// Get the threshold value
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Check if quantizer is trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }
}

impl Default for BinaryQuantization {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantizationMethod for BinaryQuantization {
    fn quantize(&self, vectors: &[Vec<f32>]) -> QuantizationResult<QuantizedVectors> {
        if vectors.is_empty() {
            return Err(QuantizationError::InvalidParameters(
                "Cannot quantize empty vector set".to_string()
            ));
        }

        if !self.trained {
            return Err(QuantizationError::InvalidParameters(
                "Binary quantizer not trained".to_string()
            ));
        }

        let dimension = vectors[0].len();
        
        // Validate all vectors have same dimension
        for vector in vectors {
            if vector.len() != dimension {
                return Err(QuantizationError::DimensionMismatch {
                    expected: dimension,
                    actual: vector.len(),
                });
            }
        }

        // Quantize all vectors
        let mut quantized_data = Vec::new();
        for vector in vectors {
            let quantized = self.quantize_vector(vector)?;
            quantized_data.extend_from_slice(&quantized);
        }

        Ok(QuantizedVectors {
            data: quantized_data,
            dimension,
            count: vectors.len(),
            parameters: QuantizationParams::Binary {
                threshold: self.threshold,
            },
        })
    }

    fn dequantize(&self, quantized: &QuantizedVectors) -> QuantizationResult<Vec<Vec<f32>>> {
        let bytes_per_vector = (quantized.dimension + 7) / 8;
        let expected_size = bytes_per_vector * quantized.count;

        if quantized.data.len() < expected_size {
            return Err(QuantizationError::InvalidParameters(format!(
                "Invalid quantized data size: expected at least {}, got {}",
                expected_size,
                quantized.data.len()
            )));
        }

        let mut result = Vec::with_capacity(quantized.count);
        
        for i in 0..quantized.count {
            let start = i * bytes_per_vector;
            let end = start + bytes_per_vector;
            let codes = &quantized.data[start..end];
            let vector = self.dequantize_vector(codes, quantized.dimension)?;
            result.push(vector);
        }

        Ok(result)
    }

    fn memory_usage(&self, vector_count: usize, dimension: usize) -> usize {
        // Binary quantization: 1 bit per dimension = dimension/8 bytes per vector
        let bytes_per_vector = (dimension + 7) / 8;
        bytes_per_vector * vector_count + std::mem::size_of::<f32>() // + threshold storage
    }

    fn quality_loss(&self) -> f32 {
        // Binary quantization has significant quality loss (estimated 0.3-0.5)
        // This is a rough estimate - actual quality depends on data distribution
        0.4
    }

    fn method_type(&self) -> QuantizationType {
        QuantizationType::Binary
    }

    fn validate_parameters(&self) -> QuantizationResult<()> {
        if !self.trained {
            return Err(QuantizationError::InvalidParameters(
                "Binary quantizer must be trained before use".to_string()
            ));
        }
        Ok(())
    }

    fn serialize_params(&self) -> QuantizationResult<QuantizationParams> {
        Ok(QuantizationParams::Binary {
            threshold: self.threshold,
        })
    }

    fn deserialize_params(&mut self, params: QuantizationParams) -> QuantizationResult<()> {
        match params {
            QuantizationParams::Binary { threshold } => {
                self.threshold = threshold;
                self.trained = true;
                Ok(())
            }
            _ => Err(QuantizationError::InvalidParameters(
                "Invalid parameters for binary quantization".to_string()
            )),
        }
    }
}

impl QuantizedSearch for BinaryQuantization {
    fn similarity(&self, query: &[f32], quantized_vector: &[u8]) -> QuantizationResult<f32> {
        // Quantize query
        let quantized_query = self.quantize_vector(query)?;
        
        // Calculate Hamming distance (XOR and count bits)
        let mut hamming_distance = 0;
        let min_len = quantized_query.len().min(quantized_vector.len());
        
        for i in 0..min_len {
            let xor = quantized_query[i] ^ quantized_vector[i];
            hamming_distance += xor.count_ones() as usize;
        }
        
        // Convert Hamming distance to similarity (normalized)
        let max_distance = query.len();
        let similarity = 1.0 - (hamming_distance as f32 / max_distance as f32);
        
        Ok(similarity)
    }

    fn quantized_similarity(&self, quantized_a: &[u8], quantized_b: &[u8]) -> QuantizationResult<f32> {
        let mut hamming_distance = 0;
        let min_len = quantized_a.len().min(quantized_b.len());
        
        for i in 0..min_len {
            let xor = quantized_a[i] ^ quantized_b[i];
            hamming_distance += xor.count_ones() as usize;
        }
        
        // Estimate dimension from byte count (assuming 8 bits per byte)
        let estimated_dim = min_len * 8;
        let similarity = 1.0 - (hamming_distance as f32 / estimated_dim as f32);
        
        Ok(similarity)
    }

    fn batch_similarity(&self, query: &[f32], quantized_vectors: &[&[u8]]) -> QuantizationResult<Vec<f32>> {
        let quantized_query = self.quantize_vector(query)?;
        
        let similarities: Vec<f32> = quantized_vectors
            .iter()
            .map(|qv| {
                self.quantized_similarity(&quantized_query, qv)
                    .unwrap_or(0.0)
            })
            .collect();
        
        Ok(similarities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_quantization_creation() {
        let quantizer = BinaryQuantization::new();
        assert!(!quantizer.is_trained());
        assert_eq!(quantizer.threshold(), 0.0);
    }

    #[test]
    fn test_binary_quantization_training() {
        let mut quantizer = BinaryQuantization::new();
        
        // Create test vectors
        let vectors = vec![
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![2.0, 3.0, 4.0, 5.0, 6.0],
            vec![3.0, 4.0, 5.0, 6.0, 7.0],
        ];
        
        let result = quantizer.train(&vectors);
        assert!(result.is_ok());
        assert!(quantizer.is_trained());
        assert!(quantizer.threshold() > 0.0);
    }

    #[test]
    fn test_binary_quantization_quantize_dequantize() {
        let mut quantizer = BinaryQuantization::new();
        
        let vectors = vec![
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![-1.0, -2.0, -3.0, -4.0, -5.0],
        ];
        
        quantizer.train(&vectors).unwrap();
        
        // Quantize
        let quantized = quantizer.quantize_vector(&vectors[0]).unwrap();
        assert!(!quantized.is_empty());
        
        // Dequantize
        let dequantized = quantizer.dequantize_vector(&quantized, 5).unwrap();
        assert_eq!(dequantized.len(), 5);
        
        // Verify values are -1.0 or 1.0
        for val in &dequantized {
            assert!(val.abs() == 1.0);
        }
    }

    #[test]
    fn test_binary_quantization_memory_usage() {
        let quantizer = BinaryQuantization::new();
        
        let memory = quantizer.memory_usage(1000, 512);
        // 512 dimensions = 64 bytes per vector (512/8)
        // 1000 vectors = 64000 bytes + overhead
        assert!(memory > 64000);
        assert!(memory < 100000); // Should be much less than full precision (2MB)
    }

    #[test]
    fn test_binary_quantization_quantized_search() {
        let mut quantizer = BinaryQuantization::new();
        
        let vectors = vec![
            vec![1.0, 2.0, 3.0, 4.0],
            vec![1.0, 2.0, 3.0, 4.0],
            vec![-1.0, -2.0, -3.0, -4.0],
        ];
        
        quantizer.train(&vectors).unwrap();
        
        let query = vec![1.0, 2.0, 3.0, 4.0];
        let quantized_vector = quantizer.quantize_vector(&vectors[0]).unwrap();
        
        let similarity = quantizer.similarity(&query, &quantized_vector).unwrap();
        assert!(similarity > 0.0);
        assert!(similarity <= 1.0);
    }

    #[test]
    fn test_binary_quantization_trait_implementation() {
        let mut quantizer = BinaryQuantization::new();
        
        let vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
        ];
        
        quantizer.train(&vectors).unwrap();
        
        // Test QuantizationMethod trait
        let quantized = quantizer.quantize(&vectors).unwrap();
        assert_eq!(quantized.count, 2);
        assert_eq!(quantized.dimension, 3);
        
        let dequantized = quantizer.dequantize(&quantized).unwrap();
        assert_eq!(dequantized.len(), 2);
        assert_eq!(dequantized[0].len(), 3);
        
        // Test method_type
        assert_eq!(quantizer.method_type(), QuantizationType::Binary);
        
        // Test quality_loss
        let loss = quantizer.quality_loss();
        assert!(loss > 0.0 && loss < 1.0);
        
        // Test validate_parameters
        assert!(quantizer.validate_parameters().is_ok());
        
        // Test serialize/deserialize
        let params = quantizer.serialize_params().unwrap();
        let mut new_quantizer = BinaryQuantization::new();
        new_quantizer.deserialize_params(params).unwrap();
        assert!(new_quantizer.is_trained());
    }
}

