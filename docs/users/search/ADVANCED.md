---
title: Advanced Search
module: search
id: advanced-search
order: 2
description: Advanced search methods in Vectorizer
tags: [search, advanced, intelligent, semantic, hybrid]
---

# Advanced Search

Advanced search methods for improved search quality and specialized use cases.

## Intelligent Search

AI-powered search with automatic query expansion, domain knowledge, and MMR diversification.

### Basic Usage

```python
results = await client.intelligent_search(
    "my_collection",
    "neural networks",
    max_results=15
)
```

### Full Configuration

```python
results = await client.intelligent_search(
    "my_collection",
    "neural networks",
    max_results=15,
    mmr_enabled=True,
    mmr_lambda=0.7,
    domain_expansion=True,
    technical_focus=True
)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `query` | string | required | Search query |
| `max_results` | number | 15 | Maximum results to return |
| `mmr_enabled` | boolean | true | Enable MMR diversification |
| `mmr_lambda` | number | 0.7 | MMR balance (0.0=diversity, 1.0=relevance) |
| `domain_expansion` | boolean | true | Enable domain knowledge expansion |
| `technical_focus` | boolean | true | Prioritize technical content |

### When to Use

- Research and discovery
- Finding diverse perspectives
- Technical documentation search
- Exploratory queries

## Semantic Search

Advanced semantic search with reranking and similarity filtering.

### Basic Usage

```python
results = await client.semantic_search(
    "my_collection",
    "deep learning frameworks",
    max_results=10
)
```

### With Reranking

```python
results = await client.semantic_search(
    "my_collection",
    "deep learning",
    max_results=10,
    semantic_reranking=True,
    similarity_threshold=0.15
)
```

### Similarity Threshold Recommendations

- **High Precision (0.15-0.2)**: Only highly relevant results
- **Balanced (0.1-0.15)**: Good balance (recommended)
- **High Recall (0.05-0.1)**: More results, may include less relevant

### When to Use

- Precise semantic matching
- High-quality result sets
- When you need accurate relevance

## Hybrid Search

Combine dense (semantic) and sparse (keyword) vectors for improved search quality.

### Basic Usage

```python
from vectorizer_sdk import HybridSearchRequest, SparseVector

sparse = SparseVector(indices=[0, 5, 10], values=[0.8, 0.6, 0.9])
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
        query="vector database",
        query_sparse=sparse,
        alpha=0.7
    )
)
```

### Scoring Algorithms

#### Reciprocal Rank Fusion (RRF) - Default

```python
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
        query="query",
        query_sparse=sparse,
        algorithm="rrf",
        alpha=0.7
    )
)
```

#### Weighted Combination

```python
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
        query="query",
        query_sparse=sparse,
        algorithm="weighted",
        alpha=0.7
    )
)
```

#### Alpha Blending

```python
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
        query="query",
        query_sparse=sparse,
        algorithm="alpha",
        alpha=0.7
    )
)
```

### Alpha Parameter

- **0.0**: Only sparse (keyword) results
- **0.5**: Equal weight
- **1.0**: Only dense (semantic) results
- **0.7**: Recommended (70% semantic, 30% keyword)

### When to Use

- Combining semantic and keyword signals
- Improving search quality
- When you have both dense and sparse vectors

## Multi-Collection Search

Search across multiple collections simultaneously.

### Basic Usage

```python
results = await client.multi_collection_search(
    query="authentication",
    collections=["docs", "code", "wiki"],
    max_results=20
)
```

### With Reranking

```python
results = await client.multi_collection_search(
    query="authentication",
    collections=["docs", "code", "wiki"],
    max_results=20,
    max_per_collection=5,
    cross_collection_reranking=True
)
```

### When to Use

- Searching across multiple data sources
- Unified search interfaces
- Cross-domain knowledge discovery

## Performance Comparison

| Method | Speed | Quality | Use Case |
|--------|-------|---------|----------|
| Basic search | Fastest | Good | Simple queries |
| Intelligent search | Slower | Best | Research, discovery |
| Semantic search | Medium | Excellent | Precise matching |
| Hybrid search | Slowest | Best | When sparse available |

## Related Topics

- [Basic Search](./BASIC.md) - Basic search operations
- [Search Guide](./SEARCH.md) - Complete search guide
- [Performance Guide](../performance/PERFORMANCE.md) - Performance optimization

