# Cluster Memory Limits - Technical Design

## Overview

This document describes the technical implementation for enforcing strict memory limits in cluster mode, making MMap storage mandatory, and disabling file watcher for multi-tenant deployments.

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Cluster Mode Server                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         ClusterConfigValidator (NEW)                  │  │
│  │  - Validates MMap enforcement                         │  │
│  │  - Validates cache limits                             │  │
│  │  - Validates file watcher disabled                    │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ▼                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         CacheMemoryManager (NEW)                      │  │
│  │  - Global 1GB limit enforcement                       │  │
│  │  - LRU eviction across all caches                     │  │
│  │  - Memory usage tracking                              │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ▼                                    │
│  ┌──────────────┬───────────────┬─────────────────────┐    │
│  │ AdvancedCache│  HNSW Cache   │  Metadata Cache     │    │
│  │  (embedding, │  (index data) │  (file info, etc.)  │    │
│  │   queries)   │               │                      │    │
│  └──────────────┴───────────────┴─────────────────────┘    │
│                          ▼                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         VectorStore (Modified)                        │  │
│  │  - Force MMap storage in cluster                      │  │
│  │  - Reject Memory storage                              │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ▼                                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Collections (MMap Only)                       │  │
│  │  - Disk-backed vector storage                         │  │
│  │  - Minimal memory footprint                           │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         FileWatcher (DISABLED)                        │  │
│  │  - Returns error if start attempted                   │  │
│  │  - Not initialized in cluster mode                    │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Implementation Details

### 1. ClusterConfigValidator

**Location**: `src/cluster/validator.rs` (NEW)

**Purpose**: Validate cluster configuration on server startup and prevent invalid configurations.

```rust
//! Cluster configuration validator
//!
//! Ensures cluster mode requirements are met before server starts.

use crate::cluster::ClusterConfig;
use crate::error::{Result, VectorizerError};
use crate::file_watcher::FileWatcherConfig;
use crate::models::{CollectionConfig, StorageType};
use tracing::{error, warn, info};

/// Validates cluster configuration
pub struct ClusterConfigValidator {
    cluster_config: ClusterConfig,
    file_watcher_config: FileWatcherConfig,
    default_collection_config: CollectionConfig,
}

impl ClusterConfigValidator {
    pub fn new(
        cluster_config: ClusterConfig,
        file_watcher_config: FileWatcherConfig,
        default_collection_config: CollectionConfig,
    ) -> Self {
        Self {
            cluster_config,
            file_watcher_config,
            default_collection_config,
        }
    }

    /// Validate all cluster requirements
    pub fn validate(&self) -> Result<()> {
        if !self.cluster_config.enabled {
            // Not in cluster mode, skip validation
            return Ok(());
        }

        info!("🔍 Validating cluster mode configuration...");

        // Validate cache memory limit
        self.validate_cache_limit()?;

        // Validate storage type
        self.validate_storage_type()?;

        // Validate file watcher is disabled
        self.validate_file_watcher()?;

        info!("✅ Cluster mode configuration is valid");
        Ok(())
    }

    fn validate_cache_limit(&self) -> Result<()> {
        let limit = self.cluster_config.max_cache_memory_bytes;

        if limit == 0 {
            error!("❌ Cache memory limit cannot be 0 in cluster mode");
            return Err(VectorizerError::ClusterConfigViolation(
                "max_cache_memory_bytes must be greater than 0".to_string(),
            ));
        }

        if limit > 2_147_483_648 {
            // 2GB
            warn!(
                "⚠️  Cache memory limit is {}GB, which is higher than recommended 1GB",
                limit / 1_073_741_824
            );
        }

        info!(
            "✓ Cache memory limit: {}MB",
            limit / 1_048_576
        );

        Ok(())
    }

    fn validate_storage_type(&self) -> Result<()> {
        match self.default_collection_config.storage_type {
            Some(StorageType::Memory) => {
                error!(
                    "❌ Memory storage is not allowed in cluster mode\n\
                     \n\
                     Fix: Set storage_type to 'mmap' in collections.defaults\n\
                     \n\
                     collections:\n\
                       defaults:\n\
                         storage_type: \"mmap\"  # Required for cluster mode"
                );
                Err(VectorizerError::ClusterConfigViolation(
                    "Memory storage is not allowed in cluster mode. Use MMap storage.".to_string(),
                ))
            }
            Some(StorageType::Mmap) => {
                info!("✓ Storage type: MMap (required for cluster mode)");
                Ok(())
            }
            None => {
                warn!("⚠️  Storage type not specified, will default to MMap in cluster mode");
                Ok(())
            }
        }
    }

    fn validate_file_watcher(&self) -> Result<()> {
        if self.file_watcher_config.enabled {
            error!(
                "❌ File Watcher must be disabled in cluster mode\n\
                 \n\
                 Fix: Set file_watcher.enabled to false\n\
                 \n\
                 file_watcher:\n\
                   enabled: false  # Required for cluster mode"
            );
            return Err(VectorizerError::ClusterConfigViolation(
                "File Watcher must be disabled in cluster mode".to_string(),
            ));
        }

        info!("✓ File Watcher: disabled (required for cluster mode)");
        Ok(())
    }
}
```

### 2. CacheMemoryManager

**Location**: `src/cache/memory_manager.rs` (NEW)

**Purpose**: Enforce global 1GB memory limit across all caches with LRU eviction.

```rust
//! Global cache memory manager for cluster mode
//!
//! Enforces memory limits across all cache types.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, warn, info};

use crate::error::{Result, VectorizerError};

/// Trait for caches that can be managed
pub trait ManagedCache: Send + Sync {
    /// Get current memory usage in bytes
    fn memory_usage(&self) -> usize;

    /// Evict items to free up memory
    /// Returns the number of bytes freed
    fn evict(&self, target_bytes: usize) -> usize;

    /// Cache name for logging
    fn name(&self) -> &str;
}

/// Global cache memory manager
pub struct CacheMemoryManager {
    /// Maximum total memory across all caches (bytes)
    max_memory_bytes: usize,

    /// Current total memory usage (bytes)
    current_usage: Arc<AtomicUsize>,

    /// Registered caches
    caches: Arc<RwLock<Vec<Arc<dyn ManagedCache>>>>,

    /// Total evictions performed
    total_evictions: Arc<AtomicUsize>,
}

impl CacheMemoryManager {
    /// Create new cache memory manager
    pub fn new(max_memory_bytes: usize) -> Self {
        info!(
            "Initializing CacheMemoryManager with {}MB limit",
            max_memory_bytes / 1_048_576
        );

        Self {
            max_memory_bytes,
            current_usage: Arc::new(AtomicUsize::new(0)),
            caches: Arc::new(RwLock::new(Vec::new())),
            total_evictions: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Register a cache for management
    pub fn register_cache(&self, cache: Arc<dyn ManagedCache>) {
        let mut caches = self.caches.write();
        info!("Registering cache: {}", cache.name());
        caches.push(cache);
    }

    /// Try to allocate memory for caching
    pub fn try_allocate(&self, bytes: usize) -> Result<()> {
        let current = self.current_usage.load(Ordering::Relaxed);

        if current + bytes > self.max_memory_bytes {
            debug!(
                "Cache memory limit would be exceeded: current={}MB, requested={}MB, limit={}MB",
                current / 1_048_576,
                bytes / 1_048_576,
                self.max_memory_bytes / 1_048_576
            );

            // Trigger eviction
            self.evict_until_available(bytes)?;
        }

        self.current_usage.fetch_add(bytes, Ordering::Relaxed);
        Ok(())
    }

    /// Free allocated memory
    pub fn free(&self, bytes: usize) {
        self.current_usage.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// Get memory usage percentage
    pub fn usage_percentage(&self) -> f64 {
        let current = self.current_usage.load(Ordering::Relaxed);
        (current as f64 / self.max_memory_bytes as f64) * 100.0
    }

    /// Evict items until we have enough memory
    fn evict_until_available(&self, needed_bytes: usize) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut total_freed = 0;

        warn!(
            "Evicting cache items to free {}MB (current usage: {}%)",
            needed_bytes / 1_048_576,
            self.usage_percentage()
        );

        // Calculate how much we need to free
        let current = self.current_usage.load(Ordering::Relaxed);
        let target_free = if current + needed_bytes > self.max_memory_bytes {
            (current + needed_bytes) - self.max_memory_bytes
        } else {
            0
        };

        if target_free == 0 {
            return Ok(());
        }

        // Evict from all caches proportionally
        let caches = self.caches.read();
        
        for cache in caches.iter() {
            let cache_usage = cache.memory_usage();
            
            if cache_usage == 0 {
                continue;
            }

            // Evict proportionally based on cache size
            let proportion = cache_usage as f64 / current as f64;
            let cache_target = (target_free as f64 * proportion) as usize;

            debug!(
                "Evicting from cache '{}': target={}MB",
                cache.name(),
                cache_target / 1_048_576
            );

            let freed = cache.evict(cache_target);
            total_freed += freed;

            if total_freed >= target_free {
                break;
            }
        }

        // Update current usage
        self.current_usage.fetch_sub(total_freed, Ordering::Relaxed);
        self.total_evictions.fetch_add(1, Ordering::Relaxed);

        let elapsed = start_time.elapsed();

        info!(
            "✅ Evicted {}MB in {:?} (new usage: {}%)",
            total_freed / 1_048_576,
            elapsed,
            self.usage_percentage()
        );

        // Verify we have enough space now
        let new_current = self.current_usage.load(Ordering::Relaxed);
        if new_current + needed_bytes > self.max_memory_bytes {
            return Err(VectorizerError::CacheMemoryLimitExceeded(
                format!(
                    "Could not free enough memory. Current: {}MB, Needed: {}MB, Limit: {}MB",
                    new_current / 1_048_576,
                    needed_bytes / 1_048_576,
                    self.max_memory_bytes / 1_048_576
                ),
            ));
        }

        Ok(())
    }

    /// Get total number of evictions
    pub fn total_evictions(&self) -> usize {
        self.total_evictions.load(Ordering::Relaxed)
    }
}
```

### 3. Modified VectorStore

**Location**: `src/db/vector_store.rs` (MODIFIED)

**Changes**: Add storage type validation in cluster mode.

```rust
impl VectorStore {
    pub fn create_collection(&self, name: &str, mut config: CollectionConfig) -> Result<()> {
        // NEW: Validate storage type in cluster mode
        if let Some(ref cluster_config) = self.cluster_config {
            if cluster_config.enabled {
                // Force MMap if not specified or if Memory is specified
                match config.storage_type {
                    Some(StorageType::Memory) => {
                        return Err(VectorizerError::ClusterConfigViolation(
                            format!(
                                "Memory storage is not allowed in cluster mode for collection '{}'. \
                                 Use MMap storage instead.",
                                name
                            ),
                        ));
                    }
                    Some(StorageType::Mmap) => {
                        // OK - MMap is allowed
                    }
                    None => {
                        // Force MMap as default
                        info!(
                            "Collection '{}': forcing MMap storage (cluster mode)",
                            name
                        );
                        config.storage_type = Some(StorageType::Mmap);
                    }
                }
            }
        }

        // ... existing collection creation logic
    }
}
```

### 4. Modified FileWatcher

**Location**: `src/file_watcher/mod.rs` (MODIFIED)

**Changes**: Prevent start in cluster mode.

```rust
impl FileWatcher {
    pub async fn start(&mut self) -> Result<()> {
        // NEW: Check if cluster mode is enabled
        if let Some(ref cluster_config) = self.cluster_config {
            if cluster_config.enabled {
                return Err(VectorizerError::FileWatcherNotAllowedInCluster(
                    "File Watcher cannot run in cluster mode. \
                     File watching is only for local development. \
                     In cluster mode, indexing should be done via API/MCP.".to_string(),
                ));
            }
        }

        // ... existing start logic
    }
}
```

## Error Types

Add new error types to `src/error.rs`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum VectorizerError {
    // ... existing errors ...

    #[error("Cluster configuration violation: {0}")]
    ClusterConfigViolation(String),

    #[error("File Watcher not allowed in cluster mode: {0}")]
    FileWatcherNotAllowedInCluster(String),

    #[error("Cache memory limit exceeded: {0}")]
    CacheMemoryLimitExceeded(String),
}
```

## Configuration Schema

Update `ClusterConfig` in `src/cluster/mod.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Whether cluster mode is enabled
    pub enabled: bool,

    /// Maximum cache memory in bytes
    /// Default: 1GB (1,073,741,824 bytes) in cluster mode
    /// Minimum: 100MB, Recommended: 1GB, Maximum: 2GB
    #[serde(default = "default_cache_memory_bytes")]
    pub max_cache_memory_bytes: usize,

    /// Enforce MMap storage (always true when cluster enabled)
    #[serde(default = "default_true")]
    pub enforce_mmap_storage: bool,

    /// Disable file watcher (always true when cluster enabled)
    #[serde(default = "default_true")]
    pub disable_file_watcher: bool,

    // ... existing fields (node_id, servers, etc.)
}

fn default_cache_memory_bytes() -> usize {
    1_073_741_824 // 1GB
}

fn default_true() -> bool {
    true
}
```

## Startup Sequence

```
1. Load configuration from config.yml
   ↓
2. Parse cluster, file_watcher, collections configs
   ↓
3. Create ClusterConfigValidator
   ↓
4. Call validator.validate()
   ├─ Check cache limit (must be > 0, warn if > 2GB)
   ├─ Check storage type (must be MMap, not Memory)
   └─ Check file watcher (must be disabled)
   ↓
5. If validation fails:
   ├─ Print clear error message with fix instructions
   └─ Exit with error code 1
   ↓
6. If validation passes:
   ├─ Create CacheMemoryManager with limit
   ├─ Create VectorStore with cluster config
   ├─ Skip FileWatcher initialization
   └─ Start server
```

## Memory Budget Breakdown

For a 1GB cache limit in cluster mode:

```
Total Available: 1GB (1,073,741,824 bytes)

Allocation:
├─ Embedding Cache:    400MB (37.3%)  - Most frequently used
├─ Query Cache:        300MB (28.0%)  - Recent queries
├─ HNSW Index Cache:   200MB (18.6%)  - Hot index pages
├─ Metadata Cache:      50MB (4.7%)   - File info, payloads
└─ Reserve:             50MB (4.7%)   - Eviction buffer

Additional (outside 1GB limit):
├─ MMap Overhead:      ~50MB          - Index structures
├─ Per-Tenant Metadata: ~5MB/tenant   - Tracking structures
└─ System Overhead:    ~100MB         - Rust runtime, etc.

Total Process Memory: ~1.2GB for 10 tenants
```

## Performance Considerations

### Memory Allocation
- **Fast Path**: If under limit, allocation is O(1) atomic operation
- **Slow Path**: If over limit, eviction is O(n) where n = cache size
- **Target**: Eviction completes in < 100ms for 100MB

### Cache Eviction
- **Strategy**: LRU (Least Recently Used) across all caches
- **Granularity**: Evict proportionally from each cache
- **Batching**: Evict in 10MB batches for efficiency

### Monitoring
- **Metrics Collection**: Every 15 seconds
- **Alert Threshold**: 80% of limit
- **Dashboard Update**: Real-time

## Migration Guide

### Automatic Migration

```bash
# Migrate existing cluster config to new format
./vectorizer migrate-cluster-config \
  --input config.yml \
  --output config.cluster.yml \
  --cache-limit 1GB
```

### Manual Migration

**Before** (old config):
```yaml
cluster:
  enabled: true
```

**After** (new config):
```yaml
cluster:
  enabled: true
  max_cache_memory_bytes: 1073741824  # 1GB
  enforce_mmap_storage: true
  disable_file_watcher: true

collections:
  defaults:
    storage_type: "mmap"

file_watcher:
  enabled: false
```

## Testing Strategy

### Unit Tests
1. ClusterConfigValidator with valid/invalid configs
2. CacheMemoryManager allocation and eviction
3. VectorStore storage type enforcement
4. FileWatcher cluster mode prevention

### Integration Tests
1. Server startup with cluster mode
2. Collection creation in cluster mode
3. Cache limit enforcement under load
4. Memory usage monitoring

### Load Tests
1. 10 concurrent users with 1GB limit
2. Cache eviction performance benchmarks
3. 24-hour stability test
4. Memory leak detection

## Rollback Plan

If issues occur after deployment:

1. **Immediate**: Disable cluster mode in config
2. **Short-term**: Revert to previous version
3. **Long-term**: Fix issues and re-deploy with additional testing

