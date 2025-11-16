---
title: Collection Operations
module: collections
id: collection-operations
order: 3
description: Managing and operating on collections
tags: [collections, operations, management]
---

# Collection Operations

Complete guide to managing collections in Vectorizer.

## Listing Collections

### List All Collections

```bash
curl http://localhost:15002/collections
```

**Response:**

```json
{
  "collections": [
    {
      "name": "my_collection",
      "vector_count": 1250,
      "dimension": 384,
      "metric": "cosine"
    }
  ]
}
```

### Using Python SDK

```python
collections = await client.list_collections()
for collection in collections:
    print(f"{collection['name']}: {collection['vector_count']} vectors")
```

### Using TypeScript SDK

```typescript
const collections = await client.listCollections();
collections.forEach((collection) => {
  console.log(`${collection.name}: ${collection.vector_count} vectors`);
});
```

## Getting Collection Information

### Get Collection Details

```bash
curl http://localhost:15002/collections/my_collection
```

**Response:**

```json
{
  "name": "my_collection",
  "vector_count": 1250,
  "dimension": 384,
  "metric": "cosine",
  "hnsw_config": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 64
  },
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 8
  }
}
```

### Using Python SDK

```python
info = await client.get_collection_info("my_collection")
print(f"Name: {info['name']}")
print(f"Vectors: {info['vector_count']}")
print(f"Dimension: {info['dimension']}")
print(f"Metric: {info['metric']}")
```

### Using TypeScript SDK

```typescript
const info = await client.getCollectionInfo("my_collection");
console.log(`Name: ${info.name}`);
console.log(`Vectors: ${info.vector_count}`);
console.log(`Dimension: ${info.dimension}`);
console.log(`Metric: ${info.metric}`);
```

## Updating Collection Configuration

### Update HNSW Parameters

```bash
curl -X PATCH http://localhost:15002/collections/my_collection \
  -H "Content-Type: application/json" \
  -d '{
    "hnsw_config": {
      "ef_search": 128
    }
  }'
```

**Note:** Some settings (dimension, metric) cannot be changed after creation.

### Update Quantization

```bash
curl -X PATCH http://localhost:15002/collections/my_collection \
  -H "Content-Type: application/json" \
  -d '{
    "quantization": {
      "enabled": true,
      "type": "scalar",
      "bits": 4
    }
  }'
```

## Deleting Collections

### Delete Collection

```bash
curl -X DELETE http://localhost:15002/collections/my_collection
```

**Warning:** This permanently deletes the collection and all its vectors!

### Using Python SDK

```python
await client.delete_collection("my_collection")
```

### Using TypeScript SDK

```typescript
await client.deleteCollection("my_collection");
```

### Safety Check Before Delete

```python
# Check collection exists and get info
try:
    info = await client.get_collection_info("my_collection")
    print(f"Collection has {info['vector_count']} vectors")

    # Confirm deletion
    confirm = input("Delete collection? (yes/no): ")
    if confirm.lower() == "yes":
        await client.delete_collection("my_collection")
        print("Collection deleted")
    else:
        print("Deletion cancelled")
except Exception as e:
    print(f"Collection not found: {e}")
```

## Collection Statistics

### Get Collection Stats

```bash
curl http://localhost:15002/collections/my_collection/stats
```

**Response:**

```json
{
  "vector_count": 1250,
  "indexed_count": 1250,
  "memory_usage_bytes": 1920000,
  "disk_usage_bytes": 960000
}
```

## Collection Health Check

### Check Collection Status

```python
async def check_collection_health(collection_name):
    """Check if collection is healthy."""
    try:
        info = await client.get_collection_info(collection_name)

        # Check if collection has vectors
        if info['vector_count'] == 0:
            return {"status": "empty", "message": "Collection has no vectors"}

        # Check if indexed
        stats = await client.get_collection_stats(collection_name)
        if stats['indexed_count'] < stats['vector_count']:
            return {
                "status": "indexing",
                "message": f"Indexing {stats['indexed_count']}/{stats['vector_count']}"
            }

        return {"status": "healthy", "message": "Collection is ready"}
    except Exception as e:
        return {"status": "error", "message": str(e)}
```

## Batch Operations

### List Multiple Collections

```python
async def list_collections_with_stats():
    """List all collections with statistics."""
    collections = await client.list_collections()

    for collection_name in collections:
        info = await client.get_collection_info(collection_name)
        stats = await client.get_collection_stats(collection_name)

        print(f"\n{collection_name}:")
        print(f"  Vectors: {info['vector_count']}")
        print(f"  Memory: {stats['memory_usage_bytes'] / 1024 / 1024:.2f} MB")
```

## Related Topics

- [Creating Collections](./CREATING.md) - How to create collections
- [Collection Configuration](./CONFIGURATION.md) - Configuration options
- [Collections Overview](./COLLECTIONS.md) - Complete guide
