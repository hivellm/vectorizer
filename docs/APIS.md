# Vectorizer APIs and Interfaces

## Interface Overview

Vectorizer provides multiple interfaces for interaction, each optimized for different use cases and execution environments.

### Available Interfaces

| Interface | Type | Description | Use Cases |
|-----------|------|-------------|-----------|
| **REST API** | Server API | Direct HTTP/JSON access to server | Custom integrations, direct API calls |
| **gRPC API** | Server API | Direct Protocol Buffers access to server | High performance, distributed systems |
| **Python SDK** | Client SDK | Python client connecting to server | Data Science, ML pipelines |
| **TypeScript SDK** | Client SDK | TypeScript/JavaScript client connecting to server | Web applications, Node.js |
| **CLI Tools** | Client Tools | Command-line client tools | Administration, scripts |

## Architecture Overview

Vectorizer follows a **client-server architecture** with **mandatory authentication** where:

- **Server**: Centralized Rust application providing REST/gRPC APIs with native vector processing, API key management, and local dashboard
- **SDKs**: Lightweight clients (Python, TypeScript) that connect to the server via HTTP/gRPC with mandatory API keys
- **Security**: All operations require valid API keys; dashboard provides key management (localhost only)
- **Network**: Configurable for internal network or cloud deployment
- **Benefits**: Single codebase maintenance, centralized processing, secure multi-language SDK support

## REST API

### Base URL and Authentication

```
Base URL: http://your-server:15001/api/v1
Authentication: Bearer token (REQUIRED - API Key)
Content-Type: application/json
Authorization: Bearer your-api-key-here
```

**Note**: API keys are mandatory for all operations. SDKs automatically include API keys in requests.

## API Keys & Dashboard

### Getting API Keys

**Option 1: CLI Tool**
```bash
# Generate a new API key
vectorizer api-keys create --name "my-app" --description "Production application"

# List all API keys
vectorizer api-keys list

# Delete an API key
vectorizer api-keys delete <key-id>
```

**Option 2: Local Dashboard**
```
URL: http://localhost:3000/dashboard (accessible only from localhost)
```

The dashboard provides a web interface for:
- Creating new API keys with custom names and descriptions
- Viewing all active API keys (masked for security)
- Deleting API keys
- Monitoring server statistics and usage

### Dashboard Features

- **API Key Management**: Create, view, and delete API keys
- **Server Statistics**: Real-time metrics on collections, vectors, and performance
- **Security**: Only accessible from localhost (127.0.0.1)
- **No Authentication**: Dashboard itself doesn't require API keys for local access

### Using API Keys

All SDKs require API keys to be specified during initialization:

**Python:**
```python
client = VectorizerClient(
    host="localhost",
    port=15001,
    api_key="your-api-key-here"  # REQUIRED
)
```

**TypeScript:**
```typescript
const client = new VectorizerClient({
  host: 'localhost',
  port: 15001,
  apiKey: 'your-api-key-here'  // REQUIRED
});
```

### Main Endpoints

#### Collections

##### Create Collection
```http
POST /collections
Content-Type: application/json

{
  "name": "documents",
  "config": {
    "dimension": 768,
    "metric": "cosine",
    "quantization": {
      "type": "pq",
      "n_centroids": 256,
      "n_subquantizers": 8
    },
    "embedding": {
      "model": "native_bow",
      "vocab_size": 50000,
      "max_sequence_length": 512
    },
    "index": {
      "type": "hnsw",
      "m": 16,
      "ef_construction": 200
    },
    "compression": {
      "enabled": true,
      "threshold_bytes": 1024,
      "algorithm": "lz4"
    }
  }
}
```

**Quantization Options:**
- `"none"`: No quantization (default)
- `"pq"`: Product Quantization - reduces memory by ~75%
- `"sq"`: Scalar Quantization - reduces memory by ~50%
- `"binary"`: Binary quantization - extreme compression (32x reduction)

**Native Embedding Models:**
- `"native_bow"`: Bag-of-Words with TF-IDF weighting
- `"native_hash"`: Feature hashing for fixed-size vectors
- `"native_ngram"`: N-gram features with dimensionality reduction

**Compression Options:**
- `"enabled"`: `true/false` - Enable automatic payload compression
- `"threshold_bytes"`: `number` - Compress payloads larger than this size (default: 1024)
- `"algorithm"`: `"lz4"` - Compression algorithm (LZ4 for speed, default: "lz4")

**Response (201 Created):**
```json
{
  "collection": {
    "name": "documents",
    "created_at": "2024-01-01T00:00:00Z",
    "config": { ... }
  }
}
```

##### List Collections
```http
GET /collections
```

**Response (200 OK):**
```json
{
  "collections": [
    {
      "name": "documents",
      "vector_count": 1000,
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

##### Get Collection Statistics
```http
GET /collections/{name}/stats
```

**Response (200 OK):**
```json
{
  "collection": "documents",
  "stats": {
    "vector_count": 1000,
    "dimension": 768,
    "index_size_bytes": 1048576,
    "avg_query_time_ms": 0.8,
    "total_queries": 50000
  }
}
```

#### Vectors

##### Insert Vectors
```http
POST /collections/{name}/vectors
Content-Type: application/json

{
  "vectors": [
    {
      "id": "vec_001",
      "data": [0.1, 0.2, 0.3, ...],
      "payload": {
        "text": "Sample document text",
        "metadata": {
          "source": "file.txt",
          "timestamp": "2024-01-01T00:00:00Z"
        }
      }
    }
  ]
}
```

##### Similarity Search
```http
POST /collections/{name}/search
Content-Type: application/json

{
  "query": {
    "type": "vector",
    "data": [0.1, 0.2, 0.3, ...]
  },
  "k": 10,
  "filter": {
    "metadata.source": "file.txt"
  },
  "include_vector": false
}
```

**Response (200 OK):**
```json
{
  "results": [
    {
      "id": "vec_001",
      "score": 0.95,
      "payload": {
        "text": "Sample document text",
        "metadata": { "source": "file.txt" }
      }
    }
  ],
  "query_time_ms": 0.8
}
```

##### Text Search (with Native Embedding)
```http
POST /collections/{name}/search
Content-Type: application/json

{
  "query": {
    "type": "text",
    "text": "machine learning algorithms",
    "model": "native_bow"
  },
  "k": 5
}
```

#### Insert Text with Automatic Embedding
```http
POST /collections/{name}/documents
Content-Type: application/json

{
  "documents": [
    {
      "id": "doc_001",
      "text": "Machine learning is a method of data analysis...",
      "metadata": {
        "source": "ml_guide.pdf",
        "chapter": 1
      }
    }
  ],
  "embedding": {
    "model": "native_bow",
    "vocab_size": 50000,
    "chunk_size": 512,
    "chunk_overlap": 50
  }
}
```

**Response (200 OK):**
```json
{
  "inserted": 1,
  "chunks_created": 3,
  "vectors_stored": 3,
  "embedding_model": "native_bow",
  "quantization_applied": "pq"
}
```

##### Update Vectors
```http
PATCH /collections/{name}/vectors/{id}
Content-Type: application/json

{
  "payload": {
    "text": "Updated document text",
    "metadata": {
      "last_modified": "2024-01-02T00:00:00Z"
    }
  }
}
```

##### Delete Vectors
```http
DELETE /collections/{name}/vectors/{id}
```

#### Batch Operations

##### Batch Insertion
```http
POST /collections/{name}/batch/insert
Content-Type: application/json

{
  "vectors": [...],
  "batch_size": 1000
}
```

##### Batch Search
```http
POST /collections/{name}/batch/search
Content-Type: application/json

{
  "queries": [
    { "type": "vector", "data": [...] },
    { "type": "text", "text": "..." }
  ],
  "k": 5
}
```

## gRPC API

### Definição do Serviço (Protocol Buffers)

```protobuf
syntax = "proto3";

package vectorizer.v1;

service VectorService {
  rpc CreateCollection(CreateCollectionRequest) returns (CreateCollectionResponse);
  rpc ListCollections(ListCollectionsRequest) returns (ListCollectionsResponse);
  rpc GetCollectionStats(GetCollectionStatsRequest) returns (GetCollectionStatsResponse);

  rpc InsertVectors(InsertVectorsRequest) returns (InsertVectorsResponse);
  rpc SearchVectors(SearchVectorsRequest) returns (SearchVectorsResponse);
  rpc UpdateVectors(UpdateVectorsRequest) returns (UpdateVectorsResponse);
  rpc DeleteVectors(DeleteVectorsRequest) returns (DeleteVectorsResponse);

  rpc BatchInsert(BatchInsertRequest) returns (BatchInsertResponse);
  rpc BatchSearch(BatchSearchRequest) returns (BatchSearchResponse);
}

// Messages
message Vector {
  string id = 1;
  repeated float data = 2;
  google.protobuf.Struct payload = 3;
}

message CollectionConfig {
  uint32 dimension = 1;
  enum MetricType {
    COSINE = 0;
    DOT_PRODUCT = 1;
    EUCLIDEAN = 2;
  }
  MetricType metric = 2;

  message HNSWConfig {
    uint32 m = 1;
    uint32 ef_construction = 2;
    uint32 ef_search = 3;
  }
  HNSWConfig hnsw_config = 3;
}

message SearchRequest {
  oneof query {
    repeated float vector = 1;
    string text = 2;
  }
  uint32 k = 3;
  map<string, google.protobuf.Value> filter = 4;
  bool include_vector = 5;
}

message SearchResult {
  string id = 1;
  float score = 2;
  google.protobuf.Struct payload = 3;
  repeated float vector = 4;
}
```

### Exemplo de Uso (Go)

```go
conn, err := grpc.Dial("localhost:8081", grpc.WithInsecure())
if err != nil {
    log.Fatal(err)
}
defer conn.Close()

client := pb.NewVectorServiceClient(conn)

// Insert vectors
vectors := []*pb.Vector{
    {
        Id: "vec_001",
        Data: []float32{0.1, 0.2, 0.3},
        Payload: &structpb.Struct{
            Fields: map[string]*structpb.Value{
                "text": structpb.NewStringValue("sample text"),
            },
        },
    },
}

resp, err := client.InsertVectors(context.Background(), &pb.InsertVectorsRequest{
    Collection: "documents",
    Vectors:    vectors,
})
```

## Python SDK (Client)

### Installation and Initialization

```bash
pip install @hivellm/vectorizer
```

```python
from vectorizer import VectorizerClient

# Connect to Vectorizer server (API key is REQUIRED)
client = VectorizerClient(
    host="localhost",
    port=15001,
    api_key="your-api-key-here"  # REQUIRED for all operations
)
```

### Main API

#### Collection Management

```python
# Create collection on server with compression
client.create_collection(
    name="documents",
    dimension=768,
    metric="cosine",
    quantization={"type": "pq", "n_centroids": 256, "n_subquantizers": 8},
    embedding={"model": "native_bow", "vocab_size": 50000},
    compression={"enabled": True, "threshold_bytes": 1024, "algorithm": "lz4"}
)

# List collections on server
collections = client.list_collections()
print(f"Collections: {collections}")

# Get collection statistics from server
stats = client.get_collection_stats("documents")
print(f"Vectors: {stats['vector_count']}")
```

#### Vector Operations

```python
# Insert vectors (vectors are processed and stored on server)
vectors = [
    [0.1, 0.2, 0.3, ...],  # 768 dimensions
    [0.4, 0.5, 0.6, ...],
]

payloads = [
    {"text": "First document", "source": "doc1.txt"},
    {"text": "Second document", "source": "doc2.txt"},
]

client.insert_vectors(
    collection="documents",
    ids=["doc1", "doc2"],
    vectors=vectors,
    payloads=payloads
)

# Similarity search (executed on server)
results = client.search(
    collection="documents",
    query_vector=[0.1, 0.2, 0.3, ...],
    k=5
)

for result in results:
    print(f"ID: {result['id']}, Score: {result['score']}")
    print(f"Text: {result['payload']['text']}")
```

#### Document Operations (Server-processed)

```python
# Insert documents (server handles embedding and chunking)
documents = [
    {
        "id": "doc_001",
        "text": "Machine learning is a subset of artificial intelligence...",
        "metadata": {"source": "ml_guide.pdf", "page": 1}
    },
    {
        "id": "doc_002",
        "text": "Deep learning uses neural networks with multiple layers...",
        "metadata": {"source": "ml_guide.pdf", "page": 2}
    }
]

# Server processes documents: chunks, embeds, quantizes, and stores
client.insert_documents(
    collection="documents",
    documents=documents,
    chunk_size=512,
    chunk_overlap=50
)
print("Documents processed and stored on server")
```

#### Text Search

```python
# Semantic search (server handles embedding of query text)
results = client.search_by_text(
    collection="documents",
    query_text="machine learning algorithms",
    k=5
)

for result in results:
    print(f"ID: {result['id']}, Score: {result['score']:.3f}")
    print(f"Text: {result['payload']['text'][:100]}...")
    print("---")
```

#### Batch Operations

```python
# Batch insertion (server processes all vectors)
batch_data = {
    "ids": ["id1", "id2", "id3"],
    "vectors": [vec1, vec2, vec3],
    "payloads": [payload1, payload2, payload3]
}

client.batch_insert("documents", batch_data)

# Batch search (server executes all queries in parallel)
queries = [
    [0.1, 0.2, 0.3, ...],
    [0.4, 0.5, 0.6, ...],
]

batch_results = client.batch_search(
    collection="documents",
    query_vectors=queries,
    k=3
)
```

## TypeScript SDK (Client)

### Installation and Initialization

```bash
npm install @hivellm/vectorizer
```

```typescript
import { VectorizerClient } from '@hivellm/vectorizer';

// Connect to Vectorizer server (API key is REQUIRED)
const client = new VectorizerClient({
  host: 'localhost',
  port: 15001,
  apiKey: 'your-api-key-here'  // REQUIRED for all operations
});
```

### Main API

#### Collection Management

```typescript
// Create collection on server with compression
await client.createCollection('documents', {
  dimension: 768,
  metric: 'cosine' as const,
  quantization: { type: 'pq', nCentroids: 256, nSubquantizers: 8 },
  embedding: { model: 'native_bow', vocabSize: 50000 },
  compression: { enabled: true, thresholdBytes: 1024, algorithm: 'lz4' }
});

// List collections on server
const collections = await client.listCollections();
console.log('Collections:', collections);

// Get collection statistics from server
const stats = await client.getCollectionStats('documents');
console.log(`Vectors: ${stats.vectorCount}`);
```

#### Vector Operations

```typescript
// Insert vectors (processed and stored on server)
const vectors = [
  [0.1, 0.2, 0.3, ...],  // 768 dimensions
  [0.4, 0.5, 0.6, ...],
];

const payloads = [
  { text: 'First document', source: 'doc1.txt' },
  { text: 'Second document', source: 'doc2.txt' },
];

await client.insertVectors('documents', ['doc1', 'doc2'], vectors, payloads);

// Similarity search (executed on server)
const results = await client.search('documents', [0.1, 0.2, 0.3, ...], 5);

results.forEach(result => {
  console.log(`ID: ${result.id}, Score: ${result.score}`);
  console.log(`Text: ${result.payload.text}`);
});
```

#### Document Operations (Server-processed)

```typescript
// Insert documents (server handles embedding and chunking)
const documents = [
  {
    id: 'doc_001',
    text: 'Machine learning is a subset of artificial intelligence...',
    metadata: { source: 'ml_guide.pdf', page: 1 }
  },
  {
    id: 'doc_002',
    text: 'Deep learning uses neural networks with multiple layers...',
    metadata: { source: 'ml_guide.pdf', page: 2 }
  }
];

// Server processes documents: chunks, embeds, quantizes, and stores
await client.insertDocuments('documents', documents, {
  chunkSize: 512,
  chunkOverlap: 50
});

console.log('Documents processed and stored on server');
```

#### Text Search

```typescript
// Semantic search (server handles embedding of query text)
const results = await client.searchByText(
  'documents',
  'machine learning algorithms',
  5
);

results.forEach(result => {
  console.log(`ID: ${result.id}, Score: ${result.score.toFixed(3)}`);
  console.log(`Text: ${result.payload.text?.substring(0, 100)}...`);
  console.log('---');
});
```

## Server Configuration

### Network Settings

The Vectorizer server can be configured for different deployment scenarios:

#### Internal Network (Default)
```bash
# Run server accessible only on localhost
vectorizer server --host 127.0.0.1 --port 15001

# Or with config file
vectorizer server --config config/internal.toml
```

**internal.toml:**
```toml
[server]
host = "127.0.0.1"  # localhost only
port = 15001
enable_dashboard = true  # dashboard available at http://localhost:3000

[security]
require_api_keys = true
dashboard_localhost_only = true

[network]
mode = "internal"  # internal network only
```

#### Cloud Deployment
```bash
# Run server accessible from external networks
vectorizer server --host 0.0.0.0 --port 15001

# Or with config file
vectorizer server --config config/cloud.toml
```

**cloud.toml:**
```toml
[server]
host = "0.0.0.0"  # accessible from external networks
port = 15001
enable_dashboard = false  # dashboard disabled for security

[security]
require_api_keys = true
dashboard_localhost_only = true  # even if disabled, this is enforced

[network]
mode = "cloud"  # external access enabled
allowed_origins = ["https://your-app.com"]  # CORS settings
rate_limit_requests_per_minute = 1000
```

### Dashboard Access

- **Internal Mode**: Dashboard available at `http://localhost:3000`
- **Cloud Mode**: Dashboard disabled for security
- **Security**: Dashboard always restricted to localhost access only

## CLI Tools

### Installation

```bash
cargo install vectorizer-cli
# or
npm install -g vectorizer-cli
```

### Main Commands

#### API Key Management

```bash
# Create a new API key
vectorizer api-keys create \
  --name "production-app" \
  --description "Production application key"

# List all API keys
vectorizer api-keys list

# Delete an API key
vectorizer api-keys delete <api-key-id>

# Get API key details
vectorizer api-keys info <api-key-id>
```

#### File Ingestion

```bash
# Basic ingestion
vectorizer ingest \
  --file document.txt \
  --collection documents \
  --chunk-size 512

# With specific model
vectorizer ingest \
  --file document.pdf \
  --collection docs \
  --chunk-strategy recursive \
  --chunk-size 1000 \
  --chunk-overlap 200

# Multiple files ingestion
vectorizer ingest \
  --dir ./documents/ \
  --pattern "*.txt" \
  --collection knowledge_base \
  --recursive
```

#### Queries

```bash
# Text search
vectorizer query \
  --collection documents \
  --text "machine learning algorithms" \
  --k 10 \
  --format json

# Search by ID
vectorizer get \
  --collection documents \
  --id vec_001 \
  --include-vector

# Search with filters
vectorizer query \
  --collection documents \
  --text "AI models" \
  --filter "metadata.source:research.pdf" \
  --k 5
```

#### Management

```bash
# List collections
vectorizer collections

# Collection statistics
vectorizer stats --collection documents

# Export collection
vectorizer export \
  --collection documents \
  --format json \
  --output documents.json

# Import collection
vectorizer import \
  --file documents.json \
  --collection documents

# Optimize collection
vectorizer optimize --collection documents

# Backup
vectorizer backup --collection documents --output backup.bin
```

#### Configuração

```bash
# Show configuration
vectorizer config

# Configure server
vectorizer config set \
  server.host 0.0.0.0 \
  server.port 15001

# Configure default embedding model
vectorizer config set \
  embedding.model native_bow \
  embedding.dimension 768
```

### Formatos de Saída

#### JSON (padrão)
```json
{
  "results": [
    {
      "id": "vec_001",
      "score": 0.95,
      "payload": {
        "text": "Machine learning algorithms...",
        "metadata": { "source": "doc1.txt" }
      }
    }
  ],
  "query_time_ms": 0.8,
  "total_results": 1
}
```

#### Tabela (legível)
```
ID       Score    Text Preview
vec_001  0.950    Machine learning algorithms are computational methods...
vec_005  0.890    Deep learning models use neural networks...
vec_012  0.870    Artificial intelligence systems learn from data...

Query time: 0.8ms, Total results: 3
```

## Tratamento de Erros

### Códigos de Erro Comuns

| Código | Descrição | Solução |
|--------|-----------|---------|
| `COLLECTION_NOT_FOUND` | Collection does not exist | Create collection first |
| `INVALID_DIMENSION` | Incorrect vector dimension | Check collection configuration |
| `DUPLICATE_ID` | Vector ID already exists | Use unique ID |
| `RATE_LIMIT_EXCEEDED` | Rate limit exceeded | Implement backoff |
| `STORAGE_FULL` | Storage full | Clean old vectors or increase capacity |

### Exemplo de Tratamento de Erro

**Python:**
```python
from vectorizer import VectorizerError

try:
    db.insert("documents", ids, vectors, payloads)
except VectorizerError as e:
    if e.code == "INVALID_DIMENSION":
        print(f"Dimensão incorreta: esperada {e.expected}, recebida {e.got}")
    elif e.code == "COLLECTION_NOT_FOUND":
        db.create_collection("documents", dimension=768)
        db.insert("documents", ids, vectors, payloads)
```

**TypeScript:**
```typescript
import { VectorizerError } from '@hivellm/vectorizer';

try {
  await db.insert('documents', ids, vectors, payloads);
} catch (error) {
  if (error instanceof VectorizerError) {
    switch (error.code) {
      case 'INVALID_DIMENSION':
        console.error(`Invalid dimension: expected ${error.expected}, got ${error.got}`);
        break;
      case 'COLLECTION_NOT_FOUND':
        await db.createCollection('documents', { dimension: 768 });
        await db.insert('documents', ids, vectors, payloads);
        break;
    }
  }
}
```

## Limites e Restrições

### Limites Recomendados

| Recurso | Limite | Recomendação |
|---------|--------|--------------|
| Vector Dimension | ≤ 2048 | 384-768 for text |
| Vectors per Collection | ≤ 10M | Sharding for >10M |
| Payload Size | ≤ 1MB | Compress large payloads |
| Queries per Second | ≤ 1000 | Implement cache |
| Concurrent Connections | ≤ 100 | Load balancer for >100 |

### Configurações de Performance

```json
{
  "server": {
    "workers": 4,
    "max_connections": 1000,
    "timeout_seconds": 30
  },
  "index": {
    "hnsw": {
      "m": 16,
      "ef_construction": 200,
      "ef_search": 64
    }
  },
  "cache": {
    "enabled": true,
    "size_mb": 512,
    "ttl_seconds": 3600
  }
}
```

---

This documentation provides a comprehensive overview of the available interfaces in Vectorizer, with practical examples for each SDK and supported protocol.
