---
title: JavaScript SDK
module: sdks
id: javascript-sdk
order: 3
description: Complete JavaScript SDK guide for Vectorizer
tags: [javascript, sdk, client-library]
---

# JavaScript SDK

Complete guide to using the Vectorizer JavaScript SDK.

## Installation

```bash
npm install @hivellm/vectorizer-sdk-js
```

## Quick Start

```javascript
const { VectorizerClient } = require("@hivellm/vectorizer-sdk-js");

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

```javascript
const client = new VectorizerClient("http://localhost:15002");
```

### With Custom Options

```javascript
const client = new VectorizerClient("http://localhost:15002", {
    timeout: 30000,  // milliseconds
    retries: 3
});
```

## Collection Operations

### Create Collection

```javascript
await client.createCollection("my_collection", {
    dimension: 384,
    metric: "cosine"
});
```

### List Collections

```javascript
const collections = await client.listCollections();
collections.forEach(collection => {
    console.log(`Collection: ${collection.name}`);
});
```

### Get Collection Info

```javascript
const info = await client.getCollectionInfo("my_collection");
console.log(`Dimension: ${info.dimension}`);
```

### Delete Collection

```javascript
await client.deleteCollection("my_collection");
```

## Vector Operations

### Insert Single Text

```javascript
const vectorId = await client.insertText(
    "my_collection",
    "Vectorizer is awesome!",
    { source: "readme.md" }
);
```

### Batch Insert

```javascript
const texts = ["Doc 1", "Doc 2", "Doc 3"];
const metadatas = [{ id: 1 }, { id: 2 }, { id: 3 }];
const vectorIds = await client.batchInsertText("my_collection", texts, metadatas);
```

### Get Vector

```javascript
const vector = await client.getVector("my_collection", "vector_id");
```

### Update Vector

```javascript
await client.updateVector("my_collection", "vector_id", {
    text: "Updated content"
});
```

### Delete Vector

```javascript
await client.deleteVector("my_collection", "vector_id");
```

## Search Operations

### Basic Search

```javascript
const results = await client.search("my_collection", "query", { limit: 10 });
```

### Intelligent Search

```javascript
const results = await client.intelligentSearch({
    collection: "my_collection",
    query: "neural networks",
    max_results: 15
});
```

### Hybrid Search

```javascript
const results = await client.hybridSearch({
    collection: "my_collection",
    query: "vector database",
    query_sparse: {
        indices: [0, 5, 10],
        values: [0.8, 0.6, 0.9]
    },
    alpha: 0.7
});
```

## Error Handling

```javascript
try {
    await client.createCollection("my_collection", { dimension: 384 });
} catch (error) {
    console.error(`Error: ${error.message}`);
}
```

## Related Topics

- [Python SDK](./PYTHON.md) - Python SDK
- [TypeScript SDK](./TYPESCRIPT.md) - TypeScript SDK
- [Rust SDK](./RUST.md) - Rust SDK

