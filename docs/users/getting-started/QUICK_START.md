---
title: Quick Start Guide
module: getting-started
id: quick-start-guide
order: 1
description: Get up and running with Vectorizer in minutes
tags: [quick-start, tutorial, getting-started]
---

# Quick Start Guide

Get up and running with Vectorizer in minutes!

## Prerequisites

- Vectorizer installed (see [Installation Guide](../installation/INSTALLATION.md))
- Service running on `http://localhost:15002`
- `curl` or similar HTTP client (or use the SDKs)

## Step 1: Create Your First Collection

```bash
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_documents",
    "dimension": 384,
    "metric": "cosine"
  }'
```

## Step 2: Insert Documents

```bash
curl -X POST http://localhost:15002/collections/my_documents/insert \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Vectorizer is a high-performance vector database",
    "metadata": {"source": "readme"}
  }'
```

## Step 3: Search

```bash
curl -X POST http://localhost:15002/collections/my_documents/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "vector database",
    "limit": 5
  }'
```

## Using SDKs

### Python SDK

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")
await client.create_collection("my_docs", dimension=384)
await client.insert_text("my_docs", "Vectorizer is awesome!")
results = await client.search("my_docs", "vector database", limit=5)
```

### TypeScript/JavaScript SDK

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

const client = new VectorizerClient("http://localhost:15002");
await client.createCollection("my_docs", { dimension: 384 });
await client.insertText("my_docs", "Vectorizer is awesome!");
const results = await client.search("my_docs", "vector database", { limit: 5 });
```

### Rust SDK

```rust
use vectorizer_sdk::VectorizerClient;

let client = VectorizerClient::new("http://localhost:15002")?;
client.create_collection("my_docs", 384).await?;
client.insert_text("my_docs", "Vectorizer is awesome!").await?;
let results = client.search("my_docs", "vector database", 5).await?;
```

## Next Steps

- [Collections Guide](../collections/COLLECTIONS.md) - Learn about collections
- [Search Guide](../search/SEARCH.md) - Advanced search features
- [SDKs Guide](../sdks/SDKS.md) - Complete SDK documentation
