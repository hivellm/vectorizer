# API Module

## Overview

The API module provides comprehensive API capabilities for the Vectorizer service, including REST, GraphQL, and Graph APIs. This module serves as the integration layer between clients and the core vector database functionality.

## Purpose

The API module exposes Vectorizer's functionality through multiple interfaces:

- **REST API**: Traditional HTTP endpoints for collection management, vector operations, and search
- **GraphQL API**: Flexible query interface with schema introspection
- **Graph API**: Relationship and graph traversal operations
- **Advanced API**: Extended capabilities including rate limiting, analytics, and versioning

## Key Components

### REST API (`src/server/rest_handlers.rs`)

Core REST endpoints for vector operations:

- Collection management (create, read, update, delete)
- Vector operations (insert, update, delete, search)
- File upload and processing
- Workspace management
- Health checks and statistics

### GraphQL API (`src/api/graphql/`)

Full GraphQL implementation with:

- Query operations for collections, vectors, and search
- Mutation operations for data modification
- GraphQL Playground for interactive exploration
- Schema introspection and type system

### Graph API (`src/api/graph.rs`)

Graph and relationship operations:

- Node and edge management
- Graph traversal and path finding
- Relationship discovery
- Graph status and configuration

### Advanced API (`src/api/advanced_api.rs`)

Extended API capabilities:

- API versioning and backward compatibility
- Rate limiting and throttling
- API analytics and monitoring
- SDK generation support
- OpenAPI/Swagger documentation

### Cluster API (`src/api/cluster.rs`)

Distributed cluster operations:

- Cluster management
- Sharding operations
- Node coordination

## API Endpoints

### Collection Management

#### List Collections

**GET** `/collections`

Returns a list of all collections in the store.

**Response:**
```json
{
  "collections": ["collection1", "collection2"]
}
```

#### Create Collection

**POST** `/collections`

Creates a new vector collection.

**Request Body:**
```json
{
  "name": "my_collection",
  "config": {
    "dimension": 512,
    "metric": "cosine"
  }
}
```

**Response:**
```json
{
  "status": "created",
  "collection": "my_collection"
}
```

#### Get Collection

**GET** `/collections/{name}`

Retrieves information about a specific collection.

**Path Parameters:**
- `name` - Collection name

**Response:**
```json
{
  "name": "my_collection",
  "vector_count": 1000,
  "dimension": 512,
  "metric": "cosine"
}
```

#### Delete Collection

**DELETE** `/collections/{name}`

Deletes a collection and all its vectors.

**Path Parameters:**
- `name` - Collection name

**Response:**
```json
{
  "status": "deleted"
}
```

### Vector Operations

#### Search Vectors

**POST** `/collections/{name}/search`

Performs similarity search in a collection.

**Path Parameters:**
- `name` - Collection name

**Request Body:**
```json
{
  "vector": [0.1, 0.2, 0.3, ...],
  "limit": 10,
  "threshold": 0.7
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "vector1",
      "score": 0.95,
      "payload": {}
    }
  ]
}
```

#### Search by Text

**POST** `/collections/{name}/search/text`

Performs text-based search using embeddings.

**Path Parameters:**
- `name` - Collection name

**Request Body:**
```json
{
  "query": "search text",
  "limit": 10,
  "threshold": 0.7
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "vector1",
      "score": 0.95,
      "payload": {}
    }
  ]
}
```

#### Insert Vector

**POST** `/insert`

Inserts a vector into a collection.

**Request Body:**
```json
{
  "collection": "my_collection",
  "id": "vector1",
  "vector": [0.1, 0.2, 0.3, ...],
  "payload": {}
}
```

**Response:**
```json
{
  "status": "inserted",
  "id": "vector1"
}
```

#### Update Vector

**POST** `/update`

Updates an existing vector.

**Request Body:**
```json
{
  "collection": "my_collection",
  "id": "vector1",
  "vector": [0.1, 0.2, 0.3, ...],
  "payload": {}
}
```

**Response:**
```json
{
  "status": "updated"
}
```

#### Delete Vector

**POST** `/delete`

Deletes a vector from a collection.

**Request Body:**
```json
{
  "collection": "my_collection",
  "id": "vector1"
}
```

**Response:**
```json
{
  "status": "deleted"
}
```

### File Operations

#### Upload File

**POST** `/files/upload`

Uploads and processes a file for indexing.

**Content-Type:** `multipart/form-data`

**Form Fields:**
- `file` - File to upload (required)
- `collection_name` - Target collection (required)
- `chunk_size` - Chunk size in characters (optional, default: 2048)
- `chunk_overlap` - Chunk overlap in characters (optional, default: 256)
- `use_transmutation` - Enable document conversion (optional, default: false)
- `metadata` - Additional metadata as JSON string (optional)

**Response:**
```json
{
  "status": "uploaded",
  "file_id": "file123",
  "chunks_created": 10
}
```

#### Get Upload Config

**GET** `/files/config`

Retrieves file upload configuration.

**Response:**
```json
{
  "max_file_size_mb": 100,
  "supported_formats": ["pdf", "docx", "txt"],
  "transmutation_enabled": true
}
```

### Graph Operations

#### List Nodes

**GET** `/graph/nodes/{collection}`

Lists all nodes in a collection's graph.

**Path Parameters:**
- `collection` - Collection name

**Response:**
```json
{
  "nodes": [
    {
      "id": "node1",
      "collection": "my_collection",
      "payload": {}
    }
  ],
  "count": 10
}
```

#### Get Neighbors

**GET** `/graph/nodes/{collection}/{node_id}/neighbors`

Gets neighbors of a specific node.

**Path Parameters:**
- `collection` - Collection name
- `node_id` - Node identifier

**Response:**
```json
{
  "neighbors": [
    {
      "node": {
        "id": "node2",
        "collection": "my_collection"
      },
      "edge": {
        "id": "edge1",
        "from": "node1",
        "to": "node2",
        "relationship_type": "related"
      }
    }
  ]
}
```

#### Find Related Nodes

**POST** `/graph/nodes/{collection}/{node_id}/related`

Finds related nodes using graph traversal.

**Path Parameters:**
- `collection` - Collection name
- `node_id` - Starting node identifier

**Request Body:**
```json
{
  "max_hops": 3,
  "relationship_type": "related"
}
```

**Response:**
```json
{
  "related": [
    {
      "node": {
        "id": "node2",
        "collection": "my_collection"
      },
      "distance": 1,
      "weight": 0.95
    }
  ]
}
```

#### Find Path

**POST** `/graph/path`

Finds a path between two nodes.

**Request Body:**
```json
{
  "collection": "my_collection",
  "from": "node1",
  "to": "node2",
  "max_hops": 5
}
```

**Response:**
```json
{
  "path": [
    {"id": "node1"},
    {"id": "node2"}
  ],
  "distance": 1
}
```

#### Create Edge

**POST** `/graph/edges`

Creates an edge between two nodes.

**Request Body:**
```json
{
  "collection": "my_collection",
  "from": "node1",
  "to": "node2",
  "relationship_type": "related",
  "weight": 0.95
}
```

**Response:**
```json
{
  "status": "created",
  "edge_id": "edge1"
}
```

#### Delete Edge

**DELETE** `/graph/edges/{edge_id}`

Deletes an edge.

**Path Parameters:**
- `edge_id` - Edge identifier

**Response:**
```json
{
  "status": "deleted"
}
```

### GraphQL API

#### GraphQL Endpoint

**POST** `/graphql`

Executes GraphQL queries and mutations.

**Request Body:**
```json
{
  "query": "query { collections { name vectorCount } }",
  "variables": {}
}
```

**Response:**
```json
{
  "data": {
    "collections": [
      {
        "name": "my_collection",
        "vectorCount": 1000
      }
    ]
  }
}
```

#### GraphQL Playground

**GET** `/graphql` or `/graphiql`

Interactive GraphQL playground for exploring the API.

### Health and Status

#### Health Check

**GET** `/health`

Returns server health status.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-01-07T12:00:00Z",
  "version": "1.0.0",
  "cache": {
    "size": 100,
    "capacity": 1000,
    "hit_rate": 0.95
  }
}
```

#### Get Stats

**GET** `/stats`

Returns server statistics.

**Response:**
```json
{
  "collections": 5,
  "total_vectors": 10000,
  "uptime_seconds": 3600,
  "version": "1.0.0"
}
```

#### Get Status

**GET** `/status`

Returns detailed server status.

**Response:**
```json
{
  "status": "running",
  "collections": 5,
  "total_vectors": 10000,
  "memory_usage_mb": 512,
  "disk_usage_mb": 1024
}
```

## Authentication

### Login

**POST** `/auth/login`

Authenticates a user and returns a JWT token.

**Request Body:**
```json
{
  "username": "user",
  "password": "password"
}
```

**Response:**
```json
{
  "token": "jwt_token_here",
  "refresh_token": "refresh_token_here",
  "expires_in": 3600
}
```

### Get Current User

**GET** `/auth/me`

Returns information about the authenticated user.

**Headers:**
- `Authorization: Bearer {token}`

**Response:**
```json
{
  "username": "user",
  "role": "admin",
  "permissions": ["read", "write"]
}
```

### Create API Key

**POST** `/auth/keys`

Creates a new API key for programmatic access.

**Headers:**
- `Authorization: Bearer {token}`

**Request Body:**
```json
{
  "name": "my_api_key",
  "permissions": ["read", "write"]
}
```

**Response:**
```json
{
  "key_id": "key123",
  "api_key": "generated_key_here",
  "created_at": "2025-01-07T12:00:00Z"
}
```

## Error Responses

All endpoints return appropriate HTTP status codes:

- `200 OK` - Success
- `201 Created` - Resource created
- `400 Bad Request` - Invalid request
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict
- `500 Internal Server Error` - Server error

### Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}
  }
}
```

## Rate Limiting

When rate limiting is enabled, responses include rate limit headers:

- `X-RateLimit-Limit` - Maximum requests per window
- `X-RateLimit-Remaining` - Remaining requests in current window
- `X-RateLimit-Reset` - Time when the rate limit resets

When rate limit is exceeded, returns `429 Too Many Requests`.

## API Versioning

The API supports versioning through the `Accept` header:

```
Accept: application/vnd.vectorizer.v1+json
```

Supported versions:
- `v1` - Current stable version (default)

## CORS

CORS is enabled by default. Configure allowed origins, methods, and headers in the server configuration.

## API Documentation

For complete API documentation, see:

- [REST API Reference](../api/README.md) - Complete REST API documentation with OpenAPI schema
- [OpenAPI Schema](../api/openapi.yaml) - OpenAPI 3.0.3 specification
- [File Upload Documentation](../api/FILE_UPLOAD_TRANSMUTATION.md) - File upload and transmutation guide
- [GraphQL Schema](../api/graphql_schema.md) - GraphQL type definitions and queries
- [Graph API Reference](../api/graph_api.md) - Graph and relationship operations

## Additional Resources

- [Authentication Guide](../specs/AUTHENTICATION.md) - Authentication and authorization
- [API Architecture](../specs/API_ARCHITECTURE.md) - API design patterns
- [Performance Guide](../specs/PERFORMANCE.md) - Performance optimization
- [Error Handling](../reference/errors.md) - Error types and handling

## Code References

- REST Handlers: `src/server/rest_handlers.rs`
- GraphQL Schema: `src/api/graphql/`
- Graph API: `src/api/graph.rs`
- Advanced API: `src/api/advanced_api.rs`
- Cluster API: `src/api/cluster.rs`
