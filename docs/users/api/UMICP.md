---
title: UMICP Protocol
module: api
id: umicp-protocol
order: 10
description: Universal Multi-Agent Communication Protocol integration
tags: [api, umicp, protocol, multi-agent]
---

# UMICP Protocol

Vectorizer supports the Universal Multi-Agent Communication Protocol (UMICP) v0.2.1 for high-performance envelope-based communication.

## Overview

UMICP provides:

- Envelope-based communication
- High-performance streaming over HTTP
- All 38+ MCP tools accessible via UMICP
- Tool discovery endpoint
- Native JSON types support

## Base URL

```
http://localhost:15002/umicp
```

## Protocol Details

- **Version**: UMICP v0.2.1
- **Transport**: HTTP POST with envelope format
- **Format**: JSON envelope with capabilities
- **Compatibility**: Full MCP tool compatibility

## Endpoints

### Main UMICP Handler

Handle UMICP envelope requests and route to MCP tools.

**Endpoint:** `POST /umicp`

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

**Example:**

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

### Health Check

Get UMICP protocol information and health status.

**Endpoint:** `GET /umicp/health`

**Response:**

```json
{
  "protocol": "UMICP",
  "version": "1.0",
  "transport": "streamable-http",
  "status": "ok",
  "vectorizer_version": "1.3.0"
}
```

**Example:**

```bash
curl http://localhost:15002/umicp/health
```

### Tool Discovery

Discover all available operations with schemas.

**Endpoint:** `GET /umicp/discover`

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
        "collection": { "type": "string", "required": true },
        "query": { "type": "string", "required": true },
        "limit": { "type": "integer", "required": false, "default": 10 }
      },
      "read_only": true,
      "idempotent": false,
      "destructive": false
    }
  ],
  "total_operations": 38
}
```

**Example:**

```bash
curl http://localhost:15002/umicp/discover
```

## Supported Operations

All 38+ MCP tools are available through UMICP:

### Collection Management (4)

- `list_collections` - List all collections
- `create_collection` - Create a new collection
- `get_collection_info` - Get collection details
- `delete_collection` - Delete a collection

### Vector Operations (6)

- `search` - Semantic search
- `insert_text` - Insert text (auto-embed)
- `embed_text` - Generate embedding
- `get_vector` - Retrieve vector by ID
- `update_vector` - Update existing vector
- `delete_vectors` - Delete vectors

### Batch Operations (5)

- `batch_insert_texts` - Batch insert texts
- `insert_texts` - Insert multiple texts
- `batch_search_vectors` - Batch search
- `batch_update_vectors` - Batch update
- `batch_delete_vectors` - Batch delete

### Intelligent Search (4)

- `search_intelligent` - AI-powered search
- `multi_collection_search` - Cross-collection search
- `search_semantic` - Semantic search with reranking
- `search_extra` - Combined search strategies

### Discovery Pipeline (9)

- `discover` - Complete discovery pipeline
- `filter_collections` - Filter collections
- `score_collections` - Score collections
- `expand_queries` - Expand queries
- `broad_discovery` - Broad search
- `semantic_focus` - Semantic focus
- `compress_evidence` - Compress evidence
- `build_answer_plan` - Build answer plan
- `render_llm_prompt` - Render LLM prompt
- `promote_readme` - Promote README files

### File Operations (7)

- `get_file_content` - Get file content
- `list_files_in_collection` - List files
- `get_file_summary` - Get file summary
- `get_file_chunks_ordered` - Get ordered chunks
- `get_project_outline` - Get project outline
- `get_related_files` - Get related files
- `search_by_file_type` - Search by file type

## Operation Types

### DATA Operations

Execute data operations (search, insert, etc.):

```json
{
  "envelope_id": "req-001",
  "operation": "DATA",
  "capabilities": {
    "operation": "search",
    "collection": "documents",
    "query": "vector database"
  }
}
```

### CONTROL Operations

Control operations (discovery, configuration):

```json
{
  "envelope_id": "req-002",
  "operation": "CONTROL",
  "capabilities": {
    "operation": "discover",
    "query": "machine learning"
  }
}
```

### ACK Operations

Acknowledgment operations:

```json
{
  "envelope_id": "req-001",
  "operation": "ACK",
  "status": "received"
}
```

## Use Cases

### Multi-Agent Communication

Use UMICP for multi-agent communication:

```python
import requests

# Create UMICP envelope
envelope = {
    "envelope_id": "agent-001",
    "operation": "DATA",
    "capabilities": {
        "operation": "search",
        "collection": "documents",
        "query": "vector database",
        "limit": 10
    }
}

# Send request
response = requests.post(
    "http://localhost:15002/umicp",
    json=envelope
)

result = response.json()
print(f"Status: {result['status']}")
print(f"Results: {result['result']}")
```

### Tool Discovery

Discover available operations:

```python
import requests

# Discover operations
response = requests.get("http://localhost:15002/umicp/discover")
discovery = response.json()

print(f"Available operations: {discovery['total_operations']}")

for op in discovery["operations"]:
    print(f"- {op['name']}: {op['description']}")
```

### Envelope-Based Communication

Use envelopes for structured communication:

```python
def create_envelope(operation, capabilities):
    return {
        "envelope_id": f"req-{uuid.uuid4()}",
        "operation": operation,
        "capabilities": capabilities,
        "metadata": {
            "timestamp": datetime.utcnow().isoformat()
        }
    }

# Search operation
search_envelope = create_envelope("DATA", {
    "operation": "search",
    "collection": "documents",
    "query": "vector database"
})

response = requests.post(
    "http://localhost:15002/umicp",
    json=search_envelope
)
```

## Best Practices

1. **Use unique envelope IDs**: Generate unique IDs for each request
2. **Include timestamps**: Add timestamps to metadata for tracking
3. **Handle errors**: Check status field in responses
4. **Use discovery**: Discover available operations before use
5. **Batch operations**: Use batch operations for efficiency
6. **Monitor health**: Check health endpoint regularly

## Related Topics

- [MCP Protocol](./API_REFERENCE.md#mcp-model-context-protocol) - MCP protocol documentation
- [REST API Reference](./API_REFERENCE.md) - Complete REST API
- [Discovery API](./DISCOVERY.md) - Discovery operations
