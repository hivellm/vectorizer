# Vectorizer TypeScript SDK

[![npm version](https://badge.fury.io/js/%40hivellm%2Fvectorizer-sdk.svg)](https://www.npmjs.com/package/@hivellm/vectorizer-sdk)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

High-performance TypeScript SDK for Vectorizer vector database.

**Package**: `@hivellm/vectorizer-sdk`  
**Version**: 1.5.1

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
- ✅ **Intelligent Search**: AI-powered search with query expansion, MMR diversification, and domain expansion
- ✅ **Semantic Search**: Advanced semantic search with reranking and similarity thresholds
- ✅ **Contextual Search**: Context-aware search with metadata filtering
- ✅ **Multi-Collection Search**: Cross-collection search with intelligent aggregation
- ✅ **Hybrid Search**: Combine dense and sparse vectors for improved search quality
- ✅ **Discovery Operations**: Collection filtering, query expansion, and intelligent discovery
- ✅ **File Operations**: File content retrieval, chunking, project outlines, and related files
- ✅ **Graph Relationships**: Automatic relationship discovery, path finding, and edge management
- ✅ **Summarization**: Text and context summarization with multiple methods
- ✅ **Workspace Management**: Multi-workspace support for project organization
- ✅ **Backup & Restore**: Collection backup and restore operations
- ✅ **Batch Operations**: Efficient bulk insert, update, delete, and search
- ✅ **Qdrant Compatibility**: Full Qdrant 1.14.x REST API compatibility for easy migration
  - Snapshots API (create, list, delete, recover)
  - Sharding API (create shard keys, distribute data)
  - Cluster Management API (status, recovery, peer management, metadata)
  - Query API (query, batch query, grouped queries with prefetch)
  - Search Groups and Matrix API (grouped results, similarity matrices)
  - Named Vectors support (partial)
  - Quantization configuration (PQ and Binary)
- ✅ **Embedding Generation**: Text embedding support

## Installation

```bash
npm install @hivellm/vectorizer-sdk

# Or specific version
npm install @hivellm/vectorizer-sdk@1.5.1
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

// Graph Operations (requires graph enabled in collection config)
// List all graph nodes
const nodes = await client.listGraphNodes("documents");
console.log(`Graph has ${nodes.count} nodes`);

// Get neighbors of a node
const neighbors = await client.getGraphNeighbors("documents", "document1");
console.log(`Node has ${neighbors.neighbors.length} neighbors`);

// Find related nodes within 2 hops
const related = await client.findRelatedNodes("documents", "document1", {
  max_hops: 2,
  relationship_type: "SIMILAR_TO",
});
console.log(`Found ${related.related.length} related nodes`);

// Find shortest path between two nodes
const path = await client.findGraphPath({
  collection: "documents",
  source: "document1",
  target: "document2",
});
if (path.found) {
  console.log(`Path found: ${path.path.map(n => n.id).join(" -> ")}`);
}

// Create explicit relationship
const edge = await client.createGraphEdge({
  collection: "documents",
  source: "document1",
  target: "document2",
  relationship_type: "REFERENCES",
  weight: 0.9,
});
console.log(`Created edge: ${edge.edge_id}`);

// Discover SIMILAR_TO edges for entire collection
const discoveryResult = await client.discoverGraphEdges("documents", {
  similarity_threshold: 0.7,
  max_per_node: 10,
});
console.log(`Discovered ${discoveryResult.edges_created} edges`);

// Discover edges for a specific node
const nodeDiscovery = await client.discoverGraphEdgesForNode(
  "documents",
  "document1",
  {
    similarity_threshold: 0.7,
    max_per_node: 10,
  }
);
console.log(`Discovered ${nodeDiscovery.edges_created} edges for node`);

// Get discovery status
const status = await client.getGraphDiscoveryStatus("documents");
console.log(
  `Discovery status: ${status.total_nodes} nodes, ` +
    `${status.total_edges} edges, ` +
    `${status.progress_percentage.toFixed(1)}% complete`
);

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

### Master/Slave Configuration (Read/Write Separation)

Vectorizer supports **Master-Replica replication** for high availability and read scaling. The SDK provides **automatic routing** - writes go to master, reads are distributed across replicas.

#### Basic Setup

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

// Configure with master and replicas - SDK handles routing automatically
const client = new VectorizerClient({
  hosts: {
    master: "http://master-node:15001",
    replicas: ["http://replica1:15001", "http://replica2:15001"],
  },
  apiKey: "your-api-key",
  readPreference: "replica", // "master" | "replica" | "nearest"
});

// Writes automatically go to master
await client.createCollection({
  name: "documents",
  dimension: 768,
  similarity_metric: "cosine",
});

await client.insertTexts("documents", [
  { id: "doc1", text: "Sample document", metadata: { source: "api" } },
]);

// Reads automatically go to replicas (load balanced)
const results = await client.searchVectors("documents", {
  query: "sample",
  limit: 10,
});

const collections = await client.listCollections();
```

#### Read Preferences

| Preference | Description | Use Case |
|------------|-------------|----------|
| `"replica"` | Route reads to replicas (round-robin) | Default for high read throughput |
| `"master"` | Route all reads to master | When you need read-your-writes consistency |
| `"nearest"` | Route to the node with lowest latency | Geo-distributed deployments |

#### Read-Your-Writes Consistency

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

#### Automatic Operation Routing

The SDK automatically classifies operations:

| Operation Type | Routed To | Methods |
|---------------|-----------|---------|
| **Writes** | Always Master | `insertTexts`, `insertVectors`, `updateVector`, `deleteVector`, `createCollection`, `deleteCollection` |
| **Reads** | Based on `readPreference` | `searchVectors`, `getVector`, `listCollections`, `intelligentSearch`, `semanticSearch`, `hybridSearch` |

#### Standalone Mode (Single Node)

For development or single-node deployments:

```typescript
// Single node - no replication
const client = new VectorizerClient({
  baseURL: "http://localhost:15001",
  apiKey: "your-api-key",
});
```

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

### Advanced Search Operations

#### Intelligent Search
AI-powered search with query expansion, MMR diversification, and domain expansion:

```typescript
const results = await client.intelligentSearch({
  query: "machine learning algorithms",
  collections: ["documents", "research"],
  max_results: 15,
  domain_expansion: true,
  technical_focus: true,
  mmr_enabled: true,
  mmr_lambda: 0.7,
});
```

#### Semantic Search
Advanced semantic search with reranking and similarity thresholds:

```typescript
const results = await client.semanticSearch({
  query: "neural networks",
  collection: "documents",
  max_results: 10,
  semantic_reranking: true,
  similarity_threshold: 0.6,
});
```

#### Contextual Search
Context-aware search with metadata filtering:

```typescript
const results = await client.contextualSearch({
  query: "API documentation",
  collection: "docs",
  context_filters: {
    category: "backend",
    language: "typescript",
  },
  max_results: 10,
});
```

#### Multi-Collection Search
Cross-collection search with intelligent aggregation:

```typescript
const results = await client.multiCollectionSearch({
  query: "authentication",
  collections: ["docs", "code", "tickets"],
  max_total_results: 20,
  max_per_collection: 5,
  cross_collection_reranking: true,
});
```

### Discovery Operations

#### Filter Collections
Filter collections based on query relevance:

```typescript
const filtered = await client.filterCollections({
  query: "machine learning",
  min_score: 0.5,
});
```

#### Expand Queries
Expand queries with related terms:

```typescript
const expanded = await client.expandQueries({
  query: "neural networks",
  max_expansions: 5,
});
```

#### Discover
Intelligent discovery across collections:

```typescript
const discovery = await client.discover({
  query: "authentication methods",
  max_results: 10,
});
```

### File Operations

#### Get File Content
Retrieve file content from collection:

```typescript
const content = await client.getFileContent({
  collection: "docs",
  file_path: "src/client.ts",
});
```

#### List Files
List all files in a collection:

```typescript
const files = await client.listFilesInCollection({
  collection: "docs",
});
```

#### Get File Chunks
Get ordered chunks of a file:

```typescript
const chunks = await client.getFileChunksOrdered({
  collection: "docs",
  file_path: "README.md",
  chunk_size: 1000,
});
```

#### Get Project Outline
Get project structure outline:

```typescript
const outline = await client.getProjectOutline({
  collection: "codebase",
});
```

#### Get Related Files
Find files related to a specific file:

```typescript
const related = await client.getRelatedFiles({
  collection: "codebase",
  file_path: "src/client.ts",
  max_results: 5,
});
```

### Summarization Operations

#### Summarize Text
Summarize text using various methods:

```typescript
const summary = await client.summarizeText({
  text: "Long document text...",
  method: "extractive", // 'extractive', 'abstractive', 'hybrid'
  max_length: 200,
});
```

#### Summarize Context
Summarize context with metadata:

```typescript
const summary = await client.summarizeContext({
  context: "Document context...",
  method: "abstractive",
  focus: "key_points",
});
```

### Workspace Management

#### Add Workspace
Add a new workspace:

```typescript
await client.addWorkspace({
  name: "my-project",
  path: "/path/to/project",
});
```

#### List Workspaces
List all workspaces:

```typescript
const workspaces = await client.listWorkspaces();
```

#### Remove Workspace
Remove a workspace:

```typescript
await client.removeWorkspace({
  name: "my-project",
});
```

### Backup Operations

#### Create Backup
Create a backup of collections:

```typescript
const backup = await client.createBackup({
  name: "backup-2024-11-24",
});
```

#### List Backups
List all available backups:

```typescript
const backups = await client.listBackups();
```

#### Restore Backup
Restore from a backup:

```typescript
await client.restoreBackup({
  filename: "backup-2024-11-24.vecdb",
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
