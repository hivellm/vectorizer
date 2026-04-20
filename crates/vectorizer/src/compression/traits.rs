//! Compression traits and interfaces
//!
//! This module defines the core traits for compression and decompression operations.

use crate::compression::{CompressionError, CompressionResult};
use serde::{Deserialize, Serialize};

/// Trait for compression operations
pub trait Compressor {
    /// Compress data
    fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>>;
    
    /// Get compression level
    fn level(&self) -> u8;
    
    /// Get algorithm name
    fn algorithm(&self) -> &str;
    
    /// Estimate compressed size
    fn estimate_compressed_size(&self, original_size: usize) -> usize;
}

/// Trait for decompression operations
pub trait Decompressor {
    /// Decompress data
    fn decompress(&self, compressed_data: &[u8], original_size: Option<usize>) -> CompressionResult<Vec<u8>>;
    
    /// Get algorithm name
    fn algorithm(&self) -> &str;
}

/// Trait for compression methods that can both compress and decompress
pub trait CompressionMethod: Compressor + Decompressor {
    /// Get compression ratio for given data
    fn compression_ratio(&self, data: &[u8]) -> CompressionResult<f64> {
        let compressed = self.compress(data)?;
        Ok(data.len() as f64 / compressed.len() as f64)
    }
    
    /// Check if data should be compressed based on size and ratio
    fn should_compress(&self, data: &[u8], min_size: usize, min_ratio: f64) -> CompressionResult<bool> {
        if data.len() < min_size {
            return Ok(false);
        }
        
        let ratio = self.compression_ratio(data)?;
        Ok(ratio >= min_ratio)
    }
}

/// Configuration for compression methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMethodConfig {
    /// Compression level
    pub level: u8,
    /// Enable auto-level selection
    pub auto_level: bool,
    /// Minimum compression ratio threshold
    pub min_ratio_threshold: f64,
    /// Maximum compression time in microseconds
    pub max_compression_time_us: u64,
}

impl Default for CompressionMethodConfig {
    fn default() -> Self {
        Self {
            level: 3,
            auto_level: true,
            min_ratio_threshold: 1.1,
            max_compression_time_us: 1_000_000, // 1 second
        }
    }
}

/// Compression performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMetrics {
    /// Compression ratio achieved
    pub ratio: f64,
    /// Compression time in microseconds
    pub compression_time_us: u64,
    /// Decompression time in microseconds
    pub decompression_time_us: u64,
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Algorithm used
    pub algorithm: String,
}

impl CompressionMetrics {
    /// Calculate space savings percentage
    pub fn space_savings_percent(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            (1.0 - self.compressed_size as f64 / self.original_size as f64) * 100.0
        }
    }
    
    /// Calculate compression throughput in MB/s
    pub fn compression_throughput_mbps(&self) -> f64 {
        if self.compression_time_us == 0 {
            0.0
        } else {
            (self.original_size as f64 / 1_000_000.0) / (self.compression_time_us as f64 / 1_000_000.0)
        }
    }
    
    /// Calculate decompression throughput in MB/s
    pub fn decompression_throughput_mbps(&self) -> f64 {
        if self.decompression_time_us == 0 {
            0.0
        } else {
            (self.original_size as f64 / 1_000_000.0) / (self.decompression_time_us as f64 / 1_000_000.0)
        }
    }
}

/// Compression benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionBenchmark {
    /// Algorithm name
    pub algorithm: String,
    /// Compression level
    pub level: u8,
    /// Average compression ratio
    pub avg_ratio: f64,
    /// Average compression time in microseconds
    pub avg_compression_time_us: u64,
    /// Average decompression time in microseconds
    pub avg_decompression_time_us: u64,
    /// Compression throughput in MB/s
    pub compression_throughput_mbps: f64,
    /// Decompression throughput in MB/s
    pub decompression_throughput_mbps: f64,
    /// Number of test samples
    pub sample_count: usize,
}

impl CompressionBenchmark {
    /// Calculate efficiency score (ratio / time)
    pub fn efficiency_score(&self) -> f64 {
        if self.avg_compression_time_us == 0 {
            0.0
        } else {
            self.avg_ratio / (self.avg_compression_time_us as f64 / 1_000_000.0)
        }
    }
    
    /// Compare with another benchmark
    pub fn compare(&self, other: &CompressionBenchmark) -> CompressionComparison {
        CompressionComparison {
            ratio_improvement: self.avg_ratio / other.avg_ratio,
            compression_speed_improvement: other.avg_compression_time_us as f64 / self.avg_compression_time_us as f64,
            decompression_speed_improvement: other.avg_decompression_time_us as f64 / self.avg_decompression_time_us as f64,
            efficiency_improvement: self.efficiency_score() / other.efficiency_score(),
        }
    }
}

/// Comparison between two compression benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionComparison {
    /// Ratio improvement factor
    pub ratio_improvement: f64,
    /// Compression speed improvement factor
    pub compression_speed_improvement: f64,
    /// Decompression speed improvement factor
    pub decompression_speed_improvement: f64,
    /// Efficiency improvement factor
    pub efficiency_improvement: f64,
}

/// Compression strategy for different data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// Fast compression for real-time applications
    Fast,
    /// Balanced compression for general use
    Balanced,
    /// Maximum compression for storage optimization
    Maximum,
    /// Custom strategy with specific parameters
    Custom(CompressionMethodConfig),
}

impl Default for CompressionStrategy {
    fn default() -> Self {
        Self::Balanced
    }
}

impl CompressionStrategy {
    /// Get configuration for this strategy
    pub fn config(&self) -> CompressionMethodConfig {
        match self {
            CompressionStrategy::Fast => CompressionMethodConfig {
                level: 1,
                auto_level: false,
                min_ratio_threshold: 1.05,
                max_compression_time_us: 100_000, // 100ms
            },
            CompressionStrategy::Balanced => CompressionMethodConfig::default(),
            CompressionStrategy::Maximum => CompressionMethodConfig {
                level: 22,
                auto_level: false,
                min_ratio_threshold: 1.2,
                max_compression_time_us: 10_000_000, // 10 seconds
            },
            CompressionStrategy::Custom(config) => config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compression_metrics() {
        let metrics = CompressionMetrics {
            ratio: 2.0,
            compression_time_us: 1000,
            decompression_time_us: 500,
            original_size: 1000,
            compressed_size: 500,
            algorithm: "zstd".to_string(),
        };
        
        assert_eq!(metrics.space_savings_percent(), 50.0);
        assert!(metrics.compression_throughput_mbps() > 0.0);
        assert!(metrics.decompression_throughput_mbps() > 0.0);
    }
    
    #[test]
    fn test_compression_benchmark() {
        let benchmark = CompressionBenchmark {
            algorithm: "zstd".to_string(),
            level: 3,
            avg_ratio: 2.0,
            avg_compression_time_us: 1000,
            avg_decompression_time_us: 500,
            compression_throughput_mbps: 100.0,
            decompression_throughput_mbps: 200.0,
            sample_count: 100,
        };
        
        assert!(benchmark.efficiency_score() > 0.0);
    }
    
    #[test]
    fn test_compression_strategy() {
        let fast_config = CompressionStrategy::Fast.config();
        assert_eq!(fast_config.level, 1);
        assert!(!fast_config.auto_level);
        
        let balanced_config = CompressionStrategy::Balanced.config();
        assert_eq!(balanced_config.level, 3);
        assert!(balanced_config.auto_level);
        
        let max_config = CompressionStrategy::Maximum.config();
        assert_eq!(max_config.level, 22);
        assert!(!max_config.auto_level);
    }
}
