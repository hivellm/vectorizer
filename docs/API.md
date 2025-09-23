# Vectorizer REST API Documentation

## Overview

The Vectorizer REST API provides HTTP endpoints for interacting with the vector database. All endpoints return JSON responses and follow RESTful conventions.

**Base URL**: `http://localhost:15001/api/v1`

## Authentication

Currently, the API does not require authentication. This will be added in Phase 2 Week 7-8.

## Endpoints

### Health Check

#### `GET /health`

Check the health status of the Vectorizer service.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 3600,
  "collections": 2,
  "total_vectors": 1500
}
```

### Collections

#### `GET /collections`

List all collections in the vector database.

**Response:**
```json
[
  {
    "name": "documents",
    "dimension": 384,
    "metric": "cosine",
    "vector_count": 1000,
    "created_at": "2025-09-23T10:00:00Z",
    "updated_at": "2025-09-23T12:30:00Z"
  }
]
```

#### `POST /collections`

Create a new collection.

**Request Body:**
```json
{
  "name": "documents",
  "dimension": 384,
  "metric": "cosine",
  "hnsw_config": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 64,
    "seed": 42
  }
}
```

**Parameters:**
- `name` (string, required): Collection name
- `dimension` (integer, required): Vector dimension (must be > 0)
- `metric` (string, required): Distance metric (`cosine`, `euclidean`, `dotproduct`)
- `hnsw_config` (object, optional): HNSW index configuration
  - `m` (integer, optional): Number of bidirectional links (default: 16)
  - `ef_construction` (integer, optional): Construction parameter (default: 200)
  - `ef_search` (integer, optional): Search parameter (default: 64)
  - `seed` (integer, optional): Random seed for reproducibility

**Response:**
```json
{
  "message": "Collection created successfully",
  "collection": "documents"
}
```

#### `GET /collections/{collection_name}`

Get information about a specific collection.

**Response:**
```json
{
  "name": "documents",
  "dimension": 384,
  "metric": "cosine",
  "vector_count": 1000,
  "created_at": "2025-09-23T10:00:00Z",
  "updated_at": "2025-09-23T12:30:00Z"
}
```

#### `DELETE /collections/{collection_name}`

Delete a collection and all its vectors.

**Response:**
```json
{
  "message": "Collection deleted successfully",
  "collection": "documents"
}
```

### Vectors

#### `POST /collections/{collection_name}/vectors`

Insert vectors into a collection.

**Request Body:**
```json
{
  "vectors": [
    {
      "id": "doc1",
      "vector": [0.1, 0.2, 0.3, ...],
      "payload": {
        "title": "Document Title",
        "content": "Document content...",
        "metadata": {
          "author": "John Doe",
          "created_at": "2025-09-23"
        }
      }
    }
  ]
}
```

**Parameters:**
- `vectors` (array, required): Array of vectors to insert
  - `id` (string, required): Unique vector identifier
  - `vector` (array of floats, required): Vector data (must match collection dimension)
  - `payload` (object, optional): Arbitrary JSON payload

**Response:**
```json
{
  "message": "Vectors inserted successfully",
  "inserted": 1
}
```

#### `GET /collections/{collection_name}/vectors/{vector_id}`

Retrieve a specific vector by ID.

**Response:**
```json
{
  "id": "doc1",
  "vector": [0.1, 0.2, 0.3, ...],
  "payload": {
    "title": "Document Title",
    "content": "Document content..."
  }
}
```

#### `DELETE /collections/{collection_name}/vectors/{vector_id}`

Delete a specific vector by ID.

**Response:**
```json
{
  "message": "Vector deleted successfully",
  "collection": "documents",
  "vector_id": "doc1"
}
```

### Search

#### `POST /collections/{collection_name}/search`

Search for similar vectors in a collection.

**Request Body:**
```json
{
  "vector": [0.1, 0.2, 0.3, ...],
  "limit": 10,
  "score_threshold": 0.7
}
```

**Parameters:**
- `vector` (array of floats, required): Query vector (must match collection dimension)
- `limit` (integer, optional): Maximum number of results (default: 10, max: 100)
- `score_threshold` (float, optional): Minimum similarity score threshold

**Response:**
```json
{
  "results": [
    {
      "id": "doc1",
      "score": 0.95,
      "vector": [0.1, 0.2, 0.3, ...],
      "payload": {
        "title": "Most Similar Document",
        "content": "..."
      }
    }
  ],
  "query_time_ms": 2.5
}
```

## Error Responses

All error responses follow this format:

```json
{
  "error": "Error description",
  "code": "ERROR_CODE",
  "details": {
    "additional": "context"
  }
}
```

### Common Error Codes

- `COLLECTION_NOT_FOUND`: Collection does not exist
- `VECTOR_NOT_FOUND`: Vector does not exist
- `INVALID_DIMENSION`: Vector dimension mismatch
- `INVALID_COLLECTION_NAME`: Invalid collection name
- `COLLECTION_ALREADY_EXISTS`: Collection already exists
- `VECTOR_INSERTION_FAILED`: Failed to insert vectors
- `SEARCH_FAILED`: Search operation failed

### HTTP Status Codes

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request parameters
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource already exists
- `500 Internal Server Error`: Server error

## Examples

### Complete Workflow Example

```bash
# 1. Check health
curl http://localhost:15001/api/v1/health

# 2. Create collection
curl -X POST http://localhost:15001/api/v1/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "documents",
    "dimension": 384,
    "metric": "cosine"
  }'

# 3. Insert vectors
curl -X POST http://localhost:15001/api/v1/collections/documents/vectors \
  -H "Content-Type: application/json" \
  -d '{
    "vectors": [
      {
        "id": "doc1",
        "vector": [0.1, 0.2, 0.3, ...],
        "payload": {"title": "Example Document"}
      }
    ]
  }'

# 4. Search
curl -X POST http://localhost:15001/api/v1/collections/documents/search \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, ...],
    "limit": 5
  }'
```

## Rate Limiting

Rate limiting is not currently implemented but will be added in Phase 2 Week 7-8.

## CORS

CORS is enabled for all origins during development. This will be configurable in production.

## Next Steps

- Authentication and API keys (Phase 2 Week 7-8)
- Rate limiting (Phase 2 Week 7-8)
- OpenAPI/Swagger documentation (Phase 2 Week 6)
- Batch operations (Phase 3)
- Filtering and metadata queries (Phase 3)
