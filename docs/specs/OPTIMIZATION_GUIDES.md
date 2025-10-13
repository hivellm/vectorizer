# Optimization Guides

**Version**: 1.0  
**Status**: ✅ Active  
**Last Updated**: 2025-10-01

---

## Cache & Incremental Indexing

### Cache Strategy

**Multi-Tier Caching**:
- **L1 (Memory)**: Hot embeddings (LFU, 10% of total)
- **L2 (Disk)**: Quantized vectors (Memory-mapped)
- **L3 (Blob)**: Raw/normalized text (Zstd compressed)

### Incremental Indexing

**File Watcher Integration**:
- Detect file changes
- Index only modified files
- Update existing chunks
- Preserve unaffected data

**Benefits**:
- Faster reindexing (only changed files)
- Reduced CPU usage
- Lower memory pressure
- Real-time updates

---

## Chunk Optimization

### Optimal Chunk Sizes

**Code Files**: 2048 chars, 256 overlap  
**Documentation**: 2048 chars, 256 overlap  
**Mixed Content**: 1536 chars, 200 overlap

### Chunking Strategies

**Semantic Chunking**:
- Split on paragraph boundaries
- Preserve code blocks
- Respect markdown structure

**Fixed-Size Chunking**:
- Consistent chunk sizes
- Configurable overlap
- Fast processing

---

## Performance Optimization

### Query Optimization
- Use appropriate similarity thresholds
- Limit result counts
- Enable caching
- Batch operations

### Indexing Optimization
- Parallel processing
- Incremental updates
- Background reindexing
- Memory-efficient loading

### Search Optimization
- Pre-computed embeddings
- Query caching
- Result caching
- SIMD operations

---

**Maintained by**: HiveLLM Team

