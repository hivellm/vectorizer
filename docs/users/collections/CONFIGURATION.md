---
title: Collection Configuration
module: collections
id: collection-configuration
order: 2
description: Advanced collection configuration options
tags: [collections, configuration, hnsw, quantization]
---

# Collection Configuration

Complete guide to configuring collections for optimal performance.

## Distance Metrics

### Cosine Similarity (Default)

Best for text embeddings and normalized vectors:

```json
{
  "metric": "cosine"
}
```

**When to use:**

- Text embeddings (BERT, GPT, etc.)
- Document similarity
- Semantic search
- Recommendation systems

**Characteristics:**

- Measures angle between vectors
- Range: -1 to 1 (typically 0 to 1 for normalized vectors)
- Invariant to vector magnitude

### Euclidean Distance

Best for spatial data:

```json
{
  "metric": "euclidean"
}
```

**When to use:**

- Geographic data
- Image embeddings
- Numerical feature vectors
- Clustering algorithms

**Characteristics:**

- Measures straight-line distance
- Range: 0 to infinity
- Sensitive to vector magnitude

### Dot Product

Best for normalized vectors:

```json
{
  "metric": "dot_product"
}
```

**When to use:**

- Normalized embeddings
- Weighted feature vectors
- Score-based ranking

**Characteristics:**

- Measures vector alignment
- Range: -infinity to infinity
- Faster computation than cosine

## HNSW Index Configuration

HNSW (Hierarchical Navigable Small World) provides fast approximate nearest neighbor search.

### M Parameter

Number of connections per layer:

```json
{
  "hnsw_config": {
    "m": 16
  }
}
```

**Guidelines:**

- **Low (8)**: Faster, less memory, lower recall
- **Medium (16)**: Balanced (default)
- **High (32)**: Slower, more memory, higher recall

**By Collection Size:**

- <10K vectors: m=8-16
- 10K-1M vectors: m=16-24
- > 1M vectors: m=24-32

### EF Construction

Search width during index building:

```json
{
  "hnsw_config": {
    "ef_construction": 200
  }
}
```

**Guidelines:**

- **Low (100)**: Faster build, lower quality
- **Medium (200)**: Balanced (default)
- **High (400)**: Slower build, higher quality

**Recommendations:**

- Fast indexing: 100-150
- Balanced: 200
- High quality: 300-400

### EF Search

Search width during queries:

```json
{
  "hnsw_config": {
    "ef_search": 64
  }
}
```

**Guidelines:**

- **Low (32)**: Fastest, lower recall
- **Medium (64)**: Balanced
- **High (128+)**: Slower, higher recall

**Recommendations:**

- Fast search: 32-48
- Balanced: 64-96
- High recall: 128-256

### Complete HNSW Configuration

```json
{
  "hnsw_config": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 64,
    "seed": null
  }
}
```

## Quantization Configuration

Quantization reduces memory usage by storing vectors with fewer bits.

### Scalar Quantization (Default)

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
| 16   | 25%              | <1%           | Maximum accuracy       |
| 8    | 50%              | 1-2%          | Recommended (default)  |
| 4    | 75%              | 3-5%          | Maximum memory savings |

### Disable Quantization

For maximum accuracy:

```json
{
  "quantization": {
    "enabled": false
  }
}
```

**When to disable:**

- Maximum accuracy required
- Memory is not a concern
- Research/development

## Compression Configuration

Compress payloads to save storage:

```json
{
  "compression": {
    "enabled": true,
    "threshold_bytes": 1024,
    "algorithm": "lz4"
  }
}
```

**Parameters:**

- **enabled**: Enable compression (default: true)
- **threshold_bytes**: Minimum size to compress (default: 1024)
- **algorithm**: Compression algorithm (lz4 or none)

## Complete Configuration Example

```json
{
  "name": "optimized_collection",
  "dimension": 512,
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
  },
  "compression": {
    "enabled": true,
    "threshold_bytes": 1024,
    "algorithm": "lz4"
  }
}
```

## Configuration Patterns

### High-Speed Configuration

```json
{
  "dimension": 384,
  "metric": "cosine",
  "hnsw_config": { "m": 8, "ef_construction": 100, "ef_search": 32 },
  "quantization": { "enabled": true, "type": "scalar", "bits": 8 }
}
```

### High-Quality Configuration

```json
{
  "dimension": 768,
  "metric": "cosine",
  "hnsw_config": { "m": 32, "ef_construction": 400, "ef_search": 128 },
  "quantization": { "enabled": false }
}
```

### Memory-Optimized Configuration

```json
{
  "dimension": 384,
  "metric": "cosine",
  "hnsw_config": { "m": 16, "ef_construction": 200, "ef_search": 64 },
  "quantization": { "enabled": true, "type": "scalar", "bits": 4 }
}
```

## Related Topics

- [Creating Collections](./CREATING.md) - How to create collections
- [Collection Operations](./OPERATIONS.md) - Managing collections
- [Performance Guide](../performance/PERFORMANCE.md) - Performance tuning
