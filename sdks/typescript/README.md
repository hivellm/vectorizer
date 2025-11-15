# Vectorizer TypeScript SDK

[![npm version](https://badge.fury.io/js/%40hivellm%2Fvectorizer-sdk.svg)](https://www.npmjs.com/package/@hivellm/vectorizer-sdk)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

High-performance TypeScript SDK for Vectorizer vector database.

**Package**: `@hivellm/vectorizer-sdk`  
**Version**: 1.2.0

## Features

- ✅ **Complete TypeScript Support**: Full type safety and IntelliSense
- ✅ **Async/Await**: Modern async programming patterns
- ✅ **Multiple Transport Protocols**: HTTP/HTTPS and UMICP support
- ✅ **HTTP Client**: Native fetch-based HTTP client with robust error handling
- ✅ **UMICP Protocol**: High-performance protocol with compression and encryption
- ✅ **Comprehensive Validation**: Input validation and error handling
- ✅ **12 Custom Exceptions**: Robust error management
- ✅ **Logging**: Configurable logging system
- ✅ **Collection Management**: CRUD operations for collections
- ✅ **Vector Operations**: Insert, search, update, delete vectors
- ✅ **Semantic Search**: Text and vector similarity search
- ✅ **Hybrid Search**: Combine dense and sparse vectors for improved search quality
- ✅ **Qdrant Compatibility**: Full Qdrant REST API compatibility for easy migration
- ✅ **Embedding Generation**: Text embedding support

## Installation

```bash
npm install @hivellm/vectorizer-sdk

# Or specific version
npm install @hivellm/vectorizer-sdk@1.0.1
```

## Quick Start

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

// Create client
const client = new VectorizerClient({
  baseURL: "http://localhost:15001",
  apiKey: "your-api-key-here",
});

// Health check
const health = await client.healthCheck();
console.log("Server status:", health.status);

// Create collection
const collection = await client.createCollection({
  name: "documents",
  dimension: 768,
  similarity_metric: "cosine",
});

// Insert vectors
const vectors = [
  {
    data: [0.1, 0.2, 0.3 /* ... 768 dimensions */],
    metadata: { source: "document1.pdf" },
  },
];

await client.insertVectors("documents", vectors);

// Search vectors
const results = await client.searchVectors("documents", {
  query_vector: [0.1, 0.2, 0.3 /* ... 768 dimensions */],
  limit: 5,
});

// Text search
const textResults = await client.searchText("documents", {
  query: "machine learning algorithms",
  limit: 5,
});

// Generate embeddings
const embedding = await client.embedText({
  text: "machine learning algorithms",
});

// Hybrid search (dense + sparse vectors)
const hybridResults = await client.hybridSearch({
  collection: "documents",
  query: "machine learning",
  query_sparse: {
    indices: [0, 5, 10, 15],
    values: [0.8, 0.6, 0.9, 0.7],
  },
  alpha: 0.7,
  algorithm: "rrf",
  dense_k: 20,
  sparse_k: 20,
  final_k: 10,
});

// Qdrant-compatible API usage
const qdrantCollections = await client.qdrantListCollections();
const qdrantResults = await client.qdrantSearchPoints(
  "documents",
  embedding.embedding,
  10
);
```

## Configuration

### HTTP Configuration (Default)

```typescript
const client = new VectorizerClient({
  baseURL: "http://localhost:15002", // API base URL
  apiKey: "your-api-key", // API key for authentication
  timeout: 30000, // Request timeout in ms
  headers: {
    // Custom headers
    "User-Agent": "MyApp/1.0",
  },
  logger: {
    // Logger configuration
    level: "info", // debug, info, warn, error
    enabled: true,
  },
});
```

### UMICP Configuration (High Performance)

[UMICP (Universal Messaging and Inter-process Communication Protocol)](https://www.npmjs.com/package/@hivellm/umicp) provides significant performance benefits:

- **Automatic Compression**: GZIP, DEFLATE, or LZ4 compression for large payloads
- **Built-in Encryption**: Optional encryption for secure communication
- **Lower Latency**: Optimized binary protocol with checksums
- **Request Validation**: Automatic request/response validation

#### Using Connection String

```typescript
const client = new VectorizerClient({
  connectionString: "umicp://localhost:15003",
  apiKey: "your-api-key",
});
```

#### Using Explicit Configuration

```typescript
const client = new VectorizerClient({
  protocol: "umicp",
  apiKey: "your-api-key",
  umicp: {
    host: "localhost",
    port: 15003,
    compression: "gzip", // 'gzip', 'deflate', 'lz4', or 'none'
    encryption: true, // Enable encryption
    priority: "normal", // 'low', 'normal', 'high'
  },
});
```

#### When to Use UMICP

Use UMICP when:

- **Large Payloads**: Inserting or searching large batches of vectors
- **High Throughput**: Need maximum performance for production workloads
- **Secure Communication**: Require encryption without TLS overhead
- **Low Latency**: Need minimal protocol overhead

Use HTTP when:

- **Development**: Quick testing and debugging
- **Firewall Restrictions**: Only HTTP/HTTPS allowed
- **Simple Deployments**: No need for custom protocol setup

#### Protocol Comparison

| Feature     | HTTP/HTTPS           | UMICP                        |
| ----------- | -------------------- | ---------------------------- |
| Compression | Manual (gzip header) | Automatic (GZIP/DEFLATE/LZ4) |
| Encryption  | TLS required         | Built-in optional            |
| Latency     | Standard             | Lower                        |
| Firewall    | Widely supported     | May require configuration    |
| Debugging   | Easy (browser tools) | Requires UMICP tools         |

## API Reference

### Collection Management

```typescript
// List collections
const collections = await client.listCollections();

// Get collection info
const info = await client.getCollection("documents");

// Create collection
const collection = await client.createCollection({
  name: "documents",
  dimension: 768,
  similarity_metric: "cosine",
  description: "Document embeddings",
});

// Update collection
const updated = await client.updateCollection("documents", {
  description: "Updated description",
});

// Delete collection
await client.deleteCollection("documents");
```

### Vector Operations

```typescript
// Insert vectors
const vectors = [
  {
    data: [0.1, 0.2, 0.3],
    metadata: { source: "doc1.pdf" },
  },
];
await client.insertVectors("documents", vectors);

// Get vector
const vector = await client.getVector("documents", "vector-id");

// Update vector
const updated = await client.updateVector("documents", "vector-id", {
  metadata: { updated: true },
});

// Delete vector
await client.deleteVector("documents", "vector-id");

// Delete multiple vectors
await client.deleteVectors("documents", ["id1", "id2", "id3"]);
```

### Search Operations

```typescript
// Vector similarity search
const results = await client.searchVectors("documents", {
  query_vector: [0.1, 0.2, 0.3],
  limit: 10,
  threshold: 0.8,
  include_metadata: true,
});

// Text semantic search
const textResults = await client.searchText("documents", {
  query: "machine learning",
  limit: 10,
  threshold: 0.8,
  include_metadata: true,
  model: "bert-base",
});
```

### Embedding Operations

```typescript
// Generate embeddings
const embedding = await client.embedText({
  text: "machine learning algorithms",
  model: "bert-base",
  parameters: {
    max_length: 512,
    normalize: true,
  },
});
```

## Error Handling

```typescript
import {
  VectorizerError,
  AuthenticationError,
  CollectionNotFoundError,
  ValidationError,
  NetworkError,
  ServerError,
} from "@hivellm/vectorizer-sdk";

try {
  await client.createCollection({
    name: "documents",
    dimension: 768,
  });
} catch (error) {
  if (error instanceof AuthenticationError) {
    console.error("Authentication failed:", error.message);
  } else if (error instanceof ValidationError) {
    console.error("Validation error:", error.message);
  } else if (error instanceof NetworkError) {
    console.error("Network error:", error.message);
  } else {
    console.error("Unknown error:", error.message);
  }
}
```

## Types

```typescript
// Vector types
interface Vector {
  id: string;
  data: number[];
  metadata?: Record<string, unknown>;
}

// Collection types
interface Collection {
  name: string;
  dimension: number;
  similarity_metric: "cosine" | "euclidean" | "dot_product";
  description?: string;
  created_at?: Date;
  updated_at?: Date;
}

// Search result types
interface SearchResult {
  id: string;
  score: number;
  data: number[];
  metadata?: Record<string, unknown>;
}

// Client configuration
interface VectorizerClientConfig {
  baseURL?: string;
  wsURL?: string;
  apiKey?: string;
  timeout?: number;
  headers?: Record<string, string>;
  logger?: LoggerConfig;
}
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
