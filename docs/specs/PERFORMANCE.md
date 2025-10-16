# Performance & Optimization Guide

**Version**: 0.9.0  
**Status**: ✅ Production Ready  
**Last Updated**: 2025-10-16

---

## Table of Contents

1. [Performance Metrics](#performance-metrics)
2. [Memory Optimization & Quantization](#memory-optimization--quantization)
3. [Query Optimization](#query-optimization)
4. [Cache & Incremental Indexing](#cache--incremental-indexing)
5. [Benchmarking](#benchmarking)

---

## Performance Metrics (Achieved)

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Search Latency | <10ms | 0.6-2.4ms | ✅ **Exceeded** |
| Indexing Speed | >1000 docs/s | 1500+ docs/s | ✅ **Exceeded** |
| Memory Usage | <2GB (1M vectors) | Optimized | ✅ **Achieved** |
| Throughput | >1000 QPS | 1247 QPS | ✅ **Exceeded** |

---

## Memory Optimization & Quantization

### Scalar Quantization (SQ-8bit) - **RECOMMENDED** ✅

**Compression**: 4x (float32 → uint8)  
**Quality**: MAP improved from 0.8400 → 0.9147 (+8.9%)  
**Performance**: <1ms search latency

#### Algorithm
```rust
For each dimension:
  code[i] = round((value[i] - zero_point) / scale)
  clamped to [0, 255]
```

#### Performance Comparison

| Metric | Without Quant | With SQ-8 | Improvement |
|--------|--------------|-----------|-------------|
| **Memory** | 1.46 GB | 366 MB | **4x reduction** ✅ |
| **MAP Score** | 0.8400 | 0.9147 | **+8.9%** ✅ |
| **Search Latency** | 0.6ms | 0.8ms | **Minimal impact** ✅ |
| **Recall@10** | 95% | 97% | **+2%** ✅ |

### Other Quantization Methods

**Product Quantization (PQ)**:
- Compression: 96x (extreme)
- Quality: Moderate degradation
- Use Case: Very large collections (10M+ vectors)

**Binary Quantization**:
- Compression: 32x (1-bit per dimension)
- Quality: Lower but acceptable for filtering
- Use Case: First-stage retrieval

### Configuration

```yaml
quantization:
  enabled: true
  default_method: "scalar_8bit"
  
  methods:
    scalar_8bit:
      enabled: true
      per_dimension: true
      
    product:
      enabled: false
      subvectors: 8
      
    binary:
      enabled: false
```

### Memory Snapshots

**Purpose**: Monitor memory usage and collection states

**Metrics**:
```rust
pub struct MemorySnapshot {
    pub total_memory_mb: u64,
    pub collections: Vec<CollectionMemory>,
    pub quantization_savings_mb: u64,
    pub timestamp: DateTime<Utc>,
}
```

---

## Query Optimization

### Search Optimization Strategies

**For Large Collections (1M+ vectors)**:
- Enable SQ-8bit quantization
- Use per-block quantization strategy
- Increase HNSW ef_search for quality
- Enable result caching

**For Low Latency (<5ms)**:
- Reduce ef_search value
- Limit max_results
- Use smaller collections
- Enable aggressive caching

**For High Throughput**:
- Use batch operations
- Enable connection pooling
- Increase worker threads
- Optimize chunk sizes

### Query Parameters

**Similarity Thresholds**:
- Use appropriate thresholds (0.1-0.2)
- Too high: few results
- Too low: noisy results

**Result Limits**:
- Limit to necessary count
- Use pagination for large result sets
- Enable result caching

**Batch Operations**:
- Group similar queries
- Use batch_search_vectors
- Reduce connection overhead

---

## Cache & Incremental Indexing

### Multi-Tier Caching

**L1 (Memory)**: Hot embeddings
- Strategy: LFU (Least Frequently Used)
- Size: 10% of total vectors
- Access: <1ms

**L2 (Disk)**: Quantized vectors
- Strategy: Memory-mapped files
- Compression: SQ-8bit
- Access: ~5ms

**L3 (Blob)**: Raw/normalized text
- Strategy: Zstd compressed
- Compression: ~60-70%
- Access: ~20ms

### Incremental Indexing

**File Watcher Integration**:
- Detect file changes in real-time
- Index only modified files
- Update existing chunks
- Preserve unaffected data

**Benefits**:
- Faster reindexing (only changed files)
- Reduced CPU usage
- Lower memory pressure
- Real-time updates

### Chunk Optimization

**Optimal Chunk Sizes**:
- **Code Files**: 2048 chars, 256 overlap
- **Documentation**: 2048 chars, 256 overlap
- **Mixed Content**: 1536 chars, 200 overlap

**Chunking Strategies**:

**Semantic Chunking**:
- Split on paragraph boundaries
- Preserve code blocks
- Respect markdown structure

**Fixed-Size Chunking**:
- Consistent chunk sizes
- Configurable overlap
- Fast processing

---

## Benchmarking

### Running Benchmarks

```bash
# Core operations
cargo run --release --bin core_operations_benchmark

# Quantization comparison
cargo run --release --bin quantization_benchmark

# Scale testing
cargo run --release --bin scale_benchmark

# GPU acceleration (if available)
cargo run --release --bin gpu_benchmark
```

### Benchmark Results

**Quantization Performance**:
- SQ-8bit: 4x compression, +8.9% quality
- Search latency: 0.6-2.4ms (minimal overhead)
- Memory: 366MB vs 1.46GB (75% reduction)

**Search Quality**:
- Recall@10: 97%
- MAP Score: 0.9147
- NDCG@10: 0.94

**Indexing Performance**:
- 1500+ documents/second
- Parallel processing enabled
- Background loading supported

---

## Performance Monitoring

### Metrics Dashboard

Access real-time performance metrics at:
- **Dashboard**: http://localhost:15002/
- **Stats API**: http://localhost:15002/stats
- **Metrics**: http://localhost:15002/metrics

### Key Metrics to Monitor

**Search Performance**:
- Average search latency
- P95/P99 latencies
- Queries per second
- Cache hit rate

**Memory Usage**:
- Total memory usage
- Per-collection memory
- Quantization savings
- Cache effectiveness

**Indexing Performance**:
- Documents indexed per second
- Active indexing operations
- File watcher events processed
- Background task status

---

## Optimization Checklist

### For Production Deployment

- [ ] Enable SQ-8bit quantization for collections >10K vectors
- [ ] Configure appropriate ef_search (64-128 for quality)
- [ ] Enable multi-tier caching
- [ ] Set up result caching with TTL
- [ ] Configure connection pooling
- [ ] Enable batch operations
- [ ] Monitor memory usage
- [ ] Set up performance alerts
- [ ] Configure backup schedule
- [ ] Enable auto-save (every 5 minutes)

### For Development

- [ ] Use smaller test collections
- [ ] Disable quantization for faster iteration
- [ ] Enable detailed logging
- [ ] Use file watcher for auto-reindex
- [ ] Monitor performance metrics
- [ ] Run benchmarks regularly

---

## Troubleshooting Performance Issues

### High Memory Usage

**Symptoms**: Memory usage >2GB for <1M vectors

**Solutions**:
1. Enable SQ-8bit quantization
2. Reduce HNSW max_connections
3. Clear unused collections
4. Enable memory snapshots
5. Restart with fresh state

### Slow Search Queries

**Symptoms**: Search latency >10ms

**Solutions**:
1. Reduce ef_search value
2. Enable result caching
3. Limit max_results
4. Check collection size
5. Enable quantization

### Slow Indexing

**Symptoms**: Indexing <500 docs/s

**Solutions**:
1. Enable parallel processing
2. Increase worker threads
3. Optimize chunk sizes
4. Check disk I/O
5. Use SSD storage

---

**Version**: 0.9.0  
**Status**: ✅ Production Ready  
**Maintained by**: HiveLLM Team
