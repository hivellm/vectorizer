//! Integration tests for cluster memory limits
//!
//! Tests the enforcement of memory limits, MMap storage, and file watcher
//! disabling in cluster mode.
#![allow(clippy::uninlined_format_args)]

use vectorizer::cache::{AllocationResult, CacheMemoryManager, CacheMemoryManagerConfig};
use vectorizer::cluster::{
    ClusterConfig, ClusterConfigValidator, ClusterMemoryConfig, ClusterValidationError,
    DiscoveryMethod, ServerConfig,
};
use vectorizer::models::StorageType;

/// Create a valid cluster configuration for testing
fn create_test_cluster_config() -> ClusterConfig {
    ClusterConfig {
        enabled: true,
        node_id: Some("test-node-1".to_string()),
        servers: vec![
            ServerConfig {
                id: "node-1".to_string(),
                address: "localhost:15002".to_string(),
                grpc_port: 15003,
            },
            ServerConfig {
                id: "node-2".to_string(),
                address: "localhost:15004".to_string(),
                grpc_port: 15005,
            },
        ],
        discovery: DiscoveryMethod::Static,
        timeout_ms: 5000,
        retry_count: 3,
        memory: ClusterMemoryConfig {
            max_cache_memory_bytes: 1024 * 1024 * 1024, // 1GB
            enforce_mmap_storage: true,
            disable_file_watcher: true,
            cache_warning_threshold: 80,
            strict_validation: true,
        },
    }
}

#[test]
fn test_cluster_config_validator_integration() {
    let config = create_test_cluster_config();
    let validator = ClusterConfigValidator::new();

    let result = validator.validate(&config);

    assert!(result.valid, "Valid cluster config should pass validation");
    assert!(!result.has_errors(), "No errors expected");
}

#[test]
fn test_cluster_storage_type_validation() {
    let config = create_test_cluster_config();
    let validator = ClusterConfigValidator::new();

    // Memory storage should be rejected
    let result = validator.validate_storage_type(&config, &StorageType::Memory);
    assert!(
        !result.valid,
        "Memory storage should be rejected in cluster mode"
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, ClusterValidationError::MemoryStorageNotAllowed))
    );

    // MMap storage should be allowed
    let result = validator.validate_storage_type(&config, &StorageType::Mmap);
    assert!(
        result.valid,
        "MMap storage should be allowed in cluster mode"
    );
}

#[test]
fn test_cluster_disabled_skips_validation() {
    let mut config = create_test_cluster_config();
    config.enabled = false;

    let validator = ClusterConfigValidator::new();

    // Should pass even with invalid settings when cluster is disabled
    config.node_id = None;
    config.servers.clear();

    let result = validator.validate(&config);
    assert!(result.valid, "Disabled cluster should skip validation");
}

#[test]
fn test_cache_memory_manager_integration() {
    let config = CacheMemoryManagerConfig {
        max_memory_bytes: 1024 * 1024, // 1MB
        warning_threshold_percent: 80,
        strict_enforcement: true,
    };

    let manager = CacheMemoryManager::new(config);

    // Allocate 500KB
    let result = manager.try_allocate(500 * 1024);
    assert!(result.is_success());
    assert_eq!(manager.current_usage(), 500 * 1024);

    // Allocate another 400KB (should trigger warning at 88%)
    let result = manager.try_allocate(400 * 1024);
    assert!(matches!(
        result,
        AllocationResult::SuccessWithWarning { .. }
    ));

    // Try to allocate 200KB more (would exceed limit)
    let result = manager.try_allocate(200 * 1024);
    assert!(matches!(result, AllocationResult::Rejected { .. }));

    // Deallocate and try again
    manager.deallocate(300 * 1024);
    let result = manager.try_allocate(200 * 1024);
    assert!(result.is_success());
}

#[test]
fn test_cache_memory_manager_non_strict_mode() {
    let config = CacheMemoryManagerConfig {
        max_memory_bytes: 1024 * 1024, // 1MB
        warning_threshold_percent: 80,
        strict_enforcement: false, // Non-strict mode
    };

    let manager = CacheMemoryManager::new(config);

    // Allocate more than limit (should succeed in non-strict mode)
    let result = manager.try_allocate(2 * 1024 * 1024);
    assert!(result.is_success());
    assert_eq!(manager.current_usage(), 2 * 1024 * 1024);
}

#[test]
fn test_cache_memory_manager_statistics() {
    let config = CacheMemoryManagerConfig {
        max_memory_bytes: 1024 * 1024,
        warning_threshold_percent: 80,
        strict_enforcement: true,
    };

    let manager = CacheMemoryManager::new(config);

    // Perform various operations
    manager.try_allocate(100 * 1024);
    manager.try_allocate(200 * 1024);
    manager.deallocate(50 * 1024);
    manager.try_allocate(10 * 1024 * 1024); // Will be rejected
    manager.record_eviction();

    let stats = manager.stats();
    assert_eq!(stats.allocation_count, 3);
    assert_eq!(stats.deallocation_count, 1);
    assert_eq!(stats.rejected_allocations, 1);
    assert_eq!(stats.forced_evictions, 1);
    assert_eq!(stats.current_usage_bytes, 250 * 1024);
    assert_eq!(stats.peak_usage_bytes, 300 * 1024);
}

#[test]
fn test_cluster_config_memory_defaults() {
    let config = ClusterConfig::default();

    assert!(!config.enabled);
    assert_eq!(config.memory.max_cache_memory_bytes, 1024 * 1024 * 1024); // 1GB
    assert!(config.memory.enforce_mmap_storage);
    assert!(config.memory.disable_file_watcher);
    assert_eq!(config.memory.cache_warning_threshold, 80);
    assert!(config.memory.strict_validation);
}

#[test]
fn test_cluster_validation_error_codes() {
    let errors = vec![
        ClusterValidationError::MemoryStorageNotAllowed,
        ClusterValidationError::CacheMemoryLimitTooHigh {
            limit_bytes: 100,
            max_bytes: 50,
        },
        ClusterValidationError::CacheMemoryLimitZero,
        ClusterValidationError::FileWatcherEnabled,
        ClusterValidationError::NoServersConfigured,
        ClusterValidationError::NodeIdMissing,
        ClusterValidationError::InvalidCacheWarningThreshold { threshold: 150 },
    ];

    for error in errors {
        assert!(!error.code().is_empty(), "Error should have a code");
        assert!(!error.message().is_empty(), "Error should have a message");
    }
}

#[test]
fn test_cache_memory_manager_would_exceed_limit() {
    let config = CacheMemoryManagerConfig {
        max_memory_bytes: 1000,
        warning_threshold_percent: 80,
        strict_enforcement: true,
    };

    let manager = CacheMemoryManager::new(config);

    assert!(!manager.would_exceed_limit(500));
    manager.try_allocate(600);
    assert!(!manager.would_exceed_limit(400));
    assert!(manager.would_exceed_limit(401));
}

#[test]
fn test_cache_memory_manager_recommended_eviction() {
    let config = CacheMemoryManagerConfig {
        max_memory_bytes: 1000,
        warning_threshold_percent: 80,
        strict_enforcement: true,
    };

    let manager = CacheMemoryManager::new(config);

    // No eviction needed when under limit
    assert!(manager.recommended_eviction_size(500).is_none());

    // Fill up and check eviction recommendation
    manager.try_allocate(900);
    let eviction = manager.recommended_eviction_size(200);
    assert!(
        eviction.is_some(),
        "Should recommend eviction when over limit"
    );
}

#[test]
fn test_disabled_cache_memory_manager() {
    let manager = CacheMemoryManager::disabled();

    assert!(!manager.is_enabled());

    // All operations should succeed without tracking
    let result = manager.try_allocate(100 * 1024 * 1024 * 1024); // 100GB
    assert!(result.is_success());
    assert_eq!(manager.current_usage(), 0); // Not tracked

    manager.deallocate(1000);
    manager.record_eviction();

    let stats = manager.stats();
    assert_eq!(stats.allocation_count, 0);
}
