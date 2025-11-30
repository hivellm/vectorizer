---
title: Collections
module: collections
id: collections-index
order: 0
description: Complete guide to collections in Vectorizer
tags: [collections, data-management, vectors]
---

# Collections

Collections are the primary way to organize and manage vectors in Vectorizer.

## Guides

### [Creating Collections](./CREATING.md)
Learn how to create collections:
- Basic collection creation
- Required parameters
- Custom configurations
- Common patterns
- Verification and error handling

### [Collection Configuration](./CONFIGURATION.md)
Advanced configuration options:
- Distance metrics (cosine, euclidean, dot product)
- HNSW index configuration
- Quantization settings
- Compression options
- Configuration patterns

### [Collection Operations](./OPERATIONS.md)
Managing collections:
- Listing collections
- Getting collection information
- Updating configuration
- Deleting collections
- Statistics and health checks

### [Sharding Configuration](./SHARDING.md)
Distributed sharding for scalability:
- When to use sharding
- Configuration parameters
- Shard management
- Monitoring and rebalancing
- Best practices and troubleshooting

### [Complete Collections Guide](./COLLECTIONS.md)
Comprehensive guide covering all aspects:
- What are collections
- Complete configuration reference
- Best practices
- Troubleshooting

## Quick Start

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Create collection
await client.create_collection("my_docs", dimension=384)

# List collections
collections = await client.list_collections()

# Get collection info
info = await client.get_collection_info("my_docs")
```

## Related Topics

- [Search Guide](../search/SEARCH.md) - Searching within collections
- [Vectors Guide](../vectors/VECTORS.md) - Managing vectors
- [Performance Guide](../configuration/PERFORMANCE_TUNING.md) - Performance tuning

