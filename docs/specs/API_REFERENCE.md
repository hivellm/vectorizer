# API Reference & Integrations

**Version**: 0.9.0  
**Base URL**: `http://localhost:15002`  
**MCP Endpoint**: `http://localhost:15002/mcp`  
**Status**: ✅ Production Ready

---

## Overview

Vectorizer provides multiple interfaces optimized for different use cases:

| Interface | Type | Description | Use Cases |
|-----------|------|-------------|-----------|
| **REST API** | Server | HTTP/JSON access | Custom integrations, direct API |
| **MCP API** | Server | Model Context Protocol | AI integration, IDE tools (StreamableHTTP) |
| **Python SDK** | Client | Python client library | Data Science, ML pipelines |
| **TypeScript SDK** | Client | TS/JS client library | Web apps, Node.js |
| **Rust SDK** | Client | Rust client library | High-performance apps |
| **CLI Tools** | Client | Command-line tools | Administration, scripts |

---

## Architecture

**Client-Server** with mandatory authentication:
- **Server**: Centralized Rust application (REST/MCP APIs)
- **SDKs**: Lightweight clients (Python, TypeScript, Rust)
- **Security**: All operations require valid API keys
- **Dashboard**: Local key management (localhost only)

---

## Framework Integrations

### LangChain

**Python**:
```python
from vectorizer_store import VectorizerStore

store = VectorizerStore(
    host="localhost",
    port=15002,
    collection="documents",
    api_key="your-key"
)

# Use with LangChain
from langchain.chains import RetrievalQA

qa = RetrievalQA.from_chain_type(
    llm=llm,
    retriever=store.as_retriever()
)
```

**TypeScript**:
```typescript
import { VectorizerStore } from '@hivellm/vectorizer-langchain';

const store = new VectorizerStore({
  host: 'localhost',
  port: 15002,
  collection: 'documents',
  apiKey: 'your-key'
});
```

### PyTorch

```python
from vectorizer.pytorch import PyTorchEmbedder

embedder = PyTorchEmbedder(
    model_name="sentence-transformers/all-MiniLM-L6-v2",
    device="cuda"  # or "cpu"
)

embeddings = embedder.embed_batch(texts)
```

### TensorFlow

```python
from vectorizer.tensorflow import TensorFlowEmbedder

embedder = TensorFlowEmbedder(
    model_name="universal-sentence-encoder"
)

embeddings = embedder.embed(texts)
```

---

## Authentication

All endpoints require API key authentication:

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:15002/health
```

### Getting API Keys

**CLI**:
```bash
vectorizer api-keys create --name "my-app"
vectorizer api-keys list
vectorizer api-keys delete <key-id>
```

**Dashboard**: `http://localhost:15002` (localhost only)

---

## REST API Endpoints

### Health & Status

**GET /health** - Server health check
**GET /status** - Detailed server status
**GET /collections** - List all collections
**GET /collections/{name}** - Collection details

### Vector Operations

**POST /collections/{name}/search** - Search vectors
**POST /collections/{name}/vectors** - Insert vectors
**DELETE /collections/{name}/vectors/{id}** - Delete vector
**PUT /collections/{name}/vectors/{id}** - Update vector

### Intelligent Search

**POST /intelligent_search** - Advanced multi-query search
**POST /semantic_search** - Pure semantic search
**POST /contextual_search** - Context-aware search
**POST /multi_collection_search** - Cross-collection search

### Collection Management

**POST /collections** - Create collection
**DELETE /collections/{name}** - Delete collection
**POST /collections/{name}/reindex** - Reindex collection

---

## MCP Tools

See [MCP.md](./MCP.md) for complete reference.

---

## SDK Usage

### Python

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    host="localhost",
    port=15002,
    api_key="your-api-key"
)

# Search
results = client.search("query", "collection")

# Insert
client.insert_text("collection", "id", "text", metadata={})
```

### TypeScript

```typescript
import { VectorizerClient } from '@hivellm/vectorizer';

const client = new VectorizerClient({
  host: 'localhost',
  port: 15002,
  apiKey: 'your-api-key'
});

// Search
const results = await client.search('query', 'collection');

// Insert
await client.insertText('collection', 'id', 'text', {});
```

---

## Error Handling

**400 Bad Request**: Invalid parameters
**401 Unauthorized**: Invalid/missing API key
**404 Not Found**: Collection/vector not found
**500 Internal Server Error**: Server error

---

**Complete API documentation**: See individual endpoint specs in this directory  
**Maintained by**: HiveLLM Team

