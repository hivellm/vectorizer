---
title: Search Guide
module: search
id: search-guide
order: 1
description: Complete guide to searching vectors in Vectorizer
tags: [search, query, similarity, vectors]
---

# Search Guide

Vectorizer provides multiple search methods to find similar vectors.

## Basic Search

### Simple Text Search

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning",
    "limit": 10
  }'
```

### Vector Search

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, ...],
    "limit": 10
  }'
```

## Advanced Search Types

### Intelligent Search

AI-powered search with query expansion:

```bash
curl -X POST http://localhost:15002/collections/my_collection/intelligent_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "neural networks",
    "max_results": 15,
    "mmr_enabled": true,
    "technical_focus": true
  }'
```

### Semantic Search

Semantic search with reranking:

```bash
curl -X POST http://localhost:15002/collections/my_collection/semantic_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "deep learning",
    "max_results": 10,
    "semantic_reranking": true
  }'
```

### Hybrid Search

Combine dense and sparse vectors:

```bash
curl -X POST http://localhost:15002/collections/my_collection/hybrid_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "vector database",
    "query_sparse": {
      "indices": [0, 5, 10],
      "values": [0.8, 0.6, 0.9]
    },
    "alpha": 0.7,
    "algorithm": "rrf"
  }'
```

### Multi-Collection Search

Search across multiple collections:

```bash
curl -X POST http://localhost:15002/multi_collection_search \
  -H "Content-Type: application/json" \
  -d '{
    "collections": ["docs", "code", "wiki"],
    "query": "authentication",
    "max_results": 20
  }'
```

## Search Parameters

### Limit

Control the number of results:

```json
{
  "query": "search term",
  "limit": 5
}
```

### Similarity Threshold

Filter results by minimum similarity:

```json
{
  "query": "search term",
  "limit": 10,
  "similarity_threshold": 0.7
}
```

### With Payload

Include metadata in results:

```json
{
  "query": "search term",
  "limit": 10,
  "with_payload": true
}
```

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Managing collections
- [SDKs Guide](../sdks/SDKS.md) - SDK search methods

