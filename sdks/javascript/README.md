# Vectorizer JavaScript SDK

[![npm version](https://badge.fury.io/js/%40hivellm%2Fvectorizer-sdk-js.svg)](https://www.npmjs.com/package/@hivellm/vectorizer-sdk-js)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

High-performance JavaScript SDK for Vectorizer vector database.

**Package**: `@hivellm/vectorizer-sdk-js`  
**Version**: 1.0.1

## Features

- ✅ **Modern JavaScript**: ES2020+ support with async/await
- ✅ **Multiple Transport Protocols**: HTTP/HTTPS and UMICP support
- ✅ **HTTP Client**: Native fetch-based HTTP client with robust error handling
- ✅ **UMICP Protocol**: High-performance protocol using @hivellm/umicp SDK
- ✅ **Comprehensive Validation**: Input validation with `isFinite()` checks for Infinity/NaN
- ✅ **12 Custom Exceptions**: Robust error management with consistent error codes
- ✅ **Logging**: Configurable logging system
- ✅ **Collection Management**: CRUD operations for collections
- ✅ **Vector Operations**: Insert, search, update, delete vectors
- ✅ **Semantic Search**: Text and vector similarity search
- ✅ **Intelligent Search**: Advanced multi-query search with domain expansion
- ✅ **Contextual Search**: Context-aware search with metadata filtering
- ✅ **Multi-Collection Search**: Cross-collection search with intelligent aggregation
- ✅ **Embedding Generation**: Text embedding support
- ✅ **Multiple Build Formats**: CommonJS, ES Modules, UMD
- ✅ **100% Test Coverage**: Comprehensive test suite with all tests passing

## Installation

```bash
npm install @hivellm/vectorizer-sdk-js
```

## Quick Start

```javascript
import { VectorizerClient } from '@hivellm/vectorizer-sdk-js';

// Create client
const client = new VectorizerClient({
  baseURL: 'http://localhost:15001',
  apiKey: 'your-api-key-here'
});

// Health check
const health = await client.healthCheck();
console.log('Server status:', health.status);

// Create collection
const collection = await client.createCollection({
  name: 'documents',
  dimension: 768,
  similarity_metric: 'cosine'
});

// Insert vectors
const vectors = [{
  data: [0.1, 0.2, 0.3, /* ... 768 dimensions */],
  metadata: { source: 'document1.pdf' }
}];

await client.insertVectors('documents', vectors);

// Search vectors
const results = await client.searchVectors('documents', {
  query_vector: [0.1, 0.2, 0.3, /* ... 768 dimensions */],
  limit: 5
});

// Text search
const textResults = await client.searchText('documents', {
  query: 'machine learning algorithms',
  limit: 5
});

// Intelligent search with multi-query expansion
const intelligentResults = await client.intelligentSearch({
  query: 'machine learning algorithms',
  collections: ['documents', 'research'],
  max_results: 15,
  domain_expansion: true,
  technical_focus: true,
  mmr_enabled: true,
  mmr_lambda: 0.7
});

// Semantic search with reranking
const semanticResults = await client.semanticSearch({
  query: 'neural networks',
  collection: 'documents',
  max_results: 10,
  semantic_reranking: true,
  similarity_threshold: 0.6
});

// Contextual search with metadata filtering
const contextualResults = await client.contextualSearch({
  query: 'deep learning',
  collection: 'documents',
  context_filters: {
    category: 'AI',
    language: 'en',
    year: 2023
  },
  max_results: 10,
  context_weight: 0.4
});

// Multi-collection search
const multiResults = await client.multiCollectionSearch({
  query: 'artificial intelligence',
  collections: ['documents', 'research', 'tutorials'],
  max_per_collection: 5,
  max_total_results: 20,
  cross_collection_reranking: true
});

// Generate embeddings
const embedding = await client.embedText({
  text: 'machine learning algorithms'
});
```

## Configuration

### HTTP Configuration (Default)

```javascript
const client = new VectorizerClient({
  baseURL: 'http://localhost:15002',     // API base URL
  apiKey: 'your-api-key',                // API key for authentication
  timeout: 30000,                        // Request timeout in ms
  headers: {                             // Custom headers
    'User-Agent': 'MyApp/1.0'
  },
  logger: {                              // Logger configuration
    level: 'info',                       // debug, info, warn, error
    enabled: true
  }
});
```

### UMICP Configuration (High Performance)

[UMICP (Universal Messaging and Inter-process Communication Protocol)](https://www.npmjs.com/package/@hivellm/umicp) provides performance benefits using the StreamableHTTP transport from the official SDK.

#### Using Connection String

```javascript
const client = new VectorizerClient({
  connectionString: 'umicp://localhost:15003',
  apiKey: 'your-api-key'
});
```

#### Using Explicit Configuration

```javascript
const client = new VectorizerClient({
  protocol: 'umicp',
  apiKey: 'your-api-key',
  umicp: {
    host: 'localhost',
    port: 15003,
    timeout: 60000
  }
});
```

#### When to Use UMICP

Use UMICP when:
- **Large Payloads**: Inserting or searching large batches of vectors
- **High Throughput**: Need maximum performance for production workloads
- **Low Latency**: Need minimal protocol overhead

Use HTTP when:
- **Development**: Quick testing and debugging
- **Firewall Restrictions**: Only HTTP/HTTPS allowed
- **Simple Deployments**: No need for custom protocol setup

#### Protocol Comparison

| Feature | HTTP/HTTPS | UMICP |
|---------|-----------|-------|
| Transport | Standard fetch API | StreamableHTTP (from @hivellm/umicp) |
| Performance | Standard | Optimized for large payloads |
| Firewall | Widely supported | May require configuration |
| Debugging | Easy (browser tools) | Requires UMICP tools |

## API Reference

### Collection Management

```javascript
// List collections
const collections = await client.listCollections();

// Get collection info
const info = await client.getCollection('documents');

// Create collection
const collection = await client.createCollection({
  name: 'documents',
  dimension: 768,
  similarity_metric: 'cosine',
  description: 'Document embeddings'
});

// Update collection
const updated = await client.updateCollection('documents', {
  description: 'Updated description'
});

// Delete collection
await client.deleteCollection('documents');
```

### Vector Operations

```javascript
// Insert vectors
const vectors = [{
  data: [0.1, 0.2, 0.3],
  metadata: { source: 'doc1.pdf' }
}];
await client.insertVectors('documents', vectors);

// Get vector
const vector = await client.getVector('documents', 'vector-id');

// Update vector
const updated = await client.updateVector('documents', 'vector-id', {
  metadata: { updated: true }
});

// Delete vector
await client.deleteVector('documents', 'vector-id');

// Delete multiple vectors
await client.deleteVectors('documents', ['id1', 'id2', 'id3']);
```

### Search Operations

```javascript
// Vector similarity search
const results = await client.searchVectors('documents', {
  query_vector: [0.1, 0.2, 0.3],
  limit: 10,
  threshold: 0.8,
  include_metadata: true
});

// Text semantic search
const textResults = await client.searchText('documents', {
  query: 'machine learning',
  limit: 10,
  threshold: 0.8,
  include_metadata: true,
  model: 'bert-base'
});
```

### Embedding Operations

```javascript
// Generate embeddings
const embedding = await client.embedText({
  text: 'machine learning algorithms',
  model: 'bert-base',
  parameters: {
    max_length: 512,
    normalize: true
  }
});
```

## Error Handling

```javascript
import {
  VectorizerError,
  AuthenticationError,
  CollectionNotFoundError,
  ValidationError,
  NetworkError,
  ServerError
} from '@hivellm/vectorizer-sdk-js';

try {
  await client.createCollection({
    name: 'documents',
    dimension: 768
  });
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error('Authentication failed:', error.message);
  } else if (error instanceof ValidationError) {
    console.error('Validation error:', error.message);
  } else if (error instanceof NetworkError) {
    console.error('Network error:', error.message);
  } else {
    console.error('Unknown error:', error.message);
  }
}
```

## Build Formats

The SDK is available in multiple formats:

- **CommonJS**: `dist/index.js` - For Node.js
- **ES Modules**: `dist/index.esm.js` - For modern bundlers
- **UMD**: `dist/index.umd.js` - For browsers
- **UMD Minified**: `dist/index.umd.min.js` - For production

### Node.js (CommonJS)

```javascript
const { VectorizerClient } = require('@hivellm/vectorizer-sdk-js');
```

### ES Modules

```javascript
import { VectorizerClient } from '@hivellm/vectorizer-sdk-js';
```

### Browser (UMD)

```html
<script src="https://unpkg.com/@hivellm/vectorizer-sdk-js/dist/index.umd.min.js"></script>
<script>
  const client = new VectorizerClient.VectorizerClient({
    baseURL: 'http://localhost:15001'
  });
</script>
```

## Development

```bash
# Install dependencies
npm install

# Build
npm run build

# Watch mode
npm run build:watch

# Test
npm test

# Test with coverage
npm run test:coverage

# Lint
npm run lint

# Lint and fix
npm run lint:fix
```

## License

MIT License - see [LICENSE](LICENSE) for details.
