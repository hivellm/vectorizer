---
title: Practical Examples
module: examples
id: examples-guide
order: 1
description: Real-world examples and code samples for Vectorizer
tags: [examples, tutorials, code-samples, practical]
---

# Practical Examples

Real-world examples showing how to use Vectorizer in common scenarios.

## Example 1: Document Search System

Build a document search system with metadata filtering.

### Setup

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Create collection for documents
await client.create_collection(
    "documents",
    dimension=384,
    metric="cosine"
)
```

### Index Documents

```python
documents = [
    {
        "id": "doc_001",
        "content": "Python is a high-level programming language...",
        "title": "Python Introduction",
        "category": "programming",
        "author": "John Doe",
        "date": "2024-01-01"
    },
    {
        "id": "doc_002",
        "content": "Rust is a systems programming language...",
        "title": "Rust Guide",
        "category": "programming",
        "author": "Jane Smith",
        "date": "2024-01-02"
    }
]

# Insert documents
for doc in documents:
    await client.insert_text(
        "documents",
        doc["content"],
        id=doc["id"],
        metadata={
            "title": doc["title"],
            "category": doc["category"],
            "author": doc["author"],
            "date": doc["date"]
        }
    )
```

### Search Documents

```python
# Simple search
results = await client.search(
    "documents",
    "programming languages",
    limit=10
)

# Search with category filter
results = await client.search(
    "documents",
    "programming",
    limit=10,
    filter={"category": "programming"}
)

# Search with multiple filters
results = await client.search(
    "documents",
    "python",
    limit=10,
    filter={
        "category": "programming",
        "author": "John Doe"
    }
)
```

## Example 2: Code Search Engine

Build a semantic code search engine.

### Setup

```python
# Create collection for code
await client.create_collection(
    "code",
    dimension=768,  # Higher dimension for code embeddings
    metric="cosine"
)
```

### Index Code Files

```python
code_files = [
    {
        "file": "src/main.rs",
        "content": "fn main() { println!(\"Hello, world!\"); }",
        "language": "rust",
        "function": "main"
    },
    {
        "file": "src/lib.rs",
        "content": "pub fn add(a: i32, b: i32) -> i32 { a + b }",
        "language": "rust",
        "function": "add"
    }
]

for code in code_files:
    await client.insert_text(
        "code",
        code["content"],
        metadata={
            "file": code["file"],
            "language": code["language"],
            "function": code["function"]
        }
    )
```

### Search Code

```python
# Use intelligent search for code discovery
results = await client.intelligent_search(
    "code",
    "function that adds numbers",
    max_results=10,
    technical_focus=True
)

# Search by language
results = await client.search(
    "code",
    "hello world",
    limit=10,
    filter={"language": "rust"}
)
```

## Example 3: Recommendation System

Build a content recommendation system.

### Setup

```python
await client.create_collection(
    "recommendations",
    dimension=384,
    metric="cosine"
)
```

### Index Content

```python
content_items = [
    {
        "id": "item_001",
        "title": "Machine Learning Basics",
        "description": "Introduction to machine learning...",
        "tags": ["ml", "ai", "tutorial"],
        "category": "education"
    },
    {
        "id": "item_002",
        "title": "Deep Learning Advanced",
        "description": "Advanced deep learning techniques...",
        "tags": ["dl", "ai", "advanced"],
        "category": "education"
    }
]

for item in content_items:
    text = f"{item['title']} {item['description']}"
    await client.insert_text(
        "recommendations",
        text,
        id=item["id"],
        metadata={
            "title": item["title"],
            "tags": item["tags"],
            "category": item["category"]
        }
    )
```

### Get Recommendations

```python
# Find similar content
def get_recommendations(item_id, limit=5):
    # Get the item
    item = await client.get_vector("recommendations", item_id)
    
    # Search for similar items
    results = await client.search(
        "recommendations",
        item["vector"],  # Use item's vector
        limit=limit + 1  # +1 to exclude the item itself
    )
    
    # Filter out the item itself
    return [r for r in results if r["id"] != item_id][:limit]

# Get recommendations for an item
recommendations = await get_recommendations("item_001")
```

## Example 4: Question Answering System

Build a Q&A system with RAG (Retrieval Augmented Generation).

### Setup

```python
await client.create_collection(
    "knowledge_base",
    dimension=384,
    metric="cosine"
)
```

### Index Knowledge Base

```python
qa_pairs = [
    {
        "question": "What is Vectorizer?",
        "answer": "Vectorizer is a high-performance vector database...",
        "category": "general"
    },
    {
        "question": "How do I install Vectorizer?",
        "answer": "You can install Vectorizer using the install script...",
        "category": "installation"
    }
]

for qa in qa_pairs:
    # Index both question and answer
    text = f"Q: {qa['question']} A: {qa['answer']}"
    await client.insert_text(
        "knowledge_base",
        text,
        metadata={
            "question": qa["question"],
            "answer": qa["answer"],
            "category": qa["category"]
        }
    )
```

### Answer Questions

```python
async def answer_question(question):
    # Search for relevant answers
    results = await client.semantic_search(
        "knowledge_base",
        question,
        max_results=3,
        similarity_threshold=0.3
    )
    
    if not results:
        return "I couldn't find a relevant answer."
    
    # Get the best answer
    best_result = results[0]
    answer = best_result["payload"]["answer"]
    
    return answer

# Use it
answer = await answer_question("How do I install Vectorizer?")
print(answer)
```

## Example 5: Multi-Collection Search

Search across multiple collections simultaneously.

### Setup Multiple Collections

```python
# Create collections for different data types
await client.create_collection("documents", dimension=384)
await client.create_collection("code", dimension=384)
await client.create_collection("wiki", dimension=384)
```

### Search Across Collections

```python
# Search all collections
results = await client.multi_collection_search(
    query="authentication",
    collections=["documents", "code", "wiki"],
    max_results=20,
    max_per_collection=5
)

# Process results by collection
for result in results:
    collection = result["collection"]
    content = result["payload"]["content"]
    print(f"[{collection}] {content}")
```

## Example 6: Hybrid Search with Keywords

Combine semantic and keyword search.

### Setup

```python
await client.create_collection(
    "hybrid_collection",
    dimension=384,
    metric="cosine"
)
```

### Index with Both Dense and Sparse

```python
# Insert document with sparse keywords
await client.insert_text(
    "hybrid_collection",
    "Vectorizer is a high-performance vector database",
    sparse={
        "indices": [0, 5, 10],  # Keyword positions
        "values": [0.8, 0.6, 0.9]  # Keyword weights
    },
    metadata={"keywords": ["vector", "database", "performance"]}
)
```

### Hybrid Search

```python
from vectorizer_sdk import HybridSearchRequest, SparseVector

# Create sparse query for keywords
sparse_query = SparseVector(
    indices=[0, 5, 10],
    values=[0.8, 0.6, 0.9]
)

# Hybrid search
results = await client.hybrid_search(
    HybridSearchRequest(
        collection="hybrid_collection",
        query="vector database",
        query_sparse=sparse_query,
        alpha=0.7,  # 70% semantic, 30% keyword
        algorithm="rrf"
    )
)
```

## Example 7: Batch Processing

Efficiently process large datasets.

### Batch Insert

```python
def process_large_dataset(documents, batch_size=100):
    """Process large dataset in batches."""
    client = VectorizerClient("http://localhost:15002")
    
    for i in range(0, len(documents), batch_size):
        batch = documents[i:i + batch_size]
        
        texts = [doc["content"] for doc in batch]
        metadatas = [doc["metadata"] for doc in batch]
        
        await client.batch_insert_text(
            "documents",
            texts,
            metadatas
        )
        
        print(f"Processed {min(i + batch_size, len(documents))}/{len(documents)}")
```

### Batch Update

```python
async def update_documents(updates):
    """Batch update multiple documents."""
    client = VectorizerClient("http://localhost:15002")
    
    await client.batch_update(
        "documents",
        [
            {
                "id": update["id"],
                "text": update["new_content"],
                "metadata": update["new_metadata"]
            }
            for update in updates
        ]
    )
```

## Example 8: Real-time Search

Build a real-time search interface.

### Setup with Auto-refresh

```python
import asyncio
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

async def real_time_search(query, collection="documents"):
    """Perform real-time search with auto-refresh."""
    while True:
        results = await client.search(
            collection,
            query,
            limit=10
        )
        
        yield results
        
        # Wait before next search
        await asyncio.sleep(1)

# Use it
async for results in real_time_search("vector database"):
    print(f"Found {len(results)} results")
    # Update UI with results
```

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection setup
- [Search Guide](../search/SEARCH.md) - Search methods
- [Vectors Guide](../vectors/VECTORS.md) - Vector operations
- [SDKs Guide](../sdks/SDKS.md) - SDK usage

