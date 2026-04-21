//! Configuration management for embedding providers
//!
//! This module provides configuration management for embedding operations,
//! including provider selection, model configuration, and performance tuning.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::embedding::{EmbeddingConfig, EmbeddingProviderType};

/// Embedding configuration builder
pub struct EmbeddingConfigBuilder {
    config: EmbeddingConfig,
}

impl EmbeddingConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: EmbeddingConfig::default(),
        }
    }

    /// Set the provider type
    pub fn provider(mut self, provider: EmbeddingProviderType) -> Self {
        self.config.provider = provider;
        self
    }

    /// Set the model name
    pub fn model(mut self, model: String) -> Self {
        self.config.model = model;
        self
    }

    /// Set the embedding dimension
    pub fn dimension(mut self, dimension: usize) -> Self {
        self.config.dimension = dimension;
        self
    }

    /// Set the maximum text length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.config.max_length = max_length;
        self
    }

    /// Enable or disable caching
    pub fn enable_caching(mut self, enable: bool) -> Self {
        self.config.enable_caching = enable;
        self
    }

    /// Set the cache size
    pub fn cache_size(mut self, size: usize) -> Self {
        self.config.cache_size = size;
        self
    }

    /// Set the batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Set the timeout
    pub fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.timeout_seconds = timeout;
        self
    }

    /// Build the configuration
    pub fn build(self) -> EmbeddingConfig {
        self.config
    }
}

impl Default for EmbeddingConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Embedding presets for common use cases
pub struct EmbeddingPresets;

impl EmbeddingPresets {
    /// Fast embeddings preset (speed over quality)
    pub fn fast() -> EmbeddingConfig {
        EmbeddingConfigBuilder::new()
            .provider(EmbeddingProviderType::BM25)
            .model("default".to_string())
            .dimension(512)
            .max_length(1024)
            .enable_caching(true)
            .cache_size(5000)
            .batch_size(64)
            .timeout_seconds(10)
            .build()
    }

    /// High-quality embeddings preset
    pub fn high_quality() -> EmbeddingConfig {
        EmbeddingConfigBuilder::new()
            .provider(EmbeddingProviderType::OpenAI)
            .model("text-embedding-ada-002".to_string())
            .dimension(1536)
            .max_length(8191)
            .enable_caching(true)
            .cache_size(10000)
            .batch_size(32)
            .timeout_seconds(60)
            .build()
    }

    /// Balanced embeddings preset
    pub fn balanced() -> EmbeddingConfig {
        EmbeddingConfigBuilder::new()
            .provider(EmbeddingProviderType::BERT)
            .model("bert-base-uncased".to_string())
            .dimension(768)
            .max_length(512)
            .enable_caching(true)
            .cache_size(8000)
            .batch_size(32)
            .timeout_seconds(30)
            .build()
    }

    /// Memory-efficient embeddings preset
    pub fn memory_efficient() -> EmbeddingConfig {
        EmbeddingConfigBuilder::new()
            .provider(EmbeddingProviderType::BM25)
            .model("default".to_string())
            .dimension(256)
            .max_length(512)
            .enable_caching(true)
            .cache_size(2000)
            .batch_size(16)
            .timeout_seconds(15)
            .build()
    }

    /// Real-time embeddings preset
    pub fn real_time() -> EmbeddingConfig {
        EmbeddingConfigBuilder::new()
            .provider(EmbeddingProviderType::BM25)
            .model("default".to_string())
            .dimension(128)
            .max_length(256)
            .enable_caching(true)
            .cache_size(1000)
            .batch_size(8)
            .timeout_seconds(5)
            .build()
    }
}

/// Configuration validator for embedding settings
pub struct EmbeddingConfigValidator;

impl EmbeddingConfigValidator {
    /// Validate an embedding configuration
    pub fn validate(config: &EmbeddingConfig) -> Result<(), ValidationError> {
        // Validate provider
        match config.provider {
            EmbeddingProviderType::BM25 => {
                // BM25 specific validation
                if config.dimension == 0 {
                    return Err(ValidationError::InvalidValue(
                        "BM25 dimension must be greater than 0".to_string(),
                    ));
                }
            }
            EmbeddingProviderType::BERT => {
                // BERT specific validation
                if config.dimension != 768 && config.dimension != 1024 {
                    return Err(ValidationError::InvalidValue(
                        "BERT dimension must be 768 or 1024".to_string(),
                    ));
                }
                if config.max_length > 512 {
                    return Err(ValidationError::InvalidValue(
                        "BERT max_length cannot exceed 512".to_string(),
                    ));
                }
            }
            EmbeddingProviderType::OpenAI => {
                // OpenAI specific validation
                if config.dimension != 1536 && config.dimension != 3072 {
                    return Err(ValidationError::InvalidValue(
                        "OpenAI dimension must be 1536 or 3072".to_string(),
                    ));
                }
                if config.max_length > 8191 {
                    return Err(ValidationError::InvalidValue(
                        "OpenAI max_length cannot exceed 8191".to_string(),
                    ));
                }
            }
            _ => {
                // Generic validation
            }
        }

        // Validate dimension
        if config.dimension == 0 {
            return Err(ValidationError::InvalidValue(
                "Dimension must be greater than 0".to_string(),
            ));
        }

        // Validate max_length
        if config.max_length == 0 {
            return Err(ValidationError::InvalidValue(
                "Max length must be greater than 0".to_string(),
            ));
        }

        // Validate cache_size
        if config.enable_caching && config.cache_size == 0 {
            return Err(ValidationError::InvalidValue(
                "Cache size must be greater than 0 when caching is enabled".to_string(),
            ));
        }

        // Validate batch_size
        if config.batch_size == 0 {
            return Err(ValidationError::InvalidValue(
                "Batch size must be greater than 0".to_string(),
            ));
        }

        // Validate timeout
        if config.timeout_seconds == 0 {
            return Err(ValidationError::InvalidValue(
                "Timeout must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Optimize configuration for performance
    pub fn optimize_for_performance(config: &mut EmbeddingConfig) {
        // Optimize batch size based on provider
        match config.provider {
            EmbeddingProviderType::BM25 => {
                config.batch_size = config.batch_size.min(100);
            }
            EmbeddingProviderType::BERT => {
                config.batch_size = config.batch_size.min(32);
            }
            EmbeddingProviderType::OpenAI => {
                config.batch_size = config.batch_size.min(100);
            }
            _ => {}
        }

        // Optimize cache size based on available memory
        if config.cache_size > 50000 {
            config.cache_size = 50000;
        }

        // Optimize timeout based on provider
        match config.provider {
            EmbeddingProviderType::BM25 => {
                config.timeout_seconds = config.timeout_seconds.min(30);
            }
            EmbeddingProviderType::BERT => {
                config.timeout_seconds = config.timeout_seconds.min(60);
            }
            EmbeddingProviderType::OpenAI => {
                config.timeout_seconds = config.timeout_seconds.min(120);
            }
            _ => {}
        }
    }

    /// Get recommended configuration for use case
    pub fn recommend_for_use_case(use_case: UseCase) -> EmbeddingConfig {
        match use_case {
            UseCase::Search => EmbeddingPresets::balanced(),
            UseCase::Similarity => EmbeddingPresets::high_quality(),
            UseCase::Classification => EmbeddingPresets::balanced(),
            UseCase::Clustering => EmbeddingPresets::high_quality(),
            UseCase::RealTime => EmbeddingPresets::real_time(),
            UseCase::BatchProcessing => EmbeddingPresets::fast(),
            UseCase::MemoryConstrained => EmbeddingPresets::memory_efficient(),
        }
    }
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// Invalid configuration value
    InvalidValue(String),
    /// Configuration conflict
    Conflict(String),
    /// Missing required configuration
    Missing(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ValidationError::Conflict(msg) => write!(f, "Configuration conflict: {}", msg),
            ValidationError::Missing(msg) => write!(f, "Missing configuration: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Use cases for embedding recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseCase {
    /// Search applications
    Search,
    /// Similarity matching
    Similarity,
    /// Text classification
    Classification,
    /// Document clustering
    Clustering,
    /// Real-time applications
    RealTime,
    /// Batch processing
    BatchProcessing,
    /// Memory-constrained environments
    MemoryConstrained,
}

/// Performance tuning recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendations {
    /// Recommended configuration
    pub recommended_config: EmbeddingConfig,
    /// Expected performance metrics
    pub expected_metrics: ExpectedMetrics,
    /// Tuning suggestions
    pub suggestions: Vec<String>,
}

/// Expected performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedMetrics {
    /// Expected processing rate (embeddings per second)
    pub processing_rate: f64,
    /// Expected memory usage (MB)
    pub memory_usage_mb: f64,
    /// Expected latency (milliseconds)
    pub latency_ms: f64,
    /// Expected accuracy score
    pub accuracy_score: f64,
}

/// Configuration comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigComparison {
    /// Configuration being compared
    pub config: EmbeddingConfig,
    /// Performance score (0.0 to 1.0)
    pub performance_score: f64,
    /// Memory efficiency score (0.0 to 1.0)
    pub memory_efficiency: f64,
    /// Quality score (0.0 to 1.0)
    pub quality_score: f64,
    /// Cost score (0.0 to 1.0, higher is better)
    pub cost_score: f64,
}

impl ConfigComparison {
    /// Calculate overall score
    pub fn overall_score(&self) -> f64 {
        (self.performance_score + self.memory_efficiency + self.quality_score + self.cost_score)
            / 4.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = EmbeddingConfigBuilder::new()
            .provider(EmbeddingProviderType::BERT)
            .model("bert-base-uncased".to_string())
            .dimension(768)
            .max_length(512)
            .enable_caching(true)
            .cache_size(5000)
            .batch_size(32)
            .timeout_seconds(30)
            .build();

        assert_eq!(config.provider, EmbeddingProviderType::BERT);
        assert_eq!(config.model, "bert-base-uncased");
        assert_eq!(config.dimension, 768);
        assert_eq!(config.max_length, 512);
        assert!(config.enable_caching);
        assert_eq!(config.cache_size, 5000);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_presets() {
        let fast = EmbeddingPresets::fast();
        assert_eq!(fast.provider, EmbeddingProviderType::BM25);
        assert_eq!(fast.dimension, 512);
        assert!(fast.enable_caching);

        let high_quality = EmbeddingPresets::high_quality();
        assert_eq!(high_quality.provider, EmbeddingProviderType::OpenAI);
        assert_eq!(high_quality.dimension, 1536);
        assert!(high_quality.enable_caching);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = EmbeddingConfig::default();
        assert!(EmbeddingConfigValidator::validate(&valid_config).is_ok());

        let mut invalid_config = EmbeddingConfig::default();
        invalid_config.dimension = 0;
        assert!(EmbeddingConfigValidator::validate(&invalid_config).is_err());
    }

    #[test]
    fn test_use_case_recommendations() {
        let search_config = EmbeddingConfigValidator::recommend_for_use_case(UseCase::Search);
        assert_eq!(search_config.provider, EmbeddingProviderType::BERT);

        let real_time_config = EmbeddingConfigValidator::recommend_for_use_case(UseCase::RealTime);
        assert_eq!(real_time_config.provider, EmbeddingProviderType::BM25);
    }

    #[test]
    fn test_config_comparison() {
        let comparison = ConfigComparison {
            config: EmbeddingConfig::default(),
            performance_score: 0.8,
            memory_efficiency: 0.7,
            quality_score: 0.9,
            cost_score: 0.6,
        };

        assert_eq!(comparison.overall_score(), 0.75);
    }
}
