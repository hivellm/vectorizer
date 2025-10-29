//! Configuration management for batch processing
//!
//! This module provides configuration management for batch operations,
//! including preset configurations, validation, and optimization.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::batch::{BatchConfig, BatchErrorType};

/// Batch processing configuration builder
pub struct BatchConfigBuilder {
    config: BatchConfig,
}

impl BatchConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: BatchConfig::default(),
        }
    }

    /// Set maximum number of workers
    pub fn max_workers(mut self, workers: usize) -> Self {
        self.config.max_workers = workers;
        self
    }

    /// Set batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Set maximum retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Set operation timeout
    pub fn operation_timeout(mut self, timeout: Duration) -> Self {
        self.config.operation_timeout = timeout.as_secs();
        self
    }

    /// Enable or disable progress tracking
    pub fn enable_progress(mut self, enable: bool) -> Self {
        self.config.enable_progress = enable;
        self
    }

    /// Enable or disable parallel processing
    pub fn enable_parallel(mut self, enable: bool) -> Self {
        self.config.enable_parallel = enable;
        self
    }

    /// Set memory limit per worker
    pub fn memory_limit_mb(mut self, limit: usize) -> Self {
        self.config.memory_limit_mb = limit;
        self
    }

    /// Build the configuration
    pub fn build(self) -> BatchConfig {
        self.config
    }
}

impl Default for BatchConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch processing presets
pub struct BatchPresets;

impl BatchPresets {
    /// Fast processing preset (speed over memory efficiency)
    pub fn fast() -> BatchConfig {
        BatchConfigBuilder::new()
            .max_workers(num_cpus::get() * 2)
            .batch_size(500)
            .max_retries(1)
            .operation_timeout(Duration::from_secs(60))
            .enable_progress(true)
            .enable_parallel(true)
            .memory_limit_mb(256)
            .build()
    }

    /// Balanced processing preset (good balance)
    pub fn balanced() -> BatchConfig {
        BatchConfigBuilder::new()
            .max_workers(num_cpus::get())
            .batch_size(1000)
            .max_retries(3)
            .operation_timeout(Duration::from_secs(300))
            .enable_progress(true)
            .enable_parallel(true)
            .memory_limit_mb(512)
            .build()
    }

    /// Memory-efficient processing preset
    pub fn memory_efficient() -> BatchConfig {
        BatchConfigBuilder::new()
            .max_workers(num_cpus::get() / 2)
            .batch_size(100)
            .max_retries(2)
            .operation_timeout(Duration::from_secs(600))
            .enable_progress(true)
            .enable_parallel(false)
            .memory_limit_mb(128)
            .build()
    }

    /// High-throughput processing preset
    pub fn high_throughput() -> BatchConfig {
        BatchConfigBuilder::new()
            .max_workers(num_cpus::get() * 4)
            .batch_size(2000)
            .max_retries(5)
            .operation_timeout(Duration::from_secs(180))
            .enable_progress(false)
            .enable_parallel(true)
            .memory_limit_mb(1024)
            .build()
    }

    /// Sequential processing preset (no parallelism)
    pub fn sequential() -> BatchConfig {
        BatchConfigBuilder::new()
            .max_workers(1)
            .batch_size(50)
            .max_retries(3)
            .operation_timeout(Duration::from_secs(300))
            .enable_progress(true)
            .enable_parallel(false)
            .memory_limit_mb(64)
            .build()
    }

    /// Real-time processing preset (low latency)
    pub fn real_time() -> BatchConfig {
        BatchConfigBuilder::new()
            .max_workers(num_cpus::get())
            .batch_size(10)
            .max_retries(0)
            .operation_timeout(Duration::from_secs(5))
            .enable_progress(false)
            .enable_parallel(true)
            .memory_limit_mb(128)
            .build()
    }
}

/// Configuration validator for batch processing
pub struct BatchConfigValidator;

impl BatchConfigValidator {
    /// Validate a batch configuration
    pub fn validate(config: &BatchConfig) -> Result<(), ValidationError> {
        // Validate max_workers
        if config.max_workers == 0 {
            return Err(ValidationError::InvalidValue(
                "max_workers must be greater than 0".to_string(),
            ));
        }

        if config.max_workers > 1000 {
            return Err(ValidationError::InvalidValue(
                "max_workers should not exceed 1000".to_string(),
            ));
        }

        // Validate batch_size
        if config.batch_size == 0 {
            return Err(ValidationError::InvalidValue(
                "batch_size must be greater than 0".to_string(),
            ));
        }

        if config.batch_size > 100_000 {
            return Err(ValidationError::InvalidValue(
                "batch_size should not exceed 100,000".to_string(),
            ));
        }

        // Validate max_retries
        if config.max_retries > 100 {
            return Err(ValidationError::InvalidValue(
                "max_retries should not exceed 100".to_string(),
            ));
        }

        // Validate operation_timeout
        if config.operation_timeout == 0 {
            return Err(ValidationError::InvalidValue(
                "operation_timeout must be greater than 0".to_string(),
            ));
        }

        if config.operation_timeout > 3600 {
            return Err(ValidationError::InvalidValue(
                "operation_timeout should not exceed 3600 seconds".to_string(),
            ));
        }

        // Validate memory_limit_mb
        if config.memory_limit_mb == 0 {
            return Err(ValidationError::InvalidValue(
                "memory_limit_mb must be greater than 0".to_string(),
            ));
        }

        if config.memory_limit_mb > 10_000 {
            return Err(ValidationError::InvalidValue(
                "memory_limit_mb should not exceed 10,000 MB".to_string(),
            ));
        }

        Ok(())
    }

    /// Optimize configuration based on system resources
    pub fn optimize_for_system(config: &mut BatchConfig) {
        let cpu_count = num_cpus::get();
        let available_memory_mb = get_available_memory_mb();

        // Optimize max_workers
        if config.max_workers > cpu_count * 2 {
            config.max_workers = cpu_count * 2;
        }

        // Optimize batch_size based on available memory
        let estimated_memory_per_item = 1024; // 1KB per item estimate
        let max_batch_size =
            (available_memory_mb * 1024 * 1024) / (config.max_workers * estimated_memory_per_item);

        if config.batch_size > max_batch_size {
            config.batch_size = max_batch_size.max(1);
        }

        // Optimize memory_limit_mb
        let memory_per_worker = available_memory_mb / config.max_workers;
        if config.memory_limit_mb > memory_per_worker {
            config.memory_limit_mb = memory_per_worker.max(64);
        }
    }

    /// Get recommended configuration for workload
    pub fn recommend_for_workload(workload: WorkloadType, data_size: usize) -> BatchConfig {
        match workload {
            WorkloadType::CpuIntensive => {
                if data_size < 1000 {
                    BatchPresets::fast()
                } else if data_size < 100_000 {
                    BatchPresets::balanced()
                } else {
                    BatchPresets::high_throughput()
                }
            }
            WorkloadType::MemoryIntensive => {
                if data_size < 1000 {
                    BatchPresets::memory_efficient()
                } else {
                    BatchPresets::balanced()
                }
            }
            WorkloadType::IoIntensive => {
                if data_size < 10_000 {
                    BatchPresets::fast()
                } else {
                    BatchPresets::high_throughput()
                }
            }
            WorkloadType::RealTime => BatchPresets::real_time(),
            WorkloadType::Sequential => BatchPresets::sequential(),
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

/// Workload types for configuration recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadType {
    /// CPU-intensive workload
    CpuIntensive,
    /// Memory-intensive workload
    MemoryIntensive,
    /// I/O-intensive workload
    IoIntensive,
    /// Real-time workload
    RealTime,
    /// Sequential workload
    Sequential,
}

/// Performance tuning recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendations {
    /// Recommended configuration
    pub recommended_config: BatchConfig,
    /// Expected performance metrics
    pub expected_metrics: ExpectedMetrics,
    /// Tuning suggestions
    pub suggestions: Vec<String>,
}

/// Expected performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedMetrics {
    /// Expected processing rate (items per second)
    pub processing_rate: f64,
    /// Expected memory usage (MB)
    pub memory_usage_mb: f64,
    /// Expected error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Expected completion time (seconds)
    pub completion_time_seconds: f64,
}

/// Get available system memory in MB
fn get_available_memory_mb() -> usize {
    // This is a simplified implementation
    // In a real implementation, you would use system APIs
    let total_memory = 8 * 1024; // Assume 8GB
    let used_memory = 2 * 1024; // Assume 2GB used
    total_memory - used_memory
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = BatchConfigBuilder::new()
            .max_workers(4)
            .batch_size(500)
            .max_retries(2)
            .operation_timeout(Duration::from_secs(120))
            .enable_progress(false)
            .enable_parallel(true)
            .memory_limit_mb(256)
            .build();

        assert_eq!(config.max_workers, 4);
        assert_eq!(config.batch_size, 500);
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.operation_timeout, 120);
        assert!(!config.enable_progress);
        assert!(config.enable_parallel);
        assert_eq!(config.memory_limit_mb, 256);
    }

    #[test]
    fn test_presets() {
        let fast = BatchPresets::fast();
        assert!(fast.max_workers > 0);
        assert!(fast.batch_size > 0);
        assert!(fast.enable_parallel);

        let sequential = BatchPresets::sequential();
        assert_eq!(sequential.max_workers, 1);
        assert!(!sequential.enable_parallel);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = BatchConfig::default();
        assert!(BatchConfigValidator::validate(&valid_config).is_ok());

        let invalid_config = BatchConfig {
            max_workers: 0,
            batch_size: 1000,
            max_retries: 3,
            operation_timeout: 300,
            enable_progress: true,
            enable_parallel: true,
            memory_limit_mb: 512,
        };

        assert!(BatchConfigValidator::validate(&invalid_config).is_err());
    }

    #[test]
    fn test_workload_recommendations() {
        let cpu_config =
            BatchConfigValidator::recommend_for_workload(WorkloadType::CpuIntensive, 1000);
        assert!(cpu_config.enable_parallel);

        let sequential_config =
            BatchConfigValidator::recommend_for_workload(WorkloadType::Sequential, 1000);
        assert!(!sequential_config.enable_parallel);
    }
}
