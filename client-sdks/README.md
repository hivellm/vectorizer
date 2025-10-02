# Hive Vectorizer Client SDKs

High-performance client SDKs for the Hive Vectorizer vector database, available in multiple languages.

## Available SDKs

### 🟦 TypeScript SDK ✅
- **Package**: `@hivellm/vectorizer-client-ts`
- **Status**: Published on npm (v0.1.0)
- **Features**: Full TypeScript support, async/await, comprehensive type safety
- **Installation**: `npm install @hivellm/vectorizer-client-ts`
- **Documentation**: [TypeScript SDK README](./typescript/README.md)

### 🟨 JavaScript SDK ✅
- **Package**: `@hivellm/vectorizer-client-js`
- **Status**: Published on npm (v0.1.0)
- **Features**: Modern JavaScript, multiple build formats (CJS, ESM, UMD)
- **Installation**: `npm install @hivellm/vectorizer-client-js`
- **Documentation**: [JavaScript SDK README](./javascript/README.md)

### 🦀 Rust SDK ✅
- **Package**: `vectorizer-rust-sdk`
- **Status**: Published on crates.io (v0.1.0)
- **Features**: High performance, async/await, MCP support, type safety
- **Installation**: Add to `Cargo.toml`: `vectorizer-rust-sdk = "0.1.0"`
- **Documentation**: [Rust SDK README](./rust/README.md)

### 🐍 Python SDK 🚧
- **Package**: `hivellm-vectorizer-client`
- **Status**: In development - PyPI publishing in progress
- **Features**: Async/await support, comprehensive testing, CLI interface
- **Installation**: Coming soon - `pip install hivellm-vectorizer-client`
- **Documentation**: [Python SDK README](./python/README.md)

## Quick Start

### TypeScript/JavaScript

```typescript
import { VectorizerClient } from '@hivellm/vectorizer-client';

const client = new VectorizerClient({
  baseURL: 'http://localhost:15001',
  apiKey: 'your-api-key'
});

// Create collection
await client.createCollection({
  name: 'documents',
  dimension: 768,
  similarity_metric: 'cosine'
});

// Insert texts
await client.insertTexts('documents', [{
  id: 'doc_1',
  text: 'This is a sample document about machine learning',
  metadata: { source: 'document.pdf', category: 'AI' }
}]);

// Search
const results = await client.searchVectors('documents', {
  query_vector: [0.1, 0.2, 0.3, /* ... 768 dimensions */],
  limit: 5
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
const batchResult = await client.batchInsertTexts('documents', {
  texts: [
    { id: 'doc1', text: 'Machine learning algorithms', metadata: { category: 'AI' } },
    { id: 'doc2', text: 'Deep learning neural networks', metadata: { category: 'AI' } },
    { id: 'doc3', text: 'Natural language processing', metadata: { category: 'NLP' } }
  ],
  config: {
    provider: 'bm25',
    max_batch_size: 100,
    parallel_workers: 4,
    atomic: true
  }
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
const searchResult = await client.batchSearchVectors('documents', {
  queries: [
    { query: 'machine learning', limit: 5 },
    { query: 'neural networks', limit: 3 },
    { query: 'NLP techniques', limit: 4 }
  ],
  config: { provider: 'bm25', parallel_workers: 2 }
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
const deleteResult = await client.batchDeleteVectors('documents', {
  vector_ids: ['doc1', 'doc2', 'doc3'],
  config: { atomic: true }
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
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   TypeScript    │    │   JavaScript    │    │     Python      │    │      Rust        │
│      SDK        │    │      SDK        │    │      SDK        │    │      SDK         │
│     ✅ v0.1.0   │    │     ✅ v0.1.0   │    │   🚧 In Dev     │    │     ✅ v0.1.0    │
│                 │    │                 │    │                 │    │                  │
│ • Type Safety   │    │ • REST-Only     │    │ • Async/Await   │    │ • High Performance│
│ • IntelliSense  │    │ • 100% Tests    │    │ • CLI Interface │    │ • Memory Safety  │
│ • ES2020+       │    │ • Browser Ready │    │ • 44+ Tests     │    │ • MCP Support    │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │                       │
         └───────────────────────┼───────────────────────┼───────────────────────┘
                                 │                       │
                    ┌─────────────────┐    ┌─────────────────┐
                    │  Vectorizer     │    │   MCP Server    │
                    │     Server      │    │                 │
                    │                 │    │ • Model Context │
                    │ • REST API      │    │ • AI Integration │
                    │ • GRPC          │    │ • Tool Calling  │
                    │ • MCP Protocol  │    │ • SSE Transport │
                    └─────────────────┘    └─────────────────┘
```

## Text Summarization

All SDKs support intelligent text and context summarization with multiple algorithms:

### Summarize Text
```typescript
// TypeScript/JavaScript
const summary = await client.summarizeText({
  text: 'Long document text here...',
  method: 'extractive', // extractive, keyword, sentence, abstractive
  compression_ratio: 0.3,
  language: 'en'
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
  context: 'Context information here...',
  method: 'keyword',
  max_length: 100,
  language: 'en'
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
  method: 'extractive',
  language: 'en',
  limit: 10
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
cd client-sdks/typescript
npm install
npm run build

# JavaScript SDK
cd client-sdks/javascript
npm install
npm run build

# Python SDK
cd client-sdks/python
pip install -r requirements.txt
python setup.py build

# Rust SDK
cd client-sdks/rust
cargo build
```

### Testing

```bash
# TypeScript SDK
cd client-sdks/typescript
npm test

# JavaScript SDK
cd client-sdks/javascript
npm test

# Python SDK
cd client-sdks/python
python run_tests.py

# Rust SDK
cd client-sdks/rust
cargo test
```

### Linting

```bash
# TypeScript SDK
cd client-sdks/typescript
npm run lint

# JavaScript SDK
cd client-sdks/javascript
npm run lint

# Python SDK
cd client-sdks/python
flake8 src/

# Rust SDK
cd client-sdks/rust
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
