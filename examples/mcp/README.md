# MCP Integration Examples

This directory contains examples and tools for integrating Vectorizer's MCP (Model Context Protocol) server with various IDEs and development tools.

## Overview

Vectorizer implements a comprehensive MCP server that enables AI-powered IDEs to interact with the vector database through WebSocket connections. The MCP server provides tools for semantic search, collection management, and embedding generation.

## Quick Start

### 1. Start Vectorizer Server

```bash
# Start the server with MCP enabled
cargo run --bin vectorizer-server --features full

# Or with custom configuration
cargo run --bin vectorizer-server --features full -- --config config.yml
```

### 2. Verify MCP Server

```bash
# Check server health
curl http://127.0.0.1:15001/health

# Check MCP status
curl http://127.0.1:15001/status | jq '.mcp'
```

### 3. Test MCP Connection

```bash
# JavaScript/Node.js
cd examples/mcp
npm install
node basic-client.js

# Python
pip install -r requirements.txt
python cursor-integration.py
```

## Examples

### Basic MCP Client (JavaScript)

A simple WebSocket client that demonstrates basic MCP operations:

```bash
node basic-client.js
```

**Features:**
- WebSocket connection management
- MCP protocol implementation
- Interactive CLI
- Tool calling examples

### Cursor IDE Integration (Python)

A comprehensive integration for Cursor IDE with code indexing and semantic search:

```bash
python cursor-integration.py
```

**Features:**
- Automatic code indexing
- Semantic code search
- Function and class detection
- Multi-language support
- Interactive mode

### Performance Testing

Load testing and performance benchmarking:

```bash
node performance-test.js
```

**Features:**
- WebSocket connection testing
- Tool call benchmarking
- Memory usage monitoring
- Response time analysis

## MCP Tools Reference

### Available Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `search_vectors` | Semantic search across collections | `collection`, `query`, `limit` |
| `list_collections` | List all available collections | None |
| `get_collection_info` | Get collection metadata | `collection` |
| `embed_text` | Generate embeddings for text | `text` |
| `insert_vectors` | Add vectors to collections | `collection`, `vectors` |
| `delete_vectors` | Remove vectors from collections | `collection`, `vector_ids` |
| `get_vector` | Retrieve specific vectors | `collection`, `vector_id` |
| `create_collection` | Create new collections | `name`, `dimension`, `metric` |
| `delete_collection` | Remove collections | `name` |
| `get_database_stats` | Database performance metrics | None |

### Example Tool Calls

**Search Vectors:**
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

**Embed Text:**
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

**Create Collection:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_collection",
    "arguments": {
      "name": "code_snippets",
      "dimension": 768,
      "metric": "cosine"
    }
  }
}
```

## Configuration

### MCP Server Configuration

```yaml
# config.yml
mcp:
  enabled: true
  host: "127.0.0.1"
  port: 15003
  max_connections: 10
  connection_timeout: 300
  auth_required: true
  allowed_api_keys:
    - "your-api-key-here"
  
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

### Client Configuration

**JavaScript:**
```javascript
const client = new VectorizerMCPClient('ws://127.0.0.1:15003/mcp');
```

**Python:**
```python
client = VectorizerMCPClient("ws://127.0.0.1:15003/mcp")
```

## IDE Integration

### VS Code Extension

```typescript
import * as vscode from 'vscode';
import WebSocket from 'ws';

export class VectorizerMCPClient {
  private ws: WebSocket | null = null;
  
  async connect() {
    this.ws = new WebSocket('ws://127.0.0.1:15003/mcp');
    
    this.ws.on('open', () => {
      vscode.window.showInformationMessage('Connected to Vectorizer MCP');
    });
  }
  
  async searchVectors(query: string, collection: string) {
    // Implementation here
  }
}
```

### Cursor IDE Integration

```python
import websocket
import json

class VectorizerMCPClient:
    def __init__(self, url="ws://127.0.0.1:15003/mcp"):
        self.url = url
        self.ws = None
    
    def connect(self):
        self.ws = websocket.WebSocketApp(
            self.url,
            on_open=self.on_open,
            on_message=self.on_message
        )
        self.ws.run_forever()
    
    def search_vectors(self, collection, query, limit=10):
        # Implementation here
        pass
```

## Testing

### Unit Tests

```bash
# Run MCP tests
cargo test mcp --verbose

# Run integration tests
cargo test --test mcp_integration --verbose
```

### Manual Testing

```bash
# Test WebSocket connection
wscat -c ws://127.0.0.1:15003/mcp

# Send test message
{"jsonrpc": "2.0", "method": "ping", "params": {}}
```

### Performance Testing

```bash
# JavaScript performance test
node performance-test.js

# Python performance test
python -m pytest tests/test_performance.py
```

## Troubleshooting

### Common Issues

**Connection Refused:**
```bash
# Check if server is running
curl http://127.0.0.1:15001/health

# Check MCP port
netstat -tlnp | grep 15003
```

**Authentication Failed:**
```bash
# Verify API key in config
grep -A 5 "allowed_api_keys" config.yml

# Test with curl
curl -H "X-API-Key: your-key" http://127.0.0.1:15001/status
```

**WebSocket Connection Issues:**
```javascript
// Test WebSocket connection
const ws = new WebSocket('ws://127.0.0.1:15003/mcp');
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

## Monitoring

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

## Contributing

### Adding New Tools

1. Define the tool in `src/mcp/mod.rs`
2. Implement the tool logic in `src/mcp/tools.rs`
3. Add the tool handler in `src/mcp/handlers.rs`
4. Update the configuration schema
5. Add tests and documentation

### Adding New Examples

1. Create a new example file
2. Add dependencies to `package.json` or `requirements.txt`
3. Update this README
4. Add tests if applicable

## Support

For issues and questions:
- GitHub Issues: [vectorizer/issues](https://github.com/your-org/vectorizer/issues)
- Documentation: [docs/MCP_INTEGRATION.md](../../docs/MCP_INTEGRATION.md)
- Examples: [examples/mcp/](.)

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.
