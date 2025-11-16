---
title: Advanced Features
module: advanced
id: advanced-index
order: 0
description: Advanced features and optimization guides
tags: [advanced, features, optimization, sparse-vectors, quantization]
---

# Advanced Features

Guides for advanced Vectorizer features and optimizations.

## Guides

### [Sparse Vectors](./SPARSE_VECTORS.md)
Complete sparse vector guide:
- What are sparse vectors
- Creating sparse vectors from keywords/TF-IDF
- Inserting sparse vectors
- Sparse vector search
- Hybrid search (dense + sparse)
- Use cases and best practices

### [Quantization](./QUANTIZATION.md)
Vector quantization guide:
- Quantization types (Scalar, Product, Binary)
- Memory savings calculations
- Performance impact
- Choosing quantization settings
- Best practices and troubleshooting

## Quick Reference

### Sparse Vectors

```python
from vectorizer_sdk import SparseVector

sparse = SparseVector(
    indices=[0, 5, 10],
    values=[0.8, 0.6, 0.9]
)

await client.insert_text(
    "collection",
    "text",
    sparse_vector=sparse
)
```

### Quantization

```python
await client.create_collection(
    "collection",
    dimension=512,
    quantization={
        "enabled": True,
        "type": "scalar",
        "bits": 8  # 4x memory reduction
    }
)
```

## Related Topics

- [Performance Guide](../performance/PERFORMANCE.md) - Performance optimization
- [Collection Configuration](../collections/CONFIGURATION.md) - Collection settings
- [Search Guide](../search/ADVANCED.md) - Advanced search methods

