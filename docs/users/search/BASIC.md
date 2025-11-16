---
title: Basic Search
module: search
id: basic-search
order: 1
description: Basic search operations in Vectorizer
tags: [search, basic, query, similarity]
---

# Basic Search

Learn the fundamentals of searching vectors in Vectorizer.

## Simple Text Search

The most common search method - converts text to embeddings and finds similar vectors.

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning algorithms",
    "limit": 10
  }'
```

### Using Python SDK

```python
results = await client.search(
    "my_collection",
    "machine learning algorithms",
    limit=10
)
```

### Using TypeScript SDK

```typescript
const results = await client.search(
  "my_collection",
  "machine learning algorithms",
  {
    limit: 10,
  }
);
```

## Vector Search

Search using a pre-computed vector instead of text.

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, 0.4, ...],
    "limit": 10
  }'
```

### When to Use Vector Search

- You already have embeddings
- Cross-collection search with same embeddings
- Custom embedding models
- Performance optimization (skip embedding step)

## Search Parameters

### Limit Results

Control the number of results returned:

```python
# Get top 5 results
results = await client.search("my_collection", "query", limit=5)

# Get top 20 results
results = await client.search("my_collection", "query", limit=20)
```

**Best practices:**

- Request only what you need (faster)
- Typical: 5-20 results
- Maximum: 100-1000 (depending on use case)

### Similarity Threshold

Filter results by minimum similarity score:

```python
results = await client.search(
    "my_collection",
    "query",
    limit=10,
    similarity_threshold=0.7
)
```

**Threshold Guidelines:**

| Threshold | Use Case        | Description                 |
| --------- | --------------- | --------------------------- |
| 0.9+      | Very strict     | Only highly similar results |
| 0.7-0.9   | High precision  | Good for exact matches      |
| 0.5-0.7   | Balanced        | Good balance (recommended)  |
| 0.3-0.5   | High recall     | More diverse results        |
| <0.3      | Very permissive | May include less relevant   |

### Include Vector Data

Return vector data in results:

```python
results = await client.search(
    "my_collection",
    "query",
    limit=10,
    with_vector=True
)
```

### Include Payload

Return metadata in results:

```python
results = await client.search(
    "my_collection",
    "query",
    limit=10,
    with_payload=True
)
```

## Search Results

### Result Structure

```python
results = await client.search("my_collection", "query", limit=5)

for result in results:
    print(f"ID: {result['id']}")
    print(f"Score: {result['score']}")
    print(f"Payload: {result.get('payload', {})}")
    if 'vector' in result:
        print(f"Vector: {result['vector']}")
```

### Processing Results

```python
async def search_and_process(collection, query, limit=10):
    """Search and process results."""
    results = await client.search(collection, query, limit=limit)

    processed = []
    for result in results:
        processed.append({
            "id": result["id"],
            "relevance": result["score"],
            "content": result.get("payload", {}).get("text", ""),
            "metadata": result.get("payload", {}).get("metadata", {})
        })

    return processed
```

## Common Patterns

### Pattern 1: Simple Query

```python
results = await client.search("documents", "python tutorial", limit=5)
```

### Pattern 2: Filtered Search

```python
results = await client.search(
    "documents",
    "python",
    limit=10,
    filter={"category": "tutorial", "language": "python"}
)
```

### Pattern 3: High Precision Search

```python
results = await client.search(
    "documents",
    "specific term",
    limit=5,
    similarity_threshold=0.8
)
```

## Best Practices

1. **Use appropriate limits**: Don't request more than needed
2. **Set similarity thresholds**: Filter low-quality results
3. **Use filters**: When you have metadata filters
4. **Cache results**: For frequently searched queries
5. **Monitor performance**: Track search latency

## Related Topics

- [Advanced Search](./ADVANCED.md) - Intelligent, semantic, hybrid search
- [Search Guide](./SEARCH.md) - Complete search guide
- [Collections Guide](../collections/COLLECTIONS.md) - Collection configuration
