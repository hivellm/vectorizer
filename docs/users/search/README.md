---
title: Search
module: search
id: search-index
order: 0
description: Complete guide to searching vectors in Vectorizer
tags: [search, query, similarity, vectors]
---

# Search

Vectorizer provides multiple search methods optimized for different use cases.

## Guides

### [Basic Search](./BASIC.md)
Fundamental search operations:
- Simple text search
- Vector search
- Search parameters
- Result processing
- Common patterns

### [Advanced Search](./ADVANCED.md)
Advanced search methods:
- Intelligent search (AI-powered)
- Semantic search (reranking)
- Hybrid search (dense + sparse)
- Multi-collection search

### [Complete Search Guide](./SEARCH.md)
Comprehensive guide covering:
- All search methods
- Detailed parameters
- Performance optimization
- Best practices
- Troubleshooting

## Quick Start

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Basic search
results = await client.search("my_collection", "query", limit=10)

# Intelligent search
results = await client.intelligent_search("my_collection", "query", max_results=15)

# Hybrid search
from vectorizer_sdk import HybridSearchRequest, SparseVector
sparse = SparseVector(indices=[0, 5], values=[0.8, 0.6])
results = await client.hybrid_search(
    HybridSearchRequest(collection="my_collection", query="query", query_sparse=sparse)
)
```

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection configuration
- [Vectors Guide](../vectors/VECTORS.md) - Vector operations
- [Performance Guide](../performance/PERFORMANCE.md) - Performance tuning

