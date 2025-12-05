//! Cluster configuration validator
//!
//! Validates cluster configuration to ensure memory limits and storage
//! requirements are properly configured for production cluster deployments.

use crate::config::FileWatcherYamlConfig;
use crate::models::StorageType;

use super::{ClusterConfig, ClusterMemoryConfig};

/// Result of cluster configuration validation
#[derive(Debug, Clone)]
pub struct ClusterValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// List of errors (validation failures)
    pub errors: Vec<ClusterValidationError>,
    /// List of warnings (non-fatal issues)
    pub warnings: Vec<ClusterValidationWarning>,
}

impl ClusterValidationResult {
    /// Create a new passing validation result
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: ClusterValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: ClusterValidationWarning) {
        self.warnings.push(warning);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get formatted error message for all errors
    pub fn error_message(&self) -> String {
        if self.errors.is_empty() {
            return String::new();
        }
        let messages: Vec<String> = self.errors.iter().map(|e| e.message()).collect();
        format!(
            "Cluster configuration validation failed:\n  - {}",
            messages.join("\n  - ")
        )
    }

    /// Get formatted warning message for all warnings
    pub fn warning_message(&self) -> String {
        if self.warnings.is_empty() {
            return String::new();
        }
        let messages: Vec<String> = self.warnings.iter().map(|w| w.message()).collect();
        format!(
            "Cluster configuration warnings:\n  - {}",
            messages.join("\n  - ")
        )
    }
}

/// Cluster validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterValidationError {
    /// Memory storage is not allowed in cluster mode
    MemoryStorageNotAllowed,
    /// Cache memory limit is too high
    CacheMemoryLimitTooHigh { limit_bytes: u64, max_bytes: u64 },
    /// Cache memory limit is zero
    CacheMemoryLimitZero,
    /// File watcher is enabled in cluster mode
    FileWatcherEnabled,
    /// No servers configured in cluster mode
    NoServersConfigured,
    /// Node ID is missing when cluster mode is enabled
    NodeIdMissing,
    /// Invalid cache warning threshold (must be 0-100)
    InvalidCacheWarningThreshold { threshold: u8 },
    /// Custom validation error
    Custom(String),
}

impl ClusterValidationError {
    /// Get human-readable error message
    pub fn message(&self) -> String {
        match self {
            Self::MemoryStorageNotAllowed => {
                "Memory storage type is not allowed in cluster mode. Use MMap storage instead."
                    .to_string()
            }
            Self::CacheMemoryLimitTooHigh { limit_bytes, max_bytes } => {
                format!(
                    "Cache memory limit ({} bytes) exceeds maximum allowed ({} bytes)",
                    limit_bytes, max_bytes
                )
            }
            Self::CacheMemoryLimitZero => {
                "Cache memory limit cannot be zero in cluster mode".to_string()
            }
            Self::FileWatcherEnabled => {
                "File watcher must be disabled in cluster mode. Set cluster.memory.disable_file_watcher = true".to_string()
            }
            Self::NoServersConfigured => {
                "No servers configured for cluster mode. Add at least one server to cluster.servers".to_string()
            }
            Self::NodeIdMissing => {
                "Node ID is required when cluster mode is enabled. Set cluster.node_id".to_string()
            }
            Self::InvalidCacheWarningThreshold { threshold } => {
                format!(
                    "Cache warning threshold ({}) must be between 0 and 100",
                    threshold
                )
            }
            Self::Custom(msg) => msg.clone(),
        }
    }

    /// Get error code for programmatic handling
    pub fn code(&self) -> &'static str {
        match self {
            Self::MemoryStorageNotAllowed => "CLUSTER_MEMORY_STORAGE_NOT_ALLOWED",
            Self::CacheMemoryLimitTooHigh { .. } => "CLUSTER_CACHE_LIMIT_TOO_HIGH",
            Self::CacheMemoryLimitZero => "CLUSTER_CACHE_LIMIT_ZERO",
            Self::FileWatcherEnabled => "CLUSTER_FILE_WATCHER_ENABLED",
            Self::NoServersConfigured => "CLUSTER_NO_SERVERS",
            Self::NodeIdMissing => "CLUSTER_NODE_ID_MISSING",
            Self::InvalidCacheWarningThreshold { .. } => "CLUSTER_INVALID_WARNING_THRESHOLD",
            Self::Custom(_) => "CLUSTER_CUSTOM_ERROR",
        }
    }
}

/// Cluster validation warning types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterValidationWarning {
    /// Cache memory limit is low for production
    CacheMemoryLimitLow {
        limit_bytes: u64,
        recommended_bytes: u64,
    },
    /// Strict validation is disabled
    StrictValidationDisabled,
    /// Cache warning threshold is set to 100 (no warnings)
    CacheWarningThresholdMax,
    /// Single server cluster (no redundancy)
    SingleServerCluster,
    /// Custom warning
    Custom(String),
}

impl ClusterValidationWarning {
    /// Get human-readable warning message
    pub fn message(&self) -> String {
        match self {
            Self::CacheMemoryLimitLow {
                limit_bytes,
                recommended_bytes,
            } => {
                format!(
                    "Cache memory limit ({} MB) is below recommended ({} MB) for production",
                    limit_bytes / (1024 * 1024),
                    recommended_bytes / (1024 * 1024)
                )
            }
            Self::StrictValidationDisabled => {
                "Strict validation is disabled. Configuration errors will be warnings only."
                    .to_string()
            }
            Self::CacheWarningThresholdMax => {
                "Cache warning threshold is 100%. No cache usage warnings will be emitted."
                    .to_string()
            }
            Self::SingleServerCluster => {
                "Only one server configured. Consider adding more servers for redundancy."
                    .to_string()
            }
            Self::Custom(msg) => msg.clone(),
        }
    }
}

/// Cluster configuration validator
pub struct ClusterConfigValidator {
    /// Maximum allowed cache memory (10GB)
    max_cache_memory_bytes: u64,
    /// Recommended minimum cache memory (256MB)
    min_recommended_cache_bytes: u64,
}

impl Default for ClusterConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterConfigValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self {
            max_cache_memory_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            min_recommended_cache_bytes: 256 * 1024 * 1024,  // 256MB
        }
    }

    /// Create a validator with custom limits
    pub fn with_limits(max_cache_bytes: u64, min_recommended_bytes: u64) -> Self {
        Self {
            max_cache_memory_bytes: max_cache_bytes,
            min_recommended_cache_bytes: min_recommended_bytes,
        }
    }

    /// Validate cluster configuration
    pub fn validate(&self, config: &ClusterConfig) -> ClusterValidationResult {
        let mut result = ClusterValidationResult::ok();

        // Only validate if cluster mode is enabled
        if !config.enabled {
            return result;
        }

        // Validate memory configuration
        self.validate_memory_config(&config.memory, &mut result);

        // Validate node configuration
        self.validate_node_config(config, &mut result);

        // Add warning if strict validation is disabled
        if !config.memory.strict_validation {
            result.add_warning(ClusterValidationWarning::StrictValidationDisabled);
        }

        result
    }

    /// Validate cluster configuration with file watcher config
    pub fn validate_with_file_watcher(
        &self,
        config: &ClusterConfig,
        file_watcher_config: &FileWatcherYamlConfig,
    ) -> ClusterValidationResult {
        let mut result = self.validate(config);

        // Only validate if cluster mode is enabled
        if !config.enabled {
            return result;
        }

        // Check if file watcher is enabled
        if config.memory.disable_file_watcher && file_watcher_config.enabled {
            result.add_error(ClusterValidationError::FileWatcherEnabled);
        }

        result
    }

    /// Validate storage type for cluster mode
    pub fn validate_storage_type(
        &self,
        config: &ClusterConfig,
        storage_type: &StorageType,
    ) -> ClusterValidationResult {
        let mut result = ClusterValidationResult::ok();

        // Only validate if cluster mode is enabled and MMap is enforced
        if !config.enabled || !config.memory.enforce_mmap_storage {
            return result;
        }

        // Check if storage type is Memory
        if matches!(storage_type, StorageType::Memory) {
            result.add_error(ClusterValidationError::MemoryStorageNotAllowed);
        }

        result
    }

    fn validate_memory_config(
        &self,
        memory: &ClusterMemoryConfig,
        result: &mut ClusterValidationResult,
    ) {
        // Check cache memory limit
        if memory.max_cache_memory_bytes == 0 {
            result.add_error(ClusterValidationError::CacheMemoryLimitZero);
        } else if memory.max_cache_memory_bytes > self.max_cache_memory_bytes {
            result.add_error(ClusterValidationError::CacheMemoryLimitTooHigh {
                limit_bytes: memory.max_cache_memory_bytes,
                max_bytes: self.max_cache_memory_bytes,
            });
        } else if memory.max_cache_memory_bytes < self.min_recommended_cache_bytes {
            result.add_warning(ClusterValidationWarning::CacheMemoryLimitLow {
                limit_bytes: memory.max_cache_memory_bytes,
                recommended_bytes: self.min_recommended_cache_bytes,
            });
        }

        // Check cache warning threshold
        if memory.cache_warning_threshold > 100 {
            result.add_error(ClusterValidationError::InvalidCacheWarningThreshold {
                threshold: memory.cache_warning_threshold,
            });
        } else if memory.cache_warning_threshold == 100 {
            result.add_warning(ClusterValidationWarning::CacheWarningThresholdMax);
        }
    }

    fn validate_node_config(&self, config: &ClusterConfig, result: &mut ClusterValidationResult) {
        // Check node ID
        if config.node_id.is_none() {
            result.add_error(ClusterValidationError::NodeIdMissing);
        }

        // Check servers
        if config.servers.is_empty() {
            result.add_error(ClusterValidationError::NoServersConfigured);
        } else if config.servers.len() == 1 {
            result.add_warning(ClusterValidationWarning::SingleServerCluster);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::ServerConfig;

    fn create_valid_cluster_config() -> ClusterConfig {
        ClusterConfig {
            enabled: true,
            node_id: Some("node1".to_string()),
            servers: vec![
                ServerConfig {
                    id: "server1".to_string(),
                    address: "localhost:15002".to_string(),
                    grpc_port: 15003,
                },
                ServerConfig {
                    id: "server2".to_string(),
                    address: "localhost:15004".to_string(),
                    grpc_port: 15005,
                },
            ],
            discovery: super::super::DiscoveryMethod::Static,
            timeout_ms: 5000,
            retry_count: 3,
            memory: ClusterMemoryConfig::default(),
        }
    }

    #[test]
    fn test_valid_cluster_config() {
        let validator = ClusterConfigValidator::new();
        let config = create_valid_cluster_config();

        let result = validator.validate(&config);

        assert!(result.valid);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_disabled_cluster_skips_validation() {
        let validator = ClusterConfigValidator::new();
        let config = ClusterConfig {
            enabled: false,
            ..ClusterConfig::default()
        };

        let result = validator.validate(&config);

        assert!(result.valid);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_missing_node_id() {
        let validator = ClusterConfigValidator::new();
        let mut config = create_valid_cluster_config();
        config.node_id = None;

        let result = validator.validate(&config);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .contains(&ClusterValidationError::NodeIdMissing)
        );
    }

    #[test]
    fn test_no_servers_configured() {
        let validator = ClusterConfigValidator::new();
        let mut config = create_valid_cluster_config();
        config.servers.clear();

        let result = validator.validate(&config);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .contains(&ClusterValidationError::NoServersConfigured)
        );
    }

    #[test]
    fn test_single_server_warning() {
        let validator = ClusterConfigValidator::new();
        let mut config = create_valid_cluster_config();
        config.servers = vec![config.servers[0].clone()];

        let result = validator.validate(&config);

        assert!(result.valid);
        assert!(result.has_warnings());
        assert!(
            result
                .warnings
                .contains(&ClusterValidationWarning::SingleServerCluster)
        );
    }

    #[test]
    fn test_cache_memory_limit_zero() {
        let validator = ClusterConfigValidator::new();
        let mut config = create_valid_cluster_config();
        config.memory.max_cache_memory_bytes = 0;

        let result = validator.validate(&config);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .contains(&ClusterValidationError::CacheMemoryLimitZero)
        );
    }

    #[test]
    fn test_cache_memory_limit_too_high() {
        let validator = ClusterConfigValidator::new();
        let mut config = create_valid_cluster_config();
        config.memory.max_cache_memory_bytes = 100 * 1024 * 1024 * 1024; // 100GB

        let result = validator.validate(&config);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, ClusterValidationError::CacheMemoryLimitTooHigh { .. }))
        );
    }

    #[test]
    fn test_cache_memory_limit_low_warning() {
        let validator = ClusterConfigValidator::new();
        let mut config = create_valid_cluster_config();
        config.memory.max_cache_memory_bytes = 100 * 1024 * 1024; // 100MB

        let result = validator.validate(&config);

        assert!(result.valid);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| matches!(w, ClusterValidationWarning::CacheMemoryLimitLow { .. }))
        );
    }

    #[test]
    fn test_invalid_warning_threshold() {
        let validator = ClusterConfigValidator::new();
        let mut config = create_valid_cluster_config();
        config.memory.cache_warning_threshold = 150;

        let result = validator.validate(&config);

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ClusterValidationError::InvalidCacheWarningThreshold { .. }
        )));
    }

    #[test]
    fn test_memory_storage_rejected() {
        let validator = ClusterConfigValidator::new();
        let config = create_valid_cluster_config();

        let result = validator.validate_storage_type(&config, &StorageType::Memory);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .contains(&ClusterValidationError::MemoryStorageNotAllowed)
        );
    }

    #[test]
    fn test_mmap_storage_allowed() {
        let validator = ClusterConfigValidator::new();
        let config = create_valid_cluster_config();

        let result = validator.validate_storage_type(&config, &StorageType::Mmap);

        assert!(result.valid);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_strict_validation_disabled_warning() {
        let validator = ClusterConfigValidator::new();
        let mut config = create_valid_cluster_config();
        config.memory.strict_validation = false;

        let result = validator.validate(&config);

        assert!(result.valid);
        assert!(
            result
                .warnings
                .contains(&ClusterValidationWarning::StrictValidationDisabled)
        );
    }

    #[test]
    fn test_error_messages() {
        let error = ClusterValidationError::MemoryStorageNotAllowed;
        assert!(!error.message().is_empty());
        assert!(!error.code().is_empty());

        let error = ClusterValidationError::CacheMemoryLimitTooHigh {
            limit_bytes: 100,
            max_bytes: 50,
        };
        assert!(error.message().contains("100"));
        assert!(error.message().contains("50"));
    }

    #[test]
    fn test_warning_messages() {
        let warning = ClusterValidationWarning::SingleServerCluster;
        assert!(!warning.message().is_empty());

        let warning = ClusterValidationWarning::CacheMemoryLimitLow {
            limit_bytes: 100 * 1024 * 1024,
            recommended_bytes: 256 * 1024 * 1024,
        };
        assert!(warning.message().contains("100"));
        assert!(warning.message().contains("256"));
    }
}
