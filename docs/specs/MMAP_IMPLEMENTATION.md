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

## Next Steps
- **Persistence**: Currently, the `id_map`, `payloads`, and `sparse` maps in `Mmap` mode are in-memory only. They need to be persisted to disk (e.g., via RocksDB or a custom WAL/snapshot) to survive restarts. The current implementation relies on the existing `save_to_disk` mechanism which might need adjustment to avoid loading everything into RAM during save.
- **Full Persistence**: Phase 2 (WAL) will address robust persistence.
