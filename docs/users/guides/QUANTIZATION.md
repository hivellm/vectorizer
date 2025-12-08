---
title: Quantization
module: advanced
id: quantization-guide
order: 2
description: Complete guide to vector quantization for memory optimization
tags: [advanced, quantization, memory, optimization, performance]
---

# Quantization Guide

Complete guide to vector quantization for memory optimization and performance improvement.

## What is Quantization?

Quantization reduces memory usage by storing vectors with fewer bits per dimension. Instead of 32-bit floats (4 bytes), vectors can be stored as:

- **16-bit floats**: 2 bytes per dimension (2x reduction)
- **8-bit integers**: 1 byte per dimension (4x reduction)
- **4-bit integers**: 0.5 bytes per dimension (8x reduction)

## Quantization Types

### Scalar Quantization (SQ)

Most common quantization method. Quantizes each dimension independently.

**Configuration:**

```json
{
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 8
  }
}
```

**Bits Options:**

| Bits | Memory Reduction | Accuracy Loss | Use Case               |
| ---- | ---------------- | ------------- | ---------------------- |
| 16   | 2x (50%)         | <1%           | Maximum accuracy       |
| 8    | 4x (75%)         | 1-2%          | Recommended (default)  |
| 4    | 8x (87.5%)       | 3-5%          | Maximum memory savings |

**Benchmark Results (1M vectors, 512-dim):**

- **Without quantization**: 1.46 GB memory
- **8-bit SQ**: 366 MB memory (4x reduction)
- **Quality**: MAP score actually improves (+8.9%: 0.8400 → 0.9147)

### Product Quantization (PQ)

Quantizes vectors in sub-vectors for better compression.

**Configuration:**

```json
{
  "quantization": {
    "enabled": true,
    "type": "product",
    "n_centroids": 256,
    "n_subquantizers": 8
  }
}
```

**Use Cases:**

- Very large collections (>10M vectors)
- Maximum compression needed
- Acceptable accuracy loss (5-10%)

### Binary Quantization

Stores vectors as binary (1 bit per dimension).

**Configuration:**

```json
{
  "quantization": {
    "enabled": true,
    "type": "binary"
  }
}
```

**Characteristics:**

- **Memory reduction**: 32x (1 bit vs 32 bits)
- **Accuracy loss**: Significant (10-20%)
- **Use cases**: Approximate search, very large collections

## Enabling Quantization

### At Collection Creation

**REST API:**

```bash
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_collection",
    "dimension": 512,
    "metric": "cosine",
    "quantization": {
      "enabled": true,
      "type": "scalar",
      "bits": 8
    }
  }'
```

**Python SDK:**

```python
await client.create_collection(
    "my_collection",
    dimension=512,
    metric="cosine",
    quantization={
        "enabled": True,
        "type": "scalar",
        "bits": 8
    }
)
```

### Update Existing Collection

**REST API:**

```bash
curl -X PATCH http://localhost:15002/collections/my_collection \
  -H "Content-Type: application/json" \
  -d '{
    "quantization": {
      "enabled": true,
      "type": "scalar",
      "bits": 8
    }
  }'
```

**Note:** Quantization is applied to new vectors. Existing vectors may need re-indexing.

## Memory Savings

### Calculation

**Without quantization:**

```
Memory = vectors × dimension × 4 bytes
Example: 1M vectors × 512 dim × 4 bytes = 2 GB
```

**With 8-bit quantization:**

```
Memory = vectors × dimension × 1 byte
Example: 1M vectors × 512 dim × 1 byte = 512 MB (4x reduction)
```

**With 4-bit quantization:**

```
Memory = vectors × dimension × 0.5 bytes
Example: 1M vectors × 512 dim × 0.5 bytes = 256 MB (8x reduction)
```

### Real-World Examples

**1M vectors, 512 dimensions:**

| Quantization  | Memory | Reduction |
| ------------- | ------ | --------- |
| None (32-bit) | 2 GB   | -         |
| 16-bit        | 1 GB   | 50%       |
| 8-bit         | 512 MB | 75%       |
| 4-bit         | 256 MB | 87.5%     |

**10M vectors, 384 dimensions:**

| Quantization  | Memory   | Reduction |
| ------------- | -------- | --------- |
| None (32-bit) | 15 GB    | -         |
| 8-bit         | 3.75 GB  | 75%       |
| 4-bit         | 1.875 GB | 87.5%     |

## Performance Impact

### Search Latency

**Benchmark results (10K vectors, 512-dim):**

- **Without quantization**: 0.61 ms average
- **With 8-bit SQ**: 0.6-2.4 ms (minimal overhead)
- **Quality**: Actually improves (MAP +8.9%)

**Conclusion:** Quantization has minimal impact on search latency while significantly reducing memory.

### Indexing Performance

- **Build time**: Slightly slower (quantization overhead)
- **Memory during build**: Significantly lower
- **Overall**: Faster due to less memory pressure

## Choosing Quantization Settings

### High Accuracy (Recommended)

```json
{
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 8
  }
}
```

**Best for:**

- Production deployments
- Balanced accuracy and memory
- Most use cases

### Maximum Accuracy

```json
{
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 16
  }
}
```

**Best for:**

- Critical accuracy requirements
- Small to medium collections
- Research/development

### Maximum Memory Savings

```json
{
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 4
  }
}
```

**Best for:**

- Very large collections (>10M vectors)
- Memory-constrained environments
- Acceptable accuracy loss (3-5%)

## Quantization Best Practices

1. **Always enable quantization** for production (8-bit recommended)
2. **Test accuracy** on your specific dataset before choosing bits
3. **Use 8-bit by default** (best balance)
4. **Consider 4-bit** only for very large collections
5. **Monitor quality metrics** (MAP, NDCG) after enabling
6. **Re-index collections** after changing quantization settings

## Troubleshooting

### Quality Degradation

**Problem:** Search quality decreases after enabling quantization.

**Solutions:**

1. **Increase bits**: Try 16-bit instead of 8-bit
2. **Check vector normalization**: Ensure vectors are normalized
3. **Verify collection metric**: Use appropriate metric (cosine recommended)
4. **Test on sample**: Verify quantization impact on your data

### Memory Not Reduced

**Problem:** Memory usage doesn't decrease after enabling quantization.

**Solutions:**

1. **Verify quantization enabled**: Check collection configuration
2. **Re-index collection**: Quantization applies to new vectors
3. **Check vector count**: Ensure vectors are actually quantized
4. **Monitor memory**: Use `/collections/{name}/stats` endpoint

## Quantization Cache

Vectorizer implements a quantization cache to speed up repeated searches with the same quantized vectors.

### Cache Architecture

```
Query → Check Cache → [Hit] → Return cached result
                   → [Miss] → Dequantize → Search → Cache result → Return
```

### Cache Metrics

Monitor cache performance via the metrics endpoint:

```bash
GET /metrics
```

**Relevant metrics:**

```
# Quantization cache hit ratio
quantization_cache_hit_ratio 0.85

# Cache hits
quantization_cache_hits_total 85000

# Cache misses
quantization_cache_misses_total 15000

# Cache size
quantization_cache_size_bytes 104857600
```

### Cache Configuration

Configure cache settings in `config.yml`:

```yaml
quantization:
  cache:
    enabled: true
    max_size_mb: 512
    ttl_seconds: 3600
```

### Cache Hit Tracking

The cache tracks hits/misses per collection:

```bash
GET /collections/{name}/stats
```

**Response includes:**

```json
{
  "name": "my_collection",
  "vector_count": 1000000,
  "quantization": {
    "type": "scalar",
    "bits": 8,
    "cache": {
      "enabled": true,
      "hit_ratio": 0.85,
      "hits": 85000,
      "misses": 15000,
      "size_bytes": 104857600
    }
  }
}
```

### HNSW Cache Integration

When using HNSW with quantization, cache hits are tracked during graph traversal:

- **Node visits**: Cached quantized vectors speed up neighbor comparisons
- **Distance calculations**: Cached results reduce redundant dequantization
- **Beam search**: Hot nodes remain in cache for faster exploration

### Best Practices

1. **Size appropriately**: Cache size should accommodate hot vectors
2. **Monitor hit ratio**: Target >80% hit ratio for optimal performance
3. **Adjust TTL**: Lower TTL for frequently updated collections
4. **Clear on reindex**: Cache is invalidated when quantization changes

## Related Topics

- [Collection Configuration](../collections/CONFIGURATION.md) - Collection settings
- [Performance Guide](../configuration/PERFORMANCE_TUNING.md) - Performance optimization
- [Memory Optimization](../configuration/PERFORMANCE_TUNING.md) - Memory tuning
- [Monitoring](../operations/MONITORING.md) - Metrics and monitoring
