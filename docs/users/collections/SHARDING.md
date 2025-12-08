# Sharding Configuration Guide

Sharding allows you to distribute vectors across multiple shards for improved scalability, performance, and fault tolerance. This guide explains how to configure and use sharding in Vectorizer, including both single-server and distributed (multi-server) sharding.

## Table of Contents

- [Overview](#overview)
- [When to Use Sharding](#when-to-use-sharding)
- [Configuration](#configuration)
- [Sharding Parameters](#sharding-parameters)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Monitoring and Management](#monitoring-and-management)
- [Troubleshooting](#troubleshooting)

## Overview

Sharding distributes a collection's vectors across multiple independent shards using consistent hashing. Each shard is a separate collection that handles a subset of the data. This provides:

- **Horizontal Scalability**: Add more shards to handle larger datasets
- **Parallel Processing**: Search operations can run in parallel across shards
- **Fault Tolerance**: Failure of one shard doesn't affect others
- **Load Distribution**: Better distribution of read/write operations

## When to Use Sharding

Consider enabling sharding when:

- **Large Datasets**: Collections with millions of vectors
- **High Throughput**: Need to handle many concurrent operations
- **Memory Constraints**: Single collection exceeds available memory
- **Performance Requirements**: Need parallel search across shards

**Note**: Sharding adds overhead for coordination and result merging. For small collections (< 100K vectors), a single collection is usually more efficient.

## Configuration

### Basic Configuration

Sharding is configured in the `CollectionConfig` when creating a collection:

```rust
use vectorizer::models::{CollectionConfig, ShardingConfig, DistanceMetric, HnswConfig};

let config = CollectionConfig {
    dimension: 128,
    metric: DistanceMetric::Cosine,
    hnsw_config: HnswConfig::default(),
    quantization: QuantizationConfig::None,
    compression: CompressionConfig::default(),
    normalization: None,
    storage_type: Some(StorageType::Memory),
    sharding: Some(ShardingConfig {
        shard_count: 4,                    // Number of shards
        virtual_nodes_per_shard: 100,       // Virtual nodes for consistent hashing
        rebalance_threshold: 0.2,           // 20% deviation threshold
    }),
};
```

### YAML Configuration

When using YAML configuration files:

```yaml
collections:
  my_sharded_collection:
    dimension: 128
    metric: cosine
    sharding:
      shard_count: 4
      virtual_nodes_per_shard: 100
      rebalance_threshold: 0.2
```

### REST API Configuration

When creating a collection via REST API:

```json
POST /api/v1/collections
{
  "name": "my_sharded_collection",
  "config": {
    "dimension": 128,
    "metric": "cosine",
    "sharding": {
      "shard_count": 4,
      "virtual_nodes_per_shard": 100,
      "rebalance_threshold": 0.2
    }
  }
}
```

## Sharding Parameters

### `shard_count` (required)

**Type**: `u32`  
**Default**: `4`  
**Description**: Number of shards to create for the collection.

**Recommendations**:
- Start with 4 shards for small to medium datasets
- Use 8-16 shards for large datasets (millions of vectors)
- Consider your CPU cores: more shards = more parallel processing
- Each shard requires memory, so balance shard count with available RAM

**Example**:
```rust
sharding: Some(ShardingConfig {
    shard_count: 8,  // 8 shards for large collection
    // ...
})
```

### `virtual_nodes_per_shard` (optional)

**Type**: `usize`  
**Default**: `100`  
**Description**: Number of virtual nodes per shard in the consistent hash ring.

**How it works**:
- Higher values provide better distribution but use more memory
- Each virtual node requires ~16 bytes of memory
- Formula: `memory = shard_count * virtual_nodes_per_shard * 16 bytes`

**Recommendations**:
- **Small collections** (< 1M vectors): 50-100 virtual nodes
- **Medium collections** (1M-10M vectors): 100-200 virtual nodes
- **Large collections** (> 10M vectors): 200-500 virtual nodes

**Example**:
```rust
sharding: Some(ShardingConfig {
    shard_count: 4,
    virtual_nodes_per_shard: 200,  // Better distribution for large dataset
    // ...
})
```

### `rebalance_threshold` (optional)

**Type**: `f32`  
**Default**: `0.2` (20%)  
**Description**: Percentage deviation from average shard size that triggers rebalancing.

**How it works**:
- When shard sizes deviate more than this threshold, rebalancing is recommended
- Value is a percentage (0.2 = 20%, 0.5 = 50%)
- Lower values = more aggressive rebalancing (more overhead)
- Higher values = less frequent rebalancing (may have uneven distribution)

**Recommendations**:
- **Balanced workloads**: 0.2 (20%) - default
- **Highly variable workloads**: 0.3-0.4 (30-40%)
- **Stable workloads**: 0.1-0.15 (10-15%)

**Example**:
```rust
sharding: Some(ShardingConfig {
    shard_count: 4,
    virtual_nodes_per_shard: 100,
    rebalance_threshold: 0.3,  // 30% threshold for variable workloads
})
```

## Examples

### Example 1: Small Collection (100K vectors)

```rust
let config = CollectionConfig {
    dimension: 384,
    metric: DistanceMetric::Cosine,
    hnsw_config: HnswConfig::default(),
    quantization: QuantizationConfig::None,
    compression: CompressionConfig::default(),
    normalization: None,
    storage_type: Some(StorageType::Memory),
    sharding: Some(ShardingConfig {
        shard_count: 2,                    // 2 shards sufficient
        virtual_nodes_per_shard: 50,       // Lower for small collection
        rebalance_threshold: 0.2,          // Default threshold
    }),
};
```

### Example 2: Medium Collection (1M vectors)

```rust
let config = CollectionConfig {
    dimension: 512,
    metric: DistanceMetric::Euclidean,
    hnsw_config: HnswConfig::default(),
    quantization: QuantizationConfig::SQ { bits: 8 },
    compression: CompressionConfig::default(),
    normalization: None,
    storage_type: Some(StorageType::Memory),
    sharding: Some(ShardingConfig {
        shard_count: 4,                    // 4 shards for good parallelism
        virtual_nodes_per_shard: 100,       // Default virtual nodes
        rebalance_threshold: 0.2,          // Default threshold
    }),
};
```

### Example 3: Large Collection (10M+ vectors)

```rust
let config = CollectionConfig {
    dimension: 768,
    metric: DistanceMetric::Cosine,
    hnsw_config: HnswConfig {
        m: 32,                              // Higher for large collection
        ef_construction: 400,
        ef_search: 200,
        seed: None,
    },
    quantization: QuantizationConfig::SQ { bits: 8 },
    compression: CompressionConfig::default(),
    normalization: None,
    storage_type: Some(StorageType::Mmap), // Use MMAP for large data
    sharding: Some(ShardingConfig {
        shard_count: 8,                    // 8 shards for large dataset
        virtual_nodes_per_shard: 200,       // More virtual nodes for better distribution
        rebalance_threshold: 0.15,         // Tighter threshold for large collections
    }),
};
```

### Example 4: High-Performance Collection

```rust
let config = CollectionConfig {
    dimension: 1024,
    metric: DistanceMetric::DotProduct,
    hnsw_config: HnswConfig {
        m: 64,
        ef_construction: 500,
        ef_search: 300,
        seed: Some(42),
    },
    quantization: QuantizationConfig::None, // No quantization for maximum accuracy
    compression: CompressionConfig::default(),
    normalization: None,
    storage_type: Some(StorageType::Memory),
    sharding: Some(ShardingConfig {
        shard_count: 16,                   // Many shards for maximum parallelism
        virtual_nodes_per_shard: 300,      // High virtual nodes for even distribution
        rebalance_threshold: 0.1,          // Aggressive rebalancing for performance
    }),
};
```

## Best Practices

### 1. Choose Appropriate Shard Count

- **Rule of thumb**: 1 shard per 1-2 CPU cores
- **Memory**: Each shard needs memory for its vectors and index
- **Start small**: Begin with 4 shards and scale up if needed

### 2. Balance Virtual Nodes

- More virtual nodes = better distribution but more memory
- Calculate memory: `shard_count * virtual_nodes_per_shard * 16 bytes`
- For 4 shards with 100 virtual nodes: `4 * 100 * 16 = 6.4 KB` (negligible)

### 3. Monitor Shard Distribution

Regularly check shard sizes to ensure even distribution:

```rust
let collection = store.get_collection("my_collection")?;
match collection.deref() {
    CollectionType::Sharded(sharded) => {
        let shard_counts = sharded.shard_counts();
        println!("Shard distribution: {:?}", shard_counts);
    }
    _ => {}
}
```

### 4. Use Appropriate Storage Type

- **Memory**: Fastest, but limited by RAM
- **MMAP**: Slower, but can handle larger datasets on disk
- For sharded collections, MMAP is often better for large datasets

### 5. Consider Rebalancing

- Monitor if rebalancing is needed: `sharded.needs_rebalancing()`
- Rebalancing moves vectors between shards (can be expensive)
- Set threshold based on your workload variability

### 6. Vector ID Distribution

- Vector IDs are hashed to determine shard assignment
- Use diverse, random IDs for better distribution
- Avoid sequential IDs that might hash to the same shard

## Monitoring and Management

### Check Shard Status

```rust
let collection = store.get_collection("my_collection")?;
match collection.deref() {
    CollectionType::Sharded(sharded) => {
        // Get all shard IDs
        let shard_ids = sharded.get_shard_ids();
        println!("Shard IDs: {:?}", shard_ids);
        
        // Get shard counts
        let counts = sharded.shard_counts();
        for (shard_id, count) in counts {
            println!("Shard {}: {} vectors", shard_id, count);
        }
        
        // Get shard metadata
        for shard_id in shard_ids {
            if let Some(metadata) = sharded.get_shard_metadata(&shard_id) {
                println!("Shard {} metadata: {:?}", shard_id, metadata);
            }
        }
        
        // Check if rebalancing is needed
        if sharded.needs_rebalancing() {
            println!("Rebalancing recommended");
        }
    }
    _ => {}
}
```

### Add/Remove Shards

```rust
match collection.deref() {
    CollectionType::Sharded(sharded) => {
        // Add a new shard
        let new_shard_id = ShardId::new(5);
        sharded.add_shard(new_shard_id, 1.0)?;
        
        // Remove a shard (WARNING: deletes all vectors in that shard)
        // sharded.remove_shard(shard_id)?;
    }
    _ => {}
}
```

## Troubleshooting

### Uneven Shard Distribution

**Problem**: Some shards have many more vectors than others.

**Solutions**:
1. Increase `virtual_nodes_per_shard` for better distribution
2. Check if vector IDs are causing hash collisions
3. Manually trigger rebalancing if available
4. Consider using more diverse vector IDs

### High Memory Usage

**Problem**: Sharded collection uses too much memory.

**Solutions**:
1. Reduce `virtual_nodes_per_shard` (saves memory, may reduce distribution quality)
2. Enable quantization: `QuantizationConfig::SQ { bits: 8 }`
3. Use MMAP storage instead of Memory
4. Reduce number of shards

### Slow Search Performance

**Problem**: Search is slower than expected with sharding.

**Solutions**:
1. Check if shards are evenly distributed (uneven = some shards overloaded)
2. Increase `ef_search` in HNSW config for better quality
3. Reduce number of shards if overhead is too high
4. Consider if sharding is appropriate for your dataset size

### Cannot Access Sharded Collection Methods

**Problem**: Trying to access shard-specific methods on non-sharded collection.

**Solution**: Always check collection type before accessing shard methods:

```rust
let collection = store.get_collection("my_collection")?;
match collection.deref() {
    CollectionType::Sharded(sharded) => {
        // Access shard methods here
    }
    _ => {
        // Collection is not sharded
    }
}
```

## Distributed Sharding (Multi-Server)

Vectorizer supports distributed sharding across multiple server instances for true horizontal scalability. This extends single-server sharding to work across a cluster of servers.

### Enabling Distributed Sharding

To enable distributed sharding, configure the cluster in your `config.yml`:

```yaml
cluster:
  enabled: true
  node_id: "node-1"  # Optional, auto-generated if not specified
  discovery: "static"
  timeout_ms: 5000
  retry_count: 3
  servers:
    - id: "node-1"
      address: "127.0.0.1"
      grpc_port: 15003
    - id: "node-2"
      address: "127.0.0.1"
      grpc_port: 15004
    - id: "node-3"
      address: "127.0.0.1"
      grpc_port: 15005
```

### How Distributed Sharding Works

1. **Shard Assignment**: Shards are automatically assigned to cluster nodes using consistent hashing
2. **Automatic Routing**: Operations are automatically routed to the correct server based on shard location
3. **Parallel Search**: Search operations query all relevant servers in parallel and merge results
4. **Fault Tolerance**: If a server fails, operations continue on available servers

### Distributed Sharding vs Single-Server Sharding

| Feature | Single-Server Sharding | Distributed Sharding |
|---------|----------------------|---------------------|
| **Scalability** | Limited to one server's resources | Scales across multiple servers |
| **Fault Tolerance** | Single point of failure | Continues operating if some servers fail |
| **Setup Complexity** | Simple | Requires cluster configuration |
| **Use Case** | Large collections on one server | Very large collections, high availability |

### Managing Distributed Clusters

Use the REST API or MCP tools to manage your cluster:

**REST API:**
```bash
# List cluster nodes
GET /api/v1/cluster/nodes

# Get shard distribution
GET /api/v1/cluster/shard-distribution

# Trigger rebalancing
POST /api/v1/cluster/rebalance

# Add a node
POST /api/v1/cluster/nodes
{
  "id": "node-4",
  "address": "127.0.0.1",
  "grpc_port": 15006
}
```

**MCP Tools:**
- `cluster_list_nodes` - List all cluster nodes
- `cluster_get_shard_distribution` - Get shard distribution
- `cluster_rebalance` - Trigger shard rebalancing
- `cluster_add_node` - Add node to cluster
- `cluster_remove_node` - Remove node from cluster
- `cluster_get_node_info` - Get node information

### Best Practices for Distributed Sharding

1. **Start Small**: Begin with 2-3 nodes and scale as needed
2. **Even Distribution**: Monitor shard distribution to ensure even load
3. **Network Latency**: Keep nodes in the same data center for low latency
4. **Health Monitoring**: Regularly check node health and shard distribution
5. **Backup Strategy**: Ensure backups are taken across all nodes

## Testing Distributed Sharding

### Running Integration Tests

Vectorizer includes comprehensive integration tests for distributed sharding:

```bash
# Run all cluster tests
cargo test --lib integration::cluster

# Run distributed sharding tests
cargo test --lib integration::distributed_sharding

# Run failure scenario tests
cargo test --lib integration::cluster_failures

# Run performance tests
cargo test --lib integration::cluster_performance

# Run end-to-end tests
cargo test --lib integration::cluster_e2e
```

### Test Coverage

The test suite covers:
- **Basic Operations**: Create, insert, search, update, delete
- **Failure Scenarios**: Node failures, network partitions, recovery
- **Scaling**: Adding/removing nodes, rebalancing
- **Performance**: Concurrent operations, throughput, latency
- **Integration**: Works with quantization, compression, payloads, sparse vectors

### Manual Testing with Real Servers

For testing with actual running servers:

1. **Start multiple servers** on different ports:
   ```bash
   # Server 1
   ./target/release/vectorizer --config config1.yml
   
   # Server 2
   ./target/release/vectorizer --config config2.yml
   
   # Server 3
   ./target/release/vectorizer --config config3.yml
   ```

2. **Configure cluster** in each config file:
   ```yaml
   cluster:
     enabled: true
     node_id: "node-1"  # Unique per server
     servers:
       - id: "node-1"
         address: "127.0.0.1:15002"
         grpc_port: 15012
       - id: "node-2"
         address: "127.0.0.1:15003"
         grpc_port: 15013
       - id: "node-3"
         address: "127.0.0.1:15004"
         grpc_port: 15014
     discovery: static
     timeout_ms: 5000
     retry_count: 3
   ```

3. **Create distributed collection** via API:
   ```bash
   curl -X POST http://127.0.0.1:15002/api/v1/collections \
     -H "Content-Type: application/json" \
     -d '{
       "name": "distributed-collection",
       "config": {
         "dimension": 128,
         "metric": "cosine",
         "sharding": {
           "shard_count": 6,
           "virtual_nodes_per_shard": 100
         }
       }
     }'
   ```

4. **Verify shard distribution**:
   ```bash
   curl http://127.0.0.1:15002/api/v1/cluster/shard-distribution
   ```

## Advanced Sharded Collection Features

### Batch Insert

Sharded collections support efficient batch insert operations that automatically route vectors to the correct shard:

```rust
let vectors = vec![
    Vector { id: "v1".to_string(), data: vec![0.1, 0.2, 0.3, 0.4], ... },
    Vector { id: "v2".to_string(), data: vec![0.5, 0.6, 0.7, 0.8], ... },
    // ... more vectors
];

// Batch insert routes vectors to appropriate shards
sharded_collection.insert_batch(vectors)?;
```

**REST API:**
```bash
POST /collections/{name}/batch_insert
Content-Type: application/json

{
  "vectors": [
    {"id": "v1", "data": [0.1, 0.2, 0.3, 0.4]},
    {"id": "v2", "data": [0.5, 0.6, 0.7, 0.8]}
  ]
}
```

### Hybrid Search

Sharded collections support hybrid search combining dense (semantic) and sparse (keyword) search:

```rust
let results = sharded_collection.hybrid_search(
    &query_vector,
    Some(&query_text),
    k,
    alpha,  // 0.0 = sparse only, 1.0 = dense only, 0.5 = balanced
    None,   // filter
)?;

// Results include both scores
for result in results {
    println!("ID: {}, Score: {}", result.id, result.score);
    println!("  Dense: {:?}, Sparse: {:?}", result.dense_score, result.sparse_score);
}
```

**REST API:**
```bash
POST /collections/{name}/hybrid_search
Content-Type: application/json

{
  "query": [0.1, 0.2, 0.3, 0.4],
  "query_text": "search query",
  "k": 10,
  "alpha": 0.5
}
```

### Document Count

Track document counts across all shards:

```rust
let total_count = sharded_collection.document_count();
println!("Total documents: {}", total_count);

// Per-shard counts
let shard_counts = sharded_collection.shard_counts();
for (shard_id, count) in shard_counts {
    println!("Shard {}: {} documents", shard_id, count);
}
```

### Requantization

Change quantization settings for a sharded collection without re-indexing:

```rust
use vectorizer::quantization::QuantizationConfig;

// Apply new quantization across all shards
let new_config = QuantizationConfig::SQ { bits: 4 };
sharded_collection.requantize(new_config)?;
```

**REST API:**
```bash
POST /collections/{name}/requantize
Content-Type: application/json

{
  "quantization": {
    "type": "scalar",
    "bits": 4
  }
}
```

## Summary

Sharding is a powerful feature for scaling large collections. Key points:

- ✅ Enable sharding for collections with > 100K vectors
- ✅ Start with 4 shards and scale based on needs
- ✅ Use 100 virtual nodes per shard as default
- ✅ Monitor shard distribution regularly
- ✅ Set rebalance threshold based on workload variability
- ✅ Consider MMAP storage for large sharded collections
- ✅ Use distributed sharding for multi-server deployments

For more information, see:
- [Collection Configuration](./CONFIGURATION.md)
- [Collection Operations](./OPERATIONS.md)
- [Performance Tuning](../../configuration/PERFORMANCE_TUNING.md)
- [Cluster Deployment](../../deployment/CLUSTER.md)

