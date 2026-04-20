//! Compression module for vector data
//!
//! This module provides compression capabilities for vector data to reduce
//! storage requirements and improve I/O performance. Supports LZ4 and Zstd
//! compression algorithms with automatic selection based on data characteristics.

pub mod lz4;
pub mod zstd;
pub mod traits;
pub mod config;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompressionAlgorithm {
    /// LZ4 fast compression
    Lz4,
    /// Zstandard compression
    Zstd,
    /// No compression
    None,
}

impl fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompressionAlgorithm::Lz4 => write!(f, "LZ4"),
            CompressionAlgorithm::Zstd => write!(f, "Zstd"),
            CompressionAlgorithm::None => write!(f, "None"),
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression algorithm to use
    pub algorithm: CompressionAlgorithm,
    /// Compression level (1-22 for Zstd, 1-9 for LZ4)
    pub level: u8,
    /// Enable auto-selection based on data characteristics
    pub auto_select: bool,
    /// Minimum size threshold for compression (bytes)
    pub min_size_threshold: usize,
    /// Enable compression statistics
    pub enable_stats: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            level: 3,
            auto_select: true,
            min_size_threshold: 1024, // 1KB
            enable_stats: true,
        }
    }
}

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Total bytes compressed
    pub bytes_compressed: u64,
    /// Total bytes before compression
    pub bytes_original: u64,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Number of compression operations
    pub operations_count: u64,
    /// Average compression time in microseconds
    pub avg_compression_time_us: u64,
    /// Average decompression time in microseconds
    pub avg_decompression_time_us: u64,
    /// Algorithm used
    pub algorithm: CompressionAlgorithm,
}

impl CompressionStats {
    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.bytes_original == 0 {
            1.0
        } else {
            self.bytes_original as f64 / self.bytes_compressed as f64
        }
    }
    
    /// Calculate space savings percentage
    pub fn space_savings_percent(&self) -> f64 {
        if self.bytes_original == 0 {
            0.0
        } else {
            (1.0 - self.bytes_compressed as f64 / self.bytes_original as f64) * 100.0
        }
    }
    
    /// Get average compression throughput in MB/s
    pub fn compression_throughput_mbps(&self) -> f64 {
        if self.avg_compression_time_us == 0 {
            0.0
        } else {
            (self.bytes_original as f64 / 1_000_000.0) / (self.avg_compression_time_us as f64 / 1_000_000.0)
        }
    }
}

/// Error types for compression operations
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    
    #[error("Invalid compression level: {level} for algorithm {algorithm}")]
    InvalidLevel { level: u8, algorithm: CompressionAlgorithm },
    
    #[error("Data too small for compression: {size} < {threshold}")]
    DataTooSmall { size: usize, threshold: usize },
    
    #[error("Compression ratio too low: {ratio:.2} < {threshold:.2}")]
    CompressionRatioTooLow { ratio: f64, threshold: f64 },
    
    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}

/// Result type for compression operations
pub type CompressionResult<T> = Result<T, CompressionError>;

/// Main compression manager
pub struct CompressionManager {
    config: CompressionConfig,
    stats: CompressionStats,
}

impl CompressionManager {
    /// Create a new compression manager
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            stats: CompressionStats {
                bytes_compressed: 0,
                bytes_original: 0,
                compression_ratio: 1.0,
                operations_count: 0,
                avg_compression_time_us: 0,
                avg_decompression_time_us: 0,
                algorithm: config.algorithm.clone(),
            },
            config,
        }
    }
    
    /// Get current configuration
    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }
    
    /// Get current statistics
    pub fn stats(&self) -> &CompressionStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = CompressionStats {
            bytes_compressed: 0,
            bytes_original: 0,
            compression_ratio: 1.0,
            operations_count: 0,
            avg_compression_time_us: 0,
            avg_decompression_time_us: 0,
            algorithm: self.config.algorithm.clone(),
        };
    }
    
    /// Update statistics after compression operation
    pub fn update_stats(&mut self, original_size: usize, compressed_size: usize, compression_time_us: u64, decompression_time_us: u64) {
        self.stats.bytes_original += original_size as u64;
        self.stats.bytes_compressed += compressed_size as u64;
        self.stats.operations_count += 1;
        
        // Update average times using exponential moving average
        let alpha = 0.1; // Smoothing factor
        if self.stats.operations_count == 1 {
            self.stats.avg_compression_time_us = compression_time_us;
            self.stats.avg_decompression_time_us = decompression_time_us;
        } else {
            self.stats.avg_compression_time_us = ((1.0 - alpha) * self.stats.avg_compression_time_us as f64 + alpha * compression_time_us as f64) as u64;
            self.stats.avg_decompression_time_us = ((1.0 - alpha) * self.stats.avg_decompression_time_us as f64 + alpha * decompression_time_us as f64) as u64;
        }
        
        self.stats.compression_ratio = self.stats.compression_ratio();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compression_algorithm_display() {
        assert_eq!(format!("{}", CompressionAlgorithm::Lz4), "LZ4");
        assert_eq!(format!("{}", CompressionAlgorithm::Zstd), "Zstd");
        assert_eq!(format!("{}", CompressionAlgorithm::None), "None");
    }
    
    #[test]
    fn test_default_config() {
        let config = CompressionConfig::default();
        assert_eq!(config.algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(config.level, 3);
        assert!(config.auto_select);
        assert_eq!(config.min_size_threshold, 1024);
        assert!(config.enable_stats);
    }
    
    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats {
            bytes_compressed: 500,
            bytes_original: 1000,
            compression_ratio: 2.0,
            operations_count: 10,
            avg_compression_time_us: 1000,
            avg_decompression_time_us: 500,
            algorithm: CompressionAlgorithm::Zstd,
        };
        
        assert_eq!(stats.compression_ratio(), 2.0);
        assert_eq!(stats.space_savings_percent(), 50.0);
        assert!(stats.compression_throughput_mbps() > 0.0);
    }
    
    #[test]
    fn test_compression_manager() {
        let config = CompressionConfig::default();
        let manager = CompressionManager::new(config);
        
        assert_eq!(manager.config().algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(manager.stats().operations_count, 0);
        
        // Test stats update
        let mut manager = manager;
        manager.update_stats(1000, 500, 1000, 500);
        assert_eq!(manager.stats().operations_count, 1);
        assert_eq!(manager.stats().bytes_original, 1000);
        assert_eq!(manager.stats().bytes_compressed, 500);
    }
}

// Re-export main types
pub use traits::{Compressor, Decompressor, CompressionMethod, CompressionStrategy, CompressionMetrics};
pub use lz4::Lz4Compressor;
pub use zstd::ZstdCompressor;
pub use config::{CompressionConfigBuilder, CompressionPresets, CompressionConfigValidator, DataType};
