# Vectorizer API Documentation

This directory contains the complete API documentation for Vectorizer, including the OpenAPI 3.0.3 schema.

## 📁 Files

- **`openapi.yaml`** - Complete OpenAPI 3.0.3 schema for Vectorizer API
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

### Health Check
```bash
curl "http://localhost:15002/health"
```

## 🔧 Configuration

### Local Server
- **Base URL**: `http://localhost:15002`
- **Port**: 15002
- **Version**: 0.3.0

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

This project is licensed under the [MIT License](../../LICENSE).
