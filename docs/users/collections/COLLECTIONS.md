---
title: Collections Guide
module: collections
id: collections-guide
order: 1
description: Complete guide to understanding and managing collections in Vectorizer
tags: [collections, data-management, vectors, configuration]
---

# Collections Guide

Collections are the primary way to organize and manage vectors in Vectorizer. This guide covers everything you need to know about creating, configuring, and managing collections.

## What is a Collection?

A collection is a named group of vectors that share the same configuration:

- **Dimension**: The size of each vector (e.g., 384, 512, 768, 1536)
- **Distance Metric**: How similarity is calculated (cosine, euclidean, dot product)
- **Index Configuration**: HNSW index settings for fast similarity search
- **Quantization**: Memory optimization settings
- **Compression**: Optional compression for storage efficiency

## Creating Collections

### Basic Collection

The simplest way to create a collection with default settings:

```bash
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_collection",
    "dimension": 384,
    "metric": "cosine"
  }'
```

### Using Python SDK

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Basic collection
await client.create_collection(
    "my_collection",
    dimension=384,
    metric="cosine"
)

# With custom configuration
await client.create_collection(
    "advanced_collection",
    dimension=512,
    metric="cosine",
    quantization={"enabled": True, "type": "scalar", "bits": 8}
)
```

### Using TypeScript SDK

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

const client = new VectorizerClient("http://localhost:15002");

// Basic collection
await client.createCollection("my_collection", {
  dimension: 384,
  metric: "cosine",
});

// With custom configuration
await client.createCollection("advanced_collection", {
  dimension: 512,
  metric: "cosine",
  quantization: { enabled: true, type: "scalar", bits: 8 },
});
```

## Collection Configuration Options

### Distance Metrics

Choose the distance metric based on your use case:

#### Cosine Similarity (Default)

Best for text embeddings and normalized vectors. Measures the angle between vectors.

```json
{
  "metric": "cosine"
}
```

**Use cases:**

- Text embeddings (BERT, GPT, etc.)
- Document similarity
- Semantic search
- Recommendation systems

#### Euclidean Distance

Best for spatial data and absolute distance measurements.

```json
{
  "metric": "euclidean"
}
```

**Use cases:**

- Geographic data
- Image embeddings
- Numerical feature vectors
- Clustering algorithms

#### Dot Product

Best for normalized vectors where magnitude matters.

```json
{
  "metric": "dot_product"
}
```

**Use cases:**

- Normalized embeddings
- Weighted feature vectors
- Score-based ranking

### Vector Dimensions

Choose dimension based on your embedding model:

| Dimension | Common Models       | Use Case                   |
| --------- | ------------------- | -------------------------- |
| 384       | BGE-small, MiniLM   | Fast, efficient embeddings |
| 512       | BM25, custom models | Balanced performance       |
| 768       | BERT-base           | High-quality embeddings    |
| 1536      | OpenAI ada-002      | Maximum quality            |

### HNSW Index Configuration

HNSW (Hierarchical Navigable Small World) is the default index type for fast similarity search.

#### Basic HNSW Configuration

```json
{
  "hnsw_config": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 64
  }
}
```

**Parameters:**

- **m** (default: 16): Number of connections per layer. Higher = better recall, more memory
- **ef_construction** (default: 200): Search width during index construction. Higher = better quality, slower build
- **ef_search** (default: 64): Search width during queries. Higher = better recall, slower queries

#### Performance Tuning

**For High Recall (Quality):**

```json
{
  "hnsw_config": {
    "m": 32,
    "ef_construction": 400,
    "ef_search": 128
  }
}
```

**For High Speed:**

```json
{
  "hnsw_config": {
    "m": 8,
    "ef_construction": 100,
    "ef_search": 32
  }
}
```

**For Balanced Performance:**

```json
{
  "hnsw_config": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 64
  }
}
```

### Quantization Configuration

Quantization reduces memory usage by storing vectors with fewer bits.

#### Scalar Quantization (Default)

Reduces memory by 50-75% with minimal accuracy loss.

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

- **8 bits**: 50% memory reduction, ~1-2% accuracy loss (recommended)
- **4 bits**: 75% memory reduction, ~3-5% accuracy loss
- **16 bits**: 25% memory reduction, <1% accuracy loss

#### Disable Quantization

For maximum accuracy (uses more memory):

```json
{
  "quantization": {
    "enabled": false
  }
}
```

### Complete Configuration Example

```bash
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production_collection",
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
    }
  }'
```

## Collection Operations

### List All Collections

```bash
curl http://localhost:15002/collections
```

**Response:**

```json
{
  "collections": [
    {
      "name": "my_collection",
      "vector_count": 1250,
      "dimension": 384,
      "metric": "cosine"
    }
  ]
}
```

### Get Collection Information

```bash
curl http://localhost:15002/collections/my_collection
```

**Response:**

```json
{
  "name": "my_collection",
  "vector_count": 1250,
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

### Update Collection Configuration

```bash
curl -X PATCH http://localhost:15002/collections/my_collection \
  -H "Content-Type: application/json" \
  -d '{
    "hnsw_config": {
      "ef_search": 128
    }
  }'
```

**Note:** Some settings (like dimension and metric) cannot be changed after creation.

### Delete Collection

```bash
curl -X DELETE http://localhost:15002/collections/my_collection
```

**Warning:** This permanently deletes the collection and all its vectors!

## Best Practices

### Choosing the Right Dimension

1. **Match your embedding model**: Use the dimension your model outputs
2. **Consider memory**: Larger dimensions use more memory
3. **Balance quality vs. speed**: Higher dimensions = better quality but slower

### Choosing the Right Metric

1. **Text embeddings**: Use `cosine` (default)
2. **Spatial data**: Use `euclidean`
3. **Normalized vectors**: Use `dot_product`

### HNSW Configuration Guidelines

1. **Small collections (<10K vectors)**: Use default settings
2. **Medium collections (10K-1M vectors)**: Increase `ef_search` to 128
3. **Large collections (>1M vectors)**: Increase `m` to 32 and `ef_search` to 256

### Quantization Guidelines

1. **Always enable for production**: Reduces memory by 50%+
2. **Use 8 bits**: Best balance of memory and accuracy
3. **Disable only if**: Maximum accuracy is critical and memory is not a concern

## Common Patterns

### Pattern 1: High-Quality Search Collection

```json
{
  "name": "high_quality",
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

### Pattern 2: Memory-Optimized Collection

```json
{
  "name": "memory_optimized",
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
    "bits": 4
  }
}
```

### Pattern 3: Fast Search Collection

```json
{
  "name": "fast_search",
  "dimension": 384,
  "metric": "cosine",
  "hnsw_config": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 32
  },
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 8
  }
}
```

## Troubleshooting

### Collection Creation Fails

**Problem:** "Collection already exists"

**Solution:** Delete existing collection or use a different name:

```bash
curl -X DELETE http://localhost:15002/collections/my_collection
```

### Dimension Mismatch

**Problem:** "Invalid dimension: expected 384, got 512"

**Solution:** Ensure all vectors inserted match the collection dimension:

```bash
# Check collection dimension
curl http://localhost:15002/collections/my_collection

# Verify your embedding model outputs the correct dimension
```

### Memory Issues

**Problem:** High memory usage

**Solution:** Enable quantization:

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

## Related Topics

- [Search Guide](../search/SEARCH.md) - Searching within collections
- [SDKs Guide](../sdks/SDKS.md) - SDK-specific collection operations
- [Configuration Guide](../configuration/CONFIGURATION.md) - Advanced configuration
- [Troubleshooting Guide](../troubleshooting/TROUBLESHOOTING.md) - Common issues
