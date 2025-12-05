# Cluster Memory Limits Specification

This document describes the memory management system for Vectorizer in cluster mode.

## Overview

When running Vectorizer in cluster mode, predictable memory usage is critical for:
- Preventing out-of-memory crashes in production
- Ensuring stable performance under load
- Supporting multi-tenant workloads
- Enabling proper capacity planning

## Configuration

### ClusterMemoryConfig

The cluster memory configuration is part of `ClusterConfig`:

```yaml
cluster:
  enabled: true
  node_id: "node-1"
  servers:
    - id: "node-1"
      address: "192.168.1.10"
      grpc_port: 15003

  # Memory limits configuration
  memory:
    # Maximum total cache memory in bytes (default: 1GB)
    max_cache_memory_bytes: 1073741824

    # Enforce MMap storage for all collections (default: true)
    enforce_mmap_storage: true

    # Disable file watcher in cluster mode (default: true)
    disable_file_watcher: true

    # Warning threshold percentage (0-100, default: 80)
    cache_warning_threshold: 80

    # Fail startup on config violations (default: true)
    strict_validation: true
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_cache_memory_bytes` | u64 | 1GB | Maximum total cache memory across all caches |
| `enforce_mmap_storage` | bool | true | Reject Memory storage type in cluster mode |
| `disable_file_watcher` | bool | true | Prevent file watcher from starting |
| `cache_warning_threshold` | u8 | 80 | Emit warning when usage exceeds this percentage |
| `strict_validation` | bool | true | Fail startup on configuration violations |

## Components

### ClusterConfigValidator

Validates cluster configuration at startup:

```rust
use vectorizer::cluster::{ClusterConfigValidator, ClusterConfig};

let validator = ClusterConfigValidator::new();
let result = validator.validate(&config);

if result.has_errors() {
    eprintln!("{}", result.error_message());
}
```

**Validation Checks:**
- Node ID is required when cluster is enabled
- At least one server must be configured
- Cache memory limit is valid (0 < limit <= 10GB)
- Cache warning threshold is 0-100
- MMap storage enforcement is enabled
- File watcher is disabled

**Error Types:**
- `MemoryStorageNotAllowed` - Memory storage rejected in cluster mode
- `CacheMemoryLimitTooHigh` - Limit exceeds maximum (10GB)
- `CacheMemoryLimitZero` - Limit cannot be zero
- `FileWatcherEnabled` - File watcher must be disabled
- `NoServersConfigured` - No cluster servers defined
- `NodeIdMissing` - Node ID required for cluster mode
- `InvalidCacheWarningThreshold` - Threshold must be 0-100

**Warning Types:**
- `CacheMemoryLimitLow` - Limit below 256MB recommended
- `StrictValidationDisabled` - Errors won't fail startup
- `CacheWarningThresholdMax` - No warnings at 100%
- `SingleServerCluster` - Single node has no redundancy

### CacheMemoryManager

Global cache memory tracking for cluster mode:

```rust
use vectorizer::cache::{
    CacheMemoryManager, CacheMemoryManagerConfig, AllocationResult
};

let config = CacheMemoryManagerConfig {
    max_memory_bytes: 1024 * 1024 * 1024, // 1GB
    warning_threshold_percent: 80,
    strict_enforcement: true,
};

let manager = CacheMemoryManager::new(config);

// Try to allocate memory
match manager.try_allocate(100 * 1024 * 1024) { // 100MB
    AllocationResult::Success => println!("Allocated successfully"),
    AllocationResult::SuccessWithWarning { current_usage, max } => {
        println!("Allocated, but usage at {}%", (current_usage * 100) / max);
    }
    AllocationResult::Rejected { requested, available } => {
        println!("Rejected: requested {} bytes, only {} available", requested, available);
    }
}

// Deallocate memory
manager.deallocate(50 * 1024 * 1024);

// Get statistics
let stats = manager.stats();
println!("Current usage: {} bytes", stats.current_usage_bytes);
println!("Peak usage: {} bytes", stats.peak_usage_bytes);
println!("Allocations: {}", stats.allocation_count);
println!("Rejections: {}", stats.rejected_allocations);
```

### Global Singleton

The cache memory manager is available as a global singleton:

```rust
use vectorizer::cache::{
    init_global_cache_memory_manager,
    get_global_cache_memory_manager,
    CacheMemoryManagerConfig,
};

// Initialize at startup (once)
let config = CacheMemoryManagerConfig {
    max_memory_bytes: 1024 * 1024 * 1024,
    warning_threshold_percent: 80,
    strict_enforcement: true,
};
init_global_cache_memory_manager(config);

// Use anywhere in the codebase
let manager = get_global_cache_memory_manager();
manager.try_allocate(1024);
```

## Enforcement Behavior

### Strict Mode (default)

When `strict_validation: true`:
- Server fails to start if configuration is invalid
- Memory allocations that exceed limit are rejected
- Clear error messages indicate what needs to be fixed

### Non-Strict Mode

When `strict_validation: false`:
- Server starts with warnings for invalid configuration
- Memory allocations over limit succeed but log warnings
- Useful for debugging but not recommended for production

## File Watcher Behavior

When cluster mode is enabled with `disable_file_watcher: true`:
- File watcher is automatically disabled at startup
- Warning logged if file watcher was enabled in config
- Prevents incompatible behavior in distributed clusters

## Storage Type Enforcement

When `enforce_mmap_storage: true`:
- Memory storage type is rejected for collections
- Only MMap storage is allowed
- Prevents unbounded memory growth from large collections

## Best Practices

### Production Cluster Configuration

```yaml
cluster:
  enabled: true
  node_id: "prod-node-1"
  servers:
    - id: "prod-node-1"
      address: "10.0.1.10"
      grpc_port: 15003
    - id: "prod-node-2"
      address: "10.0.1.11"
      grpc_port: 15003
    - id: "prod-node-3"
      address: "10.0.1.12"
      grpc_port: 15003
  timeout_ms: 5000
  retry_count: 3
  memory:
    max_cache_memory_bytes: 2147483648  # 2GB
    enforce_mmap_storage: true
    disable_file_watcher: true
    cache_warning_threshold: 75
    strict_validation: true
```

### Memory Sizing Guidelines

| Workload | Recommended Cache | Notes |
|----------|-------------------|-------|
| Small (< 1M vectors) | 512MB | Single tenant, light load |
| Medium (1-10M vectors) | 1GB | Multi-tenant, moderate load |
| Large (10-100M vectors) | 2-4GB | Heavy multi-tenant workload |
| Enterprise (> 100M) | 4-8GB | Maximum, requires careful monitoring |

### Monitoring

Monitor these metrics in production:
- `cache_memory_current_bytes` - Current cache memory usage
- `cache_memory_peak_bytes` - Peak usage since startup
- `cache_memory_rejected_total` - Number of rejected allocations
- `cache_memory_evictions_total` - Number of forced evictions

## Troubleshooting

### Server Fails to Start

**Error:** "Cluster configuration validation failed"

**Solutions:**
1. Ensure `node_id` is set
2. Add at least one server to `servers` list
3. Check `max_cache_memory_bytes` is between 1 and 10GB
4. Verify `cache_warning_threshold` is 0-100

### Memory Allocations Rejected

**Error:** "Cache memory allocation rejected"

**Solutions:**
1. Increase `max_cache_memory_bytes`
2. Reduce cache TTL to evict entries faster
3. Add more cluster nodes to distribute load
4. Review query patterns for inefficiencies

### File Watcher Won't Start

**Expected Behavior:** File watcher is disabled in cluster mode

**If needed for testing:**
1. Set `memory.disable_file_watcher: false`
2. Note: Not recommended for production clusters

## API Reference

### ClusterMemoryConfig

```rust
pub struct ClusterMemoryConfig {
    pub max_cache_memory_bytes: u64,
    pub enforce_mmap_storage: bool,
    pub disable_file_watcher: bool,
    pub cache_warning_threshold: u8,
    pub strict_validation: bool,
}
```

### CacheMemoryManager Methods

| Method | Description |
|--------|-------------|
| `new(config)` | Create new manager |
| `disabled()` | Create disabled manager |
| `try_allocate(bytes)` | Attempt to allocate memory |
| `deallocate(bytes)` | Release allocated memory |
| `current_usage()` | Get current usage in bytes |
| `peak_usage()` | Get peak usage in bytes |
| `available()` | Get available memory |
| `usage_percent()` | Get usage as percentage |
| `stats()` | Get full statistics |
| `would_exceed_limit(bytes)` | Check if allocation would exceed |
| `recommended_eviction_size(bytes)` | Get recommended eviction size |

### AllocationResult

```rust
pub enum AllocationResult {
    Success,
    Rejected { requested: u64, available: u64 },
    SuccessWithWarning { current_usage: u64, max: u64 },
}
```
