---
title: Deleting Vectors
module: vectors
id: deleting-vectors
order: 3
description: How to delete vectors from collections
tags: [vectors, delete, remove]
---

# Deleting Vectors

Complete guide to deleting vectors from Vectorizer collections.

## Delete Single Vector

### REST API

```bash
curl -X DELETE http://localhost:15002/collections/my_collection/vectors/vec_001
```

### Using Python SDK

```python
await client.delete_vector("my_collection", "vec_001")
```

### Using TypeScript SDK

```typescript
await client.deleteVector("my_collection", "vec_001");
```

## Delete Multiple Vectors

### REST API

```bash
curl -X POST http://localhost:15002/collections/my_collection/batch_delete \
  -H "Content-Type: application/json" \
  -d '{
    "ids": ["vec_001", "vec_002", "vec_003"]
  }'
```

### Using Python SDK

```python
ids_to_delete = ["vec_001", "vec_002", "vec_003"]
await client.batch_delete("my_collection", ids_to_delete)
```

### Using TypeScript SDK

```typescript
const idsToDelete = ["vec_001", "vec_002", "vec_003"];
await client.batchDelete("my_collection", idsToDelete);
```

## Delete by Filter

Delete vectors matching specific criteria.

### REST API

```bash
curl -X POST http://localhost:15002/collections/my_collection/delete_by_filter \
  -H "Content-Type: application/json" \
  -d '{
    "filter": {
      "category": "old_docs"
    }
  }'
```

### Python Example

```python
await client.delete_by_filter(
    "my_collection",
    filter={"category": "old_docs"}
)
```

## Common Patterns

### Pattern 1: Cleanup Old Documents

```python
async def cleanup_old_documents(collection_name, older_than_days=30):
    """Delete documents older than specified days."""
    cutoff_date = (datetime.now() - timedelta(days=older_than_days)).isoformat()
    
    # Search for old documents
    results = await client.search(
        collection_name,
        "",
        limit=10000,
        filter={"created_at": {"$lt": cutoff_date}}
    )
    
    # Delete found documents
    ids_to_delete = [r["id"] for r in results]
    if ids_to_delete:
        await client.batch_delete(collection_name, ids_to_delete)
```

### Pattern 2: Delete by Category

```python
async def delete_by_category(collection_name, category):
    """Delete all vectors in a category."""
    await client.delete_by_filter(
        collection_name,
        filter={"category": category}
    )
```

## Best Practices

1. **Use batch delete**: Much faster for multiple deletions
2. **Verify before delete**: Check what will be deleted
3. **Backup important data**: Before bulk deletions
4. **Use filters**: When possible, more efficient

## Related Topics

- [Inserting Vectors](./INSERT.md) - Adding vectors
- [Updating Vectors](./UPDATE.md) - Updating vectors
- [Vectors Guide](./VECTORS.md) - Complete vectors guide

