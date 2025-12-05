# MMap Storage Implementation Summary

## Overview
We have implemented memory-mapped file storage for dense vectors to support datasets larger than available RAM. This implementation allows the vectorizer to scale to millions of vectors while keeping memory usage manageable.

## Changes Implemented

### 1. `MmapVectorStorage` (`src/storage/mmap.rs`)
- Implemented a wrapper around `memmap2::MmapMut`.
- Supports appending vectors (append-only).
- Supports random access reading by index.
- Automatically handles file resizing/growth.
- **Limitation**: Only stores dense vector data (`Vec<f32>`). Payloads and sparse vectors are not stored in MMap.

### 2. `VectorStorageBackend` (`src/db/storage_backend.rs`)
- Created an abstraction enum `VectorStorageBackend` to support both `Memory` (HashMap) and `Mmap` backends.
- **Memory Mode**: Uses `HashMap<String, Vector>` (legacy behavior, fast, high RAM usage).
- **Mmap Mode**: 
  - Stores dense vectors in `MmapVectorStorage` (disk-backed).
  - Stores ID mapping (`String` -> `usize`), Payloads, and Sparse Vectors in memory (`HashMap`).
  - This hybrid approach significantly reduces memory usage for the bulk of data (dense vectors) while maintaining fast lookups for metadata.

### 3. `Collection` Integration (`src/db/collection.rs`)
- Refactored `Collection` to use `VectorStorageBackend` instead of direct `HashMap`.
- Updated `Collection::new` to initialize the backend based on `CollectionConfig`.
- Updated all CRUD operations (`insert`, `get`, `delete`, `update`) to use the backend abstraction.

### 4. Configuration (`src/models/mod.rs`)
- Added `StorageType` enum (`Memory`, `Mmap`) to `CollectionConfig`.
- Default is `Memory` to preserve backward compatibility.

## Usage

To use MMap storage, configure the collection with `storage_type: StorageType::Mmap`:

```rust
let config = CollectionConfig {
    dimension: 1536,
    metric: DistanceMetric::Cosine,
    hnsw_config: HnswConfig::default(),
    quantization: QuantizationConfig::None,
    compression: CompressionConfig::default(),
    normalization: None,
    storage_type: StorageType::Mmap, // Enable MMap
};
```

## Performance Implications
- **Writes**: Slightly slower due to disk I/O (OS page cache helps).
- **Reads**: Fast if data is in OS page cache. If data is cold, incurs disk latency.
- **Memory**: Significantly reduced. Only metadata and HNSW graph structure remain in RAM. Dense vectors (usually >90% of size) are offloaded to OS cache/disk.

## Cluster Mode Requirements

**Important**: When running in cluster mode, MMap storage is **mandatory**.

### Why MMap is Required for Clusters

Cluster deployments require predictable memory usage for:

1. **Memory Limits**: The `CacheMemoryManager` enforces strict memory limits (default: 1GB) to prevent OOM crashes
2. **Multi-Tenancy**: Multiple tenants share cluster resources, requiring fair memory distribution
3. **Stability**: Memory storage can grow unbounded, while MMap offloads to OS page cache
4. **Capacity Planning**: MMap usage is predictable and can be monitored via Prometheus metrics

### Cluster Configuration

```yaml
cluster:
  enabled: true
  memory:
    # Enforces MMap storage for all collections (default: true)
    enforce_mmap_storage: true

    # Maximum cache memory across all caches (default: 1GB)
    max_cache_memory_bytes: 1073741824

    # Fail startup if Memory storage is configured (default: true)
    strict_validation: true
```

### Validation Behavior

When `cluster.memory.enforce_mmap_storage: true`:
- Server startup validates that no collections use `StorageType::Memory`
- `ClusterConfigValidator` rejects configurations with Memory storage
- Error message: `MemoryStorageNotAllowed: Memory storage type is not allowed in cluster mode`

### Migration from Memory to MMap

If you have existing Memory-based collections, you must migrate before enabling cluster mode:

1. Export collection data via REST API
2. Recreate collection with `storage_type: Mmap`
3. Re-import data
4. Enable cluster mode

See [CLUSTER_MEMORY.md](./CLUSTER_MEMORY.md) for detailed migration guidance.

## Next Steps
- **Persistence**: Currently, the `id_map`, `payloads`, and `sparse` maps in `Mmap` mode are in-memory only. They need to be persisted to disk (e.g., via RocksDB or a custom WAL/snapshot) to survive restarts. The current implementation relies on the existing `save_to_disk` mechanism which might need adjustment to avoid loading everything into RAM during save.
- **Full Persistence**: Phase 2 (WAL) will address robust persistence.
