---
title: SDKs Guide
module: sdks
id: sdks-guide
order: 1
description: Using Vectorizer SDKs in Python, TypeScript, JavaScript, Rust, C#, and Go
tags: [sdks, python, typescript, javascript, rust, csharp, go, client-libraries]
---

# SDKs Guide

Vectorizer v1.6.0 provides official SDKs for multiple programming languages.

## Python SDK

### Installation

```bash
pip install vectorizer-sdk
```

### Basic Usage

```python
from vectorizer_sdk import VectorizerClient

# Create client
client = VectorizerClient("http://localhost:15002")

# Create collection
await client.create_collection("my_docs", dimension=384)

# Insert text
await client.insert_text("my_docs", "Hello, Vectorizer!")

# Search
results = await client.search("my_docs", "hello", limit=5)

# Hybrid search
from vectorizer_sdk import HybridSearchRequest, SparseVector

sparse = SparseVector(indices=[0, 5], values=[0.8, 0.6])
hybrid_results = await client.hybrid_search(
    HybridSearchRequest(
        collection="my_docs",
        query="search query",
        query_sparse=sparse,
        alpha=0.7
    )
)
```

## TypeScript/JavaScript SDK

### Installation

```bash
npm install @hivellm/vectorizer-sdk
```

### Basic Usage

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

// Hybrid search
const hybridResults = await client.hybridSearch({
  collection: "my_docs",
  query: "search query",
  query_sparse: {
    indices: [0, 5],
    values: [0.8, 0.6],
  },
  alpha: 0.7,
});
```

## Rust SDK

### Installation

```toml
[dependencies]
vectorizer-sdk = "1.6.0"
```

### Basic Usage

```rust
use vectorizer_sdk::VectorizerClient;

// Create client
let client = VectorizerClient::new("http://localhost:15002")?;

// Create collection
client.create_collection("my_docs", 384).await?;

// Insert text
client.insert_text("my_docs", "Hello, Vectorizer!").await?;

// Search
let results = client.search("my_docs", "hello", 5).await?;

// Hybrid search
use vectorizer_sdk::{HybridSearchRequest, SparseVector, HybridScoringAlgorithm};

let sparse = SparseVector::new(vec![0, 5], vec![0.8, 0.6])?;
let hybrid_results = client.hybrid_search(HybridSearchRequest {
    collection: "my_docs".to_string(),
    query: "search query".to_string(),
    query_sparse: Some(sparse),
    alpha: 0.7,
    algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
    dense_k: 20,
    sparse_k: 20,
    final_k: 10,
}).await?;
```

## JavaScript SDK

### Installation

```bash
npm install @hivellm/vectorizer-sdk-js
```

### Basic Usage

```javascript
const { VectorizerClient } = require("@hivellm/vectorizer-sdk-js");

const client = new VectorizerClient("http://localhost:15002");

await client.createCollection("my_docs", { dimension: 384 });
await client.insertText("my_docs", "Hello, Vectorizer!");
const results = await client.search("my_docs", "hello", { limit: 5 });
```

## C# SDK

### Installation

```bash
dotnet add package Vectorizer.Sdk
```

### Basic Usage

```csharp
using Vectorizer.Sdk;

var client = new VectorizerClient("http://localhost:15002");

// Create collection
await client.CreateCollectionAsync("my_docs", new CreateCollectionRequest
{
    Dimension = 384
});

// Insert vectors
await client.InsertVectorsAsync("my_docs", vectors);

// Search
var results = await client.SearchAsync("my_docs", new SearchRequest
{
    Vector = queryVector,
    Limit = 5
});
```

## Go SDK

### Installation

```bash
go get github.com/hivellm/vectorizer-sdk-go
```

### Basic Usage

```go
package main

import (
    "context"
    vectorizer "github.com/hivellm/vectorizer-sdk-go"
)

func main() {
    client := vectorizer.NewClient("http://localhost:15002")
    ctx := context.Background()

    // Create collection
    client.CreateCollection(ctx, "my_docs", &vectorizer.CreateCollectionRequest{
        Dimension: 384,
    })

    // Insert vectors
    client.InsertVectors(ctx, "my_docs", vectors)

    // Search
    results, _ := client.Search(ctx, "my_docs", &vectorizer.SearchRequest{
        Vector: queryVector,
        Limit:  5,
    })
}
```

## Qdrant Compatibility

All SDKs support Qdrant-compatible API methods including Snapshots, Sharding, and Cluster APIs:

```python
# Python - Qdrant API
collections = await client.qdrant_list_collections()
results = await client.qdrant_search_points("my_collection", vector, limit=10)

# Snapshots
snapshots = await client.qdrant_list_snapshots("my_collection")
await client.qdrant_create_snapshot("my_collection")

# Sharding
shard_keys = await client.qdrant_list_shard_keys("my_collection")

# Cluster
status = await client.qdrant_cluster_status()
```

```typescript
// TypeScript - Qdrant API
const collections = await client.qdrantListCollections();
const results = await client.qdrantSearchPoints("my_collection", vector, 10);

// Snapshots
const snapshots = await client.qdrantListSnapshots("my_collection");
await client.qdrantCreateSnapshot("my_collection");

// Sharding
const shardKeys = await client.qdrantListShardKeys("my_collection");

// Cluster
const status = await client.qdrantClusterStatus();
```

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection operations
- [Search Guide](../search/SEARCH.md) - Search operations
- [Qdrant Compatibility](../qdrant/API_COMPATIBILITY.md) - Qdrant API reference
- [API Reference](../../specs/API_REFERENCE.md) - Complete REST API
