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

## MCP (Model Context Protocol)

Vectorizer implements a comprehensive MCP server using StreamableHTTP (v0.9.0) for AI-powered IDE integration.

### Base URL

```
http://localhost:15002/mcp
```

### Protocol

- **Transport**: StreamableHTTP (bi-directional HTTP streaming)
- **Protocol**: JSON-RPC 2.0
- **Format**: Server-Sent Events (SSE) for streaming

### Connection

**Endpoint:** `POST /mcp`

Establishes an MCP session and handles JSON-RPC 2.0 requests.

**Request Format:**

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "tool_name",
    "arguments": {
      "param1": "value1",
      "param2": "value2"
    }
  },
  "id": 1
}
```

**Response Format:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Result data"
      }
    ]
  },
  "id": 1
}
```

### Available MCP Tools (38+ tools)

#### Collection Management

- `list_collections` - List all collections
- `create_collection` - Create a new collection
- `get_collection_info` - Get collection details
- `delete_collection` - Delete a collection

#### Vector Operations

- `insert_text` - Insert text (auto-embed)
- `get_vector` - Retrieve vector by ID
- `update_vector` - Update existing vector
- `delete_vectors` - Delete vectors
- `embed_text` - Generate embedding for text

#### Search Operations

- `search` - Basic semantic search
- `search_intelligent` - Intelligent multi-query search
- `search_semantic` - Pure semantic search
- `search_extra` - Extended search with filters
- `search_hybrid` - Hybrid dense + sparse search
- `multi_collection_search` - Cross-collection search

#### Batch Operations

- `batch_insert_texts` - Batch insert texts
- `insert_texts` - Insert multiple texts
- `batch_search_vectors` - Batch search
- `batch_update_vectors` - Batch update
- `batch_delete_vectors` - Batch delete

#### Discovery Operations

- `filter_collections` - Filter collections by query
- `expand_queries` - Expand query terms
- `discover` - Discovery pipeline
- `score_collections` - Score collection relevance
- `broad_discovery` - Broad search across collections
- `semantic_focus` - Focused semantic search
- `compress_evidence` - Compress search results
- `build_answer_plan` - Build answer plan
- `render_llm_prompt` - Render LLM prompt
- `promote_readme` - Promote README files

#### File Operations

- `get_file_content` - Get file content
- `list_files` - List files in collection
- `get_file_chunks` - Get file chunks
- `get_project_outline` - Get project outline
- `get_related_files` - Find related files
- `search_by_file_type` - Search by file type

### Example: MCP Tool Call

**Search Vectors:**

```bash
curl -X POST http://localhost:15002/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "search",
      "arguments": {
        "collection": "documents",
        "query": "machine learning",
        "limit": 5
      }
    },
    "id": 1
  }'
```

**Create Collection:**

```bash
curl -X POST http://localhost:15002/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "create_collection",
      "arguments": {
        "name": "my_collection",
        "dimension": 384,
        "metric": "cosine"
      }
    },
    "id": 2
  }'
```

## UMICP (Universal Multi-Agent Communication Protocol)

Vectorizer supports UMICP v0.2.1 for high-performance envelope-based communication.

### Base URL

```
http://localhost:15002/umicp
```

### Protocol

- **Version**: UMICP v0.2.1
- **Transport**: HTTP POST with envelope format
- **Format**: JSON envelope with capabilities

### Endpoints

#### Main UMICP Handler

**Endpoint:** `POST /umicp`

Handles UMICP envelope requests and routes to MCP tools.

**Request Format:**

```json
{
  "envelope_id": "unique-id",
  "operation": "DATA",
  "capabilities": {
    "operation": "search",
    "collection": "documents",
    "query": "machine learning",
    "limit": 5
  },
  "metadata": {
    "timestamp": "2024-11-16T10:00:00Z"
  }
}
```

**Response Format:**

```json
{
  "envelope_id": "unique-id",
  "operation": "DATA",
  "status": "success",
  "result": {
    "results": [
      {
        "id": "vec_001",
        "score": 0.95,
        "text": "..."
      }
    ]
  },
  "metadata": {
    "timestamp": "2024-11-16T10:00:01Z"
  }
}
```

#### Health Check

**Endpoint:** `GET /umicp/health`

Returns UMICP protocol information and health status.

**Response:**

```json
{
  "protocol": "UMICP",
  "version": "0.2.1",
  "status": "healthy",
  "server": "vectorizer-server",
  "server_version": "1.3.0"
}
```

#### Tool Discovery

**Endpoint:** `GET /umicp/discover`

Discovers all available operations with schemas.

**Response:**

```json
{
  "protocol": "UMICP",
  "version": "0.2.1",
  "server_info": {
    "server": "vectorizer-server",
    "version": "1.3.0",
    "protocol": "UMICP/2.0",
    "features": [
      "semantic-search",
      "vector-storage",
      "intelligent-discovery",
      "file-operations",
      "batch-operations",
      "workspace-management",
      "mcp-compatible"
    ],
    "operations_count": 38,
    "mcp_compatible": true
  },
  "operations": [
    {
      "name": "search",
      "description": "Semantic search in collection",
      "parameters": {
        "collection": {"type": "string", "required": true},
        "query": {"type": "string", "required": true},
        "limit": {"type": "integer", "required": false, "default": 10}
      },
      "read_only": true,
      "idempotent": false,
      "destructive": false
    }
    // ... more operations
  ],
  "total_operations": 38
}
```

### Example: UMICP Request

**Search:**

```bash
curl -X POST http://localhost:15002/umicp \
  -H "Content-Type: application/json" \
  -d '{
    "envelope_id": "req-001",
    "operation": "DATA",
    "capabilities": {
      "operation": "search",
      "collection": "documents",
      "query": "vector database",
      "limit": 10
    }
  }'
```

**Discover Operations:**

```bash
curl http://localhost:15002/umicp/discover
```

### Supported Operations via UMICP

All 38+ MCP tools are available through UMICP:

- **Collection Management** (4): `list_collections`, `create_collection`, `get_collection_info`, `delete_collection`
- **Vector Operations** (6): `search`, `insert_text`, `embed_text`, `get_vector`, `update_vector`, `delete_vectors`
- **Batch Operations** (5): `batch_insert_texts`, `insert_texts`, `batch_search_vectors`, `batch_update_vectors`, `batch_delete_vectors`
- **Intelligent Search** (4): `search_intelligent`, `multi_collection_search`, `search_semantic`, `search_extra`
- **Discovery Pipeline** (9): `discover`, `filter_collections`, `score_collections`, `expand_queries`, `broad_discovery`, `semantic_focus`, `compress_evidence`, `build_answer_plan`, `render_llm_prompt`, `promote_readme`
- **File Operations** (7): `get_file_content`, `list_files`, `get_file_chunks`, `get_project_outline`, `get_related_files`, `search_by_file_type`

## Qdrant-Compatible REST API

Vectorizer provides full Qdrant REST API compatibility under the `/qdrant` prefix for easy migration.

### Base URL

```
http://localhost:15002/qdrant
```

### Collection Management

#### List Collections

**Endpoint:** `GET /qdrant/collections`

Lists all collections in Qdrant format.

**Response:**

```json
{
  "result": {
    "collections": [
      {
        "name": "my_collection",
        "status": "green"
      }
    ]
  },
  "status": "ok",
  "time": 0.001
}
```

#### Get Collection Info

**Endpoint:** `GET /qdrant/collections/{name}`

Gets detailed collection information.

**Response:**

```json
{
  "result": {
    "status": "green",
    "optimizer_status": "ok",
    "vectors_count": 1000,
    "indexed_vectors_count": 1000,
    "points_count": 1000,
    "segments_count": 1,
    "config": {
      "params": {
        "vectors": {
          "size": 384,
          "distance": "Cosine"
        }
      },
      "hnsw_config": {
        "m": 16,
        "ef_construct": 200,
        "full_scan_threshold": 10000
      },
      "optimizer_config": {
        "deleted_threshold": 0.2,
        "vacuum_min_vector_number": 1000,
        "default_segment_number": 0,
        "max_segment_size": null,
        "memmap_threshold": null,
        "indexing_threshold": 20000,
        "flush_interval_sec": 5,
        "max_optimization_threads": 0
      },
      "quantization_config": {
        "scalar": {
          "type": "int8",
          "quantile": 0.99,
          "always_ram": false
        }
      }
    }
  },
  "status": "ok",
  "time": 0.001
}
```

#### Create Collection

**Endpoint:** `PUT /qdrant/collections/{name}`

Creates a new collection.

**Request Body:**

```json
{
  "vectors": {
    "size": 384,
    "distance": "Cosine"
  },
  "optimizers_config": {
    "default_segment_number": 2
  },
  "hnsw_config": {
    "m": 16,
    "ef_construct": 200
  },
  "quantization_config": {
    "scalar": {
      "type": "int8",
      "quantile": 0.99
    }
  }
}
```

**Response:**

```json
{
  "result": true,
  "status": "ok",
  "time": 0.001
}
```

#### Update Collection

**Endpoint:** `PATCH /qdrant/collections/{name}`

Updates collection configuration.

**Request Body:**

```json
{
  "optimizers_config": {
    "indexing_threshold": 10000
  }
}
```

#### Delete Collection

**Endpoint:** `DELETE /qdrant/collections/{name}`

Deletes a collection.

**Response:**

```json
{
  "result": true,
  "status": "ok",
  "time": 0.001
}
```

### Point Operations

#### Upsert Points

**Endpoint:** `PUT /qdrant/collections/{name}/points`

Upserts (insert or update) points in a collection.

**Request Body:**

```json
{
  "points": [
    {
      "id": "point-1",
      "vector": [0.1, 0.2, 0.3, ...],
      "payload": {
        "text": "Document content",
        "category": "docs"
      }
    },
    {
      "id": "point-2",
      "vector": [0.4, 0.5, 0.6, ...],
      "payload": {
        "text": "Another document",
        "category": "tutorial"
      }
    }
  ]
}
```

**Response:**

```json
{
  "result": {
    "operation_id": 0,
    "status": "completed"
  },
  "status": "ok",
  "time": 0.001
}
```

#### Retrieve Points

**Endpoint:** `GET /qdrant/collections/{name}/points`

Retrieves points by IDs.

**Query Parameters:**

- `ids` (required): Comma-separated point IDs or JSON array
- `with_payload` (optional): Include payload (default: true)
- `with_vector` (optional): Include vector (default: false)

**Example:**

```bash
curl "http://localhost:15002/qdrant/collections/my_collection/points?ids=point-1,point-2&with_payload=true"
```

**Response:**

```json
{
  "result": {
    "points": [
      {
        "id": "point-1",
        "payload": {
          "text": "Document content",
          "category": "docs"
        },
        "vector": [0.1, 0.2, 0.3, ...]
      }
    ]
  },
  "status": "ok",
  "time": 0.001
}
```

#### Delete Points

**Endpoint:** `POST /qdrant/collections/{name}/points/delete`

Deletes points by IDs or filter.

**Request Body (by IDs):**

```json
{
  "points": ["point-1", "point-2"]
}
```

**Request Body (by filter):**

```json
{
  "filter": {
    "must": [
      {
        "key": "category",
        "match": {
          "value": "docs"
        }
      }
    ]
  }
}
```

**Response:**

```json
{
  "result": {
    "operation_id": 0,
    "status": "completed"
  },
  "status": "ok",
  "time": 0.001
}
```

#### Count Points

**Endpoint:** `POST /qdrant/collections/{name}/points/count`

Counts points matching a filter.

**Request Body:**

```json
{
  "filter": {
    "must": [
      {
        "key": "category",
        "match": {
          "value": "docs"
        }
      }
    ]
  },
  "exact": true
}
```

**Response:**

```json
{
  "result": {
    "count": 150
  },
  "status": "ok",
  "time": 0.001
}
```

#### Scroll Points

**Endpoint:** `POST /qdrant/collections/{name}/points/scroll`

Scrolls through points with pagination.

**Request Body:**

```json
{
  "filter": {
    "must": [
      {
        "key": "category",
        "match": {
          "value": "docs"
        }
      }
    ]
  },
  "limit": 10,
  "offset": null,
  "with_payload": true,
  "with_vector": false
}
```

**Response:**

```json
{
  "result": {
    "points": [
      {
        "id": "point-1",
        "payload": {
          "category": "docs"
        }
      }
    ],
    "next_page_offset": "offset-token"
  },
  "status": "ok",
  "time": 0.001
}
```

### Search Operations

#### Search Points

**Endpoint:** `POST /qdrant/collections/{name}/points/search`

Searches for similar points.

**Request Body:**

```json
{
  "vector": [0.1, 0.2, 0.3, ...],
  "limit": 10,
  "offset": 0,
  "with_payload": true,
  "with_vector": false,
  "filter": {
    "must": [
      {
        "key": "category",
        "match": {
          "value": "docs"
        }
      }
    ]
  },
  "score_threshold": 0.5
}
```

**Response:**

```json
{
  "result": [
    {
      "id": "point-1",
      "score": 0.95,
      "payload": {
        "text": "Document content",
        "category": "docs"
      }
    }
  ],
  "status": "ok",
  "time": 0.001
}
```

#### Batch Search

**Endpoint:** `POST /qdrant/collections/{name}/points/search/batch`

Performs multiple searches in one request.

**Request Body:**

```json
{
  "searches": [
    {
      "vector": [0.1, 0.2, 0.3, ...],
      "limit": 5
    },
    {
      "vector": [0.4, 0.5, 0.6, ...],
      "limit": 5
    }
  ]
}
```

**Response:**

```json
{
  "result": [
    [
      {"id": "point-1", "score": 0.95, ...},
      {"id": "point-2", "score": 0.92, ...}
    ],
    [
      {"id": "point-3", "score": 0.88, ...}
    ]
  ],
  "status": "ok",
  "time": 0.002
}
```

#### Recommend Points

**Endpoint:** `POST /qdrant/collections/{name}/points/recommend`

Recommends points based on positive/negative examples.

**Request Body:**

```json
{
  "positive": ["point-1", "point-2"],
  "negative": ["point-3"],
  "limit": 10,
  "with_payload": true
}
```

#### Batch Recommend

**Endpoint:** `POST /qdrant/collections/{name}/points/recommend/batch`

Batch recommend operation.

### Collection Aliases

#### Update Aliases

**Endpoint:** `POST /qdrant/collections/aliases`

Creates or updates collection aliases.

**Request Body:**

```json
{
  "actions": [
    {
      "create_alias": {
        "collection_name": "my_collection",
        "alias_name": "my_alias"
      }
    },
    {
      "delete_alias": {
        "alias_name": "old_alias"
      }
    },
    {
      "rename_alias": {
        "old_alias_name": "old_name",
        "new_alias_name": "new_name"
      }
    }
  ]
}
```

#### List Collection Aliases

**Endpoint:** `GET /qdrant/collections/{name}/aliases`

Lists aliases for a collection.

**Response:**

```json
{
  "result": {
    "aliases": [
      {
        "alias_name": "my_alias",
        "collection_name": "my_collection"
      }
    ]
  },
  "status": "ok",
  "time": 0.001
}
```

#### List All Aliases

**Endpoint:** `GET /qdrant/aliases`

Lists all aliases across all collections.

**Response:**

```json
{
  "result": {
    "aliases": [
      {
        "alias_name": "my_alias",
        "collection_name": "my_collection"
      }
    ]
  },
  "status": "ok",
  "time": 0.001
}
```

### Qdrant Compatibility Notes

- ✅ **Full REST API compatibility** with Qdrant v1.x
- ✅ **Collection management** (create, get, update, delete, list)
- ✅ **Point operations** (upsert, retrieve, delete, count, scroll)
- ✅ **Search operations** (search, batch search, recommend, batch recommend)
- ✅ **Collection aliases** (create, delete, rename, list)
- ✅ **Payload filtering** (must, should, must_not, filter)
- ⚠️ **gRPC not supported** (REST API only)
- ⚠️ **Some advanced features** may have limitations

### Migration from Qdrant

To migrate from Qdrant to Vectorizer:

1. **Change base URL**: `http://qdrant:6333` → `http://vectorizer:15002/qdrant`
2. **Keep same request/response format**: All Qdrant API calls work as-is
3. **Test thoroughly**: Verify all operations work correctly
4. **Consider native APIs**: For better performance, migrate to Vectorizer native APIs

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
