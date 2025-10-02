//! Core traits for quantization methods

use crate::quantization::QuantizationResult;
use serde::{Deserialize, Serialize};

/// Represents quantized vector data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizedVectors {
    /// The quantized data (compressed format)
    pub data: Vec<u8>,
    /// Vector dimensions
    pub dimension: usize,
    /// Number of vectors
    pub count: usize,
    /// Quantization parameters
    pub parameters: QuantizationParams,
}

/// Parameters specific to each quantization method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantizationParams {
    /// Scalar quantization parameters
    Scalar {
        bits: u8,
        min_value: f32,
        max_value: f32,
        scale: f32,
    },
    /// Product quantization parameters
    Product {
        subvector_count: usize,
        subvector_size: usize,
        codebook_size: usize,
        codebooks: Vec<Vec<Vec<f32>>>,
    },
    /// Binary quantization parameters
    Binary {
        threshold: f32,
    },
}

/// Core trait for all quantization methods
pub trait QuantizationMethod: Send + Sync {
    /// Quantize a batch of vectors
    fn quantize(&self, vectors: &[Vec<f32>]) -> QuantizationResult<QuantizedVectors>;
    
    /// Dequantize vectors back to float32
    fn dequantize(&self, quantized: &QuantizedVectors) -> QuantizationResult<Vec<Vec<f32>>>;
    
    /// Calculate memory usage for given vector count and dimension
    fn memory_usage(&self, vector_count: usize, dimension: usize) -> usize;
    
    /// Estimate quality loss (0.0 = no loss, 1.0 = complete loss)
    fn quality_loss(&self) -> f32;
    
    /// Get quantization method type
    fn method_type(&self) -> crate::quantization::QuantizationType;
    
    /// Validate quantization parameters
    fn validate_parameters(&self) -> QuantizationResult<()>;
    
    /// Serialize quantization parameters
    fn serialize_params(&self) -> QuantizationResult<QuantizationParams>;
    
    /// Deserialize quantization parameters
    fn deserialize_params(&mut self, params: QuantizationParams) -> QuantizationResult<()>;
}

/// Trait for quantization methods that support incremental updates
pub trait IncrementalQuantization: QuantizationMethod {
    /// Add a single vector to existing quantized data
    fn add_vector(&self, quantized: &mut QuantizedVectors, vector: &[f32]) -> QuantizationResult<()>;
    
    /// Remove a vector by index
    fn remove_vector(&self, quantized: &mut QuantizedVectors, index: usize) -> QuantizationResult<()>;
    
    /// Update a vector at specific index
    fn update_vector(&self, quantized: &mut QuantizedVectors, index: usize, vector: &[f32]) -> QuantizationResult<()>;
}

/// Trait for quantization methods that support similarity search
pub trait QuantizedSearch {
    /// Calculate similarity between query and quantized vector
    fn similarity(&self, query: &[f32], quantized_vector: &[u8]) -> QuantizationResult<f32>;
    
    /// Calculate similarity between two quantized vectors
    fn quantized_similarity(&self, quantized_a: &[u8], quantized_b: &[u8]) -> QuantizationResult<f32>;
    
    /// Batch similarity calculation for multiple vectors
    fn batch_similarity(&self, query: &[f32], quantized_vectors: &[&[u8]]) -> QuantizationResult<Vec<f32>>;
}

/// Trait for quantization methods that support quality monitoring
pub trait QualityMonitoring {
    /// Calculate current quality metrics
    fn calculate_quality_metrics(&self, original: &[Vec<f32>], quantized: &QuantizedVectors) -> QuantizationResult<QualityMetrics>;
    
    /// Monitor quality degradation over time
    fn monitor_quality_degradation(&self, metrics_history: &[QualityMetrics]) -> QuantizationResult<QualityTrend>;
}

/// Quality metrics for quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Mean Absolute Precision (MAP)
    pub map_score: f32,
    /// Recall at K (default K=10)
    pub recall_at_k: f32,
    /// Precision at K (default K=10)
    pub precision_at_k: f32,
    /// Mean squared error
    pub mse: f32,
    /// Cosine similarity preservation
    pub cosine_similarity: f32,
    /// Compression ratio achieved
    pub compression_ratio: f32,
    /// Timestamp of measurement
    pub timestamp: std::time::SystemTime,
}

/// Quality trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityTrend {
    /// Quality is stable
    Stable,
    /// Quality is improving
    Improving { rate: f32 },
    /// Quality is degrading
    Degrading { rate: f32 },
    /// Quality is fluctuating
    Fluctuating { variance: f32 },
}

/// Configuration for quantization optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Target compression ratio
    pub target_compression: f32,
    /// Minimum acceptable quality
    pub min_quality: f32,
    /// Maximum acceptable quality loss
    pub max_quality_loss: f32,
    /// Enable auto-tuning
    pub auto_tune: bool,
    /// Optimization algorithm to use
    pub algorithm: OptimizationAlgorithm,
}

/// Available optimization algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationAlgorithm {
    /// Grid search optimization
    GridSearch,
    /// Bayesian optimization
    Bayesian,
    /// Genetic algorithm
    Genetic,
    /// Random search
    Random,
}

/// Trait for quantization methods that support optimization
pub trait QuantizationOptimization: QuantizationMethod {
    /// Optimize quantization parameters for given constraints
    fn optimize(&self, vectors: &[Vec<f32>], config: &OptimizationConfig) -> QuantizationResult<QuantizationParams>;
    
    /// Get optimization recommendations
    fn get_recommendations(&self, vectors: &[Vec<f32>]) -> QuantizationResult<Vec<OptimizationRecommendation>>;
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Recommended parameters
    pub parameters: QuantizationParams,
    /// Expected compression ratio
    pub expected_compression: f32,
    /// Expected quality score
    pub expected_quality: f32,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Reason for recommendation
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quantized_vectors_serialization() {
        let quantized = QuantizedVectors {
            data: vec![1, 2, 3, 4],
            dimension: 2,
            count: 2,
            parameters: QuantizationParams::Scalar {
                bits: 8,
                min_value: 0.0,
                max_value: 1.0,
                scale: 1.0 / 255.0,
            },
        };
        
        let serialized = serde_json::to_string(&quantized).unwrap();
        let deserialized: QuantizedVectors = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(quantized.dimension, deserialized.dimension);
        assert_eq!(quantized.count, deserialized.count);
        assert_eq!(quantized.data, deserialized.data);
    }
    
    #[test]
    fn test_quality_metrics() {
        let metrics = QualityMetrics {
            map_score: 0.92,
            recall_at_k: 0.88,
            precision_at_k: 0.90,
            mse: 0.001,
            cosine_similarity: 0.95,
            compression_ratio: 4.0,
            timestamp: std::time::SystemTime::now(),
        };
        
        assert!(metrics.map_score > 0.9);
        assert!(metrics.compression_ratio >= 4.0);
    }
    
    #[test]
    fn test_optimization_config() {
        let config = OptimizationConfig {
            target_compression: 4.0,
            min_quality: 0.90,
            max_quality_loss: 0.05,
            auto_tune: true,
            algorithm: OptimizationAlgorithm::Bayesian,
        };
        
        assert_eq!(config.target_compression, 4.0);
        assert!(config.min_quality >= 0.90);
    }
}
