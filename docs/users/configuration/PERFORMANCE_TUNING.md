---
title: Performance Tuning
module: configuration
id: performance-tuning-configuration
order: 4
description: Performance optimization and tuning configuration
tags: [configuration, performance, optimization, tuning, threads, memory]
---

# Performance Tuning Configuration

Complete guide to optimizing Vectorizer performance through configuration.

## Thread Configuration

### Worker Threads

Configure the number of worker threads for parallel processing.

**Default:** Auto-detected based on CPU cores

**Command Line:**

```bash
vectorizer --workers 8
```

**Environment Variable:**

```bash
export VECTORIZER_WORKERS=8
```

**YAML Configuration:**

```yaml
performance:
  cpu:
    max_threads: 8
```

### Thread Count Guidelines

**Recommended:**

- **Small servers (1-2 cores)**: 2-4 threads
- **Medium servers (4-8 cores)**: 4-8 threads
- **Large servers (8+ cores)**: 8-16 threads

**Formula:**

```
threads = CPU_cores * 1.5 (for I/O-bound workloads)
threads = CPU_cores (for CPU-bound workloads)
```

**Note:** Too many threads can cause context switching overhead. Start with CPU core count and adjust based on workload.

## Memory Configuration

### Memory Limits

**Linux (systemd):**

```ini
[Service]
MemoryMax=4G
MemoryHigh=3G
```

**Docker:**

```yaml
services:
  vectorizer:
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G
```

### Memory Optimization

**Enable quantization:**

```yaml
quantization:
  enabled: true
  type: "scalar"
  bits: 8 # 4x memory reduction
```

**Enable compression:**

```yaml
compression:
  enabled: true
  threshold_bytes: 1024
  algorithm: "lz4"
```

**Memory pool:**

```yaml
performance:
  cpu:
    memory_pool_size_mb: 1024 # Pre-allocated memory pool
```

## GPU Configuration

### macOS Metal GPU

**Enable GPU acceleration:**

```yaml
gpu:
  enabled: true
  device: "auto"
  batch_size: 1000
  fallback_to_cpu: true
  preferred_backend: "auto"
```

**Performance benefits:**

- 3-5x faster search operations
- 50-200x faster batch operations
- < 1ms search latency (vs 0.6-2.4ms CPU)

**Build with GPU support:**

```bash
cargo build --release --features hive-gpu
```

## Batch Processing

### Batch Size Configuration

**Default batch size:**

```yaml
performance:
  batch:
    default_size: 100
    max_size: 1000
    parallel_processing: true
```

**Optimal batch sizes:**

- **Small batches (10-100)**: Real-time updates, low latency
- **Medium batches (100-1000)**: Balanced performance (recommended)
- **Large batches (1000-10000)**: Maximum throughput, bulk indexing

### Parallel Batch Processing

**Enable parallel processing:**

```yaml
performance:
  batch:
    parallel_processing: true
```

**Benefits:**

- Faster batch operations
- Better CPU utilization
- Improved throughput

## Query Caching

### Cache Configuration

**Enable query cache:**

```yaml
performance:
  query_cache:
    enabled: true
    max_size: 1000
    ttl_seconds: 300
    warmup_enabled: false
```

**Parameters:**

- `max_size`: Maximum cached queries (LRU eviction)
- `ttl_seconds`: Cache entry time-to-live
- `warmup_enabled`: Pre-populate cache on startup

**Cache benefits:**

- Faster repeated queries
- Reduced CPU usage
- Lower latency for common queries

## Collection Optimization

### HNSW Index Tuning

**High-speed configuration:**

```yaml
index:
  type: "hnsw"
  hnsw:
    m: 8 # Fewer connections (faster)
    ef_construction: 100 # Faster build
    ef_search: 32 # Faster search
```

**High-quality configuration:**

```yaml
index:
  type: "hnsw"
  hnsw:
    m: 32 # More connections (better recall)
    ef_construction: 400 # Better build quality
    ef_search: 128 # Better search quality
```

**Balanced configuration:**

```yaml
index:
  type: "hnsw"
  hnsw:
    m: 16 # Balanced
    ef_construction: 200 # Balanced
    ef_search: 64 # Balanced
```

### Quantization Settings

**Memory-optimized:**

```yaml
quantization:
  enabled: true
  type: "scalar"
  bits: 4 # Maximum memory savings (8x reduction)
```

**Balanced:**

```yaml
quantization:
  enabled: true
  type: "scalar"
  bits: 8 # Recommended (4x reduction, <2% accuracy loss)
```

**Accuracy-optimized:**

```yaml
quantization:
  enabled: true
  type: "scalar"
  bits: 16 # Minimal accuracy loss (2x reduction)
```

## Search Optimization

### Search Parameters

**Fast search:**

```python
results = await client.search(
    "collection",
    "query",
    limit=5,  # Request only what you need
    similarity_threshold=0.5  # Filter early
)
```

**Optimized intelligent search:**

```python
results = await client.intelligent_search(
    "collection",
    IntelligentSearchConfig(
        query="query",
        max_results=5,  # Reduced from default 15
        mmr_enabled=False,  # Disable for speed
        domain_expansion=False,  # Disable for speed
        technical_focus=True
    )
)
```

### Search Performance Tips

1. **Limit results**: Request only what you need
2. **Use similarity thresholds**: Filter low-quality results early
3. **Choose appropriate search method**: Basic search is fastest
4. **Enable quantization**: Faster search with less memory
5. **Optimize HNSW ef_search**: Lower = faster (but less accurate)

## Insertion Optimization

### Batch Insertion

**Always use batch operations:**

```python
# ✅ Good: Batch insert
await client.batch_insert_text("collection", texts)

# ❌ Bad: Individual inserts
for text in texts:
    await client.insert_text("collection", text)
```

**Optimal batch size:**

- 500-1000 vectors per batch
- Adjust based on vector size and memory

### Parallel Insertion

**Python example:**

```python
import asyncio

async def parallel_insert(collection, texts, batch_size=1000, workers=4):
    """Insert texts in parallel batches."""
    tasks = []

    for i in range(0, len(texts), batch_size):
        batch = texts[i:i + batch_size]
        task = client.batch_insert_text(collection, batch)
        tasks.append(task)

        if len(tasks) >= workers:
            await asyncio.gather(*tasks)
            tasks = []

    if tasks:
        await asyncio.gather(*tasks)
```

## System-Level Optimization

### CPU Affinity

**Linux (taskset):**

```bash
# Pin to specific CPU cores
taskset -c 0-3 vectorizer --workers 4
```

**Docker:**

```yaml
services:
  vectorizer:
    cpuset: "0-3" # Use cores 0-3
```

### I/O Optimization

**Use SSD storage:**

- Faster random I/O for HNSW index
- Better performance for large collections

**Network optimization:**

- Use localhost for local clients
- Minimize network latency for remote clients

### Operating System Tuning

**Linux (sysctl):**

```bash
# Increase file descriptor limits
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# TCP optimization
echo "net.core.somaxconn = 1024" >> /etc/sysctl.conf
sysctl -p
```

## Monitoring Performance

### Key Metrics

**Search latency:**

- Target: < 10ms for small collections (<10K vectors)
- Target: < 20ms for medium collections (10K-100K vectors)

**Throughput:**

- Insert: 4,000+ vectors/second
- Search: 1,000+ QPS (small collections)

**Memory usage:**

- Monitor per collection
- Enable quantization if > 2GB per 1M vectors

### Performance Monitoring

**Prometheus metrics:**

```bash
curl http://localhost:15002/prometheus/metrics | grep latency
```

**Health endpoint:**

```bash
curl http://localhost:15002/health
```

## Configuration Examples

### High-Performance Server

```yaml
server:
  host: "0.0.0.0"
  port: 15002

performance:
  cpu:
    max_threads: 16
    memory_pool_size_mb: 2048
  batch:
    default_size: 1000
    max_size: 5000
    parallel_processing: true
  query_cache:
    enabled: true
    max_size: 2000
    ttl_seconds: 600

gpu:
  enabled: true
  batch_size: 2000

collections:
  defaults:
    quantization:
      type: "sq"
      sq:
        bits: 8
    index:
      hnsw:
        m: 16
        ef_construction: 200
        ef_search: 64
```

### Memory-Optimized Server

```yaml
performance:
  cpu:
    max_threads: 4
    memory_pool_size_mb: 512

collections:
  defaults:
    quantization:
      type: "sq"
      sq:
        bits: 4 # Maximum memory savings
    compression:
      enabled: true
      threshold_bytes: 512
```

### Balanced Configuration

```yaml
performance:
  cpu:
    max_threads: 8
  batch:
    default_size: 500
  query_cache:
    enabled: true
    max_size: 1000

collections:
  defaults:
    quantization:
      type: "sq"
      sq:
        bits: 8
    index:
      hnsw:
        m: 16
        ef_search: 64
```

## Troubleshooting Performance

### Slow Searches

**Check:**

1. Collection size and dimension
2. HNSW index is built
3. ef_search parameter (lower = faster)
4. System resources (CPU, memory)

**Solutions:**

- Lower ef_search (32-48 for speed)
- Enable quantization
- Reduce search limit
- Use GPU acceleration (macOS)

### High Memory Usage

**Check:**

1. Collection count and size
2. Quantization enabled
3. Compression enabled
4. Memory leaks

**Solutions:**

- Enable quantization (4x reduction)
- Enable compression
- Delete unused collections
- Lower HNSW M parameter

### Low Throughput

**Check:**

1. Thread count
2. Batch size
3. Network latency
4. Disk I/O

**Solutions:**

- Increase worker threads
- Use larger batches
- Use localhost for local clients
- Use SSD storage

## Related Topics

- [Server Configuration](./SERVER.md) - Server settings
- [Collection Configuration](../collections/CONFIGURATION.md) - Collection optimization
- [Server Configuration](./SERVER.md) - Server settings
- [Monitoring Guide](../operations/MONITORING.md) - Performance monitoring
