---
title: Search Guide
module: search
id: search-guide
order: 1
description: Complete guide to all search methods in Vectorizer
tags: [search, query, similarity, vectors, hybrid-search]
---

# Search Guide

Vectorizer provides multiple search methods optimized for different use cases. This guide covers all search types with detailed examples and best practices.

## Basic Search

### Simple Text Search

The most common search method - converts text to embeddings and finds similar vectors.

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning algorithms",
    "limit": 10
  }'
```

**Response:**

```json
{
  "results": [
    {
      "id": "vec_1",
      "score": 0.892,
      "vector": null,
      "payload": {
        "text": "Introduction to machine learning...",
        "source": "ml_book.pdf"
      }
    }
  ]
}
```

### Vector Search

Search using a pre-computed vector instead of text.

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, 0.4, ...],
    "limit": 10
  }'
```

**Use cases:**

- When you already have embeddings
- Cross-collection search with same embeddings
- Custom embedding models

### Search Parameters

#### Limit Results

```json
{
  "query": "search term",
  "limit": 5
}
```

#### Similarity Threshold

Filter results by minimum similarity score (0.0 to 1.0):

```json
{
  "query": "search term",
  "limit": 10,
  "similarity_threshold": 0.7
}
```

**Threshold Guidelines:**

- **0.9+**: Very strict, only highly similar results
- **0.7-0.9**: High precision, good for exact matches
- **0.5-0.7**: Balanced precision and recall
- **0.3-0.5**: High recall, more diverse results
- **<0.3**: Very permissive, may include less relevant results

#### Include Vector Data

```json
{
  "query": "search term",
  "limit": 10,
  "with_vector": true
}
```

#### Include Payload

```json
{
  "query": "search term",
  "limit": 10,
  "with_payload": true
}
```

## Advanced Search Types

### Intelligent Search

AI-powered search with automatic query expansion, domain knowledge, and MMR diversification.

```bash
curl -X POST http://localhost:15002/collections/my_collection/intelligent_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "neural networks",
    "max_results": 15,
    "mmr_enabled": true,
    "mmr_lambda": 0.7,
    "domain_expansion": true,
    "technical_focus": true
  }'
```

**Features:**

- **Query Expansion**: Automatically generates 4-8 related queries
- **Domain Knowledge**: Uses domain-specific terminology
- **MMR Diversification**: Reduces redundancy in results
- **Technical Focus**: Prioritizes technical content

**Parameters:**

| Parameter          | Type    | Default  | Description                                |
| ------------------ | ------- | -------- | ------------------------------------------ |
| `query`            | string  | required | Search query                               |
| `max_results`      | number  | 15       | Maximum results to return                  |
| `mmr_enabled`      | boolean | true     | Enable MMR diversification                 |
| `mmr_lambda`       | number  | 0.7      | MMR balance (0.0=diversity, 1.0=relevance) |
| `domain_expansion` | boolean | true     | Enable domain knowledge expansion          |
| `technical_focus`  | boolean | true     | Prioritize technical content               |

**Best for:**

- Research and discovery
- Finding diverse perspectives
- Technical documentation search
- Exploratory queries

### Semantic Search

Advanced semantic search with reranking and similarity filtering.

```bash
curl -X POST http://localhost:15002/collections/my_collection/semantic_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "deep learning frameworks",
    "max_results": 10,
    "semantic_reranking": true,
    "similarity_threshold": 0.15
  }'
```

**Parameters:**

| Parameter              | Type    | Default  | Description               |
| ---------------------- | ------- | -------- | ------------------------- |
| `query`                | string  | required | Search query              |
| `max_results`          | number  | 10       | Maximum results to return |
| `semantic_reranking`   | boolean | true     | Enable semantic reranking |
| `similarity_threshold` | number  | 0.5      | Minimum similarity score  |

**Similarity Threshold Recommendations:**

- **High Precision (0.15-0.2)**: Only highly relevant results
- **Balanced (0.1-0.15)**: Good balance (recommended)
- **High Recall (0.05-0.1)**: More results, may include less relevant

**Best for:**

- Precise semantic matching
- High-quality result sets
- When you need accurate relevance

### Hybrid Search

Combine dense (semantic) and sparse (keyword) vectors for improved search quality.

```bash
curl -X POST http://localhost:15002/collections/my_collection/hybrid_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "vector database performance",
    "query_sparse": {
      "indices": [0, 5, 10, 15, 20],
      "values": [0.8, 0.6, 0.9, 0.7, 0.5]
    },
    "alpha": 0.7,
    "algorithm": "rrf",
    "dense_k": 20,
    "sparse_k": 20,
    "final_k": 10
  }'
```

**Scoring Algorithms:**

#### Reciprocal Rank Fusion (RRF) - Default

Best for general use, balances dense and sparse results:

```json
{
  "algorithm": "rrf",
  "alpha": 0.7
}
```

#### Weighted Combination

Linear combination of scores:

```json
{
  "algorithm": "weighted",
  "alpha": 0.7
}
```

**Alpha parameter:**

- **0.0**: Only sparse (keyword) results
- **0.5**: Equal weight
- **1.0**: Only dense (semantic) results
- **0.7**: Recommended (70% semantic, 30% keyword)

#### Alpha Blending

Smooth blending between dense and sparse:

```json
{
  "algorithm": "alpha",
  "alpha": 0.7
}
```

**Parameters:**

| Parameter      | Type   | Default  | Description                            |
| -------------- | ------ | -------- | -------------------------------------- |
| `query`        | string | required | Dense vector query text                |
| `query_sparse` | object | optional | Sparse vector with indices/values      |
| `alpha`        | number | 0.7      | Blend factor (0.0-1.0)                 |
| `algorithm`    | string | "rrf"    | Scoring algorithm (rrf/weighted/alpha) |
| `dense_k`      | number | 20       | Number of dense results to retrieve    |
| `sparse_k`     | number | 20       | Number of sparse results to retrieve   |
| `final_k`      | number | 10       | Final number of results to return      |

**Best for:**

- Combining semantic and keyword signals
- Improving search quality
- When you have both dense and sparse vectors

### Multi-Collection Search

Search across multiple collections simultaneously with intelligent reranking.

```bash
curl -X POST http://localhost:15002/multi_collection_search \
  -H "Content-Type: application/json" \
  -d '{
    "collections": ["docs", "code", "wiki"],
    "query": "authentication",
    "max_results": 20,
    "max_per_collection": 5,
    "cross_collection_reranking": true
  }'
```

**Parameters:**

| Parameter                    | Type    | Default  | Description                          |
| ---------------------------- | ------- | -------- | ------------------------------------ |
| `collections`                | array   | required | List of collection names             |
| `query`                      | string  | required | Search query                         |
| `max_results`                | number  | 20       | Total results across all collections |
| `max_per_collection`         | number  | 5        | Max results per collection           |
| `cross_collection_reranking` | boolean | true     | Rerank across collections            |

**Best for:**

- Searching across multiple data sources
- Unified search interfaces
- Cross-domain knowledge discovery

## Search Examples by Use Case

### Example 1: Document Search

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Simple document search
results = await client.search(
    "documents",
    "Python async programming",
    limit=10
)

# With similarity threshold
results = await client.search(
    "documents",
    "Python async programming",
    limit=10,
    similarity_threshold=0.7
)
```

### Example 2: Code Search

```python
# Search code with technical focus
results = await client.intelligent_search(
    "documents",
    "async await patterns",
    max_results=15,
    technical_focus=True,
    mmr_enabled=True
)
```

### Example 3: Research Discovery

```python
# Use intelligent search for research
results = await client.intelligent_search(
    "research_papers",
    "transformer architecture",
    max_results=20,
    domain_expansion=True,
    mmr_enabled=True,
    mmr_lambda=0.6  # More diversity
)
```

### Example 4: Precise Matching

```python
# Use semantic search for precise results
results = await client.semantic_search(
    "documents",
    "vector database",
    max_results=5,
    similarity_threshold=0.2  # High precision
)
```

### Example 5: Hybrid Search

```python
from vectorizer_sdk import HybridSearchRequest, SparseVector

# Create sparse vector for keywords
sparse = SparseVector(
    indices=[0, 5, 10, 15],
    values=[0.8, 0.6, 0.9, 0.7]
)

# Hybrid search
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="documents",
        query="vector database",
        query_sparse=sparse,
        alpha=0.7,
        algorithm="rrf",
        dense_k=20,
        sparse_k=20,
        final_k=10
    )
)
```

## Performance Optimization

### Fast Search Configuration

For quick queries, use optimized settings:

```json
{
  "query": "search term",
  "limit": 5,
  "similarity_threshold": 0.3
}
```

### Quality Search Configuration

For comprehensive research:

```json
{
  "query": "search term",
  "max_results": 15,
  "mmr_enabled": true,
  "domain_expansion": true,
  "technical_focus": true
}
```

### Balanced Configuration

Good balance of speed and quality:

```json
{
  "query": "search term",
  "limit": 10,
  "similarity_threshold": 0.5
}
```

## Best Practices

### Choosing the Right Search Method

1. **Basic Search**: Simple queries, known terms
2. **Intelligent Search**: Research, discovery, diverse results
3. **Semantic Search**: Precise matching, high quality
4. **Hybrid Search**: Best quality, when you have sparse vectors
5. **Multi-Collection**: Cross-domain search

### Optimizing Search Performance

1. **Use appropriate limits**: Don't request more results than needed
2. **Set similarity thresholds**: Filter low-quality results early
3. **Enable quantization**: Reduces memory and improves speed
4. **Tune HNSW parameters**: Balance recall vs. speed

### Improving Search Quality

1. **Use intelligent search** for exploratory queries
2. **Use semantic search** for precise matching
3. **Use hybrid search** when available
4. **Tune alpha parameter** in hybrid search
5. **Enable MMR** for diverse results

## Troubleshooting

### Low Relevance Results

**Problem:** Results don't match query intent

**Solutions:**

- Use intelligent search with domain expansion
- Lower similarity threshold
- Try semantic search with reranking
- Use hybrid search if available

### Slow Search Performance

**Problem:** Searches take too long

**Solutions:**

- Reduce `limit` or `max_results`
- Lower `ef_search` in HNSW config
- Enable quantization
- Use basic search instead of intelligent search

### No Results Returned

**Problem:** Search returns empty results

**Solutions:**

- Lower similarity threshold
- Check collection has vectors
- Verify query text is valid
- Check collection name is correct

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection configuration affects search
- [SDKs Guide](../sdks/SDKS.md) - SDK search methods
- [Configuration Guide](../configuration/CONFIGURATION.md) - Performance tuning
