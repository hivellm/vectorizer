---
title: Vectors
module: vectors
id: vectors-index
order: 0
description: Complete guide to vector operations in Vectorizer
tags: [vectors, crud, operations]
---

# Vectors

Complete guide to managing vectors in Vectorizer collections.

## Guides

### [Inserting Vectors](./INSERT.md)
How to add vectors to collections:
- Single text insert
- Custom ID insertion
- Pre-computed vectors
- Batch insertion
- Sparse vectors

### [Updating Vectors](./UPDATE.md)
How to update existing vectors:
- Update content
- Update metadata
- Batch updates
- Common patterns

### [Deleting Vectors](./DELETE.md)
How to remove vectors:
- Single deletion
- Batch deletion
- Delete by filter
- Cleanup patterns

### [Complete Vectors Guide](./VECTORS.md)
Comprehensive guide covering:
- All CRUD operations
- Metadata management
- Best practices
- Troubleshooting

## Quick Start

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Insert
vector_id = await client.insert_text("my_collection", "Hello, Vectorizer!")

# Get
vector = await client.get_vector("my_collection", vector_id)

# Update
await client.update_vector("my_collection", vector_id, text="Updated!")

# Delete
await client.delete_vector("my_collection", vector_id)
```

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection operations
- [Search Guide](../search/SEARCH.md) - Searching vectors
- [SDKs Guide](../sdks/README.md) - SDK operations

