---
title: Updating Vectors
module: vectors
id: updating-vectors
order: 2
description: How to update vectors in collections
tags: [vectors, update, modify, edit]
---

# Updating Vectors

Complete guide to updating vectors in Vectorizer collections.

## Update Vector Content

Update both text content and metadata.

### REST API

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

### Using Python SDK

```python
await client.update_vector(
    "my_collection",
    "vec_001",
    text="Updated content",
    metadata={
        "source": "updated_readme.md",
        "updated_at": "2024-01-01"
    }
)
```

### Using TypeScript SDK

```typescript
await client.updateVector("my_collection", "vec_001", {
    text: "Updated content",
    metadata: {
        source: "updated_readme.md",
        updated_at: "2024-01-01"
    }
});
```

## Update Metadata Only

Update only metadata without changing vector content.

### REST API

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

### Python Example

```python
await client.update_vector(
    "my_collection",
    "vec_001",
    metadata={
        "updated_at": "2024-01-01",
        "version": 2
    }
)
```

## Update Vector Data Only

Update vector data without changing metadata.

### Python Example

```python
new_vector = [0.1, 0.2, 0.3, 0.4] * 96  # 384-dimensional
await client.update_vector(
    "my_collection",
    "vec_001",
    vector=new_vector
)
```

## Batch Update

Update multiple vectors efficiently.

### REST API

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

### Python Example

```python
updates = [
    {
        "id": "vec_001",
        "text": "Updated content 1",
        "metadata": {"updated_at": "2024-01-01"}
    },
    {
        "id": "vec_002",
        "text": "Updated content 2",
        "metadata": {"updated_at": "2024-01-01"}
    }
]

await client.batch_update("my_collection", updates)
```

## Common Patterns

### Pattern 1: Update Timestamp

```python
async def update_timestamp(collection_name, vector_id):
    """Update only the timestamp metadata."""
    vector = await client.get_vector(collection_name, vector_id)
    current_metadata = vector.get("metadata", {})
    
    await client.update_vector(
        collection_name,
        vector_id,
        metadata={
            **current_metadata,
            "updated_at": datetime.now().isoformat()
        }
    )
```

### Pattern 2: Incremental Update

```python
async def update_document(collection_name, doc_id, new_content):
    """Update document content and metadata."""
    await client.update_vector(
        collection_name,
        doc_id,
        text=new_content,
        metadata={"updated_at": datetime.now().isoformat()}
    )
```

## Best Practices

1. **Use batch updates**: Much faster for multiple updates
2. **Preserve existing metadata**: Merge with existing metadata
3. **Update timestamps**: Track when vectors were updated
4. **Validate dimensions**: Ensure new vectors match collection dimension

## Related Topics

- [Inserting Vectors](./INSERT.md) - Adding new vectors
- [Deleting Vectors](./DELETE.md) - Removing vectors
- [Vectors Guide](./VECTORS.md) - Complete vectors guide

