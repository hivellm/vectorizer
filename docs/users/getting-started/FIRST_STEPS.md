---
title: First Steps After Installation
module: getting-started
id: first-steps
order: 2
description: What to do after installing Vectorizer
tags: [getting-started, tutorial, first-steps, setup]
---

# First Steps After Installation

Complete guide to getting started with Vectorizer after installation.

## Verify Installation

### Check Service Status

**Linux:**

```bash
sudo systemctl status vectorizer
```

**Windows:**

```powershell
Get-Service Vectorizer
```

**Expected output:** Service should be running.

### Test API Endpoint

```bash
curl http://localhost:15002/health
```

**Expected response:**

```json
{
  "status": "healthy",
  "version": "1.3.0"
}
```

### Check CLI

```bash
vectorizer --version
```

**Expected output:** Version number (e.g., `1.3.0`)

## Create Your First Collection

### Using cURL

```bash
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_first_collection",
    "dimension": 384,
    "metric": "cosine"
  }'
```

### Using Python SDK

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")
await client.create_collection("my_first_collection", dimension=384)
```

### Verify Collection Created

```bash
curl http://localhost:15002/collections
```

You should see your collection in the list.

## Insert Your First Vectors

### Insert Single Vector

**Using cURL:**

```bash
curl -X POST http://localhost:15002/collections/my_first_collection/insert \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Vectorizer is a high-performance vector database",
    "metadata": {
      "source": "readme",
      "category": "introduction"
    }
  }'
```

**Using Python SDK:**

```python
await client.insert_text(
    "my_first_collection",
    "Vectorizer is a high-performance vector database",
    metadata={
        "source": "readme",
        "category": "introduction"
    }
)
```

### Insert Multiple Vectors

**Using Python SDK:**

```python
texts = [
    "Vectorizer supports semantic search",
    "Vectorizer supports hybrid search",
    "Vectorizer is written in Rust"
]

await client.batch_insert_text("my_first_collection", texts)
```

## Perform Your First Search

### Basic Search

**Using cURL:**

```bash
curl -X POST http://localhost:15002/collections/my_first_collection/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "vector database",
    "limit": 5
  }'
```

**Using Python SDK:**

```python
results = await client.search(
    "my_first_collection",
    "vector database",
    limit=5
)

for result in results:
    print(f"ID: {result['id']}, Score: {result['score']}")
```

## Next Steps

### 1. Explore Collections

- [Creating Collections](../collections/CREATING.md) - Learn how to create collections
- [Collection Configuration](../collections/CONFIGURATION.md) - Configure collections
- [Collection Operations](../collections/OPERATIONS.md) - Manage collections

### 2. Learn Search

- [Basic Search](../search/BASIC.md) - Simple search operations
- [Advanced Search](../search/ADVANCED.md) - Intelligent and hybrid search

### 3. Use SDKs

- [Python SDK](../sdks/PYTHON.md) - Python client library
- [TypeScript SDK](../sdks/TYPESCRIPT.md) - TypeScript/JavaScript client
- [Rust SDK](../sdks/RUST.md) - Rust client library

### 4. Explore Use Cases

- [Document Search](../use-cases/DOCUMENT_SEARCH.md) - Build a document search system
- [Recommendation System](../use-cases/RECOMMENDATION_SYSTEM.md) - Content-based recommendations
- [Q&A System](../use-cases/QA_SYSTEM.md) - RAG-based question answering

### 5. Configure and Optimize

- [Server Configuration](../configuration/SERVER.md) - Configure server settings
- [Performance Tuning](../configuration/PERFORMANCE_TUNING.md) - Optimize performance
- [Monitoring](../operations/MONITORING.md) - Monitor your deployment

## Common Tasks

### List All Collections

```bash
curl http://localhost:15002/collections
```

### Get Collection Info

```bash
curl http://localhost:15002/collections/my_first_collection
```

### Delete Collection

```bash
curl -X DELETE http://localhost:15002/collections/my_first_collection
```

**Warning:** This permanently deletes the collection and all its vectors!

## Troubleshooting

### Service Not Running

**Linux:**

```bash
sudo systemctl start vectorizer
sudo systemctl status vectorizer
```

**Windows:**

```powershell
Start-Service Vectorizer
Get-Service Vectorizer
```

### Cannot Connect to API

1. Verify service is running
2. Check port 15002 is not blocked by firewall
3. Verify host binding (should be `0.0.0.0` or `127.0.0.1`)

### Collection Not Found

1. List collections to verify name spelling
2. Check collection name is case-sensitive
3. Verify collection was created successfully

## Related Topics

- [Quick Start Guide](./QUICK_START.md) - Quick start tutorial
- [Installation Guide](./INSTALLATION.md) - Installation details
- [Troubleshooting Guide](../operations/TROUBLESHOOTING.md) - Common issues
