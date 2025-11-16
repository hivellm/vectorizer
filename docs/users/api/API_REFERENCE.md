---
title: REST API Reference
module: api
id: api-reference
order: 1
description: Complete REST API reference for Vectorizer
tags: [api, rest, endpoints, reference]
---

# REST API Reference

Complete reference for all Vectorizer REST API endpoints.

## Base URL

All API requests should be made to:

```
http://localhost:15002
```

Or your configured host and port.

## Authentication

Currently, Vectorizer does not require authentication. All endpoints are publicly accessible.

**Note:** For production deployments, consider using a reverse proxy with authentication.

## Response Format

### Success Response

```json
{
  "status": "success",
  "data": { ... }
}
```

### Error Response

```json
{
  "error": {
    "type": "error_type",
    "message": "Error message",
    "status_code": 400
  }
}
```

## System Endpoints

### Health Check

Check if the server is running and healthy.

**Endpoint:** `GET /health`

**Response:**

```json
{
  "status": "healthy",
  "timestamp": "2024-11-16T10:30:00Z",
  "version": "1.3.0",
  "cache": {
    "size": 123,
    "capacity": 1000,
    "hits": 500,
    "misses": 200,
    "evictions": 50,
    "hit_rate": 0.714
  }
}
```

**Example:**

```bash
curl http://localhost:15002/health
```

### System Statistics

Get system-wide statistics.

**Endpoint:** `GET /stats`

**Response:**

```json
{
  "collections": 5,
  "total_vectors": 125000,
  "memory_usage_bytes": 512000000,
  "disk_usage_bytes": 256000000
}
```

## Collection Endpoints

### List Collections

Get a list of all collections.

**Endpoint:** `GET /collections`

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

**Example:**

```bash
curl http://localhost:15002/collections
```

### Create Collection

Create a new collection.

**Endpoint:** `POST /collections`

**Request Body:**

```json
{
  "name": "my_collection",
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

**Response:**

```json
{
  "status": "created",
  "collection": {
    "name": "my_collection",
    "dimension": 384,
    "metric": "cosine"
  }
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_collection",
    "dimension": 384,
    "metric": "cosine"
  }'
```

### Get Collection

Get information about a specific collection.

**Endpoint:** `GET /collections/{name}`

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

**Example:**

```bash
curl http://localhost:15002/collections/my_collection
```

### Delete Collection

Delete a collection and all its vectors.

**Endpoint:** `DELETE /collections/{name}`

**Response:**

```json
{
  "status": "deleted",
  "collection": "my_collection"
}
```

**Example:**

```bash
curl -X DELETE http://localhost:15002/collections/my_collection
```

## Vector Endpoints

### Insert Vector

Insert a single vector into a collection.

**Endpoint:** `POST /collections/{name}/insert`

**Request Body:**

```json
{
  "id": "vector_001",
  "text": "Vectorizer is a high-performance vector database",
  "metadata": {
    "source": "readme",
    "category": "documentation"
  }
}
```

**Or with pre-computed vector:**

```json
{
  "id": "vector_001",
  "vector": [0.1, 0.2, 0.3, ...],
  "metadata": {
    "source": "custom"
  }
}
```

**Response:**

```json
{
  "id": "vector_001",
  "status": "inserted"
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/collections/my_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Vectorizer is awesome!",
    "metadata": {"source": "readme"}
  }'
```

### Get Vector

Retrieve a specific vector by ID.

**Endpoint:** `GET /collections/{name}/vectors/{id}`

**Query Parameters:**

- `with_vector` (boolean): Include vector data in response
- `with_payload` (boolean): Include metadata in response

**Response:**

```json
{
  "id": "vector_001",
  "vector": [0.1, 0.2, 0.3, ...],
  "payload": {
    "source": "readme",
    "category": "documentation"
  }
}
```

**Example:**

```bash
curl "http://localhost:15002/collections/my_collection/vectors/vector_001?with_vector=true&with_payload=true"
```

### Update Vector

Update an existing vector.

**Endpoint:** `PATCH /collections/{name}/vectors/{id}`

**Request Body:**

```json
{
  "text": "Updated content",
  "metadata": {
    "last_modified": "2024-11-16"
  }
}
```

**Response:**

```json
{
  "id": "vector_001",
  "status": "updated"
}
```

**Example:**

```bash
curl -X PATCH http://localhost:15002/collections/my_collection/vectors/vector_001 \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Updated content",
    "metadata": {"last_modified": "2024-11-16"}
  }'
```

### Delete Vector

Delete a vector from a collection.

**Endpoint:** `DELETE /collections/{name}/vectors/{id}`

**Response:**

```json
{
  "id": "vector_001",
  "status": "deleted"
}
```

**Example:**

```bash
curl -X DELETE http://localhost:15002/collections/my_collection/vectors/vector_001
```

### List Vectors

List all vectors in a collection.

**Endpoint:** `GET /collections/{name}/vectors`

**Query Parameters:**

- `limit` (integer): Maximum number of vectors to return
- `offset` (integer): Number of vectors to skip
- `with_vector` (boolean): Include vector data
- `with_payload` (boolean): Include metadata

**Response:**

```json
{
  "vectors": [
    {
      "id": "vector_001",
      "payload": {
        "source": "readme"
      }
    }
  ],
  "total": 1250
}
```

## Search Endpoints

### Basic Search

Search for similar vectors.

**Endpoint:** `POST /collections/{name}/search`

**Request Body:**

```json
{
  "query": "vector database",
  "limit": 10,
  "similarity_threshold": 0.5,
  "with_vector": false,
  "with_payload": true
}
```

**Or with vector:**

```json
{
  "vector": [0.1, 0.2, 0.3, ...],
  "limit": 10
}
```

**Response:**

```json
{
  "results": [
    {
      "id": "vector_001",
      "score": 0.95,
      "payload": {
        "source": "readme"
      }
    }
  ]
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/collections/my_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "vector database",
    "limit": 10
  }'
```

### Intelligent Search

Advanced search with query expansion and MMR.

**Endpoint:** `POST /collections/{name}/intelligent_search`

**Request Body:**

```json
{
  "query": "neural networks for image recognition",
  "max_results": 15,
  "domain_expansion": true,
  "technical_focus": true,
  "mmr_enabled": true,
  "mmr_lambda": 0.7
}
```

**Response:**

```json
{
  "results": [
    {
      "id": "vector_001",
      "score": 0.92,
      "payload": { ... }
    }
  ],
  "expanded_queries": [
    "neural networks",
    "image recognition",
    "deep learning"
  ]
}
```

### Semantic Search

High-precision semantic search with reranking.

**Endpoint:** `POST /collections/{name}/semantic_search`

**Request Body:**

```json
{
  "query": "deep learning architectures",
  "max_results": 10,
  "semantic_reranking": true,
  "similarity_threshold": 0.75
}
```

### Hybrid Search

Combine dense and sparse vector search.

**Endpoint:** `POST /collections/{name}/hybrid_search`

**Request Body:**

```json
{
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
}
```

### Multi-Collection Search

Search across multiple collections.

**Endpoint:** `POST /multi_collection_search`

**Request Body:**

```json
{
  "collections": ["docs", "code", "wiki"],
  "query": "authentication mechanism",
  "max_per_collection": 5,
  "max_total_results": 20,
  "cross_collection_reranking": true
}
```

## Batch Operations

### Batch Insert

Insert multiple vectors efficiently.

**Endpoint:** `POST /collections/{name}/batch_insert`

**Request Body:**

```json
{
  "vectors": [
    {
      "id": "vec_001",
      "text": "First document",
      "metadata": { "doc_id": 1 }
    },
    {
      "id": "vec_002",
      "text": "Second document",
      "metadata": { "doc_id": 2 }
    }
  ]
}
```

**Response:**

```json
{
  "status": "success",
  "inserted_count": 2,
  "failed_count": 0,
  "errors": []
}
```

### Batch Update

Update multiple vectors.

**Endpoint:** `POST /collections/{name}/batch_update`

**Request Body:**

```json
{
  "vectors": [
    {
      "id": "vec_001",
      "text": "Updated document 1",
      "metadata": { "updated": true }
    }
  ]
}
```

### Batch Delete

Delete multiple vectors.

**Endpoint:** `POST /collections/{name}/batch_delete`

**Request Body:**

```json
{
  "ids": ["vec_001", "vec_002", "vec_003"]
}
```

**Response:**

```json
{
  "status": "success",
  "deleted_count": 3,
  "failed_count": 0,
  "errors": []
}
```

### Batch Search

Search with multiple queries.

**Endpoint:** `POST /collections/{name}/batch_search`

**Request Body:**

```json
{
  "queries": [
    { "query": "vector database", "limit": 5 },
    { "query": "semantic search", "limit": 5 }
  ]
}
```

**Response:**

```json
{
  "results": [
    [
      {"id": "vec_001", "score": 0.95, ...},
      {"id": "vec_002", "score": 0.92, ...}
    ],
    [
      {"id": "vec_003", "score": 0.88, ...}
    ]
  ]
}
```

## Qdrant-Compatible Endpoints

Vectorizer provides Qdrant-compatible endpoints under the `/qdrant` prefix.

### List Collections (Qdrant)

**Endpoint:** `GET /qdrant/collections`

### Create Collection (Qdrant)

**Endpoint:** `PUT /qdrant/collections/{name}`

### Get Collection (Qdrant)

**Endpoint:** `GET /qdrant/collections/{name}`

### Upsert Points

**Endpoint:** `PUT /qdrant/collections/{name}/points`

### Search Points

**Endpoint:** `POST /qdrant/collections/{name}/points/search`

### Retrieve Points

**Endpoint:** `GET /qdrant/collections/{name}/points`

### Delete Points

**Endpoint:** `POST /qdrant/collections/{name}/points/delete`

### Count Points

**Endpoint:** `POST /qdrant/collections/{name}/points/count`

### Scroll Points

**Endpoint:** `POST /qdrant/collections/{name}/points/scroll`

### Collection Aliases

**Endpoint:** `POST /qdrant/collections/aliases`

**Endpoint:** `GET /qdrant/collections/{name}/aliases`

**Endpoint:** `GET /qdrant/aliases`

## File Operations

### Get File Content

**Endpoint:** `POST /file/content`

### List Files in Collection

**Endpoint:** `POST /file/list`

### Get File Summary

**Endpoint:** `POST /file/summary`

### Get File Chunks

**Endpoint:** `POST /file/chunks`

### Get Project Outline

**Endpoint:** `POST /file/outline`

### Get Related Files

**Endpoint:** `POST /file/related`

### Search by File Type

**Endpoint:** `POST /file/search_by_type`

## Discovery Endpoints

### Discover

**Endpoint:** `POST /discover`

### Filter Collections

**Endpoint:** `POST /discovery/filter_collections`

### Score Collections

**Endpoint:** `POST /discovery/score_collections`

### Expand Queries

**Endpoint:** `POST /discovery/expand_queries`

### Broad Discovery

**Endpoint:** `POST /discovery/broad_discovery`

### Semantic Focus

**Endpoint:** `POST /discovery/semantic_focus`

## Monitoring Endpoints

### Prometheus Metrics

**Endpoint:** `GET /prometheus/metrics`

Returns Prometheus-formatted metrics.

**Example:**

```bash
curl http://localhost:15002/prometheus/metrics
```

## Error Codes

| Status Code | Description                        |
| ----------- | ---------------------------------- |
| 200         | Success                            |
| 400         | Bad Request - Invalid input        |
| 404         | Not Found - Resource doesn't exist |
| 409         | Conflict - Resource already exists |
| 500         | Internal Server Error              |

## Rate Limiting

Currently, Vectorizer does not implement rate limiting. For production deployments, consider using a reverse proxy with rate limiting.

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection operations
- [Search Guide](../search/SEARCH.md) - Search operations
- [Vectors Guide](../vectors/VECTORS.md) - Vector operations
- [SDKs Guide](../sdks/README.md) - Client SDKs
