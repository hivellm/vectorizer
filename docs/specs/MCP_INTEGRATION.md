# MCP (Model Context Protocol) Integration

## Overview

Vectorizer implements a comprehensive MCP (Model Context Protocol) server that enables seamless integration with AI-powered IDEs and development tools. The MCP server provides a standardized interface for AI models to interact with the vector database through Server-Sent Events (SSE) connections.

## Architecture (v0.3.0)

```
┌─────────────────┐    SSE/HTTP     ┌──────────────────┐
│   AI IDE/Client │ ◄─────────────► │  Unified Server  │
│                 │   http://:15002 │  (Port 15002)    │
└─────────────────┘                 └──────────────────┘
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │ Vector Database │
                                    │                 │
                                    └─────────────────┘
```

### Unified Server Benefits
- **Single Process**: Reduced memory footprint
- **Unified Interface**: REST API and MCP in one server
- **Background Loading**: Non-blocking server startup
- **Automatic Quantization**: Memory optimization

## Features

### 🔌 Server-Sent Events (SSE) Communication
- Real-time unidirectional communication from server to client
- JSON-RPC 2.0 protocol compliance
- Automatic connection management
- HTTP-based with SSE transport

### 🛠️ Comprehensive Tool Set
- **search_vectors**: Semantic search across collections
- **list_collections**: Enumerate all available collections
- **get_collection_info**: Detailed collection metadata
- **embed_text**: Generate embeddings for text
- **insert_texts**: Add vectors to collections
- **delete_vectors**: Remove vectors from collections
- **get_vector**: Retrieve specific vectors
- **create_collection**: Create new collections
- **delete_collection**: Remove collections
- **get_database_stats**: System performance metrics
- **update_vector**: Update existing vectors (Phase 5)
- **batch_update_vectors**: Bulk vector updates (Phase 5)
- **reindex_incremental**: Incremental reindexing (Phase 5)
- **watch_files**: Real-time file monitoring (Phase 5)

### 🔐 Security & Authentication
- API key-based authentication
- Configurable access control
- Rate limiting per connection
- Secure WebSocket connections

### 📊 Resource Access
- **vectorizer://collections**: Live collection data
- **vectorizer://stats**: Real-time database statistics

## Configuration

### Basic MCP Configuration

```yaml
# config.yml
mcp:
  enabled: true
  host: "127.0.0.1"
  port: 15002  # MCP Server port
  internal_url: "http://127.0.0.1:15003"  # vzr internal server
  max_connections: 10
  connection_timeout: 300
  auth_required: true
  allowed_api_keys:
    - "your-api-key-here"
```

### Advanced Configuration

```yaml
mcp:
  enabled: true
  host: "127.0.0.1"
  port: 15002  # MCP Server port
  internal_url: "http://127.0.0.1:15003"  # vzr internal server
  
  # Connection management
  max_connections: 10
  connection_timeout: 300
  
  # Internal client configuration
  internal_client:
    timeout: 30
    connect_timeout: 5
    keep_alive_timeout: 30
    max_receive_message_length: 4194304  # 4MB
    max_send_message_length: 4194304     # 4MB
  
  # Authentication
  auth_required: true
  allowed_api_keys:
    - "mcp-api-key-1"
    - "mcp-api-key-2"
  
  # Server information
  server_info:
    name: "Vectorizer MCP Server"
    version: "0.3.0"
    description: "Model Context Protocol server for Vectorizer unified architecture"
  
  # Performance settings
  performance:
    connection_pooling: true
    max_message_size: 1048576  # 1MB
    heartbeat_interval: 30
    cleanup_interval: 300
  
  # Logging
  logging:
    level: "info"
    log_requests: true
    log_responses: false
    log_errors: true
```

## Getting Started

### 1. Start the Unified Server

```bash
# Start unified server with REST API and MCP
cargo run --bin vectorizer

# This starts:
# - Unified server (REST API and MCP on port 15002)
# - Background collection loading
# - Automatic quantization
```

### 2. Verify Server Status

```bash
# Check server health
curl http://127.0.0.1:15002/health

# Check MCP status
curl http://127.0.0.1:15002/mcp/sse
```

### 3. Connect via Server-Sent Events

```javascript
const EventSource = require('eventsource');
const es = new EventSource('http://127.0.0.1:15002/mcp/sse');

es.onopen = () => {
  console.log('Connected to MCP server');
  
  // SSE automatically handles connection
  console.log('SSE connection established');
});

es.onmessage = (event) => {
  const response = JSON.parse(event.data);
  console.log('Received:', response);
};

es.onerror = (error) => {
  console.error('SSE connection error:', error);
};
```

## MCP Tools Reference

### search_vectors

Search for similar vectors in a collection.

**Parameters:**
- `collection` (string): Collection name
- `query` (string): Search query
- `limit` (integer, optional): Maximum results (default: 10)

**Example:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "search_vectors",
    "arguments": {
      "collection": "documents",
      "query": "machine learning algorithms",
      "limit": 5
    }
  }
}
```

**Response:**
```json
{
  "result": {
    "results": [
      {
        "id": "doc_1",
        "score": 0.95,
        "payload": {"title": "ML Guide", "author": "John Doe"}
      }
    ],
    "query": "machine learning algorithms",
    "collection": "documents",
    "limit": 5,
    "total_results": 1
  }
}
```

### list_collections

List all available collections.

**Parameters:** None

**Example:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "list_collections",
    "arguments": {}
  }
}
```

**Response:**
```json
{
  "result": {
    "collections": [
      {
        "name": "documents",
        "vector_count": 1000,
        "dimension": 384,
        "metric": "cosine"
      }
    ],
    "total_count": 1
  }
}
```

### embed_text

Generate embeddings for text.

**Parameters:**
- `text` (string): Text to embed

**Example:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "embed_text",
    "arguments": {
      "text": "Hello, world!"
    }
  }
}
```

**Response:**
```json
{
  "result": {
    "embedding": [0.1, 0.2, 0.3, ...],
    "text": "Hello, world!",
    "dimension": 384,
    "provider": "default"
  }
}
```

### insert_texts

Insert vectors into a collection.

**Parameters:**
- `collection` (string): Collection name
- `vectors` (array): Array of vectors to insert

**Example:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "insert_texts",
    "arguments": {
      "collection": "documents",
      "vectors": [
        {
          "id": "doc_1",
          "data": [0.1, 0.2, 0.3],
          "payload": {"title": "Document 1"}
        }
      ]
    }
  }
}
```

**Response:**
```json
{
  "result": {
    "inserted_count": 1,
    "collection": "documents",
    "status": "success"
  }
}
```

## IDE Integration Examples

### VS Code Extension

```typescript
import * as vscode from 'vscode';
import WebSocket from 'ws';

export class VectorizerMCPClient {
  private ws: WebSocket | null = null;
  
  async connect() {
    this.ws = new WebSocket('ws://127.0.0.1:15002/mcp');  // Updated port
    
    this.ws.on('open', () => {
      vscode.window.showInformationMessage('Connected to Vectorizer MCP');
    });
    
    this.ws.on('message', (data) => {
      const response = JSON.parse(data.toString());
      this.handleResponse(response);
    });
  }
  
  async searchVectors(query: string, collection: string) {
    if (!this.ws) throw new Error('Not connected');
    
    this.ws.send(JSON.stringify({
      jsonrpc: '2.0',
      method: 'tools/call',
      params: {
        name: 'search_vectors',
        arguments: { collection, query }
      }
    }));
  }
}
```

### Cursor IDE Integration

```python
import websocket
import json

class VectorizerMCPClient:
    def __init__(self, url="ws://127.0.0.1:15002/mcp"):  # Updated port
        self.url = url
        self.ws = None
    
    def connect(self):
        self.ws = websocket.WebSocketApp(
            self.url,
            on_open=self.on_open,
            on_message=self.on_message,
            on_error=self.on_error
        )
        self.ws.run_forever()
    
    def on_open(self, ws):
        print("Connected to Vectorizer MCP")
        # Initialize connection
        self.send_message({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocol_version": "2024-11-05",
                "capabilities": {},
                "client_info": {
                    "name": "Cursor IDE",
                    "version": "1.0.0"
                }
            }
        })
    
    def send_message(self, message):
        if self.ws:
            self.ws.send(json.dumps(message))
    
    def search_vectors(self, collection, query, limit=10):
        self.send_message({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "search_vectors",
                "arguments": {
                    "collection": collection,
                    "query": query,
                    "limit": limit
                }
            }
        })
```

## Performance Considerations

### Connection Management
- Maximum 10 concurrent connections by default
- Automatic cleanup of inactive connections
- Heartbeat mechanism for connection health

### Message Size Limits
- Maximum message size: 1MB
- Large responses are automatically chunked
- Compression support for large payloads

### Rate Limiting
- Per-connection rate limiting
- Configurable request limits
- Automatic backoff on rate limit exceeded

## Security Best Practices

### API Key Management
```bash
# Generate secure API key
openssl rand -hex 32

# Store in environment variable
export VECTORIZER_MCP_API_KEY="your-secure-api-key"
```

### Network Security
- Use TLS for production deployments
- Restrict access to localhost in development
- Implement IP whitelisting for production

### Authentication
```yaml
mcp:
  auth_required: true
  allowed_api_keys:
    - "${VECTORIZER_MCP_API_KEY}"
```

## Troubleshooting

### Common Issues

**Connection Refused**
```bash
# Check if server is running
curl http://127.0.0.1:15001/api/v1/health

# Check MCP port
netstat -tlnp | grep 15002

# Check internal port
netstat -tlnp | grep 15003
```

**Authentication Failed**
```bash
# Verify API key in config
grep -A 5 "allowed_api_keys" config.yml

# Test with curl
curl -H "Authorization: Bearer your-key" http://127.0.0.1:15001/api/v1/status
```

**WebSocket Connection Issues**
```javascript
// Test WebSocket connection
const ws = new WebSocket('ws://127.0.0.1:15002/mcp');  // Updated port
ws.on('error', (error) => {
  console.error('WebSocket error:', error);
});
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin vectorizer-server --features full
```

### Log Analysis

```bash
# Monitor MCP logs
tail -f logs/vectorizer.log | grep MCP

# Check connection statistics
curl http://127.0.0.1:15001/status | jq '.mcp'
```

## Monitoring & Metrics

### Health Checks
```bash
# Server health
curl http://127.0.0.1:15001/health

# MCP specific health
curl http://127.0.0.1:15001/status | jq '.mcp'
```

### Performance Metrics
- Connection count
- Message throughput
- Response times
- Error rates

### Logging
```yaml
mcp:
  logging:
    level: "info"
    log_requests: true
    log_responses: false
    log_errors: true
```

## API Reference

### WebSocket Endpoints
- `ws://127.0.0.1:15002/mcp` - Main MCP endpoint

### HTTP Endpoints
- `GET /api/v1/health` - Server health check
- `GET /api/v1/status` - Server status including MCP info
- `GET /api/v1/collections` - List collections

### Internal Endpoints
- `http://127.0.0.1:15003` - vzr internal orchestrator

### MCP Protocol Methods
- `initialize` - Initialize MCP connection
- `tools/list` - List available tools
- `tools/call` - Call a specific tool
- `resources/list` - List available resources
- `resources/read` - Read a specific resource
- `ping` - Ping/pong for connection health

## Examples

See the `examples/mcp/` directory for complete integration examples:
- `basic-client.js` - Basic WebSocket client
- `cursor-integration.py` - Cursor IDE integration
- `vscode-extension.ts` - VS Code extension
- `performance-test.js` - Performance testing

## Support

For issues and questions:
- GitHub Issues: [vectorizer/issues](https://github.com/your-org/vectorizer/issues)
- Documentation: [docs/MCP_INTEGRATION.md](docs/MCP_INTEGRATION.md)
- Examples: [examples/mcp/](examples/mcp/)
