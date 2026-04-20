//! Compression configuration module
//!
//! This module provides configuration management for compression operations.

use crate::compression::{CompressionAlgorithm, CompressionConfig};
use serde::{Deserialize, Serialize};

/// Compression configuration builder
pub struct CompressionConfigBuilder {
    config: CompressionConfig,
}

impl CompressionConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: CompressionConfig::default(),
        }
    }
    
    /// Set the compression algorithm
    pub fn algorithm(mut self, algorithm: CompressionAlgorithm) -> Self {
        self.config.algorithm = algorithm;
        self
    }
    
    /// Set the compression level
    pub fn level(mut self, level: u8) -> Self {
        self.config.level = level;
        self
    }
    
    /// Enable or disable auto-selection
    pub fn auto_select(mut self, auto_select: bool) -> Self {
        self.config.auto_select = auto_select;
        self
    }
    
    /// Set the minimum size threshold
    pub fn min_size_threshold(mut self, threshold: usize) -> Self {
        self.config.min_size_threshold = threshold;
        self
    }
    
    /// Enable or disable statistics
    pub fn enable_stats(mut self, enable: bool) -> Self {
        self.config.enable_stats = enable;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> CompressionConfig {
        self.config
    }
}

impl Default for CompressionConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Compression preset configurations
pub struct CompressionPresets;

impl CompressionPresets {
    /// Fast compression preset (speed over ratio)
    pub fn fast() -> CompressionConfig {
        CompressionConfigBuilder::new()
            .algorithm(CompressionAlgorithm::Lz4)
            .level(1)
            .auto_select(false)
            .min_size_threshold(512)
            .enable_stats(true)
            .build()
    }
    
    /// Balanced compression preset (good balance)
    pub fn balanced() -> CompressionConfig {
        CompressionConfigBuilder::new()
            .algorithm(CompressionAlgorithm::Zstd)
            .level(3)
            .auto_select(true)
            .min_size_threshold(1024)
            .enable_stats(true)
            .build()
    }
    
    /// High compression preset (ratio over speed)
    pub fn high_compression() -> CompressionConfig {
        CompressionConfigBuilder::new()
            .algorithm(CompressionAlgorithm::Zstd)
            .level(22)
            .auto_select(false)
            .min_size_threshold(2048)
            .enable_stats(true)
            .build()
    }
    
    /// Storage optimization preset (maximum compression)
    pub fn storage_optimized() -> CompressionConfig {
        CompressionConfigBuilder::new()
            .algorithm(CompressionAlgorithm::Zstd)
            .level(22)
            .auto_select(false)
            .min_size_threshold(1024)
            .enable_stats(true)
            .build()
    }
    
    /// Real-time preset (minimum latency)
    pub fn real_time() -> CompressionConfig {
        CompressionConfigBuilder::new()
            .algorithm(CompressionAlgorithm::Lz4)
            .level(1)
            .auto_select(false)
            .min_size_threshold(256)
            .enable_stats(false)
            .build()
    }
    
    /// No compression preset
    pub fn none() -> CompressionConfig {
        CompressionConfigBuilder::new()
            .algorithm(CompressionAlgorithm::None)
            .level(0)
            .auto_select(false)
            .min_size_threshold(usize::MAX)
            .enable_stats(false)
            .build()
    }
}

/// Compression configuration validator
pub struct CompressionConfigValidator;

impl CompressionConfigValidator {
    /// Validate a compression configuration
    pub fn validate(config: &CompressionConfig) -> Result<(), String> {
        // Validate compression level
        match config.algorithm {
            CompressionAlgorithm::Lz4 => {
                if config.level > 9 {
                    return Err(format!("LZ4 compression level must be 1-9, got {}", config.level));
                }
            }
            CompressionAlgorithm::Zstd => {
                if config.level > 22 {
                    return Err(format!("Zstd compression level must be 1-22, got {}", config.level));
                }
            }
            CompressionAlgorithm::None => {
                if config.level != 0 {
                    return Err("No compression algorithm should have level 0".to_string());
                }
            }
        }
        
        // Validate minimum size threshold
        if config.min_size_threshold == 0 {
            return Err("Minimum size threshold must be greater than 0".to_string());
        }
        
        Ok(())
    }
    
    /// Get recommended configuration for data characteristics
    pub fn recommend_for_data(data_size: usize, data_type: DataType) -> CompressionConfig {
        match data_type {
            DataType::Text => {
                if data_size < 1024 {
                    CompressionPresets::fast()
                } else if data_size < 1024 * 1024 {
                    CompressionPresets::balanced()
                } else {
                    CompressionPresets::high_compression()
                }
            }
            DataType::Binary => {
                if data_size < 2048 {
                    CompressionPresets::fast()
                } else {
                    CompressionPresets::balanced()
                }
            }
            DataType::Vector => {
                if data_size < 512 {
                    CompressionPresets::fast()
                } else if data_size < 1024 * 1024 {
                    CompressionPresets::balanced()
                } else {
                    CompressionPresets::storage_optimized()
                }
            }
            DataType::Mixed => CompressionPresets::balanced(),
        }
    }
}

/// Data type classification for compression recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    /// Text data (highly compressible)
    Text,
    /// Binary data (moderately compressible)
    Binary,
    /// Vector data (compression depends on structure)
    Vector,
    /// Mixed data types
    Mixed,
}

impl DataType {
    /// Detect data type from sample data
    pub fn detect(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::Mixed;
        }
        
        let text_chars = data.iter()
            .filter(|&&b| b.is_ascii_alphanumeric() || b.is_ascii_whitespace() || b.is_ascii_punctuation())
            .count();
        
        let text_ratio = text_chars as f64 / data.len() as f64;
        
        if text_ratio > 0.8 {
            Self::Text
        } else if text_ratio > 0.5 {
            Self::Mixed
        } else {
            Self::Binary
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_builder() {
        let config = CompressionConfigBuilder::new()
            .algorithm(CompressionAlgorithm::Zstd)
            .level(5)
            .auto_select(true)
            .min_size_threshold(2048)
            .enable_stats(false)
            .build();
        
        assert_eq!(config.algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(config.level, 5);
        assert!(config.auto_select);
        assert_eq!(config.min_size_threshold, 2048);
        assert!(!config.enable_stats);
    }
    
    #[test]
    fn test_presets() {
        let fast = CompressionPresets::fast();
        assert_eq!(fast.algorithm, CompressionAlgorithm::Lz4);
        assert_eq!(fast.level, 1);
        
        let balanced = CompressionPresets::balanced();
        assert_eq!(balanced.algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(balanced.level, 3);
        
        let high = CompressionPresets::high_compression();
        assert_eq!(high.algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(high.level, 22);
    }
    
    #[test]
    fn test_config_validation() {
        let valid_config = CompressionConfig::default();
        assert!(CompressionConfigValidator::validate(&valid_config).is_ok());
        
        let invalid_config = CompressionConfig {
            algorithm: CompressionAlgorithm::Lz4,
            level: 15, // Invalid for LZ4
            auto_select: true,
            min_size_threshold: 1024,
            enable_stats: true,
        };
        assert!(CompressionConfigValidator::validate(&invalid_config).is_err());
    }
    
    #[test]
    fn test_data_type_detection() {
        let text_data = b"Hello, world! This is text data.";
        assert_eq!(DataType::detect(text_data), DataType::Text);
        
        let binary_data = &[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD];
        assert_eq!(DataType::detect(binary_data), DataType::Binary);
        
        let empty_data = b"";
        assert_eq!(DataType::detect(empty_data), DataType::Mixed);
    }
    
    #[test]
    fn test_recommendations() {
        let small_text = vec![0u8; 512];
        let config = CompressionConfigValidator::recommend_for_data(small_text.len(), DataType::Text);
        assert_eq!(config.algorithm, CompressionAlgorithm::Lz4);
        
        let large_text = vec![0u8; 2 * 1024 * 1024];
        let config = CompressionConfigValidator::recommend_for_data(large_text.len(), DataType::Text);
        assert_eq!(config.algorithm, CompressionAlgorithm::Zstd);
        assert_eq!(config.level, 22);
    }
}
