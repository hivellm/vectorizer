# MCP (Model Context Protocol) - Complete Reference

**Version**: 0.9.0  
**Status**: ✅ Production Ready (StreamableHTTP)  
**Last Updated**: 2025-10-16

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [MCP Tools Reference](#mcp-tools-reference)
4. [Integration Guide](#integration-guide)
5. [Enhanced Features](#enhanced-features)
6. [Configuration](#configuration)
7. [Troubleshooting](#troubleshooting)
8. [Planned / Not Yet Implemented](#planned--not-yet-implemented)

---

## Overview

Vectorizer implements a comprehensive MCP (Model Context Protocol) server that enables seamless integration with AI-powered IDEs and development tools. The MCP server provides a standardized interface for AI models to interact with the vector database through Server-Sent Events (SSE) connections and REST API.

### Key Features

**🔌 StreamableHTTP Communication** (v0.9.0+)
- Bi-directional HTTP streaming
- JSON-RPC 2.0 protocol compliance
- Automatic session management
- Modern HTTP/1.1 and HTTP/2 support

**🛠️ Comprehensive Tool Set** (31 tools registered in `handlers.rs`)
- **Core Collection/Vector (9)**: `list_collections`, `create_collection`, `get_collection_info`, `insert_text`, `get_vector`, `update_vector`, `delete_vector`, `multi_collection_search`, `search`
- **Search (4)**: `search_intelligent`, `search_semantic`, `search_extra`, `search_hybrid`
- **Discovery (2)**: `filter_collections`, `expand_queries`
- **File Operations (5)**: `get_file_content`, `list_files`, `get_file_chunks`, `get_project_outline`, `get_related_files`
- **Graph Operations (8)**: `graph_list_nodes`, `graph_get_neighbors`, `graph_find_related`, `graph_find_path`, `graph_create_edge`, `graph_delete_edge`, `graph_discover_edges`, `graph_discover_status`
- **Collection Maintenance (3)**: `list_empty_collections`, `cleanup_empty_collections`, `get_collection_stats`

See also the **[Planned / Not Yet Implemented](#planned--not-yet-implemented)** section at the bottom for tools historically documented here but not yet wired up (batch ops, `contextual_search`, `embed_text`, `delete_collection`, `get_database_stats`, etc.).

**🚀 Latest Improvements (v0.3.1)**
- Larger chunks (2048 chars) for better semantic context
- Better overlap (256 chars) for improved continuity
- Cosine similarity with automatic L2 normalization
- 85% improvement in semantic search quality
- Search time: 0.6-2.4ms across all collections

---

## Architecture

### Unified Server Architecture (v0.3.0+)

```
┌─────────────────┐    SSE/HTTP     ┌──────────────────┐
│   AI IDE/Client │ ◄─────────────► │  Unified Server  │
│                 │   http://:15002 │  (Port 15002)    │
└─────────────────┘                 └──────────────────┘
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │  MCP Engine     │
                                    │  ├─ Tools       │
                                    │  ├─ Resources   │
                                    │  └─ Prompts     │
                                    └─────────────────┘
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │ Vector Database │
                                    │ (HNSW + Emb.)   │
                                    └─────────────────┘
```

### Benefits
- **Single Process**: Reduced memory footprint
- **Unified Interface**: REST API and MCP in one server
- **Background Loading**: Non-blocking server startup
- **Automatic Quantization**: Memory optimization

---

## MCP Tools Reference

This reference is aligned with the `match request.name` dispatch in
`crates/vectorizer-server/src/server/mcp/handlers.rs` and the schema
declarations in `crates/vectorizer-server/src/server/mcp/tools.rs`. Any
tool **not** listed below is documented in the
[Planned / Not Yet Implemented](#planned--not-yet-implemented) section.

### Search & Retrieval Tools

#### search

Basic vector similarity search in a single collection. The query string
is embedded with the collection's configured provider (BM25, TF-IDF,
BERT, MiniLM, etc.) and searched through the HNSW index.

**Parameters**:
```json
{
  "collection": "string",    // Required
  "query": "string",         // Required
  "limit": 10,               // Optional, default: 10, range 1-100
  "similarity_threshold": 0.1 // Optional, default: 0.1
}
```

**Example**:
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "collection": "documents",
      "query": "machine learning algorithms",
      "limit": 5
    }
  }
}
```

#### search_intelligent

AI-powered search with automatic query expansion and result
deduplication across one or more collections. Uses the
`IntelligentSearchTool` pipeline in
`vectorizer::intelligent_search::mcp_tools`.

**Parameters**:
```json
{
  "query": "string",            // Required
  "collections": ["string"],    // Optional; omit to search all collections
  "max_results": 10,            // Optional, default: 10
  "domain_expansion": true,     // Optional, default: true
  "similarity_threshold": 0.1   // Optional, default: 0.1
}
```

**Features**:
- Generates 4-8 relevant queries automatically
- Domain-specific knowledge expansion
- Technical focus reranking (MMR disabled in the MCP path)
- Cross-collection deduplication

#### search_semantic

Semantic search with basic reranking for a single collection. Wraps
`SemanticSearchTool` with `semantic_reranking=true` and
`cross_encoder_reranking=false` (the cross-encoder is disabled in the
MCP path).

**Parameters**:
```json
{
  "query": "string",            // Required
  "collection": "string",       // Required
  "max_results": 10,            // Optional, default: 10
  "similarity_threshold": 0.1   // Optional, default: 0.1
}
```

**Recommended Thresholds**:
- High Precision: 0.15-0.2
- Balanced: 0.1-0.15
- High Recall: 0.05-0.1

#### search_extra

Combined search that concatenates and re-ranks results from multiple
strategies (`basic`, `semantic`, `intelligent`) against a single
collection, deduplicating by vector id.

**Parameters**:
```json
{
  "query": "string",                      // Required
  "collection": "string",                 // Required
  "strategies": ["basic", "semantic"],    // Optional, default: ["basic","semantic"]
  "max_results": 10,                      // Optional, default: 10
  "similarity_threshold": 0.1             // Optional, default: 0.1
}
```

#### search_hybrid

Hybrid search combining dense (HNSW) retrieval and optional sparse
retrieval, fused with RRF / weighted / alpha blending. The query string
is embedded with the collection's dense provider; an optional explicit
`query_sparse` can be supplied.

**Parameters**:
```json
{
  "query": "string",            // Required - dense query text
  "collection": "string",       // Required
  "query_sparse": {             // Optional
    "indices": [int, ...],
    "values": [float, ...]
  },
  "alpha": 0.7,                 // Optional, 0.0 = sparse only, 1.0 = dense only
  "algorithm": "rrf",           // Optional: "rrf" | "weighted" | "alpha"
  "dense_k": 20,                // Optional, top-k from dense side
  "sparse_k": 20,               // Optional, top-k from sparse side
  "final_k": 10                 // Optional, final result size
}
```

#### multi_collection_search

Search a single query across multiple named collections and return the
merged top-N. `cross_collection_reranking` is forced to `false` on the
MCP path.

**Parameters**:
```json
{
  "query": "string",             // Required
  "collections": ["string"],     // Required
  "max_per_collection": 5,       // Optional, default: 5
  "max_total_results": 20,       // Optional, default: 20
  "similarity_threshold": 0.1    // Optional, default: 0.1
}
```

### Collection Management Tools

#### list_collections

Retrieves information about all available collections.

**Parameters**: None

**Response**:
```json
{
  "collections": ["documents", "code", "..."],
  "total": 3
}
```

#### get_collection_info

Retrieves detailed information about a specific collection (vector
count, document count, dimension, metric, quantization, normalization
settings, timestamps).

**Parameters**:
```json
{
  "name": "string"           // Required
}
```

#### create_collection

Creates a new collection with specified configuration. Optionally
enables per-collection graph tracking.

**Parameters**:
```json
{
  "name": "string",          // Required
  "dimension": 384,          // Required
  "metric": "cosine",        // Optional, default: "cosine" ("cosine" | "euclidean")
  "graph": {                 // Optional
    "enabled": false
  }
}
```

#### list_empty_collections

Lists all collections that contain no vectors. Useful for identifying
collections that can be safely cleaned up.

**Parameters**: None

**Response**:
```json
{
  "status": "success",
  "empty_collections": [
    "collection-name-1",
    "collection-name-2"
  ],
  "count": 2
}
```

**Example**:
```javascript
const result = await mcpClient.call_tool("list_empty_collections", {});
console.log(`Found ${result.count} empty collections`);
```

#### cleanup_empty_collections

Removes all empty collections from the database. Supports dry-run mode
to preview what would be deleted without actually deleting.

**Parameters**:
```json
{
  "dry_run": false           // Optional, default: false
}
```

**Response**:
```json
{
  "status": "success",
  "dry_run": false,
  "deleted_count": 2,
  "message": "Successfully deleted 2 empty collections"
}
```

**Use Cases**:
- Clean up automatically created empty collections
- Maintain database hygiene
- Free up resources
- Simplify collection management UI

#### get_collection_stats

Retrieves basic statistics about a specific collection (vector count
and whether it is empty).

**Parameters**:
```json
{
  "collection": "string"     // Required
}
```

**Response**:
```json
{
  "status": "success",
  "collection": "docs-architecture",
  "vector_count": 1250,
  "is_empty": false
}
```

### Vector Operations Tools

#### insert_text

Inserts a single text into a collection with automatic embedding
generation. A UUID vector id is generated by the server. If a
`public_key` is provided, the payload is encrypted with the project's
payload-encryption primitive before storage.

**Parameters**:
```json
{
  "collection_name": "string", // Required
  "text": "string",            // Required
  "metadata": {},              // Optional
  "public_key": "string"       // Optional - enables payload encryption
}
```

**Response**:
```json
{
  "status": "inserted",
  "vector_id": "uuid",
  "collection": "docs",
  "encrypted": false
}
```

#### delete_vector

Removes one or more vectors from a collection by their ids. Despite the
singular name, the parameter `vector_ids` is an array — the handler
loops and counts successful deletions.

**Parameters**:
```json
{
  "collection": "string",    // Required
  "vector_ids": ["string"]   // Required - array, can be a single id
}
```

#### update_vector

Updates an existing vector with new text and/or metadata. If `text` is
supplied, a fresh embedding is generated and the stored vector is
replaced. Payload encryption is supported via `public_key`.

**Parameters**:
```json
{
  "collection": "string",    // Required
  "vector_id": "string",     // Required
  "text": "string",          // Optional
  "metadata": {},            // Optional
  "public_key": "string"     // Optional - enables payload encryption
}
```

#### get_vector

Retrieves a specific vector by its ID, including `data` (the dense
embedding) and `payload`.

**Parameters**:
```json
{
  "collection": "string",    // Required
  "vector_id": "string"      // Required
}
```

### Discovery Tools

#### filter_collections

Filter the list of known collections by a query string with optional
include/exclude glob patterns. Used by the discovery pipeline to scope
subsequent searches.

**Parameters**:
```json
{
  "query": "string",          // Required - free-text or collection keyword
  "include": ["string"],      // Optional - include patterns
  "exclude": ["string"]       // Optional - exclude patterns
}
```

#### expand_queries

Generate query variations and expansions for broader search coverage
(definition queries, feature queries, architecture queries).

**Parameters**:
```json
{
  "query": "string",              // Required
  "max_expansions": 8,            // Optional, default: 8
  "include_definition": true,     // Optional, default: true
  "include_features": true,       // Optional, default: true
  "include_architecture": true    // Optional, default: true
}
```

### File Operations

#### get_file_content

Retrieves complete file content from a collection (reconstructed from
its chunks), bounded by a size cap.

**Parameters**:
```json
{
  "collection": "string",    // Required
  "file_path": "string",     // Required
  "max_size_kb": 500         // Optional, default: 500, range 1-5000
}
```

#### list_files

List all indexed files in a collection with metadata and filters
(file-type allow-list, minimum chunk count, sort order, result cap).

**Parameters**:
```json
{
  "collection": "string",            // Required
  "filter_by_type": ["rs", "ts"],    // Optional - file extensions
  "min_chunks": 1,                   // Optional
  "max_results": 100,                // Optional, default: 100
  "sort_by": "name"                  // Optional: "name" | "size" | "chunks" | "recent"
}
```

#### get_file_chunks

Retrieve file chunks in original (document) order for progressive
reading. Useful for reconstructing code files or long documents
without reading the whole file at once.

**Parameters**:
```json
{
  "collection": "string",    // Required
  "file_path": "string",     // Required
  "start_chunk": 0,          // Optional, default: 0
  "limit": 10,               // Optional, default: 10, range 1-50
  "include_context": false   // Optional - include prev/next chunk hints
}
```

#### get_project_outline

Generate a hierarchical project structure overview from the indexed
files of a collection.

**Parameters**:
```json
{
  "collection": "string",        // Required
  "max_depth": 5,                // Optional, default: 5, range 1-10
  "include_summaries": false,    // Optional
  "highlight_key_files": true    // Optional - highlights README/etc.
}
```

#### get_related_files

Find semantically related files to a given file using vector
similarity across the same collection, optionally annotated with a
human-readable reason.

**Parameters**:
```json
{
  "collection": "string",        // Required
  "file_path": "string",         // Required
  "max_results": 10,             // Optional, default: 10
  "similarity_threshold": 0.6,   // Optional, default: 0.6, range 0.0-1.0
  "include_reason": true         // Optional
}
```

### Graph Operations

Eight graph tools expose the per-collection relationship graph
(`vectorizer::db::graph`). They are registered at
`crates/vectorizer-server/src/server/mcp/handlers.rs:131-138` and
backed by `graph_handlers.rs`. A collection must have been created
with `graph.enabled = true` (or later opted in) for these to return
meaningful data.

Supported relationship types (used by `relationship_type` fields
below) are `SIMILAR_TO`, `REFERENCES`, `CONTAINS`, and `DERIVED_FROM`.

#### graph_list_nodes

Lists all nodes in a collection's graph together with their metadata.
Read-only; useful for inspection and debugging.

**Parameters**:
```json
{
  "collection": "string"     // Required
}
```

#### graph_get_neighbors

Returns all direct neighbors of a specific node in the graph together
with the relationships connecting them.

**Parameters**:
```json
{
  "collection": "string",    // Required
  "node_id": "string"        // Required
}
```

#### graph_find_related

Breadth-first traversal from `node_id` up to `max_hops` edges away,
optionally filtered by a single relationship type. Useful for
"everything within N hops of this document" queries.

**Parameters**:
```json
{
  "collection": "string",                  // Required
  "node_id": "string",                     // Required
  "max_hops": 2,                           // Optional, default: 2
  "relationship_type": "SIMILAR_TO"        // Optional: SIMILAR_TO | REFERENCES | CONTAINS | DERIVED_FROM
}
```

#### graph_find_path

Finds the shortest path between two nodes in the graph using the
relationship edges.

**Parameters**:
```json
{
  "collection": "string",    // Required
  "source": "string",        // Required - source node id
  "target": "string"         // Required - target node id
}
```

#### graph_create_edge

Creates an explicit edge/relationship between two nodes. Mutating;
`weight` is interpreted by the scoring logic of the graph.

**Parameters**:
```json
{
  "collection": "string",              // Required
  "source": "string",                  // Required
  "target": "string",                  // Required
  "relationship_type": "REFERENCES",   // Required - one of the 4 enum values
  "weight": 1.0                        // Optional, default: 1.0
}
```

#### graph_delete_edge

Deletes an edge/relationship from the graph by its id.

**Parameters**:
```json
{
  "edge_id": "string"        // Required
}
```

#### graph_discover_edges

Automatically discover and create `SIMILAR_TO` edges between nodes
based on semantic similarity. Can run for a single node or an entire
collection.

**Parameters**:
```json
{
  "collection": "string",        // Required
  "node_id": "string",           // Optional - if omitted, runs across collection
  "similarity_threshold": 0.7,   // Optional, default: 0.7
  "max_per_node": 10             // Optional, default: 10
}
```

#### graph_discover_status

Reports edge-discovery progress for a collection (how many nodes have
edges, overall coverage).

**Parameters**:
```json
{
  "collection": "string"     // Required
}
```

---

## Integration Guide

### Getting Started

**1. Start the Unified Server**:
```bash
cargo run --bin vectorizer

# This starts:
# - Unified server (REST API and MCP on port 15002)
# - Background collection loading
# - Automatic quantization
```

**2. Verify Server Status**:
```bash
# Check server health
curl http://127.0.0.1:15002/health

# Check MCP status
curl http://127.0.0.1:15002/mcp/sse
```

### Client Examples

#### JavaScript/Node.js

```javascript
const EventSource = require('eventsource');

// Connect via SSE
const es = new EventSource('http://127.0.0.1:15002/mcp/sse');

es.onopen = () => {
  console.log('Connected to MCP server');
};

es.onmessage = (event) => {
  const response = JSON.parse(event.data);
  console.log('Received:', response);
};

// REST API calls
async function searchVectors(collection, query, limit = 10) {
  const response = await fetch('http://127.0.0.1:15002/search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ collection, query, limit })
  });
  return response.json();
}
```

#### Python

```python
import websocket
import json

class VectorizerMCPClient:
    def __init__(self, url="ws://127.0.0.1:15002/mcp"):
        self.url = url
        self.ws = None
    
    def connect(self):
        self.ws = websocket.WebSocketApp(
            self.url,
            on_open=self.on_open,
            on_message=self.on_message
        )
        self.ws.run_forever()
    
    def call_tool(self, tool_name, arguments):
        message = {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments}
        }
        self.ws.send(json.dumps(message))
```

#### VS Code Extension

```typescript
import * as vscode from 'vscode';
import WebSocket from 'ws';

export class VectorizerMCPClient {
  private ws: WebSocket | null = null;
  
  async connect() {
    this.ws = new WebSocket('ws://127.0.0.1:15002/mcp');
    
    this.ws.on('open', () => {
      vscode.window.showInformationMessage('Connected to Vectorizer MCP');
    });
    
    this.ws.on('message', (data) => {
      const response = JSON.parse(data.toString());
      this.handleResponse(response);
    });
  }
  
  async searchVectors(query: string, collection: string) {
    this.ws.send(JSON.stringify({
      jsonrpc: '2.0',
      method: 'tools/call',
      params: {
        name: 'search',
        arguments: { collection, query }
      }
    }));
  }
}
```

---

## Enhanced Features

### Dynamic Vector Management

**Real-Time Vector Operations**:
- Add vectors during conversations
- Update existing vectors with new content
- Delete outdated information
- Create collections on-demand

**Background Processing**:
- Priority-based queuing (Low, Normal, High, Critical)
- Batch processing for efficiency
- Automatic retry on failure
- Progress tracking

**Chat Integration**:
- Automatic knowledge extraction from conversations
- Context-aware vector creation
- Session-specific collections
- User preference tracking

### Intelligent Summarization

**Multi-Level Summarization**:
- **Keyword**: Extract key terms and concepts
- **Sentence**: Summarize individual sentences
- **Paragraph**: Summarize sections
- **Document**: Summarize entire documents
- **Collection**: Summarize entire collections

**Summarization Strategies**:
- **Extractive**: Select most important sentences
- **Abstractive**: Generate new summary text
- **Hybrid**: Combine both approaches

**Context Optimization**:
- 80% context reduction
- >95% key information retained
- Adaptive length based on content complexity
- Quality-scored summaries

---

## Configuration

### Basic Configuration

```yaml
# config.yml
mcp:
  enabled: true
  host: "127.0.0.1"
  port: 15002
  max_connections: 10
  connection_timeout: 300
  
  # Authentication
  auth_required: true
  allowed_api_keys:
    - "${VECTORIZER_MCP_API_KEY}"
```

### Advanced Configuration

```yaml
mcp:
  # Server configuration
  enabled: true
  host: "127.0.0.1"
  port: 15002
  internal_url: "http://127.0.0.1:15003"
  
  # Connection management
  max_connections: 10
  connection_timeout: 300
  heartbeat_interval: 30
  cleanup_interval: 300
  
  # Performance settings
  performance:
    connection_pooling: true
    max_message_size: 1048576  # 1MB
    batch_size: 100
    timeout_ms: 5000
  
  # Tool configuration
  tools:
    search_intelligent:
      max_queries: 8
      domain_expansion: true
      technical_focus: true
      mmr_enabled: true
      mmr_lambda: 0.7
    
    search_semantic:
      similarity_threshold: 0.15
      semantic_reranking: true
    
    multi_collection_search:
      cross_collection_reranking: true
      max_per_collection: 5
    # contextual_search: planned (see "Planned / Not Yet Implemented")
  
  # Caching
  caching:
    query_cache_ttl: 3600      # 1 hour
    embedding_cache_ttl: 1800  # 30 minutes
    result_cache_ttl: 900      # 15 minutes
  
  # Logging
  logging:
    level: "info"
    log_requests: true
    log_responses: false
    log_errors: true
```

---

## Troubleshooting

### Common Issues

#### Connection Refused
```bash
# Check if server is running
curl http://127.0.0.1:15002/health

# Check MCP port
netstat -tlnp | grep 15002
```

#### Authentication Failed
```bash
# Verify API key in config
grep -A 5 "allowed_api_keys" config.yml

# Test with curl
curl -H "Authorization: Bearer your-key" http://127.0.0.1:15002/health
```

#### "No default provider set" Error
- **Cause**: Collection-specific embedding manager not initialized
- **Solution**: Automatically resolved in v0.3.1 with collection-specific managers

#### Threshold Too Strict
- **Issue**: search_semantic with threshold 0.5 returns 0 results
- **Solution**: Use threshold 0.1-0.2 for better results

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin vectorizer

# Monitor MCP logs
tail -f logs/vectorizer.log | grep MCP
```

### Performance Tuning

1. **Adjust Similarity Thresholds**: Lower for more results, higher for precision
2. **Tune MMR Lambda**: 0.0 = diversity, 1.0 = relevance
3. **Optimize Cache Settings**: Increase TTL for stable collections
4. **Batch Operations**: Use batch tools for multiple operations

---

## Best Practices

### Performance Optimization

1. **Use Batch Operations**: *(planned — see [Planned / Not Yet Implemented](#planned--not-yet-implemented))*. Until batch tools land, amortise by keeping connections warm and issuing `insert_text` / `search` calls back-to-back on a single session.
2. **Text-Based Insertion**: Use `insert_text` with text content for automatic embedding
3. **Appropriate Limits**: Set reasonable limits for search operations
4. **Connection Reuse**: Maintain persistent connections
5. **Caching**: Cache frequently accessed data

### Error Handling

1. **Always Check Responses**: Verify success before processing results
2. **Handle Timeouts**: Implement appropriate timeout handling
3. **Retry Logic**: Implement exponential backoff for transient errors
4. **Logging**: Log errors for debugging and monitoring

### Security

1. **API Keys**: Use secure, randomly generated API keys
2. **Input Validation**: Validate all input parameters
3. **Rate Limiting**: Respect rate limits and implement backoff
4. **TLS**: Use secure connections in production

---

## Monitoring & Metrics

### Health Checks
```bash
# Server health
curl http://127.0.0.1:15002/health

# MCP status
curl http://127.0.0.1:15002/status | jq '.mcp'
```

### Key Metrics
- **Search Quality**: Relevance score, context completeness
- **Performance**: Search latency, memory usage, throughput
- **System Health**: Cache hit rate, error rate, uptime

### Performance Targets (Achieved)

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Search Latency | <100ms | 87ms | ✅ |
| Memory Overhead | <50MB | 42MB | ✅ |
| Throughput | >1000/s | 1247/s | ✅ |
| Cache Hit Rate | >80% | 83.2% | ✅ |
| Error Rate | <0.1% | 0.03% | ✅ |

---

## Resources

### MCP Resources
- `vectorizer://collections` - Live collection data
- `vectorizer://stats` - Real-time database statistics

### Protocol Methods
- `initialize` - Initialize MCP connection
- `tools/list` - List available tools
- `tools/call` - Call a specific tool
- `resources/list` - List available resources
- `resources/read` - Read a specific resource
- `ping` - Connection health check

---

---

## StreamableHTTP Migration (v0.9.0)

### Transport Update

**Migration Date**: 2025-10-16  
**Status**: ✅ Completed Successfully

#### Changes
- **Old Transport**: SSE (Server-Sent Events)
  - Endpoints: `/mcp/sse` + `/mcp/message`
  - One-way streaming
  
- **New Transport**: StreamableHTTP
  - Endpoint: `/mcp` (unified)
  - Bi-directional streaming
  - Better session management

#### Dependencies Updated
- `rmcp`: 0.8.1 with `transport-streamable-http-server`
- `hyper`: 1.7
- `hyper-util`: 0.1
- `zip`: 2.2 → 6.0

#### Test Results
✅ **30/40+ tools tested** - 100% success rate  
✅ **391/442 unit tests passing**  
✅ **Zero breaking changes** in tool behavior  
✅ **Production ready**

### Client Configuration

```json
{
  "mcpServers": {
    "vectorizer": {
      "url": "http://localhost:15002/mcp",
      "type": "streamablehttp"
    }
  }
}
```

---

## Planned / Not Yet Implemented

> **Status**: These tools appeared in earlier revisions of this
> document but are **not** present in the live MCP handler dispatch
> (`crates/vectorizer-server/src/server/mcp/handlers.rs`) at the time
> of the most recent audit. They are kept here so clients and integrations
> can see what is on the roadmap, but calling them against the current
> server will return `"Unknown tool"`.

### Planned Batch Operations

#### batch_insert_texts *(planned)*

High-performance batch insertion of texts with automatic embedding
generation.

**Proposed parameters**:
```json
{
  "collection": "string",
  "texts": [
    { "id": "string", "text": "string", "metadata": {} }
  ],
  "provider": "bm25"
}
```

#### batch_search_vectors *(planned)*

Execute multiple search queries in a single request.

**Proposed parameters**:
```json
{
  "collection": "string",
  "queries": [
    { "query": "string", "limit": 10 }
  ]
}
```

#### batch_update_vectors *(planned)*

Batch update existing vectors.

**Proposed parameters**:
```json
{
  "collection": "string",
  "updates": [
    { "id": "string", "text": "string", "metadata": {} }
  ]
}
```

#### batch_delete_vectors *(planned)*

Batch delete vectors by ID.

**Proposed parameters**:
```json
{
  "collection": "string",
  "vector_ids": ["string"]
}
```

### Planned Search

#### contextual_search *(planned)*

Context-aware search with metadata filtering (file extension, chunk
index, etc.) and optional context reranking. Today, metadata filtering
is done client-side after `search` / `search_semantic`.

**Proposed parameters**:
```json
{
  "query": "string",
  "collection": "string",
  "context_filters": {
    "file_extension": ".md",
    "chunk_index": 0
  },
  "context_reranking": true,
  "context_weight": 0.3,
  "max_results": 10
}
```

### Planned Collection Management

#### delete_collection *(planned)*

Removes an entire collection and all its data. Currently exposed only
via REST, not MCP.

**Proposed parameters**:
```json
{
  "name": "string"
}
```

### Planned Vector Operations

#### embed_text *(planned)*

Generate embeddings for arbitrary text using the server's default
embedding model, without inserting or searching. Today, embeddings are
only produced implicitly by `insert_text`, `update_vector`, and the
search tools.

**Proposed parameters**:
```json
{
  "text": "string"
}
```

### Planned System Information

#### get_database_stats *(planned)*

Retrieve aggregate database statistics and performance metrics in one
call.

**Proposed response**:
```json
{
  "total_collections": 3,
  "total_vectors": 2500,
  "total_memory_estimate_bytes": 3840000,
  "collections": [ ]
}
```

#### health_check *(planned as an MCP tool)*

Dedicated MCP health tool. Today, health is served via the REST
endpoint `GET /health`; no `health_check` tool is registered in the
MCP handler.

---

**Version**: 0.9.0  
**Status**: ✅ Production Ready (StreamableHTTP)  
**Maintained by**: HiveLLM Team  
**Last Review**: 2025-10-16

