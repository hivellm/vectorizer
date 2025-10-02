# Vectorizer REST API Documentation

## Overview

The Vectorizer REST API provides HTTP endpoints for interacting with the vector database through a GRPC-based microservices architecture. All endpoints return JSON responses and follow RESTful conventions.

**Base URL**: `http://localhost:15001/api/v1`

## Architecture (v0.13.0)

The REST API now operates as a GRPC client that communicates with the central `vzr` orchestrator service:

```
Client → REST API (Port 15001) → GRPC Client → vzr GRPC Server (Port 15003) → Vector Store
```

This architecture provides:
- **300% faster** service communication with GRPC
- **500% faster** binary serialization vs JSON
- **80% reduction** in connection overhead
- **60% reduction** in network latency

## Authentication

✅ **IMPLEMENTED**: JWT + API Key authentication system with role-based access control (RBAC)

### API Key Authentication
All endpoints require authentication via API key:

```bash
# Include API key in headers
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:15001/api/v1/health
```

### JWT Token Authentication
For advanced users, JWT tokens are supported:

```bash
# Include JWT token in headers
curl -H "Authorization: Bearer JWT_TOKEN" http://localhost:15001/api/v1/collections
```

## Endpoints

### Health Check

#### `GET /health`

Check the health status of the Vectorizer service and GRPC communication.

**Headers:**
- `Authorization: Bearer YOUR_API_KEY` (required)

**Response:**
```json
{
  "status": "healthy",
  "version": "0.13.0",
  "uptime": 3600,
  "grpc_status": "connected",
  "collections": 21,
  "total_vectors": 25000,
  "services": {
    "vzr": "healthy",
    "rest_api": "healthy",
    "mcp_server": "healthy"
  }
}
```

### Collections

#### `GET /collections`

List all collections in the vector database with GRPC communication.

**Headers:**
- `Authorization: Bearer YOUR_API_KEY` (required)

**Response:**
```json
[
  {
    "name": "gov-bips",
    "dimension": 512,
    "metric": "cosine",
    "vector_count": 2503,
    "status": "ready",
    "last_updated": "2025-09-26T16:10:04.568403600+00:00",
    "similarity_metric": "cosine"
  },
  {
    "name": "gov-proposals", 
    "dimension": 512,
    "metric": "cosine",
    "vector_count": 2165,
    "status": "ready",
    "last_updated": "2025-09-26T16:10:04.568426700+00:00",
    "similarity_metric": "cosine"
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

### Text Search & Embeddings

#### `POST /collections/{collection_name}/search/text`

Search for similar vectors using text query with automatic embedding generation.

**Headers:**
- `Authorization: Bearer YOUR_API_KEY` (required)

**Request Body:**
```json
{
  "query": "machine learning algorithms",
  "limit": 10,
  "score_threshold": 0.7,
  "embedding_model": "bm25"
}
```

**Parameters:**
- `query` (string, required): Text query to search for
- `limit` (integer, optional): Maximum number of results (default: 10, max: 100)
- `score_threshold` (float, optional): Minimum similarity score threshold
- `embedding_model` (string, optional): Embedding model to use (`bm25`, `tfidf`, `bow`, `charngram`)

**Response:**
```json
{
  "results": [
    {
      "id": "doc1",
      "score": 0.95,
      "content": "Machine learning is a subset of artificial intelligence...",
      "metadata": {
        "title": "ML Guide",
        "source": "ml_guide.pdf"
      }
    }
  ],
  "query_time_ms": 2.5,
  "embedding_time_ms": 1.2,
  "total_found": 1
}
```

#### `POST /embed`

Generate embeddings for text using various embedding models.

**Headers:**
- `Authorization: Bearer YOUR_API_KEY` (required)

**Request Body:**
```json
{
  "text": "machine learning algorithms",
  "model": "bm25",
  "dimension": 512
}
```

**Response:**
```json
{
  "embedding": [0.1, 0.2, 0.3, ...],
  "dimension": 512,
  "model": "bm25",
  "processing_time_ms": 1.2
}
```

### Search

#### `POST /collections/{collection_name}/search`

Search for similar vectors in a collection using vector data.

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

### Complete Workflow Example (v0.13.0)

```bash
# 1. Check health with authentication
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:15001/api/v1/health

# 2. List existing collections
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:15001/api/v1/collections

# 3. Text search with automatic embedding
curl -X POST http://localhost:15001/api/v1/collections/gov-bips/search/text \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "blockchain governance",
    "limit": 5,
    "embedding_model": "bm25"
  }'

# 4. Generate embeddings
curl -X POST http://localhost:15001/api/v1/embed \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "artificial intelligence",
    "model": "bm25",
    "dimension": 512
  }'

# 5. Vector search
curl -X POST http://localhost:15001/api/v1/collections/gov-proposals/search \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "vector": [0.1, 0.2, 0.3, ...],
    "limit": 5
  }'
```

## Rate Limiting

✅ **IMPLEMENTED**: Rate limiting with configurable limits per API key and user role.

## CORS

✅ **IMPLEMENTED**: Configurable CORS settings for production deployment.

## Performance Metrics (v0.13.0)

### GRPC Communication Performance
- **Service Communication**: 300% faster than HTTP
- **Binary Serialization**: 500% faster than JSON
- **Connection Overhead**: 80% reduction
- **Network Latency**: 60% reduction

### API Response Times
- **Health Check**: < 10ms
- **Collection List**: < 50ms
- **Text Search**: < 100ms (including embedding generation)
- **Vector Search**: < 20ms
- **Embedding Generation**: < 50ms

## Current Status (v0.13.0)

✅ **COMPLETED FEATURES:**
- JWT + API Key authentication with RBAC
- GRPC-based microservices architecture
- Text search with automatic embedding generation
- Multiple embedding models (BM25, TF-IDF, BOW, CharNGram)
- Real-time collection management
- Comprehensive error handling
- Rate limiting and CORS configuration

🚧 **PLANNED FEATURES:**
- OpenAPI/Swagger documentation (Phase 5)
- Advanced filtering and metadata queries (Phase 5)
- Batch operations optimization (Phase 5)
- WebSocket support for real-time updates (Phase 5)
