---
title: Python SDK
module: sdks
id: python-sdk
order: 1
description: Complete Python SDK guide for Vectorizer
tags: [python, sdk, client-library]
---

# Python SDK

Complete guide to using the Vectorizer Python SDK.

## Installation

```bash
pip install vectorizer-sdk
```

## Quick Start

```python
from vectorizer_sdk import VectorizerClient

# Create client
client = VectorizerClient("http://localhost:15002")

# Create collection
await client.create_collection("my_docs", dimension=384)

# Insert text
await client.insert_text("my_docs", "Hello, Vectorizer!")

# Search
results = await client.search("my_docs", "hello", limit=5)
```

## Client Configuration

### Basic Client

```python
client = VectorizerClient("http://localhost:15002")
```

### With Custom Timeout

```python
client = VectorizerClient(
    "http://localhost:15002",
    timeout=30.0  # seconds
)
```

### With Authentication

```python
client = VectorizerClient(
    "http://localhost:15002",
    api_key="your-api-key"
)
```

## Collection Operations

### Create Collection

```python
await client.create_collection(
    "my_collection",
    dimension=384,
    metric="cosine"
)
```

### List Collections

```python
collections = await client.list_collections()
for collection in collections:
    print(f"Collection: {collection['name']}, Vectors: {collection['vector_count']}")
```

### Get Collection Info

```python
info = await client.get_collection_info("my_collection")
print(f"Dimension: {info['dimension']}")
print(f"Metric: {info['metric']}")
print(f"Vector count: {info['vector_count']}")
```

### Delete Collection

```python
await client.delete_collection("my_collection")
```

## Vector Operations

### Insert Single Text

```python
vector_id = await client.insert_text(
    "my_collection",
    "Vectorizer is a high-performance vector database",
    metadata={"source": "readme.md"}
)
```

### Insert with Custom ID

```python
await client.insert_text(
    "my_collection",
    "Content here",
    id="custom_id_001",
    metadata={"category": "docs"}
)
```

### Batch Insert

```python
texts = ["Doc 1", "Doc 2", "Doc 3"]
metadatas = [
    {"id": 1},
    {"id": 2},
    {"id": 3}
]
vector_ids = await client.batch_insert_text("my_collection", texts, metadatas)
```

### Get Vector

```python
vector = await client.get_vector("my_collection", "vector_id")
print(f"ID: {vector['id']}")
print(f"Metadata: {vector['metadata']}")
```

### Update Vector

```python
await client.update_vector(
    "my_collection",
    "vector_id",
    text="Updated content",
    metadata={"updated_at": "2024-01-01"}
)
```

### Delete Vector

```python
await client.delete_vector("my_collection", "vector_id")
```

### Batch Delete

```python
ids_to_delete = ["id1", "id2", "id3"]
await client.batch_delete("my_collection", ids_to_delete)
```

## Search Operations

### Basic Search

```python
results = await client.search(
    "my_collection",
    "machine learning",
    limit=10
)
```

### Search with Filters

```python
results = await client.search(
    "my_collection",
    "python tutorial",
    limit=10,
    filter={"category": "tutorial", "language": "python"}
)
```

### Search with Similarity Threshold

```python
results = await client.search(
    "my_collection",
    "query",
    limit=10,
    similarity_threshold=0.7
)
```

### Intelligent Search

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

### Semantic Search

```python
results = await client.semantic_search(
    "my_collection",
    "deep learning",
    max_results=10,
    semantic_reranking=True,
    similarity_threshold=0.15
)
```

### Hybrid Search

```python
from vectorizer_sdk import HybridSearchRequest, SparseVector

sparse = SparseVector(indices=[0, 5, 10], values=[0.8, 0.6, 0.9])
hybrid_results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_collection",
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

### Multi-Collection Search

```python
results = await client.multi_collection_search(
    query="authentication",
    collections=["docs", "code", "wiki"],
    max_results=20,
    max_per_collection=5
)
```

## Qdrant Compatibility

### List Collections (Qdrant Format)

```python
collections = await client.qdrant_list_collections()
```

### Get Collection (Qdrant Format)

```python
collection = await client.qdrant_get_collection("my_collection")
```

### Upsert Points

```python
points = [
    {
        "id": "point1",
        "vector": [0.1, 0.2, 0.3, ...],
        "payload": {"text": "content"}
    }
]
await client.qdrant_upsert_points("my_collection", points)
```

### Search Points

```python
results = await client.qdrant_search_points(
    "my_collection",
    vector=[0.1, 0.2, 0.3, ...],
    limit=10,
    with_payload=True
)
```

### Delete Points

```python
await client.qdrant_delete_points(
    "my_collection",
    ids=["point1", "point2"]
)
```

### Retrieve Points

```python
points = await client.qdrant_retrieve_points(
    "my_collection",
    ids=["point1", "point2"],
    with_payload=True
)
```

### Count Points

```python
count = await client.qdrant_count_points("my_collection")
print(f"Total points: {count}")
```

## Error Handling

```python
from vectorizer_sdk import VectorizerError

try:
    await client.create_collection("my_collection", dimension=384)
except VectorizerError as e:
    print(f"Error: {e}")
```

## Async/Await

The Python SDK is fully async:

```python
import asyncio

async def main():
    client = VectorizerClient("http://localhost:15002")
    results = await client.search("my_collection", "query", limit=5)
    return results

# Run async function
results = asyncio.run(main())
```

## Type Hints

The SDK includes full type hints:

```python
from typing import List
from vectorizer_sdk import VectorizerClient, SearchResult

async def search_documents(
    client: VectorizerClient,
    query: str
) -> List[SearchResult]:
    return await client.search("my_collection", query, limit=10)
```

## Best Practices

1. **Use async/await**: All operations are async
2. **Use batch operations**: Much faster for multiple operations
3. **Handle errors**: Wrap operations in try/except
4. **Reuse client**: Create client once and reuse it
5. **Use type hints**: Improves code quality and IDE support

## Related Topics

- [TypeScript SDK](./TYPESCRIPT.md) - TypeScript/JavaScript SDK
- [Rust SDK](./RUST.md) - Rust SDK
- [Collections Guide](../collections/COLLECTIONS.md) - Collection operations
- [Search Guide](../search/SEARCH.md) - Search operations
