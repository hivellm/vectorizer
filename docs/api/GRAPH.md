# Graph API Documentation

Complete API reference for Vectorizer Graph operations.

**Version:** 1.4.0  
**Status:** ✅ Production Ready  
**Last Updated:** 2025-01-24

## Overview

Vectorizer's Graph API enables relationship discovery and querying between vectors in collections. When graph is enabled for a collection, vectors automatically become nodes, and relationships (edges) can be discovered based on similarity or explicitly created.

## Base URL

All graph endpoints are prefixed with `/graph`:

```
http://localhost:15002/graph
```

## Endpoints

### List Nodes

List all nodes (vectors) in a collection's graph.

**Endpoint:** `GET /graph/nodes/{collection}`

**Path Parameters:**
- `collection` (string, required) - Collection name

**Response:**
```json
{
  "nodes": [
    {
      "id": "vec1",
      "node_type": "document",
      "metadata": {
        "source": "doc1.md"
      }
    }
  ],
  "count": 1
}
```

**Example:**
```bash
curl "http://localhost:15002/graph/nodes/my-collection"
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Collection not found or graph not enabled
- `500 Internal Server Error` - Server error

---

### Get Neighbors

Get all neighbors (connected nodes) of a specific node.

**Endpoint:** `GET /graph/nodes/{collection}/{node_id}/neighbors`

**Path Parameters:**
- `collection` (string, required) - Collection name
- `node_id` (string, required) - Node ID (vector ID)

**Query Parameters:**
- `relationship_type` (string, optional) - Filter by relationship type (SIMILAR_TO, REFERENCES, CONTAINS, DERIVED_FROM)

**Response:**
```json
{
  "neighbors": [
    {
      "edge_id": "edge1",
      "target": {
        "id": "vec2",
        "node_type": "document",
        "metadata": {}
      },
      "relationship_type": "SIMILAR_TO",
      "weight": 0.85,
      "metadata": {}
    }
  ],
  "count": 1
}
```

**Example:**
```bash
# Get all neighbors
curl "http://localhost:15002/graph/nodes/my-collection/vec1/neighbors"

# Get only SIMILAR_TO neighbors
curl "http://localhost:15002/graph/nodes/my-collection/vec1/neighbors?relationship_type=SIMILAR_TO"
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Collection or node not found
- `500 Internal Server Error` - Server error

---

### Find Related Nodes

Find all nodes related to a specific node within a specified number of hops.

**Endpoint:** `POST /graph/nodes/{collection}/{node_id}/related`

**Path Parameters:**
- `collection` (string, required) - Collection name
- `node_id` (string, required) - Node ID (vector ID)

**Request Body:**
```json
{
  "max_hops": 2,
  "relationship_type": "SIMILAR_TO"
}
```

**Request Parameters:**
- `max_hops` (integer, required) - Maximum number of hops to traverse (1-10)
- `relationship_type` (string, optional) - Filter by relationship type

**Response:**
```json
{
  "related": [
    {
      "node": {
        "id": "vec2",
        "node_type": "document",
        "metadata": {}
      },
      "hops": 1,
      "path": ["vec1", "vec2"]
    },
    {
      "node": {
        "id": "vec3",
        "node_type": "document",
        "metadata": {}
      },
      "hops": 2,
      "path": ["vec1", "vec2", "vec3"]
    }
  ],
  "count": 2
}
```

**Example:**
```bash
curl -X POST "http://localhost:15002/graph/nodes/my-collection/vec1/related" \
  -H "Content-Type: application/json" \
  -d '{
    "max_hops": 2,
    "relationship_type": "SIMILAR_TO"
  }'
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Invalid request or node not found
- `500 Internal Server Error` - Server error

---

### Find Path

Find the shortest path between two nodes.

**Endpoint:** `POST /graph/path`

**Request Body:**
```json
{
  "collection": "my-collection",
  "source": "vec1",
  "target": "vec3"
}
```

**Request Parameters:**
- `collection` (string, required) - Collection name
- `source` (string, required) - Source node ID
- `target` (string, required) - Target node ID

**Response:**
```json
{
  "found": true,
  "path": [
    {
      "id": "vec1",
      "node_type": "document",
      "metadata": {}
    },
    {
      "id": "vec2",
      "node_type": "document",
      "metadata": {}
    },
    {
      "id": "vec3",
      "node_type": "document",
      "metadata": {}
    }
  ],
  "length": 2
}
```

**Example:**
```bash
curl -X POST "http://localhost:15002/graph/path" \
  -H "Content-Type: application/json" \
  -d '{
    "collection": "my-collection",
    "source": "vec1",
    "target": "vec3"
  }'
```

**Status Codes:**
- `200 OK` - Success (path found or not found)
- `400 Bad Request` - Invalid request or nodes not found
- `500 Internal Server Error` - Server error

---

### Create Edge

Create an explicit relationship (edge) between two nodes.

**Endpoint:** `POST /graph/edges`

**Request Body:**
```json
{
  "collection": "my-collection",
  "source": "vec1",
  "target": "vec2",
  "relationship_type": "SIMILAR_TO",
  "weight": 0.85,
  "metadata": {
    "discovered_by": "manual"
  }
}
```

**Request Parameters:**
- `collection` (string, required) - Collection name
- `source` (string, required) - Source node ID
- `target` (string, required) - Target node ID
- `relationship_type` (string, required) - Relationship type (SIMILAR_TO, REFERENCES, CONTAINS, DERIVED_FROM)
- `weight` (float, optional) - Edge weight (0.0-1.0, default: 1.0)
- `metadata` (object, optional) - Additional edge metadata

**Response:**
```json
{
  "success": true,
  "edge_id": "edge_abc123",
  "edge": {
    "id": "edge_abc123",
    "source": "vec1",
    "target": "vec2",
    "relationship_type": "SIMILAR_TO",
    "weight": 0.85,
    "metadata": {
      "discovered_by": "manual"
    }
  }
}
```

**Example:**
```bash
curl -X POST "http://localhost:15002/graph/edges" \
  -H "Content-Type: application/json" \
  -d '{
    "collection": "my-collection",
    "source": "vec1",
    "target": "vec2",
    "relationship_type": "SIMILAR_TO",
    "weight": 0.85
  }'
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Invalid request or nodes not found
- `409 Conflict` - Edge already exists
- `500 Internal Server Error` - Server error

---

### Delete Edge

Delete a specific edge by its ID.

**Endpoint:** `DELETE /graph/edges/{edge_id}`

**Path Parameters:**
- `edge_id` (string, required) - Edge ID

**Response:**
```json
{
  "success": true,
  "message": "Edge deleted successfully"
}
```

**Example:**
```bash
curl -X DELETE "http://localhost:15002/graph/edges/edge_abc123"
```

**Status Codes:**
- `200 OK` - Success
- `404 Not Found` - Edge not found
- `500 Internal Server Error` - Server error

---

### List Edges

List all edges in a collection's graph.

**Endpoint:** `GET /graph/collections/{collection}/edges`

**Path Parameters:**
- `collection` (string, required) - Collection name

**Query Parameters:**
- `relationship_type` (string, optional) - Filter by relationship type
- `limit` (integer, optional) - Maximum number of edges to return (default: 100)

**Response:**
```json
{
  "edges": [
    {
      "id": "edge1",
      "source": "vec1",
      "target": "vec2",
      "relationship_type": "SIMILAR_TO",
      "weight": 0.85,
      "metadata": {}
    }
  ],
  "count": 1
}
```

**Example:**
```bash
# List all edges
curl "http://localhost:15002/graph/collections/my-collection/edges"

# List only SIMILAR_TO edges
curl "http://localhost:15002/graph/collections/my-collection/edges?relationship_type=SIMILAR_TO"
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Collection not found
- `500 Internal Server Error` - Server error

---

### Discover Edges (Collection)

Discover SIMILAR_TO relationships for all nodes in a collection.

**Endpoint:** `POST /graph/discover/{collection}`

**Path Parameters:**
- `collection` (string, required) - Collection name

**Request Body:**
```json
{
  "similarity_threshold": 0.7,
  "max_per_node": 10,
  "enabled_types": ["SIMILAR_TO"]
}
```

**Request Parameters:**
- `similarity_threshold` (float, optional) - Minimum similarity score (0.0-1.0, default: 0.7)
- `max_per_node` (integer, optional) - Maximum edges per node (default: 10)
- `enabled_types` (array, optional) - Relationship types to discover (default: ["SIMILAR_TO"])

**Response:**
```json
{
  "success": true,
  "stats": {
    "total_nodes": 100,
    "nodes_processed": 100,
    "total_edges_created": 250,
    "nodes_with_edges": 85
  }
}
```

**Example:**
```bash
curl -X POST "http://localhost:15002/graph/discover/my-collection" \
  -H "Content-Type: application/json" \
  -d '{
    "similarity_threshold": 0.7,
    "max_per_node": 10
  }'
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Collection not found or graph not enabled
- `500 Internal Server Error` - Server error

---

### Discover Edges (Node)

Discover SIMILAR_TO relationships for a specific node.

**Endpoint:** `POST /graph/discover/{collection}/{node_id}`

**Path Parameters:**
- `collection` (string, required) - Collection name
- `node_id` (string, required) - Node ID

**Request Body:**
```json
{
  "similarity_threshold": 0.7,
  "max_per_node": 10,
  "enabled_types": ["SIMILAR_TO"]
}
```

**Response:**
```json
{
  "success": true,
  "edges_created": 5
}
```

**Example:**
```bash
curl -X POST "http://localhost:15002/graph/discover/my-collection/vec1" \
  -H "Content-Type: application/json" \
  -d '{
    "similarity_threshold": 0.7,
    "max_per_node": 10
  }'
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Node not found
- `500 Internal Server Error` - Server error

---

### Get Discovery Status

Get discovery status and statistics for a collection.

**Endpoint:** `GET /graph/discover/{collection}/status`

**Path Parameters:**
- `collection` (string, required) - Collection name

**Response:**
```json
{
  "status": "completed",
  "stats": {
    "total_nodes": 100,
    "nodes_processed": 100,
    "total_edges_created": 250,
    "nodes_with_edges": 85
  }
}
```

**Example:**
```bash
curl "http://localhost:15002/graph/discover/my-collection/status"
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Collection not found
- `500 Internal Server Error` - Server error

---

## Relationship Types

- **SIMILAR_TO** - Nodes are similar (based on vector similarity)
- **REFERENCES** - Source node references target node
- **CONTAINS** - Source node contains target node
- **DERIVED_FROM** - Source node is derived from target node

## Error Responses

All endpoints return errors in the following format:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": {}
}
```

## Usage Examples

### Complete Workflow

```bash
# 1. Create collection with graph enabled
curl -X POST "http://localhost:15002/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "documents",
    "dimension": 384,
    "metric": "cosine",
    "graph": {
      "enabled": true
    }
  }'

# 2. Insert vectors
curl -X POST "http://localhost:15002/collections/documents/vectors" \
  -H "Content-Type: application/json" \
  -d '{
    "texts": [
      {"id": "doc1", "text": "Machine learning algorithms"},
      {"id": "doc2", "text": "Deep learning neural networks"},
      {"id": "doc3", "text": "Natural language processing"}
    ]
  }'

# 3. Discover relationships
curl -X POST "http://localhost:15002/graph/discover/documents" \
  -H "Content-Type: application/json" \
  -d '{
    "similarity_threshold": 0.7,
    "max_per_node": 5
  }'

# 4. Find related documents
curl -X POST "http://localhost:15002/graph/nodes/documents/doc1/related" \
  -H "Content-Type: application/json" \
  -d '{
    "max_hops": 2
  }'

# 5. Find path between documents
curl -X POST "http://localhost:15002/graph/path" \
  -H "Content-Type: application/json" \
  -d '{
    "collection": "documents",
    "source": "doc1",
    "target": "doc3"
  }'
```

## Performance Considerations

- Graph operations are optimized for collections with up to 100,000 nodes
- Path finding uses BFS (Breadth-First Search) algorithm
- Discovery operations can be time-consuming for large collections
- Consider running discovery operations asynchronously for large datasets

## See Also

- [Graph Usage Examples](../users/guides/GRAPH.md)
- [Collection Configuration](../users/collections/CONFIGURATION.md)
- [API Reference](./README.md)

