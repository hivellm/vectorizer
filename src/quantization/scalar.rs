//! Scalar Quantization implementation
//!
//! Implements scalar quantization with configurable bit depths (8-bit, 4-bit, 2-bit).
//! Based on benchmark results showing 4x memory compression with improved quality.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::quantization::traits::*;
use crate::quantization::{QuantizationError, QuantizationResult, QuantizationType};

/// Scalar Quantization implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalarQuantization {
    /// Number of bits per dimension (8, 4, 2, or 1)
    pub bits: u8,
    /// Minimum value in the dataset
    pub min_value: f32,
    /// Maximum value in the dataset
    pub max_value: f32,
    /// Scaling factor for quantization
    pub scale: f32,
    /// Offset for quantization
    pub offset: f32,
    /// Number of possible quantized values
    pub quantized_levels: usize,
}

impl ScalarQuantization {
    /// Create a new scalar quantization instance
    pub fn new(bits: u8) -> QuantizationResult<Self> {
        if !matches!(bits, 1 | 2 | 4 | 8) {
            return Err(QuantizationError::InvalidParameters(format!(
                "Invalid bit depth: {}. Must be 1, 2, 4, or 8",
                bits
            )));
        }

        Ok(Self {
            bits,
            min_value: 0.0,
            max_value: 0.0,
            scale: 1.0,
            offset: 0.0,
            quantized_levels: 2usize.pow(bits as u32),
        })
    }

    /// Fit quantization parameters to a dataset
    pub fn fit(&mut self, vectors: &[Vec<f32>]) -> QuantizationResult<()> {
        if vectors.is_empty() {
            return Err(QuantizationError::InvalidParameters(
                "Cannot fit quantization to empty dataset".to_string(),
            ));
        }

        // Calculate min/max across all vectors and dimensions
        let mut min_val = f32::INFINITY;
        let mut max_val = f32::NEG_INFINITY;

        for vector in vectors {
            for &value in vector {
                min_val = min_val.min(value);
                max_val = max_val.max(value);
            }
        }

        self.min_value = min_val;
        self.max_value = max_val;
        self.scale = (max_val - min_val) / (self.quantized_levels - 1) as f32;
        self.offset = min_val;

        Ok(())
    }

    /// Quantize a single vector
    pub fn quantize_vector(&self, vector: &[f32]) -> QuantizationResult<Vec<u8>> {
        match self.bits {
            8 => self.quantize_8bit(vector),
            4 => self.quantize_4bit(vector),
            2 => self.quantize_2bit(vector),
            1 => self.quantize_1bit(vector),
            _ => Err(QuantizationError::InvalidParameters(format!(
                "Unsupported bit depth: {}",
                self.bits
            ))),
        }
    }

    /// Dequantize a single vector
    pub fn dequantize_vector(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        match self.bits {
            8 => self.dequantize_8bit(quantized),
            4 => self.dequantize_4bit(quantized),
            2 => self.dequantize_2bit(quantized),
            1 => self.dequantize_1bit(quantized),
            _ => Err(QuantizationError::InvalidParameters(format!(
                "Unsupported bit depth: {}",
                self.bits
            ))),
        }
    }

    /// 8-bit quantization (primary method from benchmarks)
    fn quantize_8bit(&self, vector: &[f32]) -> QuantizationResult<Vec<u8>> {
        let mut quantized = Vec::with_capacity(vector.len());

        for &value in vector {
            let normalized = (value - self.offset) / self.scale;
            let clamped = normalized.clamp(0.0, (self.quantized_levels - 1) as f32);
            quantized.push(clamped.round() as u8);
        }

        Ok(quantized)
    }

    /// 8-bit dequantization
    fn dequantize_8bit(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        let mut dequantized = Vec::with_capacity(quantized.len());

        for &q in quantized {
            let value = self.offset + (q as f32) * self.scale;
            dequantized.push(value);
        }

        Ok(dequantized)
    }

    /// 4-bit quantization (packed)
    fn quantize_4bit(&self, vector: &[f32]) -> QuantizationResult<Vec<u8>> {
        let mut quantized = Vec::with_capacity((vector.len() + 1) / 2);

        for chunk in vector.chunks(2) {
            let mut packed = 0u8;

            for (i, &value) in chunk.iter().enumerate() {
                let normalized = (value - self.offset) / self.scale;
                let clamped = normalized.clamp(0.0, (self.quantized_levels - 1) as f32);
                let quantized_value = clamped.round() as u8;

                if i == 0 {
                    packed |= quantized_value;
                } else {
                    packed |= (quantized_value << 4);
                }
            }

            quantized.push(packed);
        }

        Ok(quantized)
    }

    /// 4-bit dequantization (unpacked)
    fn dequantize_4bit(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        let mut dequantized = Vec::with_capacity(quantized.len() * 2);

        for &packed in quantized {
            // Extract lower 4 bits
            let lower = packed & 0x0F;
            let value1 = self.offset + (lower as f32) * self.scale;
            dequantized.push(value1);

            // Extract upper 4 bits
            let upper = (packed & 0xF0) >> 4;
            let value2 = self.offset + (upper as f32) * self.scale;
            dequantized.push(value2);
        }

        Ok(dequantized)
    }

    /// 2-bit quantization (packed)
    fn quantize_2bit(&self, vector: &[f32]) -> QuantizationResult<Vec<u8>> {
        let mut quantized = Vec::with_capacity((vector.len() + 3) / 4);

        for chunk in vector.chunks(4) {
            let mut packed = 0u8;

            for (i, &value) in chunk.iter().enumerate() {
                let normalized = (value - self.offset) / self.scale;
                let clamped = normalized.clamp(0.0, (self.quantized_levels - 1) as f32);
                let quantized_value = clamped.round() as u8;

                packed |= (quantized_value << (i * 2));
            }

            quantized.push(packed);
        }

        Ok(quantized)
    }

    /// 2-bit dequantization (unpacked)
    fn dequantize_2bit(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        let mut dequantized = Vec::with_capacity(quantized.len() * 4);

        for &packed in quantized {
            for i in 0..4 {
                let value_bits = (packed >> (i * 2)) & 0x03;
                let value = self.offset + (value_bits as f32) * self.scale;
                dequantized.push(value);
            }
        }

        Ok(dequantized)
    }

    /// 1-bit quantization (binary)
    fn quantize_1bit(&self, vector: &[f32]) -> QuantizationResult<Vec<u8>> {
        let threshold = (self.min_value + self.max_value) / 2.0;
        let mut quantized = Vec::with_capacity((vector.len() + 7) / 8);

        for chunk in vector.chunks(8) {
            let mut packed = 0u8;

            for (i, &value) in chunk.iter().enumerate() {
                if value >= threshold {
                    packed |= 1 << i;
                }
            }

            quantized.push(packed);
        }

        Ok(quantized)
    }

    /// 1-bit dequantization (binary)
    fn dequantize_1bit(&self, quantized: &[u8]) -> QuantizationResult<Vec<f32>> {
        let threshold = (self.min_value + self.max_value) / 2.0;
        let mut dequantized = Vec::with_capacity(quantized.len() * 8);

        for &packed in quantized {
            for i in 0..8 {
                let bit = (packed >> i) & 1;
                let value = if bit == 1 {
                    self.max_value
                } else {
                    self.min_value
                };
                dequantized.push(value);
            }
        }

        Ok(dequantized)
    }

    /// Calculate quantization error for quality assessment
    pub fn calculate_quantization_error(
        &self,
        original: &[f32],
        quantized: &[u8],
    ) -> QuantizationResult<f32> {
        let dequantized = self.dequantize_vector(quantized)?;

        if original.len() != dequantized.len() {
            return Err(QuantizationError::DimensionMismatch {
                expected: original.len(),
                actual: dequantized.len(),
            });
        }

        let mse = original
            .iter()
            .zip(dequantized.iter())
            .map(|(orig, deq)| (orig - deq).powi(2))
            .sum::<f32>()
            / original.len() as f32;

        Ok(mse)
    }

    /// Calculate theoretical compression ratio
    pub fn theoretical_compression_ratio(&self) -> f32 {
        let original_bits = 32.0; // f32
        let quantized_bits = self.bits as f32;
        original_bits / quantized_bits
    }
}

impl QuantizationMethod for ScalarQuantization {
    fn quantize(&self, vectors: &[Vec<f32>]) -> QuantizationResult<QuantizedVectors> {
        if vectors.is_empty() {
            return Err(QuantizationError::InvalidParameters(
                "Cannot quantize empty vector set".to_string(),
            ));
        }

        let dimension = vectors[0].len();
        let mut all_quantized = Vec::new();

        for vector in vectors {
            if vector.len() != dimension {
                return Err(QuantizationError::DimensionMismatch {
                    expected: dimension,
                    actual: vector.len(),
                });
            }

            let quantized = self.quantize_vector(vector)?;
            all_quantized.extend(quantized);
        }

        let parameters = self.serialize_params()?;

        Ok(QuantizedVectors {
            data: all_quantized,
            dimension,
            count: vectors.len(),
            parameters,
        })
    }

    fn dequantize(&self, quantized: &QuantizedVectors) -> QuantizationResult<Vec<Vec<f32>>> {
        let mut vectors = Vec::with_capacity(quantized.count);
        let bytes_per_vector = match self.bits {
            8 => quantized.dimension,
            4 => (quantized.dimension + 1) / 2,
            2 => (quantized.dimension + 3) / 4,
            1 => (quantized.dimension + 7) / 8,
            _ => {
                return Err(QuantizationError::InvalidParameters(format!(
                    "Unsupported bit depth: {}",
                    self.bits
                )));
            }
        };

        for i in 0..quantized.count {
            let start = i * bytes_per_vector;
            let end = start + bytes_per_vector;

            if end > quantized.data.len() {
                return Err(QuantizationError::InvalidParameters(
                    "Quantized data length mismatch".to_string(),
                ));
            }

            let vector_data = &quantized.data[start..end];
            let dequantized_vector = self.dequantize_vector(vector_data)?;

            // Truncate to original dimension (for packed formats)
            let vector = dequantized_vector[..quantized.dimension].to_vec();
            vectors.push(vector);
        }

        Ok(vectors)
    }

    fn memory_usage(&self, vector_count: usize, dimension: usize) -> usize {
        match self.bits {
            8 => vector_count * dimension,
            4 => vector_count * (dimension + 1) / 2,
            2 => vector_count * (dimension + 3) / 4,
            1 => vector_count * (dimension + 7) / 8,
            _ => 0,
        }
    }

    fn quality_loss(&self) -> f32 {
        // Theoretical quality loss based on quantization error
        let quantization_step = self.scale;
        let signal_range = self.max_value - self.min_value;

        if signal_range == 0.0 {
            return 0.0;
        }

        // Quality loss is proportional to quantization step size
        quantization_step / signal_range
    }

    fn method_type(&self) -> QuantizationType {
        QuantizationType::Scalar(self.bits)
    }

    fn validate_parameters(&self) -> QuantizationResult<()> {
        if !matches!(self.bits, 1 | 2 | 4 | 8) {
            return Err(QuantizationError::InvalidParameters(format!(
                "Invalid bit depth: {}",
                self.bits
            )));
        }

        if self.min_value >= self.max_value {
            return Err(QuantizationError::InvalidParameters(
                "min_value must be less than max_value".to_string(),
            ));
        }

        if self.scale <= 0.0 {
            return Err(QuantizationError::InvalidParameters(
                "scale must be positive".to_string(),
            ));
        }

        Ok(())
    }

    fn serialize_params(&self) -> QuantizationResult<QuantizationParams> {
        Ok(QuantizationParams::Scalar {
            bits: self.bits,
            min_value: self.min_value,
            max_value: self.max_value,
            scale: self.scale,
        })
    }

    fn deserialize_params(&mut self, params: QuantizationParams) -> QuantizationResult<()> {
        if let QuantizationParams::Scalar {
            bits,
            min_value,
            max_value,
            scale,
        } = params
        {
            self.bits = bits;
            self.min_value = min_value;
            self.max_value = max_value;
            self.scale = scale;
            self.quantized_levels = 2usize.pow(bits as u32);
            Ok(())
        } else {
            Err(QuantizationError::InvalidParameters(
                "Parameter type mismatch for ScalarQuantization".to_string(),
            ))
        }
    }
}

impl QuantizedSearch for ScalarQuantization {
    fn similarity(&self, query: &[f32], quantized_vector: &[u8]) -> QuantizationResult<f32> {
        let dequantized = self.dequantize_vector(quantized_vector)?;

        if query.len() != dequantized.len() {
            return Err(QuantizationError::DimensionMismatch {
                expected: query.len(),
                actual: dequantized.len(),
            });
        }

        // Calculate cosine similarity
        let dot_product: f32 = query
            .iter()
            .zip(dequantized.iter())
            .map(|(a, b)| a * b)
            .sum();

        let query_norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        let vector_norm: f32 = dequantized.iter().map(|x| x * x).sum::<f32>().sqrt();

        if query_norm == 0.0 || vector_norm == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (query_norm * vector_norm))
    }

    fn quantized_similarity(
        &self,
        quantized_a: &[u8],
        quantized_b: &[u8],
    ) -> QuantizationResult<f32> {
        let dequantized_a = self.dequantize_vector(quantized_a)?;
        let dequantized_b = self.dequantize_vector(quantized_b)?;

        if dequantized_a.len() != dequantized_b.len() {
            return Err(QuantizationError::DimensionMismatch {
                expected: dequantized_a.len(),
                actual: dequantized_b.len(),
            });
        }

        // Calculate cosine similarity
        let dot_product: f32 = dequantized_a
            .iter()
            .zip(dequantized_b.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = dequantized_a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = dequantized_b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm_a * norm_b))
    }

    fn batch_similarity(
        &self,
        query: &[f32],
        quantized_vectors: &[&[u8]],
    ) -> QuantizationResult<Vec<f32>> {
        let mut similarities = Vec::with_capacity(quantized_vectors.len());

        for quantized_vector in quantized_vectors {
            let similarity = self.similarity(query, quantized_vector)?;
            similarities.push(similarity);
        }

        Ok(similarities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_quantization_8bit() {
        let mut sq = ScalarQuantization::new(8).unwrap();

        let vectors = vec![
            vec![0.1, 0.5, 0.9],
            vec![0.2, 0.6, 0.8],
            vec![0.3, 0.7, 0.7],
        ];

        sq.fit(&vectors).unwrap();

        let quantized = sq.quantize(&vectors).unwrap();
        let dequantized = sq.dequantize(&quantized).unwrap();

        assert_eq!(quantized.count, 3);
        assert_eq!(quantized.dimension, 3);
        assert_eq!(dequantized.len(), 3);

        // Check that dequantized values are close to original
        for (orig, deq) in vectors.iter().zip(dequantized.iter()) {
            for (o, d) in orig.iter().zip(deq.iter()) {
                assert!(
                    (o - d).abs() < 0.1,
                    "Quantization error too large: {} vs {}",
                    o,
                    d
                );
            }
        }
    }

    #[test]
    fn test_scalar_quantization_4bit() {
        let mut sq = ScalarQuantization::new(4).unwrap();

        let vectors = vec![vec![0.1, 0.5, 0.9, 0.3], vec![0.2, 0.6, 0.8, 0.4]];

        sq.fit(&vectors).unwrap();

        let quantized = sq.quantize(&vectors).unwrap();
        let dequantized = sq.dequantize(&quantized).unwrap();

        assert_eq!(quantized.count, 2);
        assert_eq!(quantized.dimension, 4);

        // 4-bit should have more quantization error than 8-bit
        let mut total_error = 0.0;
        for (orig, deq) in vectors.iter().zip(dequantized.iter()) {
            for (o, d) in orig.iter().zip(deq.iter()) {
                total_error += (o - d).abs();
            }
        }

        assert!(total_error > 0.0, "Should have some quantization error");
    }

    #[test]
    fn test_memory_usage_calculation() {
        let sq8 = ScalarQuantization::new(8).unwrap();
        let sq4 = ScalarQuantization::new(4).unwrap();
        let sq2 = ScalarQuantization::new(2).unwrap();
        let sq1 = ScalarQuantization::new(1).unwrap();

        let vector_count = 1000;
        let dimension = 512;

        assert_eq!(sq8.memory_usage(vector_count, dimension), 1000 * 512);
        assert_eq!(
            sq4.memory_usage(vector_count, dimension),
            1000 * (512 + 1) / 2
        );
        assert_eq!(
            sq2.memory_usage(vector_count, dimension),
            1000 * (512 + 3) / 4
        );
        assert_eq!(
            sq1.memory_usage(vector_count, dimension),
            1000 * (512 + 7) / 8
        );
    }

    #[test]
    fn test_similarity_calculation() {
        let mut sq = ScalarQuantization::new(8).unwrap();

        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        sq.fit(&vectors).unwrap();

        let quantized = sq.quantize(&vectors).unwrap();
        let query = vec![1.0, 0.0, 0.0];

        // Calculate similarity with first vector (should be high)
        let first_vector_data = &quantized.data[0..quantized.dimension];
        let similarity = sq.similarity(&query, first_vector_data).unwrap();

        assert!(
            similarity > 0.9,
            "Similarity should be high for identical vectors"
        );
    }

    #[test]
    fn test_validation() {
        let mut sq = ScalarQuantization::new(8).unwrap();

        // Test valid parameters
        sq.min_value = 0.0;
        sq.max_value = 1.0;
        sq.scale = 0.1;
        assert!(sq.validate_parameters().is_ok());

        // Test invalid bit depth
        sq.bits = 16;
        assert!(sq.validate_parameters().is_err());

        // Test invalid min/max
        sq.bits = 8;
        sq.min_value = 1.0;
        sq.max_value = 0.0;
        assert!(sq.validate_parameters().is_err());
    }
}
