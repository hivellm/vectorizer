---
title: Advanced Guides
module: guides
id: guides-index
order: 0
description: Advanced features and optimization guides
tags: [advanced, guides, sparse-vectors, quantization, optimization, n8n, langflow, graph]
---

# Advanced Guides

Advanced features and optimization guides for experienced users.

## High Availability

### [Master/Replica Routing](./MASTER_REPLICA_ROUTING.md)
Automatic read/write routing for HA deployments:
- Configure master and replica URLs
- Automatic write routing to master
- Read routing based on preference (master, replica, nearest)
- Round-robin load balancing across replicas
- Per-operation override for read-your-writes patterns
- Supported in all SDKs (TypeScript, JavaScript, Python, Rust, Go, C#)

## Integrations

### [n8n Integration](./N8N_INTEGRATION.md)
No-code workflow automation:
- Official n8n community node
- Collection, Vector, and Search operations
- RAG pipeline examples
- Integration with 400+ n8n nodes

### [Langflow Integration](./LANGFLOW_INTEGRATION.md)
Visual LLM app building:
- LangChain-compatible components
- VectorStore, Retriever, Loader
- RAG pipeline examples
- Custom embeddings support

## Features

### [Graph Relationships](./GRAPH.md)
Knowledge graph features:
- Graph nodes and edges
- Relationship discovery
- Path finding
- Graph traversal

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
```

### Quantization

```python
quantization={
    "enabled": True,
    "type": "scalar",
    "bits": 8  # 4x memory reduction
}
```

## Related Topics

- [Performance Guide](../configuration/PERFORMANCE_TUNING.md) - Performance optimization
- [Collection Configuration](../collections/CONFIGURATION.md) - Collection settings
- [Search Guide](../search/ADVANCED.md) - Advanced search methods
