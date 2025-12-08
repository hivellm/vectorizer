---
title: Discovery API
module: api
id: discovery-api
order: 5
description: Discovery endpoints for intelligent content exploration
tags: [api, discovery, exploration, content-discovery]
---

# Discovery API

The Discovery API provides intelligent content exploration capabilities, helping you discover relevant information across collections through advanced search and analysis pipelines.

## Overview

Discovery endpoints enable:

- Multi-collection content discovery
- Query expansion and refinement
- Collection filtering and scoring
- Evidence compression and summarization
- Answer plan generation
- LLM prompt rendering

## Discovery Endpoint

### Main Discovery

Execute a complete discovery pipeline that searches across collections, filters results, expands queries, and generates structured output.

**Endpoint:** `POST /discover`

**Request Body:**

```json
{
  "query": "How does vector search work?",
  "include_collections": ["docs", "code"],
  "exclude_collections": ["archive"],
  "max_bullets": 10,
  "broad_k": 50,
  "focus_k": 20
}
```

**Parameters:**

| Parameter             | Type          | Required | Description                          |
| --------------------- | ------------- | -------- | ------------------------------------ |
| `query`               | string        | Yes      | Search query                         |
| `include_collections` | array[string] | No       | Collections to search (default: all) |
| `exclude_collections` | array[string] | No       | Collections to exclude               |
| `max_bullets`         | number        | No       | Maximum bullet points (default: 10)  |
| `broad_k`             | number        | No       | Broad search results (default: 50)   |
| `focus_k`             | number        | No       | Focused search results (default: 20) |

**Response:**

```json
{
  "answer_prompt": "Based on the following information...",
  "sections": 3,
  "bullets": 8,
  "chunks": 25,
  "metrics": {
    "total_time_ms": 450,
    "collections_searched": 2,
    "queries_generated": 5,
    "chunks_found": 50,
    "chunks_after_dedup": 25,
    "bullets_extracted": 8,
    "final_prompt_tokens": 1200
  }
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/discover \
  -H "Content-Type: application/json" \
  -d '{
    "query": "vector database architecture",
    "include_collections": ["docs", "code"],
    "max_bullets": 15
  }'
```

**Python SDK:**

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

result = await client.discover(
    query="vector database architecture",
    include_collections=["docs", "code"],
    max_bullets=15
)

print(f"Found {result['chunks']} chunks")
print(f"Generated {result['bullets']} bullet points")
```

## Discovery Pipeline Components

### Filter Collections

Filter collections based on relevance to a query.

**Endpoint:** `POST /discovery/filter_collections`

**Request Body:**

```json
{
  "query": "machine learning",
  "collections": ["docs", "code", "papers"],
  "min_score": 0.5
}
```

**Response:**

```json
{
  "filtered_collections": [
    {
      "name": "docs",
      "score": 0.85,
      "relevance": "high"
    },
    {
      "name": "papers",
      "score": 0.72,
      "relevance": "medium"
    }
  ]
}
```

### Score Collections

Score collections by relevance to a query.

**Endpoint:** `POST /discovery/score_collections`

**Request Body:**

```json
{
  "query": "Rust async programming",
  "collections": ["code", "docs", "tutorials"]
}
```

**Response:**

```json
{
  "scores": [
    {
      "collection": "code",
      "score": 0.92,
      "reason": "High relevance - contains Rust async examples"
    },
    {
      "collection": "docs",
      "score": 0.65,
      "reason": "Medium relevance - general Rust documentation"
    }
  ]
}
```

### Expand Queries

Generate multiple query variations for better coverage.

**Endpoint:** `POST /discovery/expand_queries`

**Request Body:**

```json
{
  "query": "vector search",
  "max_expansions": 5,
  "domain_expansion": true,
  "technical_focus": true
}
```

**Response:**

```json
{
  "expanded_queries": [
    "vector search",
    "semantic similarity search",
    "nearest neighbor search",
    "embedding-based search",
    "vector database query"
  ]
}
```

### Broad Discovery

Perform broad search across collections to find diverse content.

**Endpoint:** `POST /discovery/broad_discovery`

**Request Body:**

```json
{
  "query": "API design patterns",
  "collections": ["docs", "code"],
  "k": 50,
  "diversity_threshold": 0.3
}
```

**Response:**

```json
{
  "results": [
    {
      "collection": "docs",
      "chunks": 25,
      "diversity_score": 0.85
    },
    {
      "collection": "code",
      "chunks": 20,
      "diversity_score": 0.78
    }
  ]
}
```

### Semantic Focus

Focus search on semantically similar content.

**Endpoint:** `POST /discovery/semantic_focus`

**Request Body:**

```json
{
  "query": "authentication",
  "collection": "docs",
  "k": 20,
  "similarity_threshold": 0.7
}
```

**Response:**

```json
{
  "focused_results": [
    {
      "id": "chunk_001",
      "score": 0.92,
      "content": "..."
    }
  ],
  "total_found": 20
}
```

### Promote README

Prioritize README files and documentation in results.

**Endpoint:** `POST /discovery/promote_readme`

**Request Body:**

```json
{
  "query": "getting started",
  "collection": "docs",
  "boost_factor": 2.0
}
```

**Response:**

```json
{
  "promoted_results": [
    {
      "id": "readme_001",
      "score": 0.95,
      "boosted": true
    }
  ]
}
```

### Compress Evidence

Compress and summarize evidence chunks.

**Endpoint:** `POST /discovery/compress_evidence`

**Request Body:**

```json
{
  "chunks": [
    { "id": "chunk_1", "content": "..." },
    { "id": "chunk_2", "content": "..." }
  ],
  "max_length": 1000,
  "preserve_key_points": true
}
```

**Response:**

```json
{
  "compressed": [
    {
      "id": "chunk_1",
      "original_length": 500,
      "compressed_length": 200,
      "content": "..."
    }
  ],
  "compression_ratio": 0.4
}
```

### Build Answer Plan

Generate a structured answer plan from discovered content.

**Endpoint:** `POST /discovery/build_answer_plan`

**Request Body:**

```json
{
  "query": "How to implement vector search?",
  "chunks": [
    { "id": "chunk_1", "content": "..." },
    { "id": "chunk_2", "content": "..." }
  ],
  "max_sections": 5
}
```

**Response:**

```json
{
  "plan": {
    "sections": [
      {
        "title": "Introduction",
        "chunks": ["chunk_1"],
        "order": 1
      },
      {
        "title": "Implementation",
        "chunks": ["chunk_2"],
        "order": 2
      }
    ]
  }
}
```

### Render LLM Prompt

Generate a formatted prompt for LLM consumption.

**Endpoint:** `POST /discovery/render_llm_prompt`

**Request Body:**

```json
{
  "query": "Explain vector databases",
  "chunks": [{ "id": "chunk_1", "content": "..." }],
  "format": "markdown",
  "include_metadata": true
}
```

**Response:**

```json
{
  "prompt": "Based on the following information:\n\n## Chunk 1\n...",
  "token_count": 1200,
  "format": "markdown"
}
```

## Use Cases

### Research Assistant

Use discovery to research topics across multiple collections:

```python
result = await client.discover(
    query="Rust async programming best practices",
    include_collections=["docs", "code", "examples"],
    max_bullets=20
)

# Use the answer_prompt for LLM
llm_prompt = result["answer_prompt"]
```

### Content Exploration

Explore collections to understand available content:

```python
# Filter relevant collections
filtered = await client.filter_collections(
    query="machine learning",
    collections=["docs", "papers", "code"]
)

# Expand queries for better coverage
expanded = await client.expand_queries(
    query="neural networks",
    max_expansions=5
)
```

### Documentation Generation

Generate documentation from code and docs:

```python
# Discover relevant content
discovery = await client.discover(
    query="API endpoints",
    include_collections=["code", "docs"]
)

# Build answer plan
plan = await client.build_answer_plan(
    query="API endpoints",
    chunks=discovery["chunks"]
)

# Render prompt for documentation
prompt = await client.render_llm_prompt(
    query="API endpoints",
    chunks=plan["chunks"],
    format="markdown"
)
```

## Hybrid Search

Discovery uses hybrid search to combine dense (semantic) and sparse (keyword) search for better results.

### How Hybrid Search Works

1. **Dense Search**: HNSW-based vector similarity search using embeddings
2. **Sparse Search**: BM25/Tantivy full-text search for keyword matching
   - Uses Tantivy tokenizer for improved term extraction
   - Automatic stopword removal and lowercasing
   - Better Unicode handling
3. **Reciprocal Rank Fusion (RRF)**: Combines results from both searches

### Evidence Compression

Discovery uses intelligent evidence compression to extract the most relevant sentences from search results:

1. **Keyword Extraction**: Uses Tantivy tokenizer to extract keyphrases from text
   - Filters stopwords automatically
   - Scores terms by frequency (TF-IDF-like)
   - Identifies most important keywords

2. **Sentence Scoring**: Sentences are scored by keyword density
   - Higher scores for sentences containing important keywords
   - Prioritizes relevant content over filler text

3. **Sentence Extraction**: Improved sentence boundary detection
   - Handles multiple sentence-ending punctuation (. ! ?)
   - Proper Unicode-aware segmentation
   - Filters by minimum/maximum word count

### Collection Filtering

Collection filtering uses Tantivy's tokenizer for improved query processing:

- **Stopword Removal**: Language-specific stopwords are automatically removed
- **Term Normalization**: Lowercasing and Unicode normalization
- **Better Matching**: Improved matching accuracy for collection names

### Hybrid Search Endpoint

**Endpoint:** `POST /collections/{name}/hybrid_search`

**Request Body:**

```json
{
  "query": "vector database implementation",
  "k": 10,
  "alpha": 0.5
}
```

**Parameters:**

| Parameter | Type   | Required | Description                                    |
| --------- | ------ | -------- | ---------------------------------------------- |
| `query`   | string | Yes      | Search query                                   |
| `k`       | number | No       | Number of results (default: 10)                |
| `alpha`   | number | No       | Dense/sparse weight (0.0-1.0, default: 0.5)    |

- `alpha = 1.0`: Pure dense (semantic) search
- `alpha = 0.0`: Pure sparse (keyword) search
- `alpha = 0.5`: Balanced hybrid search

**Response:**

```json
{
  "results": [
    {
      "id": "doc_001",
      "score": 0.92,
      "dense_score": 0.85,
      "sparse_score": 0.95,
      "vector": [...],
      "payload": { "title": "..." }
    }
  ]
}
```

### RRF Algorithm

The Reciprocal Rank Fusion algorithm combines rankings from dense and sparse search:

```
RRF_score(d) = Σ (1 / (k + rank_i(d)))
```

Where:
- `k` is a constant (default: 60)
- `rank_i(d)` is the rank of document `d` in ranking `i`

### Example Usage

```python
# Balanced hybrid search
result = await client.hybrid_search(
    collection="docs",
    query="async programming in Rust",
    k=10,
    alpha=0.5
)

# Semantic-focused (good for conceptual queries)
result = await client.hybrid_search(
    collection="docs",
    query="how does memory management work",
    k=10,
    alpha=0.8
)

# Keyword-focused (good for specific terms)
result = await client.hybrid_search(
    collection="code",
    query="fn async fn tokio spawn",
    k=10,
    alpha=0.2
)
```

### When to Use Hybrid Search

| Use Case | Recommended Alpha |
|----------|-------------------|
| Conceptual questions | 0.7-0.9 |
| Code search | 0.3-0.5 |
| Documentation search | 0.5-0.6 |
| Exact term matching | 0.1-0.3 |
| General discovery | 0.5 |

## Best Practices

1. **Use appropriate collection filtering**: Include/exclude collections to focus search
2. **Adjust broad_k and focus_k**: Balance between coverage and precision
3. **Set max_bullets**: Control output size for better performance
4. **Combine discovery steps**: Use individual endpoints for fine-grained control
5. **Cache results**: Discovery can be expensive, cache when possible
6. **Use hybrid search**: Combine semantic and keyword search for better results
7. **Tune alpha parameter**: Adjust based on query type for optimal results

## Related Topics

- [Advanced Search](../search/ADVANCED.md) - Intelligent search methods
- [Multi-Collection Search](../search/ADVANCED.md#multi-collection-search) - Cross-collection search
- [API Reference](./API_REFERENCE.md) - Complete API documentation
