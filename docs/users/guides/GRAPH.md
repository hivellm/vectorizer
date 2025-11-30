# Graph Usage Guide

Complete guide to using Vectorizer's graph functionality for relationship discovery and querying.

## Overview

Vectorizer's graph feature enables you to discover and query relationships between vectors in your collections. When enabled, vectors automatically become nodes in a graph, and relationships (edges) can be discovered based on similarity or explicitly created.

## Enabling Graph

Graph must be enabled when creating a collection:

```json
{
  "name": "my-collection",
  "dimension": 384,
  "metric": "cosine",
  "graph": {
    "enabled": true,
    "auto_relationship": {
      "similarity_threshold": 0.7,
      "max_per_node": 10,
      "enabled_types": ["SIMILAR_TO"]
    }
  }
}
```

## Basic Concepts

### Nodes

Every vector in a collection with graph enabled becomes a node. Nodes have:
- **ID**: The vector ID
- **Type**: Node type (usually "document" or "vector")
- **Metadata**: Additional metadata from the vector payload

### Edges

Edges represent relationships between nodes. They have:
- **Source**: Source node ID
- **Target**: Target node ID
- **Relationship Type**: Type of relationship (SIMILAR_TO, REFERENCES, CONTAINS, DERIVED_FROM)
- **Weight**: Relationship strength (0.0-1.0)
- **Metadata**: Additional edge metadata

### Relationship Types

- **SIMILAR_TO**: Nodes are similar (based on vector similarity)
- **REFERENCES**: Source node references target node
- **CONTAINS**: Source node contains target node
- **DERIVED_FROM**: Source node is derived from target node

## Usage Examples

### Example 1: Document Similarity Graph

Create a graph to find similar documents:

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient(
    base_url="http://localhost:15002",
    api_key="your-api-key"
)

# Create collection with graph enabled
await client.create_collection({
    "name": "documents",
    "dimension": 384,
    "metric": "cosine",
    "graph": {
        "enabled": True,
        "auto_relationship": {
            "similarity_threshold": 0.7,
            "max_per_node": 10
        }
    }
})

# Insert documents
documents = [
    {"id": "doc1", "text": "Machine learning algorithms"},
    {"id": "doc2", "text": "Deep learning neural networks"},
    {"id": "doc3", "text": "Natural language processing"},
    {"id": "doc4", "text": "Computer vision techniques"},
    {"id": "doc5", "text": "Reinforcement learning"}
]

await client.insert_texts("documents", documents)

# Discover relationships
await client.discover_graph_edges("documents", {
    "similarity_threshold": 0.7,
    "max_per_node": 5
})

# Find related documents
related = await client.find_related_nodes(
    "documents",
    "doc1",
    {"max_hops": 2}
)

for item in related.related:
    print(f"Related: {item.node.id} (hops: {item.hops})")
```

### Example 2: Code Reference Graph

Create a graph to track code references:

```python
# Insert code files as vectors
code_files = [
    {
        "id": "file1.py",
        "text": "def calculate_sum(a, b): return a + b",
        "metadata": {"file_path": "utils.py", "language": "python"}
    },
    {
        "id": "file2.py",
        "text": "from utils import calculate_sum",
        "metadata": {"file_path": "main.py", "language": "python"}
    },
    {
        "id": "file3.py",
        "text": "import main",
        "metadata": {"file_path": "app.py", "language": "python"}
    }
]

await client.insert_texts("codebase", code_files)

# Create explicit references
await client.create_graph_edge({
    "collection": "codebase",
    "source": "file2.py",
    "target": "file1.py",
    "relationship_type": "REFERENCES",
    "weight": 1.0
})

await client.create_graph_edge({
    "collection": "codebase",
    "source": "file3.py",
    "target": "file2.py",
    "relationship_type": "REFERENCES",
    "weight": 1.0
})

# Find all files that reference a specific file
neighbors = await client.get_graph_neighbors(
    "codebase",
    "file1.py",
    {"relationship_type": "REFERENCES"}
)

for neighbor in neighbors.neighbors:
    print(f"Referenced by: {neighbor.target.id}")
```

### Example 3: Knowledge Graph

Build a knowledge graph from documents:

```python
# Insert knowledge base documents
knowledge_docs = [
    {
        "id": "concept1",
        "text": "Machine learning is a subset of artificial intelligence",
        "metadata": {"category": "AI", "concept": "machine_learning"}
    },
    {
        "id": "concept2",
        "text": "Deep learning uses neural networks",
        "metadata": {"category": "AI", "concept": "deep_learning"}
    },
    {
        "id": "concept3",
        "text": "Neural networks are inspired by the brain",
        "metadata": {"category": "AI", "concept": "neural_networks"}
    }
]

await client.insert_texts("knowledge", knowledge_docs)

# Discover similarity relationships
await client.discover_graph_edges("knowledge", {
    "similarity_threshold": 0.6,
    "max_per_node": 5
})

# Find path between concepts
path = await client.find_graph_path({
    "collection": "knowledge",
    "source": "concept1",
    "target": "concept3"
})

if path.found:
    print("Path found:")
    for node in path.path:
        print(f"  - {node.id}")
```

### Example 4: Using REST API

```bash
# 1. Create collection with graph
curl -X POST "http://localhost:15002/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "research",
    "dimension": 384,
    "metric": "cosine",
    "graph": {"enabled": true}
  }'

# 2. Insert papers
curl -X POST "http://localhost:15002/collections/research/vectors" \
  -H "Content-Type: application/json" \
  -d '{
    "texts": [
      {"id": "paper1", "text": "Attention is all you need"},
      {"id": "paper2", "text": "BERT: Pre-training of Deep Bidirectional Transformers"},
      {"id": "paper3", "text": "GPT: Generative Pre-trained Transformer"}
    ]
  }'

# 3. Discover relationships
curl -X POST "http://localhost:15002/graph/discover/research" \
  -H "Content-Type: application/json" \
  -d '{
    "similarity_threshold": 0.7,
    "max_per_node": 5
  }'

# 4. Find related papers
curl -X POST "http://localhost:15002/graph/nodes/research/paper1/related" \
  -H "Content-Type: application/json" \
  -d '{
    "max_hops": 2
  }'
```

## Best Practices

### 1. Similarity Threshold

Choose an appropriate similarity threshold:
- **0.8-0.9**: Very similar (high precision, low recall)
- **0.6-0.7**: Moderately similar (balanced)
- **0.4-0.5**: Loosely related (high recall, lower precision)

### 2. Max Per Node

Limit edges per node to avoid:
- Overwhelming graphs
- Performance degradation
- Noise from weak relationships

Recommended: 5-10 edges per node for most use cases.

### 3. Relationship Discovery

- Run discovery after bulk inserts for better performance
- Use batch discovery for large collections
- Monitor discovery status for long-running operations

### 4. Graph Persistence

Graph data is automatically persisted when collections are saved:
- Graph files: `{collection_name}_graph.json`
- Saved alongside vector data
- Automatically loaded when collection is restored

### 5. Performance Optimization

- Enable graph only for collections that need it
- Use appropriate similarity thresholds
- Limit max_per_node to reasonable values
- Consider running discovery asynchronously for large datasets

## Common Use Cases

### Document Clustering

Use graph to cluster similar documents:

```python
# Discover relationships
await client.discover_graph_edges("documents", {
    "similarity_threshold": 0.7,
    "max_per_node": 10
})

# Get connected components (clusters)
# This can be done by finding all nodes reachable from a seed node
related = await client.find_related_nodes(
    "documents",
    "seed_doc",
    {"max_hops": 3}
)
```

### Citation Networks

Build citation networks from academic papers:

```python
# Create explicit citation edges
for citation in citations:
    await client.create_graph_edge({
        "collection": "papers",
        "source": citation["citing"],
        "target": citation["cited"],
        "relationship_type": "REFERENCES",
        "weight": 1.0
    })
```

### Knowledge Graphs

Build knowledge graphs from structured data:

```python
# Create edges based on metadata relationships
for entity in entities:
    if entity.get("parent"):
        await client.create_graph_edge({
            "collection": "knowledge",
            "source": entity["id"],
            "target": entity["parent"],
            "relationship_type": "DERIVED_FROM",
            "weight": 1.0
        })
```

## Troubleshooting

### Graph Not Enabled

**Problem:** Graph operations fail with "Graph not enabled"

**Solution:** Enable graph when creating the collection:
```json
{
  "graph": {"enabled": true}
}
```

### No Relationships Found

**Problem:** Discovery finds no relationships

**Solution:**
- Lower similarity threshold
- Check if vectors are actually similar
- Verify collection has enough vectors

### Performance Issues

**Problem:** Graph operations are slow

**Solution:**
- Reduce max_per_node
- Increase similarity_threshold
- Use smaller collections
- Run discovery asynchronously

## See Also

- [GraphQL API Documentation](../api/GRAPHQL.md)
- [Collection Configuration](../collections/CONFIGURATION.md)
- [SDK Documentation](../sdks/README.md)

