# Proposal: enforce-cluster-memory-limits

## Why

In cluster mode, multiple users share the same Vectorizer instance. Without strict memory control, this causes:

1. **OOM (Out of Memory)**: One user with large datasets can consume all available RAM
2. **Performance Degradation**: Insufficient memory forces swap, degrading performance for everyone
3. **Cluster Instability**: Memory crashes affect all users
4. **High Infrastructure Costs**: Need to over-provision memory beyond actual requirements

**Currently**:
- **MMap** (Memory-Mapped storage) implementation exists in `src/storage/mmap.rs`
- MMap allows storing vectors on disk with direct access, without loading everything into RAM
- Configurable cache in `src/cache/advanced_cache.rs` with `max_memory_bytes`
- File Watcher consumes memory monitoring file changes
- In cluster mode, these optimizations are not mandatory

**Main Problem**:
- Cluster mode doesn't enforce MMap (still uses Memory storage by default)
- No global cache limit (1GB as per requirement)
- File Watcher enabled in cluster mode (unnecessary and consumes memory)
- Missing configuration validation on startup for cluster mode

## What Changes

### 1. Make MMap Mandatory in Cluster Mode
- Detect when `cluster.enabled = true`
- Force all collections to use `StorageType::Mmap`
- Reject collection creation with `StorageType::Memory` in cluster mode
- Convert existing collections to MMap on startup (if needed)

### 2. Implement Global Cache Limit (1GB)
- Add `max_cache_memory_bytes` to cluster configuration
- Default: **1GB (1,073,741,824 bytes)** in cluster mode
- Apply limit across all caches:
  - `AdvancedCache` (embedding cache, query cache, etc.)
  - HNSW index cache
  - Metadata cache
- Monitor cache usage and force eviction when limit is reached

### 3. Disable File Watcher in Cluster Mode
- Force `file_watcher.enabled = false` when `cluster.enabled = true`
- File watcher is only useful for local development
- In cluster, indexing is done via API/MCP, not file monitoring
- Reduces memory and CPU usage

### 4. Configuration Validation on Startup
- Add `ClusterConfigValidator` that checks:
  - MMap is configured (reject Memory storage)
  - Cache limit is configured (≤ 1GB)
  - File watcher is disabled
- Fail on startup if configuration is invalid
- Clear logging of unmet requirements

### 5. Memory Usage Monitoring
- Add metric `vectorizer_cluster_memory_usage_bytes`
- Add metric `vectorizer_cluster_cache_usage_bytes`
- Dashboard showing memory usage per tenant (if multi-tenant)
- Alerts when usage > 80% of limit

## Impact

### Affected Specs
- `docs/specs/CLUSTER.md` - Cluster mode requirements (TO CREATE)
- `docs/specs/MMAP_IMPLEMENTATION.md` - MMap mandatory in cluster
- `docs/specs/FILE_WATCHER.md` - Disabled in cluster mode
- `docs/specs/API_REFERENCE.md` - Error responses for invalid configs

### Affected Code
- `src/cluster/mod.rs` - Add memory limit config
- `src/cluster/validator.rs` - TO CREATE: Config validation
- `src/db/collection.rs` - Force MMap in cluster mode
- `src/db/vector_store.rs` - Storage type validation
- `src/file_watcher/mod.rs` - Prevent start in cluster mode
- `src/cache/advanced_cache.rs` - Apply global memory limit
- `src/server/mod.rs` - Startup validation
- `src/monitoring/metrics.rs` - Add memory metrics
- `config.example.yml` - Document cluster requirements

### Breaking Change
**YES** - For existing cluster configurations:

**Before** (allowed):
```yaml
cluster:
  enabled: true
  # ... other settings

collections:
  defaults:
    storage_type: "memory"  # ❌ Will be rejected

file_watcher:
  enabled: true  # ❌ Will be rejected
```

**After** (required):
```yaml
cluster:
  enabled: true
  max_cache_memory_bytes: 1073741824  # 1GB mandatory
  # ... other settings

collections:
  defaults:
    storage_type: "mmap"  # ✅ Required

file_watcher:
  enabled: false  # ✅ Required
```

**Migration Path**:
1. Server prints warnings before failing
2. Grace period of 1 version (warnings only)
3. Next version: fail hard if configuration is invalid
4. Automatic config migration script

### User Benefit

**For Cluster Operators**:
- ✅ Predictable memory usage (max 1GB cache + MMap overhead)
- ✅ Higher user density per server
- ✅ Lower infrastructure costs (less RAM required)
- ✅ Greater stability (no OOM crashes)

**For End Users**:
- ✅ Consistent performance (no degradation from memory shortage)
- ✅ Higher availability (more stable cluster)
- ✅ Support for larger datasets (MMap + disk vs. limited RAM)

**Example Savings**:
- **Before**: 10 users × 2GB RAM each = 20GB RAM required
- **After**: 10 users × 1GB cache + MMap = ~12GB RAM total
- **Savings**: 40% reduction in infrastructure costs
