# Batch Operations - Usage Examples

This document provides practical examples of how batch operations will be used in the Vectorizer system, demonstrating the significant performance and efficiency improvements for AI models and applications.

## 1. AI Model Workflow Examples

### 1.1 Bulk Document Indexing

**Scenario**: An AI model needs to index 1,000 documents from a dataset.

**Traditional Approach** (1,000 individual API calls):
```python
# Old way - 1,000 separate calls
for document in documents:
    vector = embed_text(document.text)
    insert_vector(collection="documents", id=document.id, data=vector, metadata=document.metadata)
    # Each call takes ~50ms = 50 seconds total
```

**Batch Approach** (1 API call):
```python
# New way - 1 batch call
vectors = []
for document in documents:
    vector = embed_text(document.text)
    vectors.append({
        "id": document.id,
        "data": vector,
        "metadata": document.metadata
    })

# Single batch insert - takes ~200ms total
batch_insert_texts(collection="documents", vectors=vectors, atomic=True)
# 250x performance improvement!
```

### 1.2 Multi-Query Search

**Scenario**: An AI model needs to search for multiple concepts simultaneously.

**Traditional Approach**:
```python
# Old way - 10 separate search calls
results = []
for query in queries:
    result = search_vectors(collection="knowledge", query=query, limit=5)
    results.append(result)
    # 10 calls × 10ms = 100ms total
```

**Batch Approach**:
```python
# New way - 1 batch search call
batch_results = batch_search_vectors(
    collection="knowledge",
    queries=[
        {"query_text": "machine learning algorithms", "limit": 5},
        {"query_text": "neural network architectures", "limit": 5},
        {"query_text": "natural language processing", "limit": 5}
    ],
    atomic=False  # Partial results are acceptable
)
# Single call takes ~15ms total - 6.7x improvement!
```

### 1.3 Dataset Updates

**Scenario**: Updating metadata for 500 vectors based on new analysis.

**Traditional Approach**:
```python
# Old way - 500 separate update calls
for vector_id, new_metadata in updates.items():
    update_vector(collection="analysis", id=vector_id, metadata=new_metadata)
    # 500 calls × 30ms = 15 seconds total
```

**Batch Approach**:
```python
# New way - 1 batch update call
batch_updates = [
    {"id": vector_id, "metadata": new_metadata}
    for vector_id, new_metadata in updates.items()
]

batch_update_vectors(collection="analysis", updates=batch_updates, atomic=True)
# Single call takes ~300ms total - 50x improvement!
```

## 2. MCP Tool Usage Examples

### 2.1 Batch Vector Insert via MCP

```javascript
// Using MCP in Cursor IDE or other MCP-compatible tools
const batchInsertResult = await mcp_hive_vectorizer_batch_insert_texts({
  collection: "research_papers",
  vectors: [
    {
      id: "paper_001",
      data: [0.1, 0.2, 0.3, /* ... 512 dimensions */],
      metadata: {
        title: "Advanced Machine Learning Techniques",
        authors: ["John Doe", "Jane Smith"],
        year: 2024,
        category: "AI/ML"
      }
    },
    {
      id: "paper_002", 
      data: [0.4, 0.5, 0.6, /* ... 512 dimensions */],
      metadata: {
        title: "Natural Language Processing Advances",
        authors: ["Alice Johnson"],
        year: 2024,
        category: "NLP"
      }
    }
    // ... up to 1000 vectors in single call
  ],
  atomic: true,
  batch_size_limit: 1000
});

console.log(`Inserted ${batchInsertResult.inserted_count} vectors in ${batchInsertResult.processing_time_ms}ms`);
```

### 2.2 Batch Search via MCP

```javascript
// Multi-concept search in single call
const searchResults = await mcp_hive_vectorizer_batch_search_vectors({
  collection: "knowledge_base",
  queries: [
    {
      query_text: "transformer architecture attention mechanism",
      limit: 10,
      threshold: 0.8
    },
    {
      query_text: "reinforcement learning policy gradient",
      limit: 5,
      threshold: 0.85
    },
    {
      query_text: "computer vision object detection YOLO",
      limit: 8,
      threshold: 0.75
    }
  ],
  atomic: false
});

// Process results for each query
searchResults.batch_results.forEach((result, index) => {
  console.log(`Query ${index + 1} found ${result.results.length} matches`);
  result.results.forEach(match => {
    console.log(`- ${match.id}: ${match.score} (${match.metadata.title})`);
  });
});
```

## 3. REST API Examples

### 3.1 Batch Insert via REST

```bash
# Insert 100 vectors in single HTTP call
curl -X POST "http://localhost:15001/api/v1/collections/documents/vectors/batch" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "vectors": [
      {
        "id": "doc_001",
        "data": [0.1, 0.2, 0.3],
        "metadata": {"title": "Document 1", "category": "tech"}
      },
      {
        "id": "doc_002", 
        "data": [0.4, 0.5, 0.6],
        "metadata": {"title": "Document 2", "category": "science"}
      }
    ],
    "atomic": true,
    "batch_size_limit": 1000
  }'
```

**Response:**
```json
{
  "inserted_count": 100,
  "failed_count": 0,
  "errors": [],
  "processing_time_ms": 45.2,
  "status": "success"
}
```

### 3.2 Batch Delete via REST

```bash
# Delete 50 vectors in single HTTP call
curl -X DELETE "http://localhost:15001/api/v1/collections/documents/vectors/batch" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "vector_ids": ["doc_001", "doc_002", "doc_003", /* ... 47 more */],
    "atomic": true
  }'
```

## 4. Performance Comparison

### 4.1 Insert Operations

| Operation Count | Traditional (ms) | Batch (ms) | Improvement |
|----------------|------------------|------------|-------------|
| 10 vectors     | 500             | 25         | 20x         |
| 100 vectors    | 5,000           | 80         | 62.5x       |
| 1,000 vectors  | 50,000          | 200        | 250x        |
| 10,000 vectors | 500,000         | 1,500      | 333x        |

### 4.2 Search Operations

| Query Count | Traditional (ms) | Batch (ms) | Improvement |
|-------------|------------------|------------|-------------|
| 5 queries   | 50              | 15         | 3.3x        |
| 10 queries  | 100             | 25         | 4x          |
| 50 queries  | 500             | 80         | 6.25x       |
| 100 queries | 1,000           | 120        | 8.3x        |

### 4.3 Update Operations

| Update Count | Traditional (ms) | Batch (ms) | Improvement |
|--------------|------------------|------------|-------------|
| 10 updates   | 300             | 20         | 15x         |
| 100 updates  | 3,000           | 60         | 50x         |
| 1,000 updates| 30,000          | 300        | 100x        |

## 5. Error Handling Examples

### 5.1 Partial Success Handling

```python
# Batch operation with some failures
result = batch_insert_texts(
    collection="test",
    vectors=[
        {"id": "valid_1", "data": [0.1, 0.2, 0.3]},  # Success
        {"id": "invalid", "data": []},                # Failure - empty vector
        {"id": "valid_2", "data": [0.4, 0.5, 0.6]}   # Success
    ],
    atomic=False  # Allow partial success
)

print(f"Status: {result.status}")  # "partial"
print(f"Inserted: {result.inserted_count}")  # 2
print(f"Failed: {result.failed_count}")      # 1
print(f"Errors: {result.errors}")            # [{"operation_id": "invalid", "error_code": "INVALID_VECTOR", "error_message": "Vector data cannot be empty"}]
```

### 5.2 Atomic Transaction Example

```python
# All-or-nothing batch operation
result = batch_update_vectors(
    collection="critical_data",
    updates=[
        {"id": "item_1", "metadata": {"status": "processed"}},
        {"id": "item_2", "metadata": {"status": "processed"}},
        {"id": "nonexistent", "metadata": {"status": "processed"}}  # This will fail
    ],
    atomic=True  # All operations must succeed
)

print(f"Status: {result.status}")  # "failed"
print(f"Updated: {result.updated_count}")  # 0
print(f"Failed: {result.failed_count}")    # 3 (all failed due to atomic constraint)
```

## 6. Real-World Use Cases

### 6.1 AI Model Training Data Management

```python
# Loading training examples in batch
def load_training_batch(examples):
    vectors = []
    for example in examples:
        vector = model.encode(example.text)
        vectors.append({
            "id": f"train_{example.id}",
            "data": vector,
            "metadata": {
                "label": example.label,
                "source": example.dataset,
                "timestamp": example.created_at
            }
        })
    
    # Insert 1000 training examples in single call
    result = batch_insert_texts(
        collection="training_data",
        vectors=vectors,
        atomic=True
    )
    
    return result.inserted_count
```

### 6.2 Knowledge Base Updates

```python
# Batch update knowledge base with new information
def update_knowledge_base(updates):
    batch_updates = []
    for update in updates:
        batch_updates.append({
            "id": update.vector_id,
            "metadata": {
                **update.current_metadata,
                "last_updated": datetime.now().isoformat(),
                "confidence": update.new_confidence,
                "source": update.new_source
            }
        })
    
    # Update 500 knowledge entries atomically
    result = batch_update_vectors(
        collection="knowledge_base",
        updates=batch_updates,
        atomic=True
    )
    
    if result.status == "success":
        print(f"Successfully updated {result.updated_count} knowledge entries")
    else:
        print(f"Update failed: {result.errors}")
```

### 6.3 Multi-Model Search and Retrieval

```python
# Search multiple concepts for comprehensive retrieval
def comprehensive_search(concepts, collection="research"):
    queries = []
    for concept in concepts:
        queries.append({
            "query_text": concept,
            "limit": 10,
            "threshold": 0.8
        })
    
    # Search all concepts in parallel
    results = batch_search_vectors(
        collection=collection,
        queries=queries,
        atomic=False
    )
    
    # Combine and rank results
    all_results = []
    for i, concept_result in enumerate(results.batch_results):
        for match in concept_result.results:
            match["source_concept"] = concepts[i]
            all_results.append(match)
    
    # Sort by score and remove duplicates
    unique_results = {}
    for result in sorted(all_results, key=lambda x: x["score"], reverse=True):
        if result["id"] not in unique_results:
            unique_results[result["id"]] = result
    
    return list(unique_results.values())
```

## 7. Configuration Examples

### 7.1 High-Performance Configuration

```yaml
# For maximum throughput
batch_operations:
  max_batch_size: 10000
  max_memory_usage_mb: 2048
  parallel_workers: 16
  chunk_size: 1000
  atomic_by_default: false
  progress_reporting: true
```

### 7.2 Memory-Conscious Configuration

```yaml
# For memory-constrained environments
batch_operations:
  max_batch_size: 1000
  max_memory_usage_mb: 256
  parallel_workers: 2
  chunk_size: 100
  atomic_by_default: true
  progress_reporting: false
```

### 7.3 Development Configuration

```yaml
# For development and testing
batch_operations:
  max_batch_size: 100
  max_memory_usage_mb: 128
  parallel_workers: 1
  chunk_size: 10
  atomic_by_default: true
  progress_reporting: true
```

---

These examples demonstrate how batch operations will dramatically improve the efficiency and usability of the Vectorizer system for AI models and applications, providing 10-300x performance improvements for bulk operations while maintaining data consistency and error handling capabilities.
