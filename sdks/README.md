# Hive Vectorizer Client SDKs

High-performance client SDKs for the Hive Vectorizer vector database, available in multiple languages.

## Available SDKs

### 🟦 TypeScript SDK ✅

- **Package**: `@hivehub/vectorizer-sdk`
- **Status**: Published on npm
- **Features**: Full TypeScript support, async/await, comprehensive type safety, intelligent search, Master/Replica routing. Ships compiled CommonJS and works from plain JavaScript projects too.
- **Installation**: `npm install @hivehub/vectorizer-sdk`
- **Documentation**: [TypeScript SDK README](./typescript/README.md)

### 🦀 Rust SDK ✅

- **Package**: `vectorizer-sdk`
- **Status**: Published on crates.io (v1.8.0)
- **Features**: High performance, async/await, MCP support, type safety, intelligent search, Master/Replica routing
- **Installation**: Add to `Cargo.toml`: `vectorizer-sdk = "1.8.0"`
- **Documentation**: [Rust SDK README](./rust/README.md)

### 🐍 Python SDK ✅

- **Package**: `vectorizer-sdk`
- **Status**: Published on PyPI (v1.8.0)
- **Features**: Async/await support, comprehensive testing, CLI interface, intelligent search, Master/Replica routing
- **Installation**: `pip install vectorizer-sdk==1.8.0`
- **Documentation**: [Python SDK README](./python/README.md)

### 🐹 Go SDK 🚧

- **Package**: `github.com/hivellm/vectorizer-sdk-go`
- **Status**: In Development (v1.8.0)
- **Features**: High performance, simple API, comprehensive error handling, intelligent search, Master/Replica routing
- **Installation**: `go get github.com/hivellm/vectorizer-sdk-go`
- **Repository**: https://github.com/hivellm/vectorizer/tree/main/sdks/go
- **Documentation**: [Go SDK README](./go/README.md)

### 🔷 C# SDK ✅

- **Package**: `Vectorizer.Sdk`
- **Status**: Published on NuGet (v1.8.0)
- **Features**: Async/await support, .NET 8.0+, type-safe models, intelligent search, SourceLink, Master/Replica routing
- **Installation**: `dotnet add package Vectorizer.Sdk`
- **NuGet**: https://www.nuget.org/packages/Vectorizer.Sdk
- **Documentation**: [C# SDK README](./csharp/README.md)

### Removed framework integrations

LangChain (Python + JS), Langflow, n8n, TensorFlow, and PyTorch
integrations were dropped in v3.0.0. They were thin adapters over the
core SDK and added support burden out of proportion to their usage.
Build directly against the language-native SDKs (TypeScript, Python,
Rust, Go, C#) instead — every operation those integrations exposed is
one call away on the corresponding SDK.

### Removed standalone JavaScript SDK

The standalone `@hivehub/vectorizer-sdk-js` package was dropped in
v3.0.0. The TypeScript SDK ships compiled CommonJS + ESM and is fully
usable from plain JavaScript — keeping two parallel packages doubled
maintenance for no functional difference. Replace `@hivehub/vectorizer-sdk-js`
with `@hivehub/vectorizer-sdk`; the import path and runtime API are
identical.

## 🧠 Intelligent Search Features (v1.5.0+)

All SDKs now support advanced intelligent search capabilities:

### 🔍 Intelligent Search

- **Multi-query expansion**: Automatically generates multiple search queries
- **Domain knowledge**: Technology-specific term expansion
- **MMR diversification**: Ensures diverse, high-quality results
- **Technical focus**: Prioritizes technical content and API documentation

### 🎯 Semantic Search

- **Advanced reranking**: Multi-factor scoring system
- **Similarity thresholds**: Configurable relevance filtering
- **Cross-encoder support**: Optional neural reranking

### 🎪 Contextual Search

- **Metadata filtering**: Search within specific contexts
- **Context-aware reranking**: Considers metadata relevance
- **Weighted scoring**: Balance between semantic and contextual factors

### 🔗 Multi-Collection Search

- **Cross-collection search**: Search across multiple collections simultaneously
- **Intelligent aggregation**: Unified ranking across collections
- **Collection-specific limits**: Control results per collection

### Example Usage

```typescript
// Intelligent search with domain expansion
const results = await client.intelligentSearch({
  query: "machine learning algorithms",
  collections: ["docs", "research"],
  max_results: 15,
  domain_expansion: true,
  technical_focus: true,
  mmr_enabled: true,
  mmr_lambda: 0.7,
});

// Contextual search with metadata filtering
const contextualResults = await client.contextualSearch({
  query: "deep learning",
  collection: "docs",
  context_filters: {
    category: "AI",
    language: "en",
    year: 2023,
  },
  max_results: 10,
  context_weight: 0.4,
});
```

## Quick Start

### TypeScript / JavaScript

```typescript
import { VectorizerClient } from "@hivehub/vectorizer-sdk";

const client = new VectorizerClient({
  baseURL: "http://localhost:15001",
  apiKey: "your-api-key",
});

// Create collection
await client.createCollection({
  name: "documents",
  dimension: 768,
  similarity_metric: "cosine",
});

// Insert texts
await client.insertTexts("documents", [
  {
    id: "doc_1",
    text: "This is a sample document about machine learning",
    metadata: { source: "document.pdf", category: "AI" },
  },
]);

// Search
const results = await client.searchVectors("documents", {
  query_vector: [0.1, 0.2, 0.3 /* ... 768 dimensions */],
  limit: 5,
});
```

### Python

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    base_url="http://localhost:15001",
    api_key="your-api-key"
)

# Create collection
await client.create_collection(
    name="documents",
    dimension=768,
    metric="cosine"
)

# Insert texts
texts = [{
    "id": "doc_1",
    "text": "This is a sample document about machine learning",
    "metadata": {"source": "document.pdf", "category": "AI"}
}]
await client.insert_texts("documents", texts)

# Search
results = await client.search_vectors(
    collection="documents",
    query_vector=[0.1, 0.2, 0.3, ...],
    limit=5
)
```

### Rust

```rust
use vectorizer_sdk::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = VectorizerClient::new_default()?;

    // Create collection
    client.create_collection("documents", 768, Some(SimilarityMetric::Cosine)).await?;

    // Insert texts
    let texts = vec![BatchTextRequest {
        id: "doc_1".to_string(),
        text: "This is a sample document about machine learning".to_string(),
        metadata: Some({
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "document.pdf".to_string());
            meta.insert("category".to_string(), "AI".to_string());
            meta
        }),
    }];

    client.insert_texts("documents", texts).await?;

    // Search
    let results = client.search_vectors("documents", "machine learning", Some(5), None).await?;
    println!("Found {} results", results.results.len());

    Ok(())
}
```

**Note**: Add to `Cargo.toml`:

```toml
[dependencies]
vectorizer-sdk = "1.8.0"
```

## SDK Comparison Table

| Feature                     | TypeScript   | Rust         | Python       | Go         | C#           |
| --------------------------- | ------------ | ------------ | ------------ | ---------- | ------------ |
| **Status**                  | ✅ Published | ✅ Published | ✅ Published | 🚧 Dev     | ✅ Published |
| **Master/Replica Routing**  | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Package Manager**         | npm          | crates.io    | PyPI         | Go Modules | NuGet        |
| **Collection Management**   | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Vector Operations**       | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Text Search**             | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Vector Search**           | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Intelligent Search**      | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Semantic Search**         | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Contextual Search**       | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Multi-Collection Search** | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Hybrid Search**           | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Discovery API**           | ✅           | ✅           | ✅           | 🚧         | ✅           |
| **File Operations**         | ✅           | ✅           | ✅           | 🚧         | ✅           |
| **Summarization**           | ✅           | ✅           | ✅           | 🚧         | ✅           |
| **Embedding Generation**    | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Batch Insert**            | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Batch Search**            | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Batch Update**            | ✅           | ✅           | ✅           | 🚧         | ✅           |
| **Batch Delete**            | ✅           | ✅           | ✅           | 🚧         | ✅           |
| **Qdrant Compatibility**    | ✅           | ✅           | ✅           | 🚧         | 🚧           |
| **Async/Await**             | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Type Safety**             | ✅           | ✅           | ✅           | ✅         | ✅           |
| **Error Handling**          | ✅           | ✅           | ✅           | ✅         | ✅           |
| **SourceLink**              | ❌           | ✅           | ❌           | ❌         | ✅           |
| **Code Analysis**           | ❌           | ✅           | ❌           | ❌         | ✅           |
| **Documentation**           | ✅           | ✅           | ✅           | ✅         | ✅           |

## Features

All SDKs provide:

- ✅ **Collection Management**: Create, read, update, delete collections
- ✅ **Vector Operations**: Insert, search, update, delete vectors
- ✅ **Semantic Search**: Text and vector similarity search
- ✅ **Embedding Generation**: Text embedding support
- ✅ **Batch Operations**: High-performance batch processing
- ✅ **REST-Only Architecture**: Pure HTTP REST API communication
- ✅ **Authentication**: API key-based authentication
- ✅ **Error Handling**: Comprehensive exception handling
- ✅ **Logging**: Configurable logging system
- ✅ **Validation**: Input validation and type checking
- ✅ **100% Test Coverage**: Comprehensive test suites for all SDKs

## Batch Operations

All SDKs support high-performance batch operations for efficient processing of large datasets:

### Batch Insert Texts

```typescript
// TypeScript/JavaScript
const batchResult = await client.batchInsertTexts("documents", {
  texts: [
    {
      id: "doc1",
      text: "Machine learning algorithms",
      metadata: { category: "AI" },
    },
    {
      id: "doc2",
      text: "Deep learning neural networks",
      metadata: { category: "AI" },
    },
    {
      id: "doc3",
      text: "Natural language processing",
      metadata: { category: "NLP" },
    },
  ],
  config: {
    provider: "bm25",
    max_batch_size: 100,
    parallel_workers: 4,
    atomic: true,
  },
});
```

```python
# Python
from vectorizer.models import BatchInsertRequest, BatchTextRequest, BatchConfig

batch_result = await client.batch_insert_texts('documents', BatchInsertRequest(
    texts=[
        BatchTextRequest(id='doc1', text='Machine learning algorithms', metadata={'category': 'AI'}),
        BatchTextRequest(id='doc2', text='Deep learning neural networks', metadata={'category': 'AI'}),
        BatchTextRequest(id='doc3', text='Natural language processing', metadata={'category': 'NLP'})
    ],
    config=BatchConfig(provider='bm25', max_batch_size=100, parallel_workers=4, atomic=True)
))
```

### Batch Search

```typescript
// TypeScript/JavaScript
const searchResult = await client.batchSearchVectors("documents", {
  queries: [
    { query: "machine learning", limit: 5 },
    { query: "neural networks", limit: 3 },
    { query: "NLP techniques", limit: 4 },
  ],
  config: { provider: "bm25", parallel_workers: 2 },
});
```

```python
# Python
from vectorizer.models import BatchSearchRequest, BatchSearchQuery

search_result = await client.batch_search_vectors('documents', BatchSearchRequest(
    queries=[
        BatchSearchQuery(query='machine learning', limit=5),
        BatchSearchQuery(query='neural networks', limit=3),
        BatchSearchQuery(query='NLP techniques', limit=4)
    ],
    config=BatchConfig(provider='bm25', parallel_workers=2)
))
```

### Batch Delete

```typescript
// TypeScript/JavaScript
const deleteResult = await client.batchDeleteVectors("documents", {
  vector_ids: ["doc1", "doc2", "doc3"],
  config: { atomic: true },
});
```

```python
# Python
from vectorizer.models import BatchDeleteRequest

delete_result = await client.batch_delete_vectors('documents', BatchDeleteRequest(
    vector_ids=['doc1', 'doc2', 'doc3'],
    config=BatchConfig(atomic=True)
))
```

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   TypeScript    │    │     Python      │    │      Rust        │
│      SDK        │    │      SDK        │    │      SDK         │
│                 │    │                 │    │                  │
│ • Type Safety   │    │ • Async/Await   │    │ • High Performance│
│ • IntelliSense  │    │ • CLI Interface │    │ • Memory Safety  │
│ • ES2020+       │    │ • Full Features │    │ • MCP Support    │
│ • Works from JS │    │                 │    │ • SourceLink     │
│                 │    │                 │    │                  │
│      C# SDK     │    │      Go SDK     │    │                  │
│  ✅ Published   │    │   🚧 In Dev     │    │                  │
│                 │    │                 │    │                  │
│ • .NET 8.0+     │    │ • High Perf     │    │                  │
│ • SourceLink    │    │ • Simple API    │    │                  │
│ • Code Analysis │    │ • Go Modules    │    │                  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐    ┌─────────────────┐
                    │  Vectorizer     │    │   MCP Server    │
                    │     Server      │    │                 │
                    │                 │    │ • Model Context │
                    │ • REST API      │    │ • AI Integration │
                    │ • MCP Protocol  │    │ • Tool Calling  │
                    │                 │    │ • SSE Transport │
                    └─────────────────┘    └─────────────────┘
```

## Master/Slave Replication (Read/Write Separation)

Vectorizer supports **Master-Replica replication** for high availability and read scaling. The SDK provides **automatic routing** - writes go to master, reads are distributed across replicas.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Your Application                                  │
│                                                                          │
│                    ┌─────────────────────────┐                          │
│                    │   VectorizerClient      │                          │
│                    │   (Single Interface)    │                          │
│                    └───────────┬─────────────┘                          │
│                                │                                         │
│              ┌─────────────────┴─────────────────┐                      │
│              │        Automatic Routing          │                      │
│              │  writes → master | reads → replica │                      │
│              └─────────────────┬─────────────────┘                      │
└────────────────────────────────┼─────────────────────────────────────────┘
                                 │
               ┌─────────────────┴─────────────────┐
               ▼                                   ▼
┌──────────────────────────┐    ┌──────────────────────────────────────────┐
│      Master Node         │    │           Replica Nodes                   │
│  ┌────────────────────┐  │    │  ┌──────────────┐  ┌──────────────┐     │
│  │ Writes: INSERT,    │  │───▶│  │  Replica 1   │  │  Replica 2   │     │
│  │ UPDATE, DELETE     │  │    │  │  (Read-Only) │  │  (Read-Only) │     │
│  │ CREATE COLLECTION  │  │    │  └──────────────┘  └──────────────┘     │
│  └────────────────────┘  │    │                                          │
└──────────────────────────┘    └──────────────────────────────────────────┘
```

### Quick Start - All Languages

Configure once, use everywhere. The SDK automatically routes operations:

```typescript
// TypeScript/JavaScript
const client = new VectorizerClient({
  hosts: {
    master: "http://master-node:15001",
    replicas: ["http://replica1:15001", "http://replica2:15001"],
  },
  apiKey: "your-api-key",
  readPreference: "replica", // "master" | "replica" | "nearest"
});

// Writes automatically go to master
await client.insertTexts("documents", [
  { id: "doc1", text: "Sample document", metadata: { source: "api" } },
]);

// Reads automatically go to replicas (load balanced)
const results = await client.searchVectors("documents", {
  query: "sample",
  limit: 10,
});

// Force read from master when you need read-your-writes consistency
const fresh = await client.getVector("documents", "doc1", {
  readPreference: "master",
});
```

```python
# Python
client = VectorizerClient(
    hosts={
        "master": "http://master-node:15001",
        "replicas": ["http://replica1:15001", "http://replica2:15001"]
    },
    api_key="your-api-key",
    read_preference="replica"  # "master" | "replica" | "nearest"
)

# Writes automatically go to master
await client.insert_texts("documents", [{
    "id": "doc1",
    "text": "Sample document",
    "metadata": {"source": "api"}
}])

# Reads automatically go to replicas (load balanced)
results = await client.search_vectors("documents", query="sample", limit=10)

# Force read from master when you need read-your-writes consistency
fresh = await client.get_vector("documents", "doc1", read_preference="master")
```

```rust
// Rust
let client = VectorizerClient::builder()
    .master("http://master-node:15001")
    .replica("http://replica1:15001")
    .replica("http://replica2:15001")
    .api_key("your-api-key")
    .read_preference(ReadPreference::Replica)
    .build()?;

// Writes automatically go to master
client.insert_texts("documents", vec![
    BatchTextRequest {
        id: "doc1".to_string(),
        text: "Sample document".to_string(),
        metadata: Some(metadata),
    }
]).await?;

// Reads automatically go to replicas (load balanced)
let results = client.search_vectors("documents", &query_vector, 10).await?;

// Force read from master
let fresh = client.get_vector_with_preference("documents", "doc1", ReadPreference::Master).await?;
```

```go
// Go
client := vectorizer.NewClient(&vectorizer.Config{
    Hosts: vectorizer.HostConfig{
        Master:   "http://master-node:15001",
        Replicas: []string{"http://replica1:15001", "http://replica2:15001"},
    },
    APIKey:         "your-api-key",
    ReadPreference: vectorizer.ReadPreferenceReplica, // Master | Replica | Nearest
})

// Writes automatically go to master
client.InsertTexts(ctx, "documents", []vectorizer.TextInput{
    {ID: "doc1", Text: "Sample document", Metadata: map[string]any{"source": "api"}},
})

// Reads automatically go to replicas (load balanced)
results, _ := client.SearchVectors(ctx, "documents", queryVector, 10)

// Force read from master
fresh, _ := client.GetVectorWithPreference(ctx, "documents", "doc1", vectorizer.ReadPreferenceMaster)
```

```csharp
// C#
var client = new VectorizerClient(new ClientConfig
{
    Hosts = new HostConfig
    {
        Master = "http://master-node:15001",
        Replicas = new[] { "http://replica1:15001", "http://replica2:15001" }
    },
    ApiKey = "your-api-key",
    ReadPreference = ReadPreference.Replica // Master | Replica | Nearest
});

// Writes automatically go to master
await client.InsertTextsAsync("documents", new[] {
    new TextInput { Id = "doc1", Text = "Sample document", Metadata = new { source = "api" } }
});

// Reads automatically go to replicas (load balanced)
var results = await client.SearchVectorsAsync("documents", queryVector, limit: 10);

// Force read from master
var fresh = await client.GetVectorAsync("documents", "doc1", ReadPreference.Master);
```

### Read Preferences

| Preference | Description | Use Case |
|------------|-------------|----------|
| `replica` | Route reads to replicas (round-robin) | Default for high read throughput |
| `master` | Route all reads to master | When you need read-your-writes consistency |
| `nearest` | Route to the node with lowest latency | Geo-distributed deployments |

### Automatic Operation Routing

The SDK automatically classifies operations:

| Operation Type | Routed To | Methods |
|---------------|-----------|---------|
| **Writes** | Always Master | `insertTexts`, `insertVectors`, `updateVector`, `deleteVector`, `createCollection`, `deleteCollection` |
| **Reads** | Based on `readPreference` | `searchVectors`, `getVector`, `listCollections`, `intelligentSearch`, `semanticSearch`, `hybridSearch` |

### Read-Your-Writes Consistency

For operations that need to immediately read what was just written:

```typescript
// Option 1: Override read preference for specific operation
await client.insertTexts("docs", [newDoc]);
const result = await client.getVector("docs", newDoc.id, { readPreference: "master" });

// Option 2: Use a transaction-like pattern
const result = await client.withMaster(async (masterClient) => {
  await masterClient.insertTexts("docs", [newDoc]);
  return await masterClient.getVector("docs", newDoc.id);
});
```

```python
# Option 1: Override read preference for specific operation
await client.insert_texts("docs", [new_doc])
result = await client.get_vector("docs", new_doc["id"], read_preference="master")

# Option 2: Use a transaction-like pattern
async with client.with_master() as master_client:
    await master_client.insert_texts("docs", [new_doc])
    result = await master_client.get_vector("docs", new_doc["id"])
```

### Standalone Mode (Single Node)

For development or single-node deployments:

```typescript
// Single node - no replication
const client = new VectorizerClient({
  baseURL: "http://localhost:15001",
  apiKey: "your-api-key",
});
```

### Server Configuration

Configure master and replica nodes on the **server side**:

**Master Node** (`config.yml`):
```yaml
replication:
  enabled: true
  role: "master"
  bind_address: "0.0.0.0:7001"
  heartbeat_interval_secs: 5
  log_size: 1000000
```

**Replica Node** (`config.yml`):
```yaml
replication:
  enabled: true
  role: "replica"
  master_address: "192.168.1.10:7001"
  reconnect_interval_secs: 5
```

For detailed server configuration, see [Replication Documentation](../docs/specs/REPLICATION.md).

## Text Summarization

All SDKs support intelligent text and context summarization with multiple algorithms:

### Summarize Text

```typescript
// TypeScript/JavaScript
const summary = await client.summarizeText({
  text: "Long document text here...",
  method: "extractive", // extractive, keyword, sentence, abstractive
  compression_ratio: 0.3,
  language: "en",
});

console.log(`Summary: ${summary.summary}`);
console.log(`Compression: ${summary.compression_ratio}`);
```

```python
# Python
from vectorizer.models import SummarizeTextRequest

summary = await client.summarize_text(SummarizeTextRequest(
    text='Long document text here...',
    method='extractive',
    compression_ratio=0.3,
    language='en'
))

print(f"Summary: {summary.summary}")
print(f"Compression: {summary.compression_ratio}")
```

### Summarize Context

```typescript
// TypeScript/JavaScript
const contextSummary = await client.summarizeContext({
  context: "Context information here...",
  method: "keyword",
  max_length: 100,
  language: "en",
});
```

```python
# Python
from vectorizer.models import SummarizeContextRequest

context_summary = await client.summarize_context(SummarizeContextRequest(
    context='Context information here...',
    method='keyword',
    max_length=100,
    language='en'
))
```

### Summary Management

```typescript
// TypeScript/JavaScript
// Get summary by ID
const retrieved = await client.getSummary(summary.summary_id);

// List summaries with filtering
const summaries = await client.listSummaries({
  method: "extractive",
  language: "en",
  limit: 10,
});
```

```python
# Python
# Get summary by ID
retrieved = await client.get_summary(summary.summary_id)

# List summaries with filtering
summaries = await client.list_summaries(
    method='extractive',
    language='en',
    limit=10
)
```

### Available Summarization Methods

- **extractive**: Uses MMR (Maximal Marginal Relevance) algorithm for extractive summarization
- **keyword**: Extracts key terms and phrases
- **sentence**: Selects most important sentences
- **abstractive**: Generates abstract summaries (experimental)

## Development

### Building SDKs

```bash
# TypeScript SDK
cd sdks/typescript
npm install
npm run build

# Python SDK
cd sdks/python
pip install -r requirements.txt
python setup.py build

# Rust SDK
cd sdks/rust
cargo build
```

### Testing

```bash
# TypeScript SDK
cd sdks/typescript
npm test

# Python SDK
cd sdks/python
python run_tests.py

# Rust SDK
cd sdks/rust
cargo test
```

### Linting

```bash
# TypeScript SDK
cd sdks/typescript
npm run lint

# Python SDK
cd sdks/python
flake8 src/

# Rust SDK
cd sdks/rust
cargo clippy
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

## License

MIT License - see [LICENSE](./LICENSE) for details.

## Support

- **Documentation**: [Vectorizer Documentation](../docs/)
- **Issues**: [GitHub Issues](https://github.com/hivellm/vectorizer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hivellm/vectorizer/discussions)
