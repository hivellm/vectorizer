---
title: Vector Operations Guide
module: vectors
id: vectors-guide
order: 1
description: Complete guide to inserting, updating, and managing vectors
tags: [vectors, insert, update, delete, crud]
---

# Vector Operations Guide

This guide covers all operations for managing vectors in Vectorizer collections.

## Inserting Vectors

### Insert Single Text

The simplest way to add a vector - text is automatically converted to embeddings.

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

**Response:**
```json
{
  "id": "auto_generated_id",
  "status": "inserted"
}
```

### Insert with Custom ID

```bash
curl -X POST http://localhost:15002/collections/my_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "id": "doc_001",
    "text": "Vectorizer is a high-performance vector database",
    "metadata": {
      "source": "readme.md"
    }
  }'
```

### Insert Pre-computed Vector

```bash
curl -X POST http://localhost:15002/collections/my_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "id": "vec_001",
    "vector": [0.1, 0.2, 0.3, 0.4, ...],
    "metadata": {
      "source": "custom_embedding"
    }
  }'
```

### Batch Insert

Insert multiple vectors efficiently:

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

**Response:**
```json
{
  "inserted": 3,
  "ids": ["id_1", "id_2", "id_3"]
}
```

### Using Python SDK

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Single insert
await client.insert_text(
    "my_collection",
    "Vectorizer is awesome!",
    metadata={"source": "readme"}
)

# Batch insert
texts = ["Doc 1", "Doc 2", "Doc 3"]
metadatas = [{"id": i} for i in range(3)]
await client.batch_insert_text("my_collection", texts, metadatas)
```

### Using TypeScript SDK

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

const client = new VectorizerClient("http://localhost:15002");

// Single insert
await client.insertText("my_collection", "Vectorizer is awesome!", {
    source: "readme"
});

// Batch insert
const texts = ["Doc 1", "Doc 2", "Doc 3"];
const metadatas = [{ id: 0 }, { id: 1 }, { id: 2 }];
await client.batchInsertText("my_collection", texts, metadatas);
```

## Retrieving Vectors

### Get Vector by ID

```bash
curl http://localhost:15002/collections/my_collection/vectors/vec_001
```

**Response:**
```json
{
  "id": "vec_001",
  "vector": [0.1, 0.2, 0.3, ...],
  "metadata": {
    "source": "readme.md",
    "category": "documentation"
  }
}
```

### Get Multiple Vectors

```bash
curl "http://localhost:15002/collections/my_collection/vectors?ids=vec_001,vec_002,vec_003"
```

### Get with Options

```bash
curl "http://localhost:15002/collections/my_collection/vectors/vec_001?with_vector=true&with_payload=true"
```

**Query Parameters:**
- `with_vector`: Include vector data (default: false)
- `with_payload`: Include metadata (default: true)

## Updating Vectors

### Update Vector Content

```bash
curl -X PUT http://localhost:15002/collections/my_collection/vectors/vec_001 \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Updated content",
    "metadata": {
      "source": "updated_readme.md",
      "updated_at": "2024-01-01"
    }
  }'
```

### Update Metadata Only

```bash
curl -X PATCH http://localhost:15002/collections/my_collection/vectors/vec_001 \
  -H "Content-Type: application/json" \
  -d '{
    "metadata": {
      "updated_at": "2024-01-01",
      "version": 2
    }
  }'
```

### Batch Update

```bash
curl -X POST http://localhost:15002/collections/my_collection/batch_update \
  -H "Content-Type: application/json" \
  -d '{
    "updates": [
      {
        "id": "vec_001",
        "text": "Updated content 1"
      },
      {
        "id": "vec_002",
        "text": "Updated content 2"
      }
    ]
  }'
```

## Deleting Vectors

### Delete Single Vector

```bash
curl -X DELETE http://localhost:15002/collections/my_collection/vectors/vec_001
```

### Delete Multiple Vectors

```bash
curl -X POST http://localhost:15002/collections/my_collection/batch_delete \
  -H "Content-Type: application/json" \
  -d '{
    "ids": ["vec_001", "vec_002", "vec_003"]
  }'
```

### Delete by Filter

```bash
curl -X POST http://localhost:15002/collections/my_collection/delete_by_filter \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {
      "category": "old_docs"
    }
  }'
```

## Metadata and Payloads

### Metadata Structure

Metadata (payload) is stored as JSON and can contain any structure:

```json
{
  "metadata": {
    "source": "document.pdf",
    "page": 1,
    "author": "John Doe",
    "tags": ["python", "tutorial"],
    "created_at": "2024-01-01T00:00:00Z",
    "custom_field": "any value"
  }
}
```

### Filtering by Metadata

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "python tutorial",
    "limit": 10,
    "filter": {
      "category": "tutorial",
      "language": "python"
    }
  }'
```

### Metadata Best Practices

1. **Use consistent keys**: Standardize metadata field names
2. **Index important fields**: Use payload indexes for frequently filtered fields
3. **Keep it lightweight**: Large metadata slows down operations
4. **Use structured data**: Prefer objects over nested strings

## Sparse Vectors

### Insert Sparse Vector

Sparse vectors are efficient for high-dimensional data with many zeros:

```bash
curl -X POST http://localhost:15002/collections/my_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "id": "sparse_001",
    "text": "keyword-based document",
    "sparse": {
      "indices": [0, 5, 10, 15, 20],
      "values": [0.8, 0.6, 0.9, 0.7, 0.5]
    },
    "metadata": {
      "type": "sparse"
    }
  }'
```

**Use cases:**
- Keyword-based search
- BM25 embeddings
- High-dimensional sparse data
- Hybrid search scenarios

## Best Practices

### Insertion Performance

1. **Use batch operations**: Much faster than individual inserts
2. **Optimal batch size**: 100-1000 vectors per batch
3. **Parallel inserts**: Use multiple threads/processes for large datasets
4. **Pre-compute embeddings**: Faster than on-the-fly conversion

### Memory Management

1. **Enable quantization**: Reduces memory usage significantly
2. **Use sparse vectors**: When appropriate, saves memory
3. **Limit metadata size**: Keep payloads small
4. **Regular cleanup**: Delete unused vectors

### Data Quality

1. **Validate dimensions**: Ensure all vectors match collection dimension
2. **Normalize vectors**: For cosine similarity, normalize vectors
3. **Consistent metadata**: Use consistent structure across vectors
4. **Unique IDs**: Use meaningful, unique identifiers

## Common Patterns

### Pattern 1: Document Indexing Pipeline

```python
async def index_documents(collection_name, documents):
    """Index a batch of documents efficiently."""
    client = VectorizerClient("http://localhost:15002")
    
    # Prepare batch
    texts = [doc["content"] for doc in documents]
    metadatas = [
        {
            "source": doc["source"],
            "page": doc.get("page"),
            "timestamp": doc.get("timestamp")
        }
        for doc in documents
    ]
    
    # Batch insert
    await client.batch_insert_text(collection_name, texts, metadatas)
```

### Pattern 2: Incremental Updates

```python
async def update_document(collection_name, doc_id, new_content):
    """Update a document's content and metadata."""
    client = VectorizerClient("http://localhost:15002")
    
    await client.update_vector(
        collection_name,
        doc_id,
        text=new_content,
        metadata={"updated_at": datetime.now().isoformat()}
    )
```

### Pattern 3: Bulk Delete

```python
async def cleanup_old_documents(collection_name, older_than_days=30):
    """Delete documents older than specified days."""
    client = VectorizerClient("http://localhost:15002")
    
    # Search for old documents
    cutoff_date = (datetime.now() - timedelta(days=older_than_days)).isoformat()
    
    # Delete by filter (if supported)
    # Or retrieve IDs and batch delete
    results = await client.search(
        collection_name,
        "",
        limit=10000,
        filter={"created_at": {"$lt": cutoff_date}}
    )
    
    ids_to_delete = [r["id"] for r in results]
    await client.batch_delete(collection_name, ids_to_delete)
```

## Troubleshooting

### Dimension Mismatch

**Problem:** "Invalid dimension: expected 384, got 512"

**Solution:** Ensure vector dimension matches collection:
```python
# Check collection dimension
info = await client.get_collection_info("my_collection")
print(f"Collection dimension: {info['dimension']}")

# Verify your embedding model outputs correct dimension
```

### Insertion Fails

**Problem:** "Vector insertion failed"

**Solutions:**
- Check collection exists
- Verify vector dimension matches
- Check metadata is valid JSON
- Ensure sufficient memory

### Update Doesn't Work

**Problem:** Vector not updating

**Solutions:**
- Verify vector ID exists
- Check collection name is correct
- Ensure update payload is valid
- Check logs for errors

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection configuration
- [Search Guide](../search/SEARCH.md) - Searching vectors
- [SDKs Guide](../sdks/SDKS.md) - SDK vector operations

