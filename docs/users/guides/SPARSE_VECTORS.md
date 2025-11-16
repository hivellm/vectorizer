---
title: Sparse Vectors
module: advanced
id: sparse-vectors
order: 1
description: Complete guide to using sparse vectors in Vectorizer
tags: [advanced, sparse-vectors, hybrid-search, optimization]
---

# Sparse Vectors

Complete guide to using sparse vectors for efficient keyword-based search and hybrid search.

## What are Sparse Vectors?

Sparse vectors are efficient representations for high-dimensional vectors where most values are zero. They store only non-zero values with their indices, making them ideal for:

- **Keyword-based search**: Each dimension represents a keyword/token
- **TF-IDF features**: Term frequency-inverse document frequency
- **BM25 features**: BM25 scoring features
- **Hybrid search**: Combining semantic (dense) and keyword (sparse) signals

## Sparse Vector Format

### Structure

A sparse vector consists of two arrays:

- **indices**: Positions of non-zero values (must be sorted and unique)
- **values**: The actual non-zero values

**Example:**

```json
{
  "indices": [0, 5, 10, 15],
  "values": [0.8, 0.6, 0.9, 0.7]
}
```

This represents a vector where:

- Position 0 has value 0.8
- Position 5 has value 0.6
- Position 10 has value 0.9
- Position 15 has value 0.7
- All other positions are 0

## Creating Sparse Vectors

### From Keywords

**Python example:**

```python
from collections import Counter

def create_sparse_from_keywords(text, vocab):
    """Create sparse vector from keywords."""
    words = text.lower().split()
    word_counts = Counter(words)

    indices = []
    values = []

    for word, count in word_counts.items():
        if word in vocab:
            idx = vocab[word]
            indices.append(idx)
            values.append(count / len(words))  # Normalize

    return {"indices": sorted(indices), "values": values}
```

### From TF-IDF

**Python example:**

```python
from sklearn.feature_extraction.text import TfidfVectorizer

vectorizer = TfidfVectorizer(max_features=10000)
vectorizer.fit(documents)

def text_to_sparse(text):
    """Convert text to sparse vector using TF-IDF."""
    tfidf = vectorizer.transform([text])

    # Extract non-zero values
    indices = tfidf.indices.tolist()
    values = tfidf.data.tolist()

    return {"indices": indices, "values": values}
```

## Inserting Sparse Vectors

### Insert with Sparse Vector

**REST API:**

```bash
curl -X POST http://localhost:15002/collections/my_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "id": "doc_001",
    "text": "Vectorizer is a high-performance vector database",
    "sparse_vector": {
      "indices": [0, 5, 10],
      "values": [0.8, 0.6, 0.9]
    },
    "metadata": {
      "source": "readme"
    }
  }'
```

**Python SDK:**

```python
from vectorizer_sdk import VectorizerClient, SparseVector

client = VectorizerClient("http://localhost:15002")

sparse = SparseVector(
    indices=[0, 5, 10],
    values=[0.8, 0.6, 0.9]
)

await client.insert_text(
    "my_collection",
    "Vectorizer is a high-performance vector database",
    id="doc_001",
    sparse_vector=sparse,
    metadata={"source": "readme"}
)
```

### Insert Mixed Dense and Sparse

You can insert vectors with both dense (semantic) and sparse (keyword) representations:

```python
await client.insert_text(
    "my_collection",
    "Vectorizer is a high-performance vector database",
    id="doc_001",
    sparse_vector=sparse,
    metadata={"source": "readme"}
)
```

Vectorizer will:

- Use dense vector for semantic search (HNSW index)
- Use sparse vector for keyword search (SparseVectorIndex)
- Enable hybrid search combining both

## Sparse Vector Search

### Basic Sparse Search

**REST API:**

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "sparse_vector": {
      "indices": [0, 5, 10],
      "values": [0.8, 0.6, 0.9]
    },
    "limit": 10
  }'
```

**Python SDK:**

```python
sparse_query = SparseVector(
    indices=[0, 5, 10],
    values=[0.8, 0.6, 0.9]
)

results = await client.search(
    "my_collection",
    sparse_vector=sparse_query,
    limit=10
)
```

### Sparse Search with Filters

```python
results = await client.search(
    "my_collection",
    sparse_vector=sparse_query,
    limit=10,
    filter={"category": "documentation"}
)
```

## Hybrid Search

Hybrid search combines dense (semantic) and sparse (keyword) search for improved results.

### Basic Hybrid Search

**REST API:**

```bash
curl -X POST http://localhost:15002/collections/my_collection/hybrid_search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "vector database for large-scale applications",
    "query_sparse": {
      "indices": [0, 5, 10],
      "values": [0.8, 0.6, 0.9]
    },
    "alpha": 0.7,
    "algorithm": "rrf",
    "dense_k": 20,
    "sparse_k": 20,
    "final_k": 10
  }'
```

**Python SDK:**

```python
from vectorizer_sdk import HybridSearchRequest, SparseVector

sparse_query = SparseVector(
    indices=[0, 5, 10],
    values=[0.8, 0.6, 0.9]
)

results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
        query="vector database for large-scale applications",
        query_sparse=sparse_query,
        alpha=0.7,  # 70% dense, 30% sparse
        algorithm="rrf",
        dense_k=20,
        sparse_k=20,
        final_k=10
    )
)
```

### Hybrid Search Algorithms

**Reciprocal Rank Fusion (RRF) - Default:**

```python
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
        query="query",
        query_sparse=sparse_query,
        algorithm="rrf"  # Combines rankings
    )
)
```

**Weighted Combination:**

```python
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
        query="query",
        query_sparse=sparse_query,
        algorithm="weighted"  # Weighted score combination
    )
)
```

**Alpha Blending:**

```python
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
        query="query",
        query_sparse=sparse_query,
        algorithm="alpha"  # Alpha blending
    )
)
```

### Alpha Parameter

The `alpha` parameter controls the blend between dense and sparse:

- **0.0**: Pure sparse (keyword-only) search
- **0.5**: Equal weight to both
- **1.0**: Pure dense (semantic-only) search
- **0.7** (default): Favor semantic search with keyword boost

**Recommendations:**

- **High alpha (0.7-0.9)**: When semantic meaning is more important
- **Low alpha (0.3-0.5)**: When exact keywords are critical
- **Medium alpha (0.5-0.7)**: Balanced approach

## Use Cases

### Use Case 1: Document Search with Keywords

**Scenario:** Search documents by both meaning and specific keywords.

```python
# Create sparse vector from query keywords
query_keywords = ["vector", "database", "performance"]
sparse_query = create_sparse_from_keywords(
    " ".join(query_keywords),
    keyword_vocab
)

# Hybrid search
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="documents",
        query="high-performance vector database",
        query_sparse=sparse_query,
        alpha=0.6  # Balance semantic and keyword
    )
)
```

### Use Case 2: E-commerce Product Search

**Scenario:** Search products by description (semantic) and specific attributes (keywords).

```python
# Sparse vector for product attributes
sparse_attrs = SparseVector(
    indices=[color_idx, size_idx, brand_idx],
    values=[1.0, 1.0, 1.0]  # Exact match for attributes
)

results = await client.hybrid_search(
    HybridSearchRequest(
        collection="products",
        query="comfortable running shoes",
        query_sparse=sparse_attrs,
        alpha=0.5  # Equal weight
    )
)
```

### Use Case 3: Code Search

**Scenario:** Search code by functionality (semantic) and specific function/class names (keywords).

```python
# Sparse vector for code symbols
code_symbols = extract_code_symbols(query)  # Functions, classes, etc.
sparse_symbols = SparseVector(
    indices=[symbol_indices],
    values=[1.0] * len(symbol_indices)
)

results = await client.hybrid_search(
    HybridSearchRequest(
        collection="code",
        query="authentication middleware",
        query_sparse=sparse_symbols,
        alpha=0.7  # Favor semantic, boost exact symbol matches
    )
)
```

## Performance Considerations

### Memory Efficiency

Sparse vectors are memory-efficient for high-dimensional data:

- **Dense vector (10K dim)**: 40 KB per vector
- **Sparse vector (100 non-zero)**: ~800 bytes per vector (50x reduction)

### Search Performance

- **Sparse search**: Very fast for keyword matching
- **Hybrid search**: Slightly slower than pure dense/sparse, but better quality
- **Index size**: Sparse index is typically smaller than dense index

### Best Practices

1. **Use sparse vectors** when you have keyword/token features
2. **Use hybrid search** for queries that benefit from both semantic and keyword signals
3. **Tune alpha** based on your use case (start with 0.7)
4. **Pre-compute sparse vectors** for better performance
5. **Use appropriate vocabulary size** (typically 10K-100K dimensions)

## Troubleshooting

### Invalid Sparse Vector

**Error:** "Sparse vector indices must be sorted and unique"

**Solution:**

```python
# Sort indices and remove duplicates
indices = sorted(set(indices))
values = [values[i] for i in sorted_indices]
```

### Dimension Mismatch

**Error:** "Sparse vector dimension mismatch"

**Solution:** Ensure sparse vector indices don't exceed vocabulary size.

### Empty Sparse Vector

**Error:** "Sparse vector cannot be empty"

**Solution:** Ensure at least one non-zero value.

## Related Topics

- [Hybrid Search](../search/ADVANCED.md) - Advanced search methods
- [Search Guide](../search/SEARCH.md) - Complete search guide
- [Performance Guide](../configuration/PERFORMANCE_TUNING.md) - Performance optimization
