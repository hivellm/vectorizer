# Vectorizer API Documentation

This directory contains the complete API documentation for Vectorizer, including the OpenAPI 3.0.3 schema.

**Version:** 1.4.0  
**Status:** ✅ Production Ready  
**Last Updated:** 2025-01-24

## 📁 Files

- **`openapi.yaml`** - Complete OpenAPI 3.0.3 schema for Vectorizer API
- **`openapi.json`** - JSON format of the OpenAPI schema
- **`README.md`** - This file with usage instructions

## 🚀 How to Use

### 1. View Documentation

#### Swagger UI Online
1. Visit [Swagger Editor](https://editor.swagger.io/)
2. Copy the content from `openapi.yaml` file
3. Paste in the editor to view interactive documentation

#### Swagger UI Local
```bash
# Install Swagger UI
npm install -g swagger-ui-serve

# Serve documentation locally
swagger-ui-serve vectorizer/docs/api/openapi.yaml
```

#### Redoc
```bash
# Install Redoc CLI
npm install -g redoc-cli

# Generate HTML documentation
redoc-cli build vectorizer/docs/api/openapi.yaml --output vectorizer/docs/api/index.html
```

### 2. Generate SDKs

#### OpenAPI Generator
```bash
# Install OpenAPI Generator
npm install -g @openapitools/openapi-generator-cli

# Generate SDK for TypeScript
openapi-generator-cli generate -i vectorizer/docs/api/openapi.yaml -g typescript-fetch -o ./sdks/typescript

# Generate SDK for Python
openapi-generator-cli generate -i vectorizer/docs/api/openapi.yaml -g python -o ./sdks/python

# Generate SDK for Rust
openapi-generator-cli generate -i vectorizer/docs/api/openapi.yaml -g rust -o ./sdks/rust
```

#### Swagger Codegen
```bash
# Install Swagger Codegen
npm install -g swagger-codegen

# Generate SDK for JavaScript
swagger-codegen generate -i vectorizer/docs/api/openapi.yaml -l javascript -o ./sdks/javascript

# Generate SDK for Java
swagger-codegen generate -i vectorizer/docs/api/openapi.yaml -l java -o ./sdks/java
```

### 3. Validate Schema

```bash
# Install Swagger CLI
npm install -g swagger-cli

# Validate schema
swagger-cli validate vectorizer/docs/api/openapi.yaml

# Bundle (resolve references)
swagger-cli bundle vectorizer/docs/api/openapi.yaml -o vectorizer/docs/api/openapi-bundled.yaml
```

## 📋 Main Endpoints

### 🏥 System
- `GET /health` - Health check
- `GET /stats` - System statistics

### 📚 Collections
- `GET /collections` - List collections
- `POST /collections` - Create collection
- `GET /collections/{name}` - Get collection info
- `DELETE /collections/{name}` - Delete collection

### 🔍 Vectors
- `POST /collections/{name}/vectors` - Insert texts
- `GET /collections/{name}/vectors` - List vectors
- `GET /collections/{name}/vectors/{id}` - Get specific vector
- `DELETE /collections/{name}/vectors/{id}` - Delete vector

### 🔎 Search
- `POST /collections/{name}/search` - Search vectors
- `POST /collections/{name}/search/text` - Search by text

### 🧠 Intelligent Search (New in v0.3.1)
- `POST /intelligent_search` - Advanced intelligent search with multi-query generation
- `POST /semantic_search` - High-precision semantic search with reranking
- `POST /multi_collection_search` - Cross-collection search with intelligent reranking
- `POST /contextual_search` - Context-aware search with metadata filtering

### 📦 Batch Operations
- `POST /collections/{name}/batch/insert` - Batch insertion
- `POST /collections/{name}/batch/update` - Batch update
- `POST /collections/{name}/batch/delete` - Batch deletion
- `POST /collections/{name}/batch/search` - Batch search

### 🧠 Embedding
- `GET /embedding/providers` - List providers
- `POST /embedding/providers/set` - Set provider

### 📊 Indexing
- `GET /indexing/progress` - Indexing progress

### 📝 Summarization
- `POST /summarize/text` - Summarize text
- `GET /summaries` - List summaries
- `GET /summaries/{id}` - Get specific summary

### 🕸️ Graph Operations
- `GET /graph/nodes/{collection}` - List all nodes in a collection
- `GET /graph/nodes/{collection}/{node_id}/neighbors` - Get neighbors of a node
- `POST /graph/nodes/{collection}/{node_id}/related` - Find related nodes
- `POST /graph/path` - Find shortest path between nodes
- `POST /graph/edges` - Create an edge
- `DELETE /graph/edges/{edge_id}` - Delete an edge
- `GET /graph/collections/{collection}/edges` - List all edges in a collection
- `POST /graph/discover/{collection}` - Discover SIMILAR_TO edges for entire collection
- `POST /graph/discover/{collection}/{node_id}` - Discover SIMILAR_TO edges for a specific node
- `GET /graph/discover/{collection}/status` - Get discovery status and statistics

**📖 See [Graph API Documentation](./GRAPH.md) for detailed documentation and examples.**

## 🎯 Usage Examples

### Create Collection
```bash
curl -X POST "http://localhost:15002/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-collection",
    "dimension": 512,
    "metric": "cosine"
  }'
```

### Insert Texts
```bash
curl -X POST "http://localhost:15002/collections/my-collection/vectors" \
  -H "Content-Type: application/json" \
  -d '{
    "texts": [
      {
        "id": "doc1",
        "text": "This is an example text to index",
        "metadata": {"source": "example"}
      }
    ]
  }'
```

### Search by Text
```bash
curl -X POST "http://localhost:15002/collections/my-collection/search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "example search",
    "limit": 10,
    "score_threshold": 0.1
  }'
```

### Intelligent Search (New in v0.3.1)
```bash
# Advanced intelligent search with domain expansion
curl -X POST "http://localhost:15002/intelligent_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "CMMV framework architecture",
    "collections": ["cmmv-core-docs"],
    "max_results": 10,
    "domain_expansion": true,
    "technical_focus": true,
    "mmr_enabled": true,
    "mmr_lambda": 0.7
  }'

# High-precision semantic search
curl -X POST "http://localhost:15002/semantic_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "authentication system",
    "collection": "cmmv-core-docs",
    "similarity_threshold": 0.15,
    "semantic_reranking": true,
    "max_results": 10
  }'

# Multi-collection search
curl -X POST "http://localhost:15002/multi_collection_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "API documentation",
    "collections": ["cmmv-core-docs", "vectorizer-docs"],
    "max_per_collection": 5,
    "max_total_results": 15,
    "cross_collection_reranking": true
  }'

# Context-aware search
curl -X POST "http://localhost:15002/contextual_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "database integration",
    "collection": "cmmv-core-docs",
    "context_filters": {
      "file_extension": ".md",
      "chunk_index": 0
    },
    "context_reranking": true,
    "context_weight": 0.3,
    "max_results": 10
  }'
```

### Graph Edge Discovery
```bash
# Discover edges for entire collection
curl -X POST "http://localhost:15002/graph/discover/my-collection" \
  -H "Content-Type: application/json" \
  -d '{
    "similarity_threshold": 0.7,
    "max_per_node": 10
  }'

# Discover edges for a specific node
curl -X POST "http://localhost:15002/graph/discover/my-collection/node1" \
  -H "Content-Type: application/json" \
  -d '{
    "similarity_threshold": 0.7,
    "max_per_node": 10
  }'

# Get discovery status
curl "http://localhost:15002/graph/discover/my-collection/status"
```

### Health Check
```bash
curl "http://localhost:15002/health"
```

## 🔧 Configuration

### Local Server
- **Base URL**: `http://localhost:15002`
- **Port**: 15002
- **Version**: 1.4.0

### Authentication
Currently no authentication is implemented. For production, consider implementing:
- API Keys
- JWT Tokens
- OAuth 2.0

## 📖 Technical Specifications

### Supported Formats
- **Input**: JSON
- **Output**: JSON
- **Encoding**: UTF-8

### Distance Metrics
- `cosine` - Cosine similarity
- `euclidean` - Euclidean distance
- `dot_product` - Dot product

### Embedding Providers
- `bm25` - BM25 (default)
- `tfidf` - TF-IDF
- `bert` - BERT
- `minilm` - MiniLM
- `bagofwords` - Bag of Words
- `charngram` - Character N-grams

### Summarization Methods
- `extractive` - Extractive (default)
- `keyword` - By keywords
- `sentence` - By sentences
- `abstractive` - Abstractive

## 🐛 Troubleshooting

### Error 404 - Collection Not Found
```bash
# Check existing collections
curl "http://localhost:15002/collections"
```

### Error 400 - Bad Request
- Check JSON format
- Validate required parameters
- Verify data types

### Error 500 - Internal Server Error
- Check server logs
- Confirm Vectorizer is running
- Check system resources

## 📝 Updates

This schema is automatically updated when:
- New endpoints are added
- Data structures are modified
- New parameters are included

To contribute with documentation improvements, see [CONTRIBUTING.md](../../CONTRIBUTING.md).

## 🔗 Useful Links

- [OpenAPI Specification](https://swagger.io/specification/)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [Redoc](https://redoc.ly/)
- [OpenAPI Generator](https://openapi-generator.tech/)
- [Swagger Codegen](https://swagger.io/tools/swagger-codegen/)

## 📄 License

This project is licensed under the [Apache-2.0 License](../../LICENSE).
