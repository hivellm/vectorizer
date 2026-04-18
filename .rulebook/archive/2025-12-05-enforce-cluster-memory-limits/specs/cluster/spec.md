# Cluster Memory Limits Specification

## ADDED Requirements

### Requirement: Mandatory MMap Storage in Cluster Mode
The system SHALL enforce Memory-Mapped storage for all collections in cluster mode.

##### Scenario: Create Collection in Cluster Mode with Memory Storage
Given cluster mode is enabled (`cluster.enabled = true`)
When a user attempts to create a collection with `storage_type: Memory`
Then the system MUST reject the request
And the system MUST return error `ClusterConfigViolation`
And the error message MUST indicate MMap is required in cluster mode

##### Scenario: Create Collection in Cluster Mode with MMap Storage
Given cluster mode is enabled (`cluster.enabled = true`)
When a user creates a collection with `storage_type: Mmap`
Then the system MUST accept the request
And the system MUST create the collection successfully
And the system MUST use disk-backed storage for vectors

##### Scenario: Existing Memory Collections on Cluster Startup
Given cluster mode is enabled (`cluster.enabled = true`)
And existing collections use Memory storage
When the server starts
Then the system MUST log a warning about incompatible storage
And the system MAY offer automatic migration to MMap
And the system MUST prevent new Memory collections

### Requirement: Global Cache Memory Limit
The system SHALL enforce a maximum cache memory limit of 1GB in cluster mode.

##### Scenario: Cache Usage Below Limit
Given cluster mode is enabled
And cache memory limit is 1GB
And current cache usage is 800MB
When a new item is cached
Then the system MUST cache the item successfully
And the system MUST update cache usage metrics

##### Scenario: Cache Usage Exceeds Limit
Given cluster mode is enabled
And cache memory limit is 1GB
And current cache usage is 1000MB
When a new item attempts to be cached
Then the system MUST evict least recently used items
And the system MUST free sufficient memory for the new item
And the system MUST cache the new item
And the system MUST keep total usage ≤ 1GB

##### Scenario: Cache Eviction Under Memory Pressure
Given cluster mode is enabled
And cache is at memory limit (1GB)
When system needs to cache 100MB of new data
Then the system MUST evict at least 100MB of old data
And the system MUST use LRU (Least Recently Used) eviction policy
And the system MUST complete eviction within 100ms

### Requirement: File Watcher Disabled in Cluster Mode
The system SHALL prevent File Watcher from running in cluster mode.

##### Scenario: Start File Watcher in Cluster Mode
Given cluster mode is enabled (`cluster.enabled = true`)
When the File Watcher attempts to start
Then the system MUST prevent the start
And the system MUST return error `FileWatcherNotAllowedInCluster`
And the system MUST log a clear error message

##### Scenario: Server Startup with File Watcher Enabled in Config
Given cluster mode is enabled
And configuration has `file_watcher.enabled = true`
When the server starts
Then the system MUST fail startup validation
And the system MUST log error about incompatible configuration
And the system MUST provide instructions to fix configuration

##### Scenario: Server Startup with File Watcher Disabled
Given cluster mode is enabled
And configuration has `file_watcher.enabled = false`
When the server starts
Then the system MUST pass validation
And the system MUST start successfully
And the system MUST NOT initialize File Watcher

### Requirement: Cluster Configuration Validation
The system SHALL validate cluster configuration on startup.

##### Scenario: Valid Cluster Configuration
Given the configuration file contains:
```yaml
cluster:
  enabled: true
  max_cache_memory_bytes: 1073741824

collections:
  defaults:
    storage_type: "mmap"

file_watcher:
  enabled: false
```
When the server starts
Then the system MUST pass all validation checks
And the system MUST start successfully
And the system MUST log cluster mode is active

##### Scenario: Invalid Storage Type in Cluster
Given the configuration file contains:
```yaml
cluster:
  enabled: true

collections:
  defaults:
    storage_type: "memory"
```
When the server starts
Then the system MUST fail validation
And the system MUST print error message
And the error MUST include "Memory storage not allowed in cluster mode"
And the error MUST include "Set storage_type to 'mmap'"

##### Scenario: Missing Cache Limit in Cluster
Given the configuration file contains:
```yaml
cluster:
  enabled: true
  # max_cache_memory_bytes not set
```
When the server starts
Then the system MUST use default value (1GB)
And the system MUST log the default being used
And the system MUST start successfully

## MODIFIED Requirements

### Requirement: Collection Creation
**BEFORE**: Collections could use any storage type (Memory or MMap) regardless of mode.

**AFTER**: In cluster mode, collections MUST use MMap storage exclusively.

##### Delta: Storage Type Validation
```rust
// ADDED: In src/db/vector_store.rs

impl VectorStore {
    pub fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        // NEW: Validate storage type in cluster mode
        if let Some(cluster_config) = &self.cluster_config {
            if cluster_config.enabled {
                if config.storage_type == Some(StorageType::Memory) {
                    return Err(VectorizerError::ClusterConfigViolation(
                        "Memory storage is not allowed in cluster mode. Use MMap storage.".to_string()
                    ));
                }
                
                // Force MMap if not specified
                let mut config = config;
                if config.storage_type.is_none() {
                    config.storage_type = Some(StorageType::Mmap);
                }
            }
        }
        
        // ... existing collection creation logic
    }
}
```

### Requirement: Cache Management
The system SHALL enforce a global cache limit across all cache types in cluster mode.

**BEFORE**: Caches had individual limits but no global limit enforced.

**AFTER**: In cluster mode, all caches SHALL share a global 1GB limit.

##### Delta: Global Cache Manager
```rust
// ADDED: In src/cache/memory_manager.rs

pub struct CacheMemoryManager {
    /// Maximum total memory across all caches (bytes)
    max_memory_bytes: usize,
    
    /// Current total memory usage (bytes)
    current_usage: Arc<AtomicUsize>,
    
    /// Registered caches
    caches: Arc<RwLock<Vec<Box<dyn Cache>>>>,
}

impl CacheMemoryManager {
    pub fn new(max_memory_bytes: usize) -> Self {
        Self {
            max_memory_bytes,
            current_usage: Arc::new(AtomicUsize::new(0)),
            caches: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn try_allocate(&self, bytes: usize) -> Result<()> {
        let current = self.current_usage.load(Ordering::Relaxed);
        
        if current + bytes > self.max_memory_bytes {
            // Trigger eviction
            self.evict_until_available(bytes)?;
        }
        
        self.current_usage.fetch_add(bytes, Ordering::Relaxed);
        Ok(())
    }
    
    fn evict_until_available(&self, needed_bytes: usize) -> Result<()> {
        // Evict LRU items across all caches until we have space
        // ...
    }
}
```

## Configuration Requirements

### ClusterConfig Extension
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Whether cluster mode is enabled
    pub enabled: bool,
    
    /// Maximum cache memory in bytes (mandatory in cluster mode)
    /// Default: 1GB (1,073,741,824 bytes)
    #[serde(default = "default_cache_memory_bytes")]
    pub max_cache_memory_bytes: usize,
    
    /// Enforce MMap storage (always true in cluster)
    #[serde(default = "default_enforce_mmap")]
    pub enforce_mmap_storage: bool,
    
    /// Disable file watcher (always true in cluster)
    #[serde(default = "default_disable_file_watcher")]
    pub disable_file_watcher: bool,
    
    // ... existing fields
}

fn default_cache_memory_bytes() -> usize {
    1_073_741_824 // 1GB
}

fn default_enforce_mmap() -> bool {
    true
}

fn default_disable_file_watcher() -> bool {
    true
}
```

### config.yml Extension
```yaml
# =============================================================================
# CLUSTER MODE CONFIGURATION
# =============================================================================
cluster:
  # Enable cluster mode (enforces memory limits, MMap storage, disables file watcher)
  enabled: false
  
  # Node configuration
  node_id: "node-1"
  
  # MEMORY MANAGEMENT (Mandatory in cluster mode)
  # Maximum total cache memory usage across all caches
  # Default: 1GB (1,073,741,824 bytes)
  # Recommended: 1GB for multi-tenant, 2GB for single-tenant
  max_cache_memory_bytes: 1073741824
  
  # Enforce MMap storage (always true in cluster mode)
  # Collections using Memory storage will be rejected
  enforce_mmap_storage: true
  
  # Disable file watcher (always true in cluster mode)
  # File watcher is not compatible with cluster deployments
  disable_file_watcher: true
  
  # Cluster servers
  servers:
    - id: "node-1"
      address: "localhost:15002"
      grpc_port: 50051
    - id: "node-2"
      address: "localhost:15003"
      grpc_port: 50052

# =============================================================================
# COLLECTIONS - CLUSTER MODE REQUIREMENTS
# =============================================================================
collections:
  defaults:
    dimension: 512
    metric: "cosine"
    
    # IMPORTANT: In cluster mode, storage_type MUST be "mmap"
    # Memory storage is not allowed in cluster deployments
    storage_type: "mmap"  # Required for cluster mode

# =============================================================================
# FILE WATCHER - DISABLED IN CLUSTER MODE
# =============================================================================
file_watcher:
  # IMPORTANT: Must be disabled (false) in cluster mode
  # File watcher is only for local development
  enabled: false
```

## Error Responses

### ClusterConfigViolation Error
```json
{
  "error": "ClusterConfigViolation",
  "message": "Memory storage is not allowed in cluster mode",
  "details": {
    "violation": "storage_type_not_allowed",
    "current_value": "Memory",
    "required_value": "Mmap",
    "fix": "Set storage_type to 'mmap' in collection configuration"
  },
  "code": "CLUSTER_001"
}
```

### FileWatcherNotAllowedInCluster Error
```json
{
  "error": "FileWatcherNotAllowedInCluster",
  "message": "File Watcher cannot run in cluster mode",
  "details": {
    "reason": "File watcher is incompatible with cluster deployments",
    "fix": "Set file_watcher.enabled to false in configuration"
  },
  "code": "CLUSTER_002"
}
```

### CacheMemoryLimitExceeded Error
```json
{
  "error": "CacheMemoryLimitExceeded",
  "message": "Cache memory limit exceeded",
  "details": {
    "limit_bytes": 1073741824,
    "current_bytes": 1073741824,
    "requested_bytes": 104857600,
    "action": "Evicting LRU items to free memory"
  },
  "code": "CLUSTER_003"
}
```

## Performance Requirements

### Memory Usage Limits
- **Total Cache Memory**: MUST NOT exceed configured limit (default 1GB)
- **MMap Overhead**: Estimated 10-50MB for index structures
- **Per-Tenant Overhead**: ~1-5MB for metadata and tracking

### Cache Eviction Performance
- **Eviction Latency**: MUST complete within 100ms for 100MB eviction
- **Eviction Throughput**: MUST handle at least 1GB/second eviction rate
- **Lock Contention**: MUST use read-write locks to minimize blocking

### Startup Validation Performance
- **Validation Time**: MUST complete within 1 second
- **Config Parsing**: MUST be synchronous and fail-fast
- **Error Reporting**: MUST provide actionable error messages

## Monitoring Requirements

### Metrics
```
# Cache memory usage
vectorizer_cluster_cache_usage_bytes{type="embedding|query|hnsw|metadata"}

# Total memory usage
vectorizer_cluster_memory_usage_bytes{component="cache|mmap|metadata"}

# Cache evictions
vectorizer_cluster_cache_evictions_total{reason="memory_limit|size_limit|ttl"}

# Config violations
vectorizer_cluster_config_violations_total{type="storage|file_watcher|cache_limit"}
```

### Alerts
```yaml
# High memory usage alert
- alert: ClusterHighMemoryUsage
  expr: vectorizer_cluster_memory_usage_bytes > 0.8 * 1073741824
  for: 5m
  annotations:
    summary: "Cluster memory usage above 80%"
    
# Frequent evictions alert
- alert: ClusterFrequentEvictions
  expr: rate(vectorizer_cluster_cache_evictions_total[5m]) > 10
  for: 5m
  annotations:
    summary: "Cache evictions happening frequently"
```

## Testing Requirements

### Unit Tests
- ClusterConfigValidator MUST have 100% branch coverage
- Memory limit enforcement MUST be tested with various cache sizes
- Eviction logic MUST be tested under memory pressure

### Integration Tests
- Cluster startup MUST fail with invalid configuration
- Collection creation MUST respect storage type restrictions
- File watcher MUST not start in cluster mode

### Load Tests
- System MUST remain stable with 1GB cache limit for 24 hours
- System MUST support at least 10 concurrent users
- Memory usage MUST never exceed configured limit + 10% margin

