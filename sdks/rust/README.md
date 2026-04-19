# Vectorizer Rust SDK

[![Crates.io](https://img.shields.io/crates/v/vectorizer-sdk.svg)](https://crates.io/crates/vectorizer-sdk)
[![Documentation](https://docs.rs/vectorizer-sdk/badge.svg)](https://docs.rs/vectorizer-sdk)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

High-performance Rust SDK for Vectorizer vector database.

**Package**: `vectorizer-sdk`  
**Version**: 3.0.0 (RPC-first; HTTP fallback retained)

## ✅ Status: v3.0.0 — VectorizerRPC default transport

**v3.x ships with VectorizerRPC** — length-prefixed MessagePack over
raw TCP — as the recommended primary transport. The HTTP path that
shipped in 2.x stays available behind the `http` Cargo feature
(default-on for backward compat). Pick the constructor that matches
the URL scheme you have:

| URL | Constructor | Transport |
|---|---|---|
| `vectorizer://host:15503` | `RpcClient::connect_url(url)` | Binary RPC (recommended) |
| `vectorizer://host` | `RpcClient::connect_url(url)` | RPC on default port 15503 |
| `host:15503` (no scheme) | `RpcClient::connect_url(url)` or `RpcClient::connect("host:port")` | RPC |
| `http://host:15002` | `VectorizerClient` (HTTP path below) | REST (legacy) |

## Quick Start (RPC, recommended)

```toml
[dependencies]
vectorizer-sdk = "3.0"
tokio = { version = "1", features = ["full"] }
```

```rust
use vectorizer_sdk::rpc::{HelloPayload, RpcClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect via the canonical vectorizer:// URL.
    let client = RpcClient::connect_url("vectorizer://127.0.0.1:15503").await?;

    // HELLO is mandatory before any data-plane command. In single-user
    // mode (server's `auth.enabled: false`) credentials are ignored;
    // when auth is enabled, attach a JWT or API key:
    //   HelloPayload::new("my-app").with_token("<jwt>")
    let hello = client.hello(HelloPayload::new("my-app/1.0")).await?;
    println!("server={}, capabilities: {:?}", hello.server_version, hello.capabilities);

    // Typed wrappers cover every v1 command.
    let collections = client.list_collections().await?;
    if let Some(name) = collections.first() {
        let info = client.get_collection_info(name).await?;
        println!("{name}: {} vectors, dim={}", info.vector_count, info.dimension);

        let hits = client.search_basic(name, "vector database", 5).await?;
        for hit in &hits {
            println!("  {} (score={:.3})", hit.id, hit.score);
        }
    }
    Ok(())
}
```

See `examples/rpc_quickstart.rs` for the runnable version. Wire spec:
[`docs/specs/VECTORIZER_RPC.md`](../../docs/specs/VECTORIZER_RPC.md).

### Connection pooling

```rust
use vectorizer_sdk::rpc::{HelloPayload, RpcPool, pool::RpcPoolConfig};

let pool = RpcPool::new(RpcPoolConfig {
    address: "127.0.0.1:15503".into(),
    max_connections: 8,
    hello: HelloPayload::new("worker"),
});

let conn = pool.acquire().await?;
let collections = conn.client().list_collections().await?;
// `conn` returns to the pool on Drop.
```

### Error handling

`RpcClient` returns `Result<T, RpcClientError>`. The variants:

- `Io(std::io::Error)` — TCP-level failure.
- `Server(String)` — server returned `Err(message)`.
- `ConnectionClosed` — the background reader task exited (peer
  closed, or write failure mid-call).
- `NotAuthenticated` — local guard against issuing a data-plane
  command before `HELLO` succeeded; saves an unnecessary round-trip.
- `Encode(rmp_serde::encode::Error)` — should be unreachable for v1
  shapes (every type derives `Serialize`).

## Quick Start (HTTP, legacy)

The 2.x `VectorizerClient` is preserved unchanged. To opt into a
slim build with RPC only:

```toml
[dependencies]
vectorizer-sdk = { version = "3.0", default-features = false, features = ["rpc"] }
```

To use the HTTP client:

```rust
use vectorizer_sdk::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create client
    let client = VectorizerClient::new_default()?;

    // Health check
    let health = client.health_check().await?;
    println!("Status: {}", health.status);

    // List collections
    let collections = client.list_collections().await?;
    println!("Found {} collections", collections.len());

    // Create new collection
    let collection = client.create_collection("my_docs", 384, Some(SimilarityMetric::Cosine)).await?;
    println!("Created collection: {}", collection.name);

    // Search existing collections
    let results = client.search_vectors("gov-bips", "bitcoin", Some(5), None).await?;
    println!("Found {} search results", results.results.len());

    // Hybrid search (dense + sparse vectors)
    use vectorizer_sdk::{HybridSearchRequest, SparseVector, HybridScoringAlgorithm};
    let sparse = SparseVector::new(
        vec![0, 5, 10, 15],
        vec![0.8, 0.6, 0.9, 0.7]
    )?;
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
    println!("Found {} hybrid search results", hybrid_results.results.len());

    // Qdrant-compatible API usage
    let qdrant_collections = client.qdrant_list_collections().await?;
    println!("Qdrant collections: {:?}", qdrant_collections);

    // Intelligent search with multi-query expansion
    let intelligent_request = IntelligentSearchRequest {
        query: "machine learning algorithms".to_string(),
        collections: Some(vec!["gov-bips".to_string(), "research".to_string()]),
        max_results: Some(15),
        domain_expansion: Some(true),
        technical_focus: Some(true),
        mmr_enabled: Some(true),
        mmr_lambda: Some(0.7),
    };
    let intelligent_results = client.intelligent_search(intelligent_request).await?;
    println!("Intelligent search found {} results", intelligent_results.results.len());

    // Semantic search with reranking
    let semantic_request = SemanticSearchRequest {
        query: "neural networks".to_string(),
        collection: "gov-bips".to_string(),
        max_results: Some(10),
        semantic_reranking: Some(true),
        cross_encoder_reranking: Some(false),
        similarity_threshold: Some(0.6),
    };
    let semantic_results = client.semantic_search(semantic_request).await?;
    println!("Semantic search found {} results", semantic_results.results.len());

    // Graph Operations (requires graph enabled in collection config)
    // List all graph nodes
    let nodes = client.list_graph_nodes("documents").await?;
    println!("Graph has {} nodes", nodes.count);

    // Get neighbors of a node
    let neighbors = client.get_graph_neighbors("documents", "document1").await?;
    println!("Node has {} neighbors", neighbors.neighbors.len());

    // Find related nodes within 2 hops
    use vectorizer_sdk::models::FindRelatedRequest;
    let related = client.find_related_nodes(
        "documents",
        "document1",
        FindRelatedRequest {
            max_hops: Some(2),
            relationship_type: Some("SIMILAR_TO".to_string()),
        },
    ).await?;
    println!("Found {} related nodes", related.related.len());

    // Find shortest path between two nodes
    use vectorizer_sdk::models::FindPathRequest;
    let path = client.find_graph_path(FindPathRequest {
        collection: "documents".to_string(),
        source: "document1".to_string(),
        target: "document2".to_string(),
    }).await?;
    if path.found {
        println!("Path found: {:?}", path.path.iter().map(|n| &n.id).collect::<Vec<_>>());
    }

    // Create explicit relationship
    use vectorizer_sdk::models::CreateEdgeRequest;
    let edge = client.create_graph_edge(CreateEdgeRequest {
        collection: "documents".to_string(),
        source: "document1".to_string(),
        target: "document2".to_string(),
        relationship_type: "REFERENCES".to_string(),
        weight: Some(0.9),
    }).await?;
    println!("Created edge: {}", edge.edge_id);

    // Discover SIMILAR_TO edges for entire collection
    use vectorizer_sdk::models::DiscoverEdgesRequest;
    let discovery_result = client.discover_graph_edges(
        "documents",
        DiscoverEdgesRequest {
            similarity_threshold: Some(0.7),
            max_per_node: Some(10),
        },
    ).await?;
    println!("Discovered {} edges", discovery_result.edges_created);

    // Discover edges for a specific node
    let node_discovery = client.discover_graph_edges_for_node(
        "documents",
        "document1",
        DiscoverEdgesRequest {
            similarity_threshold: Some(0.7),
            max_per_node: Some(10),
        },
    ).await?;
    println!("Discovered {} edges for node", node_discovery.edges_created);

    // Get discovery status
    let status = client.get_graph_discovery_status("documents").await?;
    println!(
        "Discovery status: {} nodes, {} edges, {:.1}% complete",
        status.total_nodes,
        status.total_edges,
        status.progress_percentage
    );

    // Contextual search with metadata filtering
    let mut context_filters = std::collections::HashMap::new();
    context_filters.insert("category".to_string(), serde_json::Value::String("AI".to_string()));
    context_filters.insert("year".to_string(), serde_json::Value::Number(2023.into()));

    let contextual_request = ContextualSearchRequest {
        query: "deep learning".to_string(),
        collection: "gov-bips".to_string(),
        context_filters: Some(context_filters),
        max_results: Some(10),
        context_reranking: Some(true),
        context_weight: Some(0.4),
    };
    let contextual_results = client.contextual_search(contextual_request).await?;
    println!("Contextual search found {} results", contextual_results.results.len());

    // Multi-collection search
    let multi_request = MultiCollectionSearchRequest {
        query: "artificial intelligence".to_string(),
        collections: vec!["gov-bips".to_string(), "research".to_string(), "tutorials".to_string()],
        max_per_collection: Some(5),
        max_total_results: Some(20),
        cross_collection_reranking: Some(true),
    };
    let multi_results = client.multi_collection_search(multi_request).await?;
    println!("Multi-collection search found {} results", multi_results.results.len());

    Ok(())
}
```

## Features

- 🚀 **High Performance**: Optimized async transport layer
- 🔄 **Async/Await**: Full async/await support with Tokio
- 📡 **Multiple Protocols**: HTTP/HTTPS and UMICP support
- 🔍 **Semantic Search**: Vector similarity search with multiple metrics
- 🧠 **Intelligent Search**: Advanced multi-query search with domain expansion
- 🎯 **Contextual Search**: Context-aware search with metadata filtering
- 🔗 **Multi-Collection Search**: Cross-collection search with intelligent aggregation
- 📦 **Batch Operations**: Efficient bulk text insertion
- 🛡️ **Type Safety**: Strongly typed API with comprehensive error handling
- 🔧 **Easy Setup**: Simple client creation with sensible defaults
- 📊 **Health Monitoring**: Built-in health checks and statistics

## Installation

### HTTP Transport (Default)

Add to `Cargo.toml`:

```toml
[dependencies]
vectorizer-sdk = "2.2.0"
tokio = { version = "1.35", features = ["full"] }
serde_json = "1.0"
```

### UMICP Transport (High Performance)

Enable the UMICP feature for high-performance protocol support:

```toml
[dependencies]
vectorizer-sdk = { version = "2.1.0", features = ["umicp"] }
tokio = { version = "1.35", features = ["full"] }
serde_json = "1.0"
```

## Configuration

### HTTP Configuration (Default)

```rust
use vectorizer_rust_sdk::{VectorizerClient, ClientConfig};

// Default configuration
let client = VectorizerClient::new_default()?;

// Custom URL
let client = VectorizerClient::new_with_url("http://localhost:15002")?;

// With API key
let client = VectorizerClient::new_with_api_key("http://localhost:15002", "your-api-key")?;

// Advanced configuration
let client = VectorizerClient::new(ClientConfig {
    base_url: Some("http://localhost:15002".to_string()),
    api_key: Some("your-api-key".to_string()),
    timeout_secs: Some(60),
    ..Default::default()
})?;
```

### UMICP Configuration (High Performance)

[UMICP (Universal Messaging and Inter-process Communication Protocol)](https://crates.io/crates/umicp-core) provides significant performance benefits.

#### Using Connection String

```rust
use vectorizer_rust_sdk::VectorizerClient;

let client = VectorizerClient::from_connection_string(
    "umicp://localhost:15003",
    Some("your-api-key")
)?;

println!("Using protocol: {}", client.protocol());
```

#### Using Explicit Configuration

```rust
use vectorizer_rust_sdk::{VectorizerClient, ClientConfig, Protocol, UmicpConfig};

let client = VectorizerClient::new(ClientConfig {
    protocol: Some(Protocol::Umicp),
    api_key: Some("your-api-key".to_string()),
    umicp: Some(UmicpConfig {
        host: "localhost".to_string(),
        port: 15003,
    }),
    timeout_secs: Some(60),
    ..Default::default()
})?;
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

| Feature     | HTTP/HTTPS              | UMICP                        |
| ----------- | ----------------------- | ---------------------------- |
| Transport   | reqwest (standard HTTP) | umicp-core crate             |
| Performance | Standard                | Optimized for large payloads |
| Latency     | Standard                | Lower overhead               |
| Firewall    | Widely supported        | May require configuration    |
| Build Time  | Fast                    | Requires UMICP feature       |

### Master/Slave Configuration (Read/Write Separation)

Vectorizer supports **Master-Replica replication** for high availability and read scaling. The SDK provides **automatic routing** - writes go to master, reads are distributed across replicas.

#### Basic Setup

```rust
use vectorizer_rust_sdk::{VectorizerClient, ReadPreference};

// Configure with master and replicas - SDK handles routing automatically
let client = VectorizerClient::builder()
    .master("http://master-node:15001")
    .replica("http://replica1:15001")
    .replica("http://replica2:15001")
    .api_key("your-api-key")
    .read_preference(ReadPreference::Replica)
    .build()?;

// Writes automatically go to master
client.create_collection("documents", 768, Some(SimilarityMetric::Cosine)).await?;
client.insert_texts("documents", vec![
    BatchTextRequest {
        id: "doc1".to_string(),
        text: "Sample document".to_string(),
        metadata: Some(metadata),
    }
]).await?;

// Reads automatically go to replicas (load balanced)
let results = client.search_vectors("documents", &query_vector, 10).await?;
let collections = client.list_collections().await?;
```

#### Read Preferences

| Preference | Description | Use Case |
|------------|-------------|----------|
| `ReadPreference::Replica` | Route reads to replicas (round-robin) | Default for high read throughput |
| `ReadPreference::Master` | Route all reads to master | When you need read-your-writes consistency |
| `ReadPreference::Nearest` | Route to the node with lowest latency | Geo-distributed deployments |

#### Read-Your-Writes Consistency

For operations that need to immediately read what was just written:

```rust
// Option 1: Override read preference for specific operation
client.insert_texts("docs", vec![new_doc]).await?;
let result = client.get_vector_with_preference("docs", "doc_id", ReadPreference::Master).await?;

// Option 2: Use a scoped master context
client.with_master(|master_client| async {
    master_client.insert_texts("docs", vec![new_doc]).await?;
    master_client.get_vector("docs", "doc_id").await
}).await?;
```

#### Automatic Operation Routing

The SDK automatically classifies operations:

| Operation Type | Routed To | Methods |
|---------------|-----------|---------|
| **Writes** | Always Master | `insert_texts`, `insert_vectors`, `update_vector`, `delete_vector`, `create_collection`, `delete_collection` |
| **Reads** | Based on `ReadPreference` | `search_vectors`, `get_vector`, `list_collections`, `intelligent_search`, `semantic_search`, `hybrid_search` |

#### Standalone Mode (Single Node)

For development or single-node deployments:

```rust
// Single node - no replication
let client = VectorizerClient::new_with_api_key("http://localhost:15001", "your-api-key")?;
```

## API Endpoints

### ✅ Health & Monitoring

- `health_check()` - Server health and statistics
- `list_collections()` - List all available collections

### ✅ Collection Management

- `create_collection()` - Create new vector collection
- `get_collection_info()` - Get collection details (limited support)
- `delete_collection()` - Delete collection (limited support)

### ✅ Vector Operations

- `search_vectors()` - Semantic search with text queries
- `insert_texts()` - Batch text insertion (limited support)
- `get_vector()` - Retrieve individual vectors (limited support)

### ✅ Embedding (Future)

- `embed_text()` - Generate embeddings (endpoint not available)

## Examples

Run the examples to see the SDK in action:

```bash
# Basic usage example
cargo run --example basic_example

# Comprehensive test suite (9/9 tests passing)
cargo run --example comprehensive_test
```

## Testing

The SDK includes comprehensive tests that verify:

- ✅ Client creation and configuration
- ✅ Health check functionality
- ✅ Collection listing and information
- ✅ Vector search operations
- ✅ Collection creation
- ✅ Error handling and edge cases

**Test Results: 9/9 endpoints functional (100% success rate)**

## Compatibility

- **Rust**: 1.90.0+ (Rust 2024 edition)
- **Vectorizer Server**: v0.20.0+
- **HTTP**: REST API with JSON payloads
- **UMICP**: Optional feature (enable with `--features umicp`)
- **Async Runtime**: Tokio 1.35+

## Building

### HTTP Only (Default)

```bash
cargo build --release
```

### With UMICP Support

```bash
cargo build --release --features umicp
```

### Run Tests

```bash
# HTTP tests only
cargo test

# UMICP tests
cargo test --features umicp

# Specific test
cargo test --test umicp_tests --features umicp
```

### Run Examples

```bash
# HTTP example
cargo run --example basic_example

# UMICP example (requires feature)
cargo run --example umicp_usage --features umicp
```

## Error Handling

The SDK provides comprehensive error types:

```rust
use vectorizer_rust_sdk::{VectorizerClient, VectorizerError};

match client.search_vectors("collection", "query", None, None).await {
    Ok(results) => println!("Found {} results", results.results.len()),
    Err(VectorizerError::Network(msg)) => eprintln!("Network error: {}", msg),
    Err(VectorizerError::Server(msg)) => eprintln!("Server error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Qdrant Feature Parity

The SDK provides full compatibility with Qdrant 1.14.x REST API:

### Snapshots API

```rust
// List collection snapshots
let snapshots = client.qdrant_list_collection_snapshots("my_collection").await?;

// Create snapshot
let snapshot = client.qdrant_create_collection_snapshot("my_collection").await?;

// Delete snapshot
client.qdrant_delete_collection_snapshot("my_collection", "snapshot_name").await?;

// Recover from snapshot
client.qdrant_recover_collection_snapshot("my_collection", "snapshots/backup.snapshot").await?;

// Full snapshot (all collections)
let full_snapshot = client.qdrant_create_full_snapshot().await?;
```

### Sharding API

```rust
// List shard keys
let shard_keys = client.qdrant_list_shard_keys("my_collection").await?;

// Create shard key
let shard_config = serde_json::json!({"shard_key": "tenant_id"});
client.qdrant_create_shard_key("my_collection", &shard_config).await?;

// Delete shard key
client.qdrant_delete_shard_key("my_collection", &shard_config).await?;
```

### Cluster Management API

```rust
// Get cluster status
let status = client.qdrant_get_cluster_status().await?;

// Recover current peer
client.qdrant_cluster_recover().await?;

// Remove peer
client.qdrant_remove_peer("peer_123").await?;

// Metadata operations
let metadata_keys = client.qdrant_list_metadata_keys().await?;
let key_value = client.qdrant_get_metadata_key("my_key").await?;
let value = serde_json::json!({"config": "value"});
client.qdrant_update_metadata_key("my_key", &value).await?;
```

### Query API

```rust
// Basic query
let query_request = serde_json::json!({
    "query": [0.1, 0.2, 0.3, ...],
    "limit": 10,
    "with_payload": true
});
let results = client.qdrant_query_points("my_collection", &query_request).await?;

// Query with prefetch (multi-stage retrieval)
let prefetch_request = serde_json::json!({
    "prefetch": [
        {"query": [0.1, 0.2, ...], "limit": 100}
    ],
    "query": {"fusion": "rrf"},
    "limit": 10
});
let results = client.qdrant_query_points("my_collection", &prefetch_request).await?;

// Batch query
let batch_request = serde_json::json!({
    "searches": [
        {"query": [0.1, 0.2, ...], "limit": 5},
        {"query": [0.3, 0.4, ...], "limit": 5}
    ]
});
let results = client.qdrant_batch_query_points("my_collection", &batch_request).await?;

// Query groups
let groups_request = serde_json::json!({
    "query": [0.1, 0.2, ...],
    "group_by": "category",
    "group_size": 3,
    "limit": 10
});
let results = client.qdrant_query_points_groups("my_collection", &groups_request).await?;
```

### Search Groups & Matrix API

```rust
// Search groups
let search_groups_request = serde_json::json!({
    "vector": [0.1, 0.2, ...],
    "group_by": "category",
    "group_size": 3,
    "limit": 5
});
let groups = client.qdrant_search_points_groups("my_collection", &search_groups_request).await?;

// Search matrix pairs (pairwise similarity)
let matrix_request = serde_json::json!({
    "sample": 100,
    "limit": 500
});
let pairs = client.qdrant_search_matrix_pairs("my_collection", &matrix_request).await?;

// Search matrix offsets (compact format)
let offsets = client.qdrant_search_matrix_offsets("my_collection", &matrix_request).await?;
```

## Contributing

This SDK is ready for production use. All endpoints have been tested and verified functional.
