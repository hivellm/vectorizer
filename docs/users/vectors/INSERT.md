---
title: Inserting Vectors
module: vectors
id: inserting-vectors
order: 1
description: How to insert vectors into collections
tags: [vectors, insert, add, create]
---

# Inserting Vectors

Complete guide to inserting vectors into Vectorizer collections.

## Insert Single Text

The simplest way to add a vector - text is automatically converted to embeddings.

### Basic Insert

```bash
curl -X POST http://localhost:15002/collections/my_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Vectorizer is a high-performance vector database",
    "metadata": {
      "source": "readme.md",
      "category": "documentation"
    }
  }'
```

### Using Python SDK

```python
vector_id = await client.insert_text(
    "my_collection",
    "Vectorizer is a high-performance vector database",
    metadata={"source": "readme.md", "category": "documentation"}
)
```

### Using TypeScript SDK

```typescript
const vectorId = await client.insertText(
  "my_collection",
  "Vectorizer is a high-performance vector database",
  { source: "readme.md", category: "documentation" }
);
```

## Insert with Custom ID

### Using Custom ID

```bash
curl -X POST http://localhost:15002/collections/my_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "id": "doc_001",
    "text": "Content here",
    "metadata": {"source": "readme.md"}
  }'
```

### Python Example

```python
await client.insert_text(
    "my_collection",
    "Content here",
    id="doc_001",
    metadata={"source": "readme.md"}
)
```

## Insert Pre-computed Vector

Insert a vector directly without text-to-embedding conversion.

### REST API

```bash
curl -X POST http://localhost:15002/collections/my_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "id": "vec_001",
    "vector": [0.1, 0.2, 0.3, 0.4, ...],
    "metadata": {"source": "custom_embedding"}
  }'
```

### Python Example

```python
vector = [0.1, 0.2, 0.3, 0.4] * 96  # 384-dimensional vector
await client.insert_vector(
    "my_collection",
    vector,
    id="vec_001",
    metadata={"source": "custom_embedding"}
)
```

## Batch Insert

Insert multiple vectors efficiently.

### REST API

```bash
curl -X POST http://localhost:15002/collections/my_collection/batch_insert \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": [
      {
        "text": "First document",
        "metadata": {"doc_id": 1}
      },
      {
        "text": "Second document",
        "metadata": {"doc_id": 2}
      },
      {
        "text": "Third document",
        "metadata": {"doc_id": 3}
      }
    ]
  }'
```

### Python Example

```python
texts = ["Doc 1", "Doc 2", "Doc 3"]
metadatas = [
    {"id": 1},
    {"id": 2},
    {"id": 3}
]
vector_ids = await client.batch_insert_text("my_collection", texts, metadatas)
```

### Optimal Batch Size

- **Small batches (10-100)**: Good for real-time updates
- **Medium batches (100-1000)**: Recommended for most cases
- **Large batches (1000-10000)**: Best for bulk indexing

## Insert with Sparse Vector

Insert vectors with sparse representation for memory efficiency.

### Python Example

```python
from vectorizer_sdk import SparseVector

sparse = SparseVector(
    indices=[0, 5, 10, 15, 20],
    values=[0.8, 0.6, 0.9, 0.7, 0.5]
)

await client.insert_text(
    "my_collection",
    "keyword-based document",
    sparse=sparse,
    metadata={"type": "sparse"}
)
```

## Best Practices

1. **Use batch operations**: Much faster for multiple inserts
2. **Optimal batch size**: 100-1000 vectors per batch
3. **Pre-compute embeddings**: Faster than on-the-fly conversion
4. **Validate dimensions**: Ensure vectors match collection dimension
5. **Use meaningful IDs**: Easier to manage and retrieve

## Common Patterns

### Pattern 1: Document Indexing

```python
async def index_documents(collection_name, documents):
    """Index a batch of documents."""
    texts = [doc["content"] for doc in documents]
    metadatas = [
        {
            "source": doc["source"],
            "page": doc.get("page"),
            "timestamp": doc.get("timestamp")
        }
        for doc in documents
    ]

    await client.batch_insert_text(collection_name, texts, metadatas)
```

### Pattern 2: Incremental Updates

```python
async def add_new_documents(collection_name, new_docs):
    """Add new documents incrementally."""
    for doc in new_docs:
        await client.insert_text(
            collection_name,
            doc["content"],
            id=doc["id"],
            metadata=doc["metadata"]
        )
```

## Related Topics

- [Updating Vectors](./UPDATE.md) - Updating existing vectors
- [Deleting Vectors](./DELETE.md) - Removing vectors
- [Vectors Guide](./VECTORS.md) - Complete vectors guide
