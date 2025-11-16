---
title: Document Search System
module: use-cases
id: document-search
order: 1
description: Build a document search system with Vectorizer
tags: [use-cases, document-search, semantic-search, examples]
---

# Document Search System

Build a production-ready document search system using Vectorizer.

## Overview

This use case demonstrates how to build a semantic document search system that can:

- Index large collections of documents
- Search by meaning, not just keywords
- Filter by metadata (category, author, date, etc.)
- Scale to millions of documents

## Architecture

```
Documents → Text Extraction → Embedding → Vectorizer → Search API
```

## Implementation

### Step 1: Create Collection

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Create optimized collection for documents
await client.create_collection(
    "documents",
    dimension=384,  # BGE-small or MiniLM embeddings
    metric="cosine",
    quantization={"enabled": True, "type": "scalar", "bits": 8},
    hnsw_config={"m": 16, "ef_construction": 200, "ef_search": 64}
)
```

### Step 2: Index Documents

```python
async def index_documents(documents):
    """Index a batch of documents."""
    texts = []
    metadatas = []

    for doc in documents:
        texts.append(doc["content"])
        metadatas.append({
            "title": doc["title"],
            "category": doc["category"],
            "author": doc["author"],
            "date": doc["date"],
            "source": doc.get("source", "unknown")
        })

    # Batch insert for efficiency
    await client.batch_insert_text("documents", texts, metadatas)
    print(f"Indexed {len(documents)} documents")

# Example usage
documents = [
    {
        "content": "Python is a high-level programming language...",
        "title": "Python Introduction",
        "category": "programming",
        "author": "John Doe",
        "date": "2024-01-01"
    },
    # ... more documents
]

await index_documents(documents)
```

### Step 3: Search Documents

```python
async def search_documents(query, category=None, limit=10):
    """Search documents with optional filtering."""
    results = await client.search(
        "documents",
        query,
        limit=limit,
        filter={"category": category} if category else None,
        with_payload=True
    )

    return [
        {
            "id": r["id"],
            "score": r["score"],
            "title": r["payload"]["title"],
            "content": r["payload"].get("content", ""),
            "author": r["payload"]["author"],
            "date": r["payload"]["date"]
        }
        for r in results
    ]

# Search all documents
results = await search_documents("machine learning algorithms")

# Search in specific category
results = await search_documents("async programming", category="programming")
```

### Step 4: Advanced Search with Intelligent Search

```python
async def intelligent_document_search(query, max_results=15):
    """Use intelligent search for better results."""
    results = await client.intelligent_search(
        "documents",
        query,
        max_results=max_results,
        mmr_enabled=True,
        mmr_lambda=0.7,
        domain_expansion=True,
        technical_focus=True
    )

    return results

results = await intelligent_document_search("neural networks")
```

## Performance Optimization

### Batch Indexing

```python
async def bulk_index_documents(documents, batch_size=1000):
    """Index large document collections efficiently."""
    for i in range(0, len(documents), batch_size):
        batch = documents[i:i + batch_size]
        await index_documents(batch)
        print(f"Indexed {min(i + batch_size, len(documents))}/{len(documents)} documents")
```

### Caching Frequent Queries

```python
from functools import lru_cache
import hashlib

@lru_cache(maxsize=100)
async def cached_search(query_hash, query, limit):
    """Cache search results for frequent queries."""
    return await client.search("documents", query, limit=limit)

async def search_with_cache(query, limit=10):
    """Search with automatic caching."""
    query_hash = hashlib.md5(query.encode()).hexdigest()
    return await cached_search(query_hash, query, limit)
```

## Real-World Example

```python
import asyncio
from vectorizer_sdk import VectorizerClient

async def main():
    client = VectorizerClient("http://localhost:15002")

    # Create collection
    await client.create_collection("docs", dimension=384)

    # Index documents
    docs = [
        {"content": "Vectorizer is a high-performance vector database...", "title": "Vectorizer Docs"},
        {"content": "Python async programming guide...", "title": "Python Guide"},
    ]
    await index_documents(docs)

    # Search
    results = await search_documents("vector database")
    for r in results:
        print(f"{r['title']}: {r['score']:.3f}")

asyncio.run(main())
```

## Best Practices

1. **Use batch operations**: Index documents in batches of 100-1000
2. **Enable quantization**: Reduces memory by 75% with minimal quality loss
3. **Filter by metadata**: Use filters to narrow search results
4. **Monitor performance**: Track search latency and throughput
5. **Cache frequent queries**: Improve response time for common searches

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection configuration
- [Search Guide](../search/SEARCH.md) - Advanced search features
- [Performance Guide](../configuration/PERFORMANCE_TUNING.md) - Optimization tips
