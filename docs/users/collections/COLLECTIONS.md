---
title: Collections Guide
module: collections
id: collections-guide
order: 1
description: Understanding and managing collections in Vectorizer
tags: [collections, data-management, vectors]
---

# Collections Guide

Collections are the primary way to organize and manage vectors in Vectorizer.

## What is a Collection?

A collection is a named group of vectors that share the same:
- **Dimension**: The size of each vector (e.g., 384, 512, 768)
- **Distance Metric**: How similarity is calculated (cosine, euclidean, dot product)
- **Configuration**: Index settings, quantization, compression

## Creating Collections

### Using REST API

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
await client.create_collection(
    "my_collection",
    dimension=384,
    metric="cosine"
)
```

### Using TypeScript SDK

```typescript
await client.createCollection('my_collection', {
    dimension: 384,
    metric: 'cosine'
});
```

## Collection Configuration

### Distance Metrics

- **cosine**: Best for text embeddings (default)
- **euclidean**: Best for spatial data
- **dot_product**: Best for normalized vectors

### Dimensions

Common embedding dimensions:
- **384**: BGE-small, MiniLM
- **512**: BM25, custom models
- **768**: BERT-base
- **1536**: OpenAI ada-002

## Listing Collections

```bash
curl http://localhost:15002/collections
```

## Getting Collection Info

```bash
curl http://localhost:15002/collections/my_collection
```

## Deleting Collections

```bash
curl -X DELETE http://localhost:15002/collections/my_collection
```

## Related Topics

- [Search Guide](../search/SEARCH.md) - Searching within collections
- [SDKs Guide](../sdks/SDKS.md) - SDK-specific collection operations

