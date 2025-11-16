---
title: Performance Guide
module: performance
id: performance-guide
order: 1
description: Performance optimization and tuning guide for Vectorizer
tags: [performance, optimization, tuning, benchmarks]
---

# Performance Guide

Optimize Vectorizer for your specific use case and workload.

## Performance Metrics

Benchmark results from real-world testing (512-dimensional vectors, HNSW M=16, ef_construction=200).

### Search Latency

**Small collections (10K vectors):**

- **Search k=1**: 0.62 ms average (P95: 0.68 ms, P99: 0.75 ms)
- **Search k=10**: 0.61 ms average (P95: 0.67 ms, P99: 0.72 ms)
- **Search k=100**: 0.72 ms average (P95: 0.81 ms, P99: 0.90 ms)

**Medium collections (100K vectors):**

- **Search k=1**: 12.95 ms average (P95: 16.78 ms, P99: 21.06 ms)
- **Search k=10**: 13.95 ms average (P95: 20.03 ms, P99: 23.59 ms)
- **Search k=100**: 16.23 ms average (P95: 23.09 ms, P99: 27.77 ms)

**With quantization (SQ-8bit):**

- **Search latency**: 0.6-2.4 ms (minimal overhead)
- **Memory reduction**: 4x (1.46 GB → 366 MB for 1M vectors)
- **Quality improvement**: MAP score +8.9% (0.8400 → 0.9147)

**GPU acceleration (macOS Metal):**

- **Search latency**: < 1 ms (3-5x faster than CPU)
- **Batch operations**: 50-200x speedup

### Throughput

**Insert operations:**

- **Single insert**: 4,300-4,600 ops/sec (0.22-0.23 ms per vector)
- **Batch insert**: 4,200-4,400 ops/sec (119-125 ms per batch of 1000)
- **Concurrent mixed workload**: 108-1,742 ops/sec (depending on collection size)

**Search operations:**

- **Small collections (10K)**: 1,385-1,631 QPS
- **Medium collections (100K)**: 61-77 QPS
- **With quantization**: 1,247 QPS (measured)

**Update/Delete operations:**

- **Single update**: < 1 μs (near-instant)
- **Batch update**: 2,300-3,200 ops/sec
- **Batch delete**: < 25 μs per batch

### Memory Usage

**Without quantization:**

- **Memory per vector**: 4 bytes × dimension
- **Example**: 512-dim vectors = 2 KB per vector
- **1M vectors**: ~2 GB memory

**With 8-bit scalar quantization:**

- **Memory per vector**: 1 byte × dimension (4x reduction)
- **Example**: 512-dim vectors = 512 bytes per vector
- **1M vectors**: ~512 MB memory (75% savings)
- **Accuracy loss**: < 2% (MAP score actually improves)

**With 4-bit quantization:**

- **Memory per vector**: 0.5 bytes × dimension (8x reduction)
- **Example**: 512-dim vectors = 256 bytes per vector
- **1M vectors**: ~256 MB memory (87.5% savings)
- **Accuracy loss**: 3-5%

## Collection Configuration Optimization

### High-Speed Configuration

Optimize for fastest search:

```json
{
  "dimension": 384,
  "metric": "cosine",
  "hnsw_config": {
    "m": 8,
    "ef_construction": 100,
    "ef_search": 32
  },
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 8
  }
}
```

**Trade-offs:**

- ✅ Fastest search (0.6-0.8 ms for 10K vectors)
- ✅ Lower memory usage (4x reduction with quantization)
- ⚠️ Slightly lower recall (acceptable for most use cases)

**Benchmark results (10K vectors, 512-dim):**

- Search k=10: 0.61 ms average, 1,631 QPS
- Memory: ~20 MB (with quantization)

### High-Quality Configuration

Optimize for best search quality:

```json
{
  "dimension": 768,
  "metric": "cosine",
  "hnsw_config": {
    "m": 32,
    "ef_construction": 400,
    "ef_search": 128
  },
  "quantization": {
    "enabled": false
  }
}
```

**Trade-offs:**

- ✅ Highest recall and precision
- ✅ Best search quality
- ⚠️ Slower search (13-16 ms for 100K vectors)
- ⚠️ Higher memory usage (~2 GB for 1M vectors)

**Benchmark results (100K vectors, 512-dim):**

- Search k=10: 13.95 ms average, 71 QPS
- Memory: ~200 MB (without quantization)

### Balanced Configuration

Good balance of speed and quality:

```json
{
  "dimension": 384,
  "metric": "cosine",
  "hnsw_config": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 64
  },
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 8
  }
}
```

**Trade-offs:**

- ✅ Good balance
- ✅ Reasonable speed (0.6-14 ms depending on collection size)
- ✅ Good recall
- ✅ Memory efficient (4x reduction with quantization)

**Benchmark results:**

- Small collections (10K): 0.6-0.7 ms, 1,400+ QPS
- Medium collections (100K): 13-16 ms, 60-70 QPS
- Memory: 75% reduction with 8-bit quantization

## HNSW Parameter Tuning

### M Parameter (Connections per Layer)

Controls the number of connections in the graph:

- **Low (8)**: Faster, less memory, lower recall
- **Medium (16)**: Balanced (default)
- **High (32)**: Slower, more memory, higher recall

**Recommendations:**

- Small collections (<10K): m=8-16
- Medium collections (10K-1M): m=16-24
- Large collections (>1M): m=24-32

### EF Construction

Controls search width during index building:

- **Low (100)**: Faster build, lower quality
- **Medium (200)**: Balanced (default)
- **High (400)**: Slower build, higher quality

**Recommendations:**

- Fast indexing: ef_construction=100-150
- Balanced: ef_construction=200
- High quality: ef_construction=300-400

### EF Search

Controls search width during queries:

- **Low (32)**: Fastest, lower recall
- **Medium (64)**: Balanced
- **High (128+)**: Slower, higher recall

**Recommendations:**

- Fast search: ef_search=32-48
- Balanced: ef_search=64-96
- High recall: ef_search=128-256

## Quantization Optimization

### Memory Savings

| Bits | Memory Reduction | Accuracy Loss |
| ---- | ---------------- | ------------- |
| 16   | 25%              | <1%           |
| 8    | 50%              | 1-2%          |
| 4    | 75%              | 3-5%          |

### Choosing Bits

- **16 bits**: Maximum accuracy, minimal memory savings
- **8 bits**: Recommended (best balance)
- **4 bits**: Maximum memory savings, noticeable accuracy loss

## Search Optimization

### Limit Results Appropriately

```python
# Fast: Request only what you need
results = await client.search("collection", "query", limit=5)

# Slower: Requesting too many results
results = await client.search("collection", "query", limit=1000)
```

### Use Similarity Thresholds

```python
# Filter low-quality results early
results = await client.search(
    "collection",
    "query",
    limit=10,
    similarity_threshold=0.5  # Filter out low-similarity results
)
```

### Choose the Right Search Method

| Method             | Speed   | Quality   | Use Case              |
| ------------------ | ------- | --------- | --------------------- |
| Basic search       | Fastest | Good      | Simple queries        |
| Intelligent search | Slower  | Best      | Research, discovery   |
| Semantic search    | Medium  | Excellent | Precise matching      |
| Hybrid search      | Slowest | Best      | When sparse available |

## Insertion Optimization

### Batch Operations

Always use batch operations for multiple inserts:

```python
# ✅ Good: Batch insert
texts = ["doc1", "doc2", "doc3", ...]
await client.batch_insert_text("collection", texts)

# ❌ Bad: Individual inserts
for text in texts:
    await client.insert_text("collection", text)
```

### Optimal Batch Size

- **Small batches (10-100)**: Good for real-time updates
- **Medium batches (100-1000)**: Recommended for most cases
- **Large batches (1000-10000)**: Best for bulk indexing

### Parallel Insertion

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

## Memory Optimization

### Enable Quantization

Always enable quantization for production:

```json
{
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 8
  }
}
```

### Use Appropriate Dimensions

- **384**: Good for most use cases, fast
- **512**: Balanced quality and speed
- **768**: Higher quality, slower
- **1536**: Maximum quality, slowest

### Limit Metadata Size

Keep payloads small:

```python
# ✅ Good: Small metadata
metadata = {"id": 1, "category": "docs"}

# ❌ Bad: Large metadata
metadata = {"full_content": "..." * 10000}
```

## CPU Optimization

### Use GPU Acceleration (macOS)

Enable Metal GPU acceleration:

```bash
cargo build --release --features hive-gpu
```

**Benefits (macOS Metal GPU):**

- 3-5x faster search (measured)
- < 1ms search latency (vs 0.6-2.4 ms CPU)
- 50-200x faster batch operations
- Better throughput for concurrent workloads

**Benchmark results:**

- Single search: 1-2 ms (vs 10 ms CPU) = **5-10x faster**
- Batch insert (1K): 5-10 ms (vs 500 ms CPU) = **50-100x faster**
- Batch search (100): 5-10 ms (vs 1000 ms CPU) = **100-200x faster**

### Thread Configuration

Set appropriate thread count:

```bash
export VECTORIZER_THREADS=4  # Match CPU cores
```

## Monitoring Performance

### Check Collection Stats

```bash
curl http://localhost:15002/collections/my_collection
```

### Monitor Search Latency

```python
import time

start = time.time()
results = await client.search("collection", "query")
latency = (time.time() - start) * 1000  # ms
print(f"Search latency: {latency:.2f}ms")
```

### Track Memory Usage

```bash
# Linux
ps aux | grep vectorizer

# Check collection memory
curl http://localhost:15002/collections/my_collection/stats
```

## Benchmarking

### Search Performance Test

```python
import asyncio
import time

async def benchmark_search(client, collection, query, iterations=1000):
    """Benchmark search performance."""
    latencies = []

    for _ in range(iterations):
        start = time.time()
        await client.search(collection, query, limit=10)
        latency = (time.time() - start) * 1000
        latencies.append(latency)

    avg_latency = sum(latencies) / len(latencies)
    p95_latency = sorted(latencies)[int(len(latencies) * 0.95)]
    p99_latency = sorted(latencies)[int(len(latencies) * 0.99)]

    print(f"Average: {avg_latency:.2f}ms")
    print(f"P95: {p95_latency:.2f}ms")
    print(f"P99: {p99_latency:.2f}ms")
```

## Best Practices Summary

1. **Always enable quantization** for production (8 bits recommended)
2. **Use batch operations** for multiple inserts/updates
3. **Choose appropriate dimensions** (384-512 for most cases)
4. **Tune HNSW parameters** based on collection size
5. **Use similarity thresholds** to filter low-quality results
6. **Limit result counts** to what you actually need
7. **Monitor performance** regularly
8. **Use GPU acceleration** when available (macOS)

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection configuration
- [Configuration Guide](../configuration/CONFIGURATION.md) - Server configuration
- [Troubleshooting Guide](../troubleshooting/TROUBLESHOOTING.md) - Performance issues
