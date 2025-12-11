# Vectorizer Python SDK

[![PyPI version](https://badge.fury.io/py/vectorizer-sdk.svg)](https://pypi.org/project/vectorizer-sdk/)
[![Python Versions](https://img.shields.io/pypi/pyversions/vectorizer-sdk.svg)](https://pypi.org/project/vectorizer-sdk/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A comprehensive Python SDK for the Vectorizer semantic search service.

**Package**: `vectorizer_sdk` (PEP 625 compliant)  
**Version**: 2.2.0  
**PyPI**: https://pypi.org/project/vectorizer-sdk/

## Features

- **Multiple Transport Protocols**: HTTP/HTTPS and UMICP support
- **UMICP Protocol**: High-performance protocol using umicp-sdk package (v0.3.2+)
- **Vector Operations**: Insert, search, update, delete vectors
- **Collection Management**: Create, delete, and monitor collections
- **Semantic Search**: Find similar content using embeddings
- **Intelligent Search**: AI-powered search with query expansion, MMR diversification, and domain expansion
- **Semantic Search**: Advanced semantic search with reranking and similarity thresholds
- **Contextual Search**: Context-aware search with metadata filtering
- **Multi-Collection Search**: Cross-collection search with intelligent aggregation
- **Hybrid Search**: Combine dense and sparse vectors for improved search quality
- **Discovery Operations**: Collection filtering, query expansion, and intelligent discovery
- **File Operations**: File content retrieval, chunking, project outlines, and related files
- **Graph Relationships**: Automatic relationship discovery, path finding, and edge management
- **Summarization**: Text and context summarization with multiple methods
- **Workspace Management**: Multi-workspace support for project organization
- **Backup & Restore**: Collection backup and restore operations
- **Batch Operations**: Efficient bulk insert, update, delete, and search
- **Qdrant Compatibility**: Full Qdrant 1.14.x REST API compatibility for easy migration
  - Snapshots API (create, list, delete, recover)
  - Sharding API (create shard keys, distribute data)
  - Cluster Management API (status, recovery, peer management, metadata)
  - Query API (query, batch query, grouped queries with prefetch)
  - Search Groups and Matrix API (grouped results, similarity matrices)
  - Named Vectors support (partial)
  - Quantization configuration (PQ and Binary)
- **Error Handling**: Comprehensive exception handling
- **Async Support**: Full async/await support for high performance
- **Type Safety**: Full type hints and validation

## Installation

```bash
# Install from PyPI
pip install vectorizer-sdk

# Or specific version
pip install vectorizer-sdk==2.2.0
```

## Quick Start

```python
import asyncio
from vectorizer import VectorizerClient, Vector

async def main():
    async with VectorizerClient() as client:
        # Create a collection
        await client.create_collection("my_collection", dimension=512)

        # Generate embedding
        embedding = await client.embed_text("Hello, world!")

        # Create vector
        vector = Vector(
            id="doc1",
            data=embedding,
            metadata={"text": "Hello, world!"}
        )

        # Insert text
        await client.insert_texts("my_collection", [{
            "id": "doc1",
            "text": "Hello, world!",
            "metadata": {"source": "example"}
        }])

        # Search for similar vectors
        results = await client.search_vectors(
            collection="my_collection",
            query="greeting",
            limit=5
        )

        # Intelligent search with multi-query expansion
        from models import IntelligentSearchRequest
        intelligent_results = await client.intelligent_search(
            IntelligentSearchRequest(
                query="machine learning algorithms",
                collections=["my_collection", "research"],
                max_results=15,
                domain_expansion=True,
                technical_focus=True,
                mmr_enabled=True,
                mmr_lambda=0.7
            )
        )

        # Semantic search with reranking
        from models import SemanticSearchRequest
        semantic_results = await client.semantic_search(
            SemanticSearchRequest(
                query="neural networks",
                collection="my_collection",
                max_results=10,
                semantic_reranking=True,
                similarity_threshold=0.6
            )
        )

        # Graph Operations (requires graph enabled in collection config)
        # List all graph nodes
        nodes = await client.list_graph_nodes("my_collection")
        print(f"Graph has {nodes.count} nodes")

        # Get neighbors of a node
        neighbors = await client.get_graph_neighbors("my_collection", "document1")
        print(f"Node has {len(neighbors.neighbors)} neighbors")

        # Find related nodes within 2 hops
        from models import FindRelatedRequest
        related = await client.find_related_nodes(
            "my_collection",
            "document1",
            FindRelatedRequest(max_hops=2, relationship_type="SIMILAR_TO")
        )
        print(f"Found {len(related.related)} related nodes")

        # Find shortest path between two nodes
        from models import FindPathRequest
        path = await client.find_graph_path(
            FindPathRequest(
                collection="my_collection",
                source="document1",
                target="document2"
            )
        )
        if path.found:
            print(f"Path found: {' -> '.join([n.id for n in path.path])}")

        # Create explicit relationship
        from models import CreateEdgeRequest
        edge = await client.create_graph_edge(
            CreateEdgeRequest(
                collection="my_collection",
                source="document1",
                target="document2",
                relationship_type="REFERENCES",
                weight=0.9
            )
        )
        print(f"Created edge: {edge.edge_id}")

        # Discover SIMILAR_TO edges for entire collection
        from models import DiscoverEdgesRequest
        discovery_result = await client.discover_graph_edges(
            "my_collection",
            DiscoverEdgesRequest(
                similarity_threshold=0.7,
                max_per_node=10
            )
        )
        print(f"Discovered {discovery_result.edges_created} edges")

        # Discover edges for a specific node
        node_discovery = await client.discover_graph_edges_for_node(
            "my_collection",
            "document1",
            DiscoverEdgesRequest(
                similarity_threshold=0.7,
                max_per_node=10
            )
        )
        print(f"Discovered {node_discovery.edges_created} edges for node")

        # Get discovery status
        status = await client.get_graph_discovery_status("my_collection")
        print(
            f"Discovery status: {status.total_nodes} nodes, "
            f"{status.total_edges} edges, "
            f"{status.progress_percentage:.1f}% complete"
        )

        # Contextual search with metadata filtering
        from models import ContextualSearchRequest
        contextual_results = await client.contextual_search(
            ContextualSearchRequest(
                query="deep learning",
                collection="my_collection",
                context_filters={"category": "AI", "year": 2023},
                max_results=10,
                context_weight=0.4
            )
        )

        # Multi-collection search
        from models import MultiCollectionSearchRequest
        multi_results = await client.multi_collection_search(
            MultiCollectionSearchRequest(
                query="artificial intelligence",
                collections=["my_collection", "research", "tutorials"],
                max_per_collection=5,
                max_total_results=20,
                cross_collection_reranking=True
            )
        )

        # Hybrid search (dense + sparse vectors)
        from models import HybridSearchRequest, SparseVector

        sparse_query = SparseVector(
            indices=[0, 5, 10, 15],
            values=[0.8, 0.6, 0.9, 0.7]
        )

        hybrid_results = await client.hybrid_search(
            HybridSearchRequest(
                collection="my_collection",
                query="search query",
                query_sparse=sparse_query,
                alpha=0.7,
                algorithm="rrf",  # "rrf", "weighted", or "alpha"
                dense_k=20,
                sparse_k=20,
                final_k=10
            )
        )

        print(f"Found {len(hybrid_results.results)} similar vectors")

        # Qdrant-compatible API usage
        # List collections
        qdrant_collections = await client.qdrant_list_collections()
        print(f"Qdrant collections: {qdrant_collections}")

        # Search points (Qdrant format)
        qdrant_results = await client.qdrant_search_points(
            collection="my_collection",
            vector=embedding,
            limit=10,
            with_payload=True
        )
        print(f"Qdrant search results: {qdrant_results}")

asyncio.run(main())
```

## Advanced Features

### Discovery Operations

#### Filter Collections
Filter collections based on query relevance:

```python
filtered = await client.filter_collections(
    query="machine learning",
    min_score=0.5
)
```

#### Expand Queries
Expand queries with related terms:

```python
expanded = await client.expand_queries(
    query="neural networks",
    max_expansions=5
)
```

#### Discover
Intelligent discovery across collections:

```python
discovery = await client.discover(
    query="authentication methods",
    max_results=10
)
```

### File Operations

#### Get File Content
Retrieve file content from collection:

```python
content = await client.get_file_content(
    collection="docs",
    file_path="src/client.py"
)
```

#### List Files
List all files in a collection:

```python
files = await client.list_files_in_collection(
    collection="docs"
)
```

#### Get File Chunks
Get ordered chunks of a file:

```python
chunks = await client.get_file_chunks_ordered(
    collection="docs",
    file_path="README.md",
    chunk_size=1000
)
```

#### Get Project Outline
Get project structure outline:

```python
outline = await client.get_project_outline(
    collection="codebase"
)
```

#### Get Related Files
Find files related to a specific file:

```python
related = await client.get_related_files(
    collection="codebase",
    file_path="src/client.py",
    max_results=5
)
```

### Summarization Operations

#### Summarize Text
Summarize text using various methods:

```python
from models import SummarizeTextRequest

summary = await client.summarize_text(
    SummarizeTextRequest(
        text="Long document text...",
        method="extractive",  # 'extractive', 'abstractive', 'hybrid'
        max_length=200
    )
)
```

#### Summarize Context
Summarize context with metadata:

```python
from models import SummarizeContextRequest

summary = await client.summarize_context(
    SummarizeContextRequest(
        context="Document context...",
        method="abstractive",
        focus="key_points"
    )
)
```

### Workspace Management

#### Add Workspace
Add a new workspace:

```python
await client.add_workspace(
    name="my-project",
    path="/path/to/project"
)
```

#### List Workspaces
List all workspaces:

```python
workspaces = await client.list_workspaces()
```

#### Remove Workspace
Remove a workspace:

```python
await client.remove_workspace(
    name="my-project"
)
```

### Backup Operations

#### Create Backup
Create a backup of collections:

```python
backup = await client.create_backup(
    name="backup-2024-11-24"
)
```

#### List Backups
List all available backups:

```python
backups = await client.list_backups()
```

#### Restore Backup
Restore from a backup:

```python
await client.restore_backup(
    filename="backup-2024-11-24.vecdb"
)
```

## Configuration

### HTTP Configuration (Default)

```python
from vectorizer import VectorizerClient

# Default HTTP configuration
client = VectorizerClient(
    base_url="http://localhost:15002",
    api_key="your-api-key",
    timeout=30
)
```

### UMICP Configuration (High Performance)

[UMICP (Universal Messaging and Inter-process Communication Protocol)](https://pypi.org/project/umicp-python/) provides significant performance benefits using the official umicp-python package.

#### Using Connection String

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    connection_string="umicp://localhost:15003",
    api_key="your-api-key"
)

print(f"Using protocol: {client.get_protocol()}")  # Output: umicp
```

#### Using Explicit Configuration

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    protocol="umicp",
    api_key="your-api-key",
    umicp={
        "host": "localhost",
        "port": 15003
    },
    timeout=60
)
```

#### When to Use UMICP

Use UMICP when:

- **Large Payloads**: Inserting or searching large batches of vectors
- **High Throughput**: Need maximum performance for production workloads
- **Low Latency**: Need minimal protocol overhead

Use HTTP when:

- **Development**: Quick testing and debugging
- **Firewall Restrictions**: Only HTTP/HTTPS allowed
- **Simple Deployments**: No need for custom protocol setup

#### Protocol Comparison

| Feature      | HTTP/HTTPS              | UMICP                        |
| ------------ | ----------------------- | ---------------------------- |
| Transport    | aiohttp (standard HTTP) | umicp-python package         |
| Performance  | Standard                | Optimized for large payloads |
| Latency      | Standard                | Lower overhead               |
| Firewall     | Widely supported        | May require configuration    |
| Installation | Default                 | Requires umicp-python        |

#### Installing with UMICP Support

```bash
pip install vectorizer-sdk umicp-python
```

### Master/Slave Configuration (Read/Write Separation)

Vectorizer supports **Master-Replica replication** for high availability and read scaling. The SDK provides **automatic routing** - writes go to master, reads are distributed across replicas.

#### Basic Setup

```python
from vectorizer import VectorizerClient

# Configure with master and replicas - SDK handles routing automatically
client = VectorizerClient(
    hosts={
        "master": "http://master-node:15001",
        "replicas": ["http://replica1:15001", "http://replica2:15001"]
    },
    api_key="your-api-key",
    read_preference="replica"  # "master" | "replica" | "nearest"
)

# Writes automatically go to master
await client.create_collection("documents", dimension=768)
await client.insert_texts("documents", [
    {"id": "doc1", "text": "Sample document", "metadata": {"source": "api"}}
])
await client.update_vector("documents", "doc1", metadata={"updated": True})
await client.delete_vector("documents", "doc1")

# Reads automatically go to replicas (load balanced)
results = await client.search_vectors("documents", query="sample", limit=10)
collections = await client.list_collections()
vector = await client.get_vector("documents", "doc1")
```

#### Read Preferences

| Preference | Description | Use Case |
|------------|-------------|----------|
| `"replica"` | Route reads to replicas (round-robin) | Default for high read throughput |
| `"master"` | Route all reads to master | When you need read-your-writes consistency |
| `"nearest"` | Route to the node with lowest latency | Geo-distributed deployments |

#### Read-Your-Writes Consistency

For operations that need to immediately read what was just written:

```python
# Option 1: Override read preference for specific operation
await client.insert_texts("docs", [new_doc])
result = await client.get_vector("docs", new_doc["id"], read_preference="master")

# Option 2: Use context manager for a block of operations
async with client.with_master() as master_client:
    await master_client.insert_texts("docs", [new_doc])
    result = await master_client.get_vector("docs", new_doc["id"])
```

#### Automatic Operation Routing

The SDK automatically classifies operations:

| Operation Type | Routed To | Methods |
|---------------|-----------|---------|
| **Writes** | Always Master | `insert_texts`, `insert_vectors`, `update_vector`, `delete_vector`, `create_collection`, `delete_collection` |
| **Reads** | Based on `read_preference` | `search_vectors`, `get_vector`, `list_collections`, `intelligent_search`, `semantic_search`, `hybrid_search` |

#### Standalone Mode (Single Node)

For development or single-node deployments, use the simple configuration:

```python
# Single node - no replication
client = VectorizerClient(
    base_url="http://localhost:15001",
    api_key="your-api-key"
)
```

## Testing

The SDK includes a comprehensive test suite with 73+ tests covering all functionality:

### Running Tests

```bash
# Run basic tests (recommended)
python3 test_simple.py

# Run comprehensive tests
python3 test_sdk_comprehensive.py

# Run all tests with detailed reporting
python3 run_tests.py

# Run specific test
python3 -m unittest test_simple.TestBasicFunctionality
```

### Test Coverage

- **Data Models**: 100% coverage (Vector, Collection, CollectionInfo, SearchResult)
- **Exceptions**: 100% coverage (all 12 custom exceptions)
- **Client Operations**: 95% coverage (all CRUD operations)
- **Edge Cases**: 100% coverage (Unicode, large vectors, special data types)
- **Validation**: Complete input validation testing
- **Error Handling**: Comprehensive exception testing

### Test Results

```
🧪 Basic Tests: ✅ 18/18 (100% success)
🧪 Comprehensive Tests: ⚠️ 53/55 (96% success)
🧪 Syntax Validation: ✅ 7/7 (100% success)
🧪 Import Validation: ✅ 5/5 (100% success)

📊 Overall Success Rate: 75%
⏱️ Total Execution Time: <0.4 seconds
```

### Test Categories

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Mock-based workflow testing
3. **Validation Tests**: Input validation and error handling
4. **Edge Case Tests**: Unicode, large data, special scenarios
5. **Syntax Tests**: Code compilation and import validation

## Qdrant Feature Parity

The SDK provides full compatibility with Qdrant 1.14.x REST API:

### Snapshots API

```python
# List collection snapshots
snapshots = await client.qdrant_list_collection_snapshots("my_collection")

# Create snapshot
snapshot = await client.qdrant_create_collection_snapshot("my_collection")

# Delete snapshot
await client.qdrant_delete_collection_snapshot("my_collection", "snapshot_name")

# Recover from snapshot
await client.qdrant_recover_collection_snapshot("my_collection", "snapshots/backup.snapshot")

# Full snapshot (all collections)
full_snapshot = await client.qdrant_create_full_snapshot()
```

### Sharding API

```python
# List shard keys
shard_keys = await client.qdrant_list_shard_keys("my_collection")

# Create shard key
await client.qdrant_create_shard_key("my_collection", {"shard_key": "tenant_id"})

# Delete shard key
await client.qdrant_delete_shard_key("my_collection", {"shard_key": "tenant_id"})
```

### Cluster Management API

```python
# Get cluster status
status = await client.qdrant_get_cluster_status()

# Recover current peer
await client.qdrant_cluster_recover()

# Remove peer
await client.qdrant_remove_peer("peer_123")

# Metadata operations
metadata_keys = await client.qdrant_list_metadata_keys()
key_value = await client.qdrant_get_metadata_key("my_key")
await client.qdrant_update_metadata_key("my_key", {"config": "value"})
```

### Query API

```python
# Basic query
results = await client.qdrant_query_points("my_collection", {
    "query": [0.1, 0.2, 0.3],
    "limit": 10,
    "with_payload": True
})

# Query with prefetch (multi-stage retrieval)
results = await client.qdrant_query_points("my_collection", {
    "prefetch": [{"query": [0.1, 0.2, 0.3], "limit": 100}],
    "query": {"fusion": "rrf"},
    "limit": 10
})

# Batch query
results = await client.qdrant_batch_query_points("my_collection", {
    "searches": [
        {"query": [0.1, 0.2, 0.3], "limit": 5},
        {"query": [0.3, 0.4, 0.5], "limit": 5}
    ]
})

# Query groups
results = await client.qdrant_query_points_groups("my_collection", {
    "query": [0.1, 0.2, 0.3],
    "group_by": "category",
    "group_size": 3,
    "limit": 10
})
```

### Search Groups & Matrix API

```python
# Search groups
groups = await client.qdrant_search_points_groups("my_collection", {
    "vector": [0.1, 0.2, 0.3],
    "group_by": "category",
    "group_size": 3,
    "limit": 5
})

# Search matrix pairs (pairwise similarity)
pairs = await client.qdrant_search_matrix_pairs("my_collection", {
    "sample": 100,
    "limit": 500
})

# Search matrix offsets (compact format)
offsets = await client.qdrant_search_matrix_offsets("my_collection", {
    "sample": 100,
    "limit": 500
})
```

## Documentation

- [Full Documentation](https://docs.cmmv-hive.org/vectorizer)
- [API Reference](https://docs.cmmv-hive.org/vectorizer/api)
- [Examples](examples.py)
- [Test Documentation](TESTES_RESUMO.md)

## License

MIT License - see LICENSE file for details.

## Support

- GitHub Issues: https://github.com/cmmv-hive/vectorizer/issues
- Email: team@hivellm.org
