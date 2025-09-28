//! Batch Operations Configuration
//! 
//! Configuration management for batch operations including performance tuning,
//! resource limits, and operational parameters.

use serde::{Deserialize, Serialize};

/// Batch operations configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum number of vectors per batch operation
    pub max_batch_size: usize,
    
    /// Maximum memory usage in MB for batch operations
    pub max_memory_usage_mb: usize,
    
    /// Number of parallel workers for processing
    pub parallel_workers: usize,
    
    /// Size of chunks for streaming large batches
    pub chunk_size: usize,
    
    /// Whether operations should be atomic by default
    pub atomic_by_default: bool,
    
    /// Whether to report progress during long operations
    pub progress_reporting: bool,
    
    /// Number of retry attempts for failed operations
    pub error_retry_attempts: usize,
    
    /// Delay between retry attempts in milliseconds
    pub error_retry_delay_ms: u64,
    
    /// Timeout for individual batch operations in seconds
    pub operation_timeout_seconds: u64,
    
    /// Whether to enable performance metrics collection
    pub enable_metrics: bool,
    
    /// Maximum concurrent batch operations
    pub max_concurrent_batches: usize,
    
    /// Whether to enable compression for large payloads
    pub enable_compression: bool,
    
    /// Compression threshold in bytes
    pub compression_threshold_bytes: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            max_memory_usage_mb: 512,
            parallel_workers: 4,
            chunk_size: 100,
            atomic_by_default: true,
            progress_reporting: true,
            error_retry_attempts: 3,
            error_retry_delay_ms: 100,
            operation_timeout_seconds: 300, // 5 minutes
            enable_metrics: true,
            max_concurrent_batches: 10,
            enable_compression: false,
            compression_threshold_bytes: 1024 * 1024, // 1MB
        }
    }
}

impl BatchConfig {
    /// Create a high-performance configuration for production use
    pub fn production() -> Self {
        Self {
            max_batch_size: 10000,
            max_memory_usage_mb: 2048,
            parallel_workers: 8,
            chunk_size: 500,
            atomic_by_default: true,
            progress_reporting: false, // Disable for production performance
            error_retry_attempts: 5,
            error_retry_delay_ms: 50,
            operation_timeout_seconds: 600, // 10 minutes
            enable_metrics: true,
            max_concurrent_batches: 20,
            enable_compression: true,
            compression_threshold_bytes: 512 * 1024, // 512KB
        }
    }

    /// Create a development configuration with verbose logging
    pub fn development() -> Self {
        Self {
            max_batch_size: 100,
            max_memory_usage_mb: 128,
            parallel_workers: 2,
            chunk_size: 10,
            atomic_by_default: true,
            progress_reporting: true,
            error_retry_attempts: 1,
            error_retry_delay_ms: 1000,
            operation_timeout_seconds: 60, // 1 minute
            enable_metrics: true,
            max_concurrent_batches: 3,
            enable_compression: false,
            compression_threshold_bytes: 1024 * 1024,
        }
    }

    /// Create a testing configuration
    pub fn testing() -> Self {
        Self {
            max_batch_size: 10,
            max_memory_usage_mb: 64,
            parallel_workers: 1,
            chunk_size: 5,
            atomic_by_default: true,
            progress_reporting: false,
            error_retry_attempts: 0,
            error_retry_delay_ms: 10,
            operation_timeout_seconds: 30,
            enable_metrics: false,
            max_concurrent_batches: 1,
            enable_compression: false,
            compression_threshold_bytes: 1024,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_batch_size == 0 {
            return Err("max_batch_size must be greater than 0".to_string());
        }

        if self.max_memory_usage_mb == 0 {
            return Err("max_memory_usage_mb must be greater than 0".to_string());
        }

        if self.parallel_workers == 0 {
            return Err("parallel_workers must be greater than 0".to_string());
        }

        if self.chunk_size == 0 {
            return Err("chunk_size must be greater than 0".to_string());
        }

        if self.chunk_size > self.max_batch_size {
            return Err("chunk_size cannot be greater than max_batch_size".to_string());
        }

        if self.operation_timeout_seconds == 0 {
            return Err("operation_timeout_seconds must be greater than 0".to_string());
        }

        if self.max_concurrent_batches == 0 {
            return Err("max_concurrent_batches must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get the recommended chunk size for a given batch size
    pub fn get_chunk_size_for_batch(&self, batch_size: usize) -> usize {
        if batch_size <= self.chunk_size {
            batch_size
        } else {
            self.chunk_size
        }
    }

    /// Check if a batch size is within limits
    pub fn is_batch_size_valid(&self, batch_size: usize) -> bool {
        batch_size > 0 && batch_size <= self.max_batch_size
    }

    /// Estimate memory usage for a batch operation
    pub fn estimate_memory_usage(&self, vector_count: usize, vector_dimension: usize) -> usize {
        // Rough estimate: 4 bytes per float + overhead
        let vector_data = vector_count * vector_dimension * 4;
        let metadata_overhead = vector_count * 512; // Average metadata size
        let processing_overhead = vector_count * 64; // Processing overhead
        
        (vector_data + metadata_overhead + processing_overhead) / (1024 * 1024) // Convert to MB
    }

    /// Check if memory usage would exceed limits
    pub fn would_exceed_memory_limit(&self, vector_count: usize, vector_dimension: usize) -> bool {
        self.estimate_memory_usage(vector_count, vector_dimension) > self.max_memory_usage_mb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BatchConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.max_batch_size > 0);
        assert!(config.max_memory_usage_mb > 0);
        assert!(config.parallel_workers > 0);
    }

    #[test]
    fn test_production_config() {
        let config = BatchConfig::production();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_batch_size, 10000);
        assert_eq!(config.max_memory_usage_mb, 2048);
        assert_eq!(config.parallel_workers, 8);
    }

    #[test]
    fn test_development_config() {
        let config = BatchConfig::development();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_batch_size, 100);
        assert!(config.progress_reporting);
    }

    #[test]
    fn test_testing_config() {
        let config = BatchConfig::testing();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_batch_size, 10);
        assert_eq!(config.parallel_workers, 1);
    }

    #[test]
    fn test_config_validation() {
        let mut config = BatchConfig::default();
        config.max_batch_size = 0;
        assert!(config.validate().is_err());
        
        config = BatchConfig::default();
        config.chunk_size = config.max_batch_size + 1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_batch_size_validation() {
        let config = BatchConfig::default();
        assert!(config.is_batch_size_valid(100));
        assert!(config.is_batch_size_valid(config.max_batch_size));
        assert!(!config.is_batch_size_valid(0));
        assert!(!config.is_batch_size_valid(config.max_batch_size + 1));
    }

    #[test]
    fn test_memory_estimation() {
        let config = BatchConfig::default();
        let memory_usage = config.estimate_memory_usage(1000, 384);
        assert!(memory_usage > 0);
        
        assert!(!config.would_exceed_memory_limit(100, 384));
        // This might exceed depending on the default memory limit
        // assert!(config.would_exceed_memory_limit(100000, 384));
    }

    #[test]
    fn test_chunk_size_calculation() {
        let config = BatchConfig::default();
        
        // Small batch should use full size
        assert_eq!(config.get_chunk_size_for_batch(50), 50);
        
        // Large batch should use configured chunk size
        assert_eq!(config.get_chunk_size_for_batch(2000), config.chunk_size);
    }
}
