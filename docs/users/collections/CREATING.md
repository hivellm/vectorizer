---
title: Creating Collections
module: collections
id: creating-collections
order: 1
description: How to create collections in Vectorizer
tags: [collections, create, setup]
---

# Creating Collections

Learn how to create collections in Vectorizer with different configurations.

## Basic Collection Creation

### Simple Collection

The simplest way to create a collection:

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
await client.create_collection("my_collection", dimension=384, metric="cosine")
```

### Using TypeScript SDK

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

const client = new VectorizerClient("http://localhost:15002");
await client.createCollection("my_collection", {
  dimension: 384,
  metric: "cosine",
});
```

## Required Parameters

### Name

Collection name must be:

- Unique within the server
- Valid identifier (letters, numbers, underscores, hyphens)
- Not empty

```python
# ✅ Valid names
"my_collection"
"documents-2024"
"collection_1"

# ❌ Invalid names
""  # Empty
"my collection"  # Spaces
"123"  # Numbers only (discouraged)
```

### Dimension

Vector dimension must match your embedding model:

```python
# Common dimensions
384   # BGE-small, MiniLM
512   # BM25, custom models
768   # BERT-base
1536  # OpenAI ada-002
```

### Metric

Distance metric for similarity calculations:

- **cosine**: Best for text embeddings (default)
- **euclidean**: Best for spatial data
- **dot_product**: Best for normalized vectors

## Collection with Custom Configuration

### Full Configuration Example

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

### Python Example

```python
await client.create_collection(
    "production_collection",
    dimension=512,
    metric="cosine",
    hnsw_config={
        "m": 16,
        "ef_construction": 200,
        "ef_search": 64
    },
    quantization={
        "enabled": True,
        "type": "scalar",
        "bits": 8
    }
)
```

## Common Collection Patterns

### Pattern 1: Fast Search Collection

Optimized for speed:

```python
await client.create_collection(
    "fast_collection",
    dimension=384,
    metric="cosine",
    hnsw_config={"m": 8, "ef_construction": 100, "ef_search": 32},
    quantization={"enabled": True, "type": "scalar", "bits": 8}
)
```

### Pattern 2: High Quality Collection

Optimized for accuracy:

```python
await client.create_collection(
    "quality_collection",
    dimension=768,
    metric="cosine",
    hnsw_config={"m": 32, "ef_construction": 400, "ef_search": 128},
    quantization={"enabled": False}
)
```

### Pattern 3: Memory Optimized Collection

Optimized for memory:

```python
await client.create_collection(
    "memory_collection",
    dimension=384,
    metric="cosine",
    quantization={"enabled": True, "type": "scalar", "bits": 4}
)
```

## Verification

After creating a collection, verify it was created:

```bash
curl http://localhost:15002/collections/my_collection
```

```python
info = await client.get_collection_info("my_collection")
print(f"Created: {info['name']}, Dimension: {info['dimension']}")
```

## Common Errors

### Collection Already Exists

**Error:** "Collection already exists"

**Solution:**

```bash
# Delete existing collection first
curl -X DELETE http://localhost:15002/collections/my_collection

# Or use a different name
```

### Invalid Dimension

**Error:** "Invalid dimension"

**Solution:** Ensure dimension matches your embedding model output.

### Invalid Metric

**Error:** "Invalid metric"

**Solution:** Use one of: `cosine`, `euclidean`, `dot_product`

## Related Topics

- [Collection Configuration](./CONFIGURATION.md) - Advanced configuration options
- [Collection Operations](./OPERATIONS.md) - Managing collections
- [Collections Overview](./COLLECTIONS.md) - Complete collections guide
