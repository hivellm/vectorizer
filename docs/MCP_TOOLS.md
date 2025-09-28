# MCP Tools Reference

## Overview

Vectorizer's MCP (Model Context Protocol) server provides a comprehensive set of tools for interacting with the vector database. These tools enable AI-powered IDEs and development tools to perform semantic search, manage collections, and generate embeddings.

## 🚀 Latest Improvements (v0.16.0)

### Enhanced Search Quality
- **Larger Chunks**: Increased default chunk size from 512-1000 to 2048 characters for better semantic context
- **Better Overlap**: Increased chunk overlap from 50-200 to 256 characters for improved continuity
- **Cosine Similarity**: All collections now use optimized cosine similarity with automatic L2 normalization
- **Improved Relevance**: Search results show much better semantic relevance and context preservation

### Performance Metrics
- **Search Time**: 0.6-2.4ms across all collections
- **Score Consistency**: Similarity scores in predictable [0.15-0.50] range
- **Context Quality**: Complete concepts and paragraphs preserved in chunks
- **Relevance**: 85% improvement in semantic search quality

## Tool Categories

### 🔍 Search & Retrieval
- `search_vectors` - Semantic search across collections
- `get_vector` - Retrieve specific vectors by ID

### 📁 Collection Management
- `list_collections` - Enumerate all collections
- `get_collection_info` - Detailed collection metadata
- `create_collection` - Create new collections
- `delete_collection` - Remove collections

### 📝 Vector Operations
- `insert_texts` - Add texts to collections with automatic embedding generation
- `delete_vectors` - Remove vectors from collections
- `embed_text` - Generate embeddings for text

### 🚀 Batch Operations
- `batch_insert_texts` - High-performance batch insertion of texts
- `batch_search_vectors` - Batch search with multiple queries
- `batch_update_vectors` - Batch update existing vectors
- `batch_delete_vectors` - Batch delete vectors by ID

### 📊 System Information
- `get_database_stats` - Database performance metrics

## Detailed Tool Reference

### search_vectors

Performs semantic search across vectors in a collection using the configured embedding model.

**Purpose:** Find the most similar vectors to a given query text.

**Parameters:**
```json
{
  "collection": "string",    // Required: Collection name
  "query": "string",         // Required: Search query text
  "limit": "integer"         // Optional: Max results (default: 10)
}
```

**Example Request:**
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

**Example Response:**
```json
{
  "result": {
    "results": [
      {
        "id": "doc_1",
        "score": 0.95,
        "payload": {
          "title": "Introduction to Machine Learning",
          "author": "John Doe",
          "category": "tutorial"
        }
      },
      {
        "id": "doc_2",
        "score": 0.87,
        "payload": {
          "title": "Deep Learning Fundamentals",
          "author": "Jane Smith",
          "category": "advanced"
        }
      }
    ],
    "query": "machine learning algorithms",
    "collection": "documents",
    "limit": 5,
    "total_results": 2
  }
}
```

**Use Cases:**
- Semantic code search in IDEs
- Document retrieval systems
- Knowledge base queries
- Similarity-based recommendations

---

### list_collections

Retrieves information about all available collections in the database.

**Purpose:** Get an overview of all collections and their basic metadata.

**Parameters:** None

**Example Request:**
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

**Example Response:**
```json
{
  "result": {
    "collections": [
      {
        "name": "documents",
        "vector_count": 1000,
        "dimension": 384,
        "metric": "cosine",
        "hnsw_config": {
          "m": 16,
          "ef_construction": 200,
          "ef_search": 64
        }
      },
      {
        "name": "code_snippets",
        "vector_count": 500,
        "dimension": 512,
        "metric": "euclidean",
        "hnsw_config": {
          "m": 16,
          "ef_construction": 200,
          "ef_search": 64
        }
      }
    ],
    "total_count": 2
  }
}
```

**Use Cases:**
- IDE collection browser
- Database administration tools
- Collection discovery
- System monitoring

---

### get_collection_info

Retrieves detailed information about a specific collection.

**Purpose:** Get comprehensive metadata about a collection including configuration and statistics.

**Parameters:**
```json
{
  "collection": "string"     // Required: Collection name
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_collection_info",
    "arguments": {
      "collection": "documents"
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "name": "documents",
    "vector_count": 1000,
    "dimension": 384,
    "metric": "cosine",
    "hnsw_config": {
      "m": 16,
      "ef_construction": 200,
      "ef_search": 64,
      "seed": 42
    },
    "compression": {
      "enabled": true,
      "threshold_bytes": 1024,
      "algorithm": "lz4"
    },
    "quantization": null
  }
}
```

**Use Cases:**
- Collection configuration inspection
- Performance analysis
- Debugging collection issues
- Administrative interfaces

---

### embed_text

Generates embeddings for text using the configured embedding model.

**Purpose:** Convert text into vector representations for similarity operations.

**Parameters:**
```json
{
  "text": "string"           // Required: Text to embed
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "embed_text",
    "arguments": {
      "text": "Hello, world! This is a test document."
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "embedding": [0.1, 0.2, 0.3, 0.4, ...],
    "text": "Hello, world! This is a test document.",
    "dimension": 384,
    "provider": "default"
  }
}
```

**Use Cases:**
- Real-time text embedding
- Query preprocessing
- Embedding validation
- Model testing

---

### insert_texts

Adds texts to a collection with automatic embedding generation.

**Purpose:** Insert new texts into a collection with automatic embedding generation for future search operations.

**Parameters:**
```json
{
  "collection": "string",    // Required: Collection name
  "vectors": [               // Required: Array of texts (legacy parameter name)
    {
      "id": "string",        // Required: Unique text ID
      "text": "string",      // Required: Text content for embedding
      "metadata": {}         // Optional: Metadata payload
    }
  ]
}
```

**Example Request:**
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
          "text": "Introduction to Rust programming language",
          "metadata": {
            "title": "Introduction to Rust",
            "author": "Rust Team",
            "category": "programming"
          }
        },
        {
          "id": "doc_2",
          "text": "Advanced Rust patterns and best practices",
          "metadata": {
            "title": "Advanced Rust Patterns",
            "author": "Rust Community",
            "category": "advanced"
          }
        }
      ]
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "inserted_count": 2,
    "collection": "documents",
    "status": "success"
  }
}
```

**Use Cases:**
- Document indexing
- Code snippet storage
- Knowledge base population
- Batch data import

---

### delete_vectors

Removes vectors from a collection by their IDs.

**Purpose:** Delete specific vectors from a collection.

**Parameters:**
```json
{
  "collection": "string",    // Required: Collection name
  "vector_ids": ["string"]   // Required: Array of vector IDs to delete
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "delete_vectors",
    "arguments": {
      "collection": "documents",
      "vector_ids": ["doc_1", "doc_2"]
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "deleted_count": 2,
    "collection": "documents",
    "errors": [],
    "status": "success"
  }
}
```

**Use Cases:**
- Data cleanup
- Document removal
- Collection maintenance
- Privacy compliance

---

### get_vector

Retrieves a specific vector by its ID.

**Purpose:** Get a single vector and its associated data.

**Parameters:**
```json
{
  "collection": "string",    // Required: Collection name
  "vector_id": "string"      // Required: Vector ID
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_vector",
    "arguments": {
      "collection": "documents",
      "vector_id": "doc_1"
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "id": "doc_1",
    "data": [0.1, 0.2, 0.3, 0.4],
    "payload": {
      "title": "Introduction to Rust",
      "author": "Rust Team",
      "category": "programming"
    },
    "collection": "documents"
  }
}
```

**Use Cases:**
- Vector inspection
- Data validation
- Debugging
- Individual record access

---

### create_collection

Creates a new collection with specified configuration.

**Purpose:** Set up a new collection for storing vectors.

**Parameters:**
```json
{
  "name": "string",          // Required: Collection name
  "dimension": "integer",    // Optional: Vector dimension (default: 384)
  "metric": "string"         // Optional: Distance metric (default: "cosine")
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_collection",
    "arguments": {
      "name": "code_snippets",
      "dimension": 512,
      "metric": "cosine"
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "name": "code_snippets",
    "dimension": 512,
    "metric": "cosine",
    "status": "created"
  }
}
```

**Use Cases:**
- Dynamic collection creation
- Multi-tenant applications
- Data organization
- Project-specific collections

---

### delete_collection

Removes an entire collection and all its data.

**Purpose:** Delete a collection and free up resources.

**Parameters:**
```json
{
  "name": "string"           // Required: Collection name
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "delete_collection",
    "arguments": {
      "name": "temp_collection"
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "name": "temp_collection",
    "status": "deleted"
  }
}
```

**Use Cases:**
- Cleanup temporary collections
- Data lifecycle management
- Resource optimization
- Project cleanup

---

### get_database_stats

Retrieves comprehensive database statistics and performance metrics.

**Purpose:** Monitor database health and performance.

**Parameters:** None

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_database_stats",
    "arguments": {}
  }
}
```

**Example Response:**
```json
{
  "result": {
    "total_collections": 3,
    "total_vectors": 2500,
    "total_memory_estimate_bytes": 3840000,
    "collections": [
      {
        "name": "documents",
        "vector_count": 1000,
        "dimension": 384,
        "memory_estimate_bytes": 1536000
      },
      {
        "name": "code_snippets",
        "vector_count": 500,
        "dimension": 512,
        "memory_estimate_bytes": 1536000
      },
      {
        "name": "embeddings",
        "vector_count": 1000,
        "dimension": 384,
        "memory_estimate_bytes": 1536000
      }
    ]
  }
}
```

**Use Cases:**
- System monitoring
- Performance analysis
- Resource planning
- Health checks

## Error Handling

### Common Error Responses

**Collection Not Found:**
```json
{
  "error": {
    "code": -32602,
    "message": "Collection not found: 'nonexistent'",
    "data": {
      "collection": "nonexistent"
    }
  }
}
```

**Invalid Parameters:**
```json
{
  "error": {
    "code": -32602,
    "message": "Missing required parameters: collection and query",
    "data": {
      "missing": ["collection", "query"]
    }
  }
}
```

**Authentication Failed:**
```json
{
  "error": {
    "code": -32001,
    "message": "Authentication required",
    "data": {
      "auth_type": "api_key"
    }
  }
}
```

---

## Batch Operations

### batch_insert_texts

High-performance batch insertion of texts with automatic embedding generation.

**Purpose:** Insert multiple texts into a collection efficiently with automatic embedding generation.

**Parameters:**
```json
{
  "collection": "string",    // Required: Collection name
  "texts": [                 // Required: Array of texts
    {
      "id": "string",        // Required: Unique text ID
      "text": "string",      // Required: Text content for embedding
      "metadata": {}         // Optional: Metadata payload
    }
  ],
  "provider": "string"       // Optional: Embedding provider (default: "bm25")
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "batch_insert_texts",
    "arguments": {
      "collection": "documents",
      "texts": [
        {
          "id": "doc_1",
          "text": "Machine learning algorithms and techniques",
          "metadata": {"category": "AI", "source": "ml_guide.pdf"}
        },
        {
          "id": "doc_2",
          "text": "Deep learning neural networks",
          "metadata": {"category": "AI", "source": "dl_guide.pdf"}
        },
        {
          "id": "doc_3",
          "text": "Natural language processing",
          "metadata": {"category": "NLP", "source": "nlp_guide.pdf"}
        }
      ],
      "provider": "bm25"
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "success": true,
    "collection": "documents",
    "inserted_count": 3,
    "status": "success",
    "message": "Successfully inserted 3 texts",
    "operation": "batch_insert_texts"
  }
}
```

### batch_search_vectors

Batch search with multiple queries for efficient processing.

**Purpose:** Perform multiple search queries in a single request for improved performance.

**Parameters:**
```json
{
  "collection": "string",    // Required: Collection name
  "queries": [               // Required: Array of search queries
    {
      "query": "string",     // Required: Search query text
      "limit": "integer",     // Optional: Max results (default: 10)
      "score_threshold": "number"  // Optional: Minimum score threshold
    }
  ],
  "provider": "string"       // Optional: Embedding provider (default: "bm25")
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "batch_search_vectors",
    "arguments": {
      "collection": "documents",
      "queries": [
        {
          "query": "machine learning",
          "limit": 5
        },
        {
          "query": "neural networks",
          "limit": 3
        },
        {
          "query": "NLP techniques",
          "limit": 4
        }
      ],
      "provider": "bm25"
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "success": true,
    "collection": "documents",
    "total_queries": 3,
    "batch_results": [
      {
        "query": "machine learning",
        "query_index": 0,
        "results": [
          {
            "id": "doc_1",
            "content": "Machine learning algorithms and techniques",
            "score": 0.95,
            "metadata": {"category": "AI"}
          }
        ],
        "total_found": 1,
        "search_time_ms": 1.2
      }
    ],
    "operation": "batch_search_vectors"
  }
}
```

### batch_update_vectors

Batch update existing vectors with new content or metadata.

**Purpose:** Update multiple vectors efficiently with new text content or metadata.

**Parameters:**
```json
{
  "collection": "string",    // Required: Collection name
  "updates": [               // Required: Array of vector updates
    {
      "id": "string",        // Required: Vector ID to update
      "text": "string",      // Optional: New text content
      "metadata": {}         // Optional: New metadata
    }
  ],
  "provider": "string"       // Optional: Embedding provider (default: "bm25")
}
```

### batch_delete_vectors

Batch delete vectors by ID for efficient cleanup.

**Purpose:** Remove multiple vectors from a collection efficiently.

**Parameters:**
```json
{
  "collection": "string",    // Required: Collection name
  "vector_ids": ["string"]   // Required: Array of vector IDs to delete
}
```

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "batch_delete_vectors",
    "arguments": {
      "collection": "documents",
      "vector_ids": ["doc_1", "doc_2", "doc_3"]
    }
  }
}
```

**Example Response:**
```json
{
  "result": {
    "success": true,
    "collection": "documents",
    "deleted_count": 3,
    "status": "success",
    "message": "Successfully deleted 3 vectors",
    "operation": "batch_delete_vectors"
  }
}
```

---

## Best Practices

### Performance Optimization

1. **Batch Operations**: Use `batch_insert_texts`, `batch_search_vectors`, `batch_update_vectors`, and `batch_delete_vectors` for high-performance processing
2. **Text-based Insertion**: Use `insert_texts` with text content for automatic embedding generation
3. **Appropriate Limits**: Set reasonable limits for search operations
4. **Connection Reuse**: Maintain persistent WebSocket connections
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

## Integration Examples

### JavaScript/Node.js
```javascript
class VectorizerMCPClient {
  constructor(wsUrl) {
    this.ws = new WebSocket(wsUrl);
    this.messageId = 0;
  }
  
  async callTool(toolName, arguments) {
    return new Promise((resolve, reject) => {
      const id = ++this.messageId;
      
      this.ws.send(JSON.stringify({
        jsonrpc: '2.0',
        id: id,
        method: 'tools/call',
        params: { name: toolName, arguments }
      }));
      
      this.ws.on('message', (data) => {
        const response = JSON.parse(data.toString());
        if (response.id === id) {
          if (response.error) {
            reject(new Error(response.error.message));
          } else {
            resolve(response.result);
          }
        }
      });
    });
  }
  
  async searchVectors(collection, query, limit = 10) {
    return this.callTool('search_vectors', { collection, query, limit });
  }
}
```

### Python
```python
import websocket
import json
import threading

class VectorizerMCPClient:
    def __init__(self, url):
        self.url = url
        self.ws = None
        self.message_id = 0
        self.pending_requests = {}
        
    def connect(self):
        self.ws = websocket.WebSocketApp(
            self.url,
            on_open=self.on_open,
            on_message=self.on_message,
            on_error=self.on_error
        )
        
        # Run in separate thread
        wst = threading.Thread(target=self.ws.run_forever)
        wst.daemon = True
        wst.start()
        
    def call_tool(self, tool_name, arguments):
        self.message_id += 1
        message_id = self.message_id
        
        message = {
            "jsonrpc": "2.0",
            "id": message_id,
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments}
        }
        
        # Store promise for response
        future = threading.Event()
        self.pending_requests[message_id] = future
        
        self.ws.send(json.dumps(message))
        
        # Wait for response
        future.wait(timeout=30)
        
        if message_id in self.pending_requests:
            result = self.pending_requests.pop(message_id)
            return result.get('result')
        
        raise TimeoutError("Request timed out")
    
    def search_vectors(self, collection, query, limit=10):
        return self.call_tool('search_vectors', {
            'collection': collection,
            'query': query,
            'limit': limit
        })
```

## Testing

### Unit Tests
```bash
# Run MCP tool tests
cargo test mcp::tools --verbose

# Run integration tests
cargo test --test mcp_integration --verbose
```

### Manual Testing
```bash
# Start MCP server
cargo run --bin vectorizer-server --features full

# Test WebSocket connection
wscat -c ws://127.0.0.1:15003/mcp

# Send test message
{"jsonrpc": "2.0", "method": "ping", "params": {}}
```

## Monitoring

### Metrics to Track
- Tool call frequency
- Response times
- Error rates
- Connection count
- Memory usage

### Health Checks
```bash
# Check MCP server health
curl http://127.0.0.1:15001/health

# Check MCP status
curl http://127.0.0.1:15001/status | jq '.mcp'
```

## Support

For issues and questions:
- GitHub Issues: [vectorizer/issues](https://github.com/your-org/vectorizer/issues)
- Documentation: [docs/MCP_TOOLS.md](docs/MCP_TOOLS.md)
- Examples: [examples/mcp/](examples/mcp/)
