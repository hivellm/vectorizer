//! Quantization module for memory optimization
//! 
//! This module implements various quantization methods to reduce memory usage
//! while maintaining search quality. Based on benchmark results showing
//! 4x memory compression with improved quality using Scalar Quantization (SQ-8bit).

pub mod scalar;
pub mod traits;
pub mod storage;

// TODO: Implement these modules in future phases
// pub mod product;
// pub mod binary;
// pub mod metrics;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Enumeration of supported quantization methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuantizationType {
    /// Scalar Quantization - 8-bit, 4-bit, 2-bit
    Scalar(u8),
    /// Product Quantization
    Product,
    /// Binary Quantization (1-bit)
    Binary,
    /// No quantization (baseline)
    None,
}

impl fmt::Display for QuantizationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantizationType::Scalar(bits) => write!(f, "Scalar-{}bit", bits),
            QuantizationType::Product => write!(f, "Product"),
            QuantizationType::Binary => write!(f, "Binary"),
            QuantizationType::None => write!(f, "None"),
        }
    }
}

impl From<traits::QuantizationParams> for QuantizationType {
    fn from(params: traits::QuantizationParams) -> Self {
        match params {
            traits::QuantizationParams::Scalar { bits, .. } => QuantizationType::Scalar(bits),
            traits::QuantizationParams::Product { .. } => QuantizationType::Product,
            traits::QuantizationParams::Binary { .. } => QuantizationType::Binary,
        }
    }
}

/// Configuration for quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationConfig {
    /// Quantization method to use
    pub method: QuantizationType,
    /// Auto-optimize quantization parameters
    pub auto_optimize: bool,
    /// Minimum quality threshold (0.0 - 1.0)
    pub quality_threshold: f32,
    /// Enable quality monitoring
    pub monitor_quality: bool,
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self {
            method: QuantizationType::Scalar(8),
            auto_optimize: true,
            quality_threshold: 0.95,
            monitor_quality: true,
        }
    }
}

/// Statistics about quantization performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationStats {
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Compression ratio (original_size / compressed_size)
    pub compression_ratio: f64,
    /// Quality score (MAP)
    pub quality_score: f64,
    /// Search latency in milliseconds
    pub search_latency_ms: f64,
    /// Throughput in queries per second
    pub throughput_qps: f64,
    /// Number of vectors quantized
    pub vector_count: usize,
    /// Quantization method used
    pub method: QuantizationType,
}

impl QuantizationStats {
    /// Calculate memory savings percentage
    pub fn memory_savings_percent(&self) -> f64 {
        (1.0 - 1.0 / self.compression_ratio) * 100.0
    }
    
    /// Check if quality meets threshold
    pub fn meets_quality_threshold(&self, threshold: f32) -> bool {
        self.quality_score >= threshold as f64
    }
}

/// Error types for quantization operations
#[derive(Debug, thiserror::Error)]
pub enum QuantizationError {
    #[error("Invalid quantization parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Quality threshold not met: {actual:.3} < {threshold:.3}")]
    QualityThresholdNotMet { actual: f32, threshold: f32 },
    
    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    
    #[error("Quantization method not supported: {0}")]
    MethodNotSupported(String),
    
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Internal quantization error: {0}")]
    Internal(String),
}

/// Result type for quantization operations
pub type QuantizationResult<T> = Result<T, QuantizationError>;

/// Main quantization manager
pub struct QuantizationManager {
    config: QuantizationConfig,
    stats: QuantizationStats,
}

impl QuantizationManager {
    /// Create a new quantization manager
    pub fn new(config: QuantizationConfig) -> Self {
        Self {
            stats: QuantizationStats {
                memory_usage_mb: 0.0,
                compression_ratio: 1.0,
                quality_score: 1.0,
                search_latency_ms: 0.0,
                throughput_qps: 0.0,
                vector_count: 0,
                method: config.method.clone(),
            },
            config,
        }
    }
    
    /// Get current configuration
    pub fn config(&self) -> &QuantizationConfig {
        &self.config
    }
    
    /// Get current statistics
    pub fn stats(&self) -> &QuantizationStats {
        &self.stats
    }
    
    /// Update statistics
    pub fn update_stats(&mut self, stats: QuantizationStats) {
        self.stats = stats;
    }
    
    /// Check if current quantization meets quality requirements
    pub fn meets_quality_requirements(&self) -> bool {
        self.stats.meets_quality_threshold(self.config.quality_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quantization_type_display() {
        assert_eq!(format!("{}", QuantizationType::Scalar(8)), "Scalar-8bit");
        assert_eq!(format!("{}", QuantizationType::Product), "Product");
        assert_eq!(format!("{}", QuantizationType::Binary), "Binary");
        assert_eq!(format!("{}", QuantizationType::None), "None");
    }
    
    #[test]
    fn test_default_config() {
        let config = QuantizationConfig::default();
        assert_eq!(config.method, QuantizationType::Scalar(8));
        assert!(config.auto_optimize);
        assert_eq!(config.quality_threshold, 0.95);
        assert!(config.monitor_quality);
    }
    
    #[test]
    fn test_quantization_stats() {
        let stats = QuantizationStats {
            memory_usage_mb: 100.0,
            compression_ratio: 4.0,
            quality_score: 0.92,
            search_latency_ms: 2.5,
            throughput_qps: 15000.0,
            vector_count: 1000000,
            method: QuantizationType::Scalar(8),
        };
        
        assert_eq!(stats.memory_savings_percent(), 75.0);
        assert!(stats.meets_quality_threshold(0.90));
        assert!(!stats.meets_quality_threshold(0.95));
    }
    
    #[test]
    fn test_quantization_manager() {
        let config = QuantizationConfig::default();
        let manager = QuantizationManager::new(config);
        
        assert_eq!(manager.config().method, QuantizationType::Scalar(8));
        assert_eq!(manager.stats().vector_count, 0);
        assert!(manager.meets_quality_requirements()); // Default stats have quality_score = 1.0
    }
}

// Re-export main types
pub use scalar::ScalarQuantization;
pub use storage::{QuantizedVectorStorage, StorageConfig, StorageStats};
pub use traits::{
    QuantizationMethod, QuantizedVectors, QuantizationParams, 
    QualityMetrics, OptimizationConfig, QuantizedSearch
};
