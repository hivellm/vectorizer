---
title: TypeScript SDK
module: sdks
id: typescript-sdk
order: 2
description: Complete TypeScript/JavaScript SDK guide for Vectorizer
tags: [typescript, javascript, sdk, client-library]
---

# TypeScript SDK

Complete guide to using the Vectorizer TypeScript/JavaScript SDK.

## Installation

```bash
npm install @hivellm/vectorizer-sdk
```

## Quick Start

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

// Create client
const client = new VectorizerClient("http://localhost:15002");

// Create collection
await client.createCollection("my_docs", { dimension: 384 });

// Insert text
await client.insertText("my_docs", "Hello, Vectorizer!");

// Search
const results = await client.search("my_docs", "hello", { limit: 5 });
```

## Client Configuration

### Basic Client

```typescript
const client = new VectorizerClient("http://localhost:15002");
```

### With Custom Options

```typescript
const client = new VectorizerClient("http://localhost:15002", {
  timeout: 30000, // milliseconds
  retries: 3,
});
```

### With Authentication

```typescript
const client = new VectorizerClient("http://localhost:15002", {
  apiKey: "your-api-key",
});
```

## Collection Operations

### Create Collection

```typescript
await client.createCollection("my_collection", {
  dimension: 384,
  metric: "cosine",
});
```

### List Collections

```typescript
const collections = await client.listCollections();
collections.forEach((collection) => {
  console.log(
    `Collection: ${collection.name}, Vectors: ${collection.vector_count}`
  );
});
```

### Get Collection Info

```typescript
const info = await client.getCollectionInfo("my_collection");
console.log(`Dimension: ${info.dimension}`);
console.log(`Metric: ${info.metric}`);
console.log(`Vector count: ${info.vector_count}`);
```

### Delete Collection

```typescript
await client.deleteCollection("my_collection");
```

## Vector Operations

### Insert Single Text

```typescript
const vectorId = await client.insertText(
  "my_collection",
  "Vectorizer is a high-performance vector database",
  { source: "readme.md" }
);
```

### Insert with Custom ID

```typescript
await client.insertText("my_collection", "Content here", {
  id: "custom_id_001",
  category: "docs",
});
```

### Batch Insert

```typescript
const texts = ["Doc 1", "Doc 2", "Doc 3"];
const metadatas = [{ id: 1 }, { id: 2 }, { id: 3 }];
const vectorIds = await client.batchInsertText(
  "my_collection",
  texts,
  metadatas
);
```

### Get Vector

```typescript
const vector = await client.getVector("my_collection", "vector_id");
console.log(`ID: ${vector.id}`);
console.log(`Metadata: ${vector.metadata}`);
```

### Update Vector

```typescript
await client.updateVector("my_collection", "vector_id", {
  text: "Updated content",
  metadata: { updated_at: "2024-01-01" },
});
```

### Delete Vector

```typescript
await client.deleteVector("my_collection", "vector_id");
```

### Batch Delete

```typescript
const idsToDelete = ["id1", "id2", "id3"];
await client.batchDelete("my_collection", idsToDelete);
```

## Search Operations

### Basic Search

```typescript
const results = await client.search("my_collection", "machine learning", {
  limit: 10,
});
```

### Search with Filters

```typescript
const results = await client.search("my_collection", "python tutorial", {
  limit: 10,
  filter: { category: "tutorial", language: "python" },
});
```

### Search with Similarity Threshold

```typescript
const results = await client.search("my_collection", "query", {
  limit: 10,
  similarity_threshold: 0.7,
});
```

### Intelligent Search

```typescript
const results = await client.intelligentSearch({
  collection: "my_collection",
  query: "neural networks",
  max_results: 15,
  mmr_enabled: true,
  mmr_lambda: 0.7,
  domain_expansion: true,
  technical_focus: true,
});
```

### Semantic Search

```typescript
const results = await client.semanticSearch({
  collection: "my_collection",
  query: "deep learning",
  max_results: 10,
  semantic_reranking: true,
  similarity_threshold: 0.15,
});
```

### Hybrid Search

```typescript
const hybridResults = await client.hybridSearch({
  collection: "my_collection",
  query: "vector database",
  query_sparse: {
    indices: [0, 5, 10],
    values: [0.8, 0.6, 0.9],
  },
  alpha: 0.7,
  algorithm: "rrf",
  dense_k: 20,
  sparse_k: 20,
  final_k: 10,
});
```

### Multi-Collection Search

```typescript
const results = await client.multiCollectionSearch({
  query: "authentication",
  collections: ["docs", "code", "wiki"],
  max_results: 20,
  max_per_collection: 5,
});
```

## Qdrant Compatibility

### List Collections (Qdrant Format)

```typescript
const collections = await client.qdrantListCollections();
```

### Get Collection (Qdrant Format)

```typescript
const collection = await client.qdrantGetCollection("my_collection");
```

### Upsert Points

```typescript
const points = [
  {
    id: "point1",
    vector: [0.1, 0.2, 0.3],
    payload: { text: "content" },
  },
];
await client.qdrantUpsertPoints("my_collection", points);
```

### Search Points

```typescript
const results = await client.qdrantSearchPoints(
  "my_collection",
  [0.1, 0.2, 0.3],
  10,
  { with_payload: true }
);
```

### Delete Points

```typescript
await client.qdrantDeletePoints("my_collection", ["point1", "point2"]);
```

### Retrieve Points

```typescript
const points = await client.qdrantRetrievePoints(
  "my_collection",
  ["point1", "point2"],
  { with_payload: true }
);
```

### Count Points

```typescript
const count = await client.qdrantCountPoints("my_collection");
console.log(`Total points: ${count}`);
```

## TypeScript Types

The SDK includes full TypeScript types:

```typescript
import {
  VectorizerClient,
  SearchResult,
  CollectionInfo,
  HybridSearchRequest,
  SparseVector,
} from "@hivellm/vectorizer-sdk";

async function searchDocuments(
  client: VectorizerClient,
  query: string
): Promise<SearchResult[]> {
  return await client.search("my_collection", query, { limit: 10 });
}
```

## Error Handling

```typescript
import { VectorizerError } from "@hivellm/vectorizer-sdk";

try {
  await client.createCollection("my_collection", { dimension: 384 });
} catch (error) {
  if (error instanceof VectorizerError) {
    console.error(`Error: ${error.message}`);
  }
}
```

## Best Practices

1. **Use TypeScript**: Full type safety and IDE support
2. **Use async/await**: All operations are async
3. **Use batch operations**: Much faster for multiple operations
4. **Handle errors**: Wrap operations in try/catch
5. **Reuse client**: Create client once and reuse it

## Related Topics

- [Python SDK](./PYTHON.md) - Python SDK
- [Rust SDK](./RUST.md) - Rust SDK
- [JavaScript SDK](./JAVASCRIPT.md) - JavaScript SDK
- [Collections Guide](../collections/COLLECTIONS.md) - Collection operations
- [Search Guide](../search/SEARCH.md) - Search operations
