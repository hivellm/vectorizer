# Add Distributed Horizontal Sharding - Proposal

## Why

Currently, Vectorizer's sharding implementation only distributes vectors across multiple shards within a single server instance. This limits scalability to the resources of a single machine and prevents true horizontal scaling across multiple servers. For production deployments handling millions of vectors and high concurrent load, we need the ability to distribute collections and workload across multiple server instances, enabling:

1. **Horizontal Scalability**: Add more servers to handle increased load without upgrading existing hardware
2. **Load Distribution**: Distribute read/write operations across multiple servers to improve throughput
3. **Fault Tolerance**: Continue operating even if some servers fail
4. **Resource Optimization**: Better utilize available resources across multiple machines
5. **Geographic Distribution**: Support multi-region deployments with servers in different locations

The current sharding implementation is a good foundation, but it needs to be extended to support distributed sharding across multiple server instances.

## What Changes

This task introduces distributed horizontal sharding that enables Vectorizer to operate as a cluster of multiple server instances, with collections and vectors distributed across them:

1. **Cluster Management**: Server discovery, membership management, and cluster state coordination
2. **Distributed Shard Routing**: Route operations to the correct server based on shard location
3. **Cross-Server Communication**: gRPC-based communication between servers for distributed operations
4. **Shard Placement**: Automatic or manual assignment of shards to servers with load balancing
5. **Replication Support**: Optional replication of shards across multiple servers for fault tolerance
6. **Cluster Configuration**: Configuration for cluster topology, server addresses, and shard distribution
7. **REST and MCP APIs**: Expose cluster management and distributed sharding through REST endpoints and MCP tools
8. **Integration with Existing Sharding**: Extend current sharding implementation to work across servers

This builds on the existing `ShardedCollection` implementation but adds the distributed layer that routes shards to different server instances.

## Impact

- **Affected specs**: 
  - `specs/cluster/spec.md` - Add distributed sharding and cluster management specification
  - `specs/api-rest/spec.md` - Add cluster management REST endpoints
  - `specs/db/spec.md` - Extend sharding specification for distributed mode
- **Affected code**: 
  - New module: `src/cluster/mod.rs` - Cluster management and server discovery
  - New module: `src/cluster/shard_router.rs` - Distributed shard routing
  - New module: `src/cluster/server_client.rs` - gRPC client for cross-server communication
  - Modified: `src/db/sharded_collection.rs` - Support distributed shard routing
  - Modified: `src/server/mod.rs` - Add cluster membership and coordination
  - Modified: `src/mcp/tools.rs` - Add cluster management MCP tools
  - Modified: `src/models/mod.rs` - Add cluster configuration models
- **Breaking change**: NO (distributed sharding is opt-in via configuration)
- **User benefit**: 
  - Horizontal scalability across multiple servers
  - Better fault tolerance and availability
  - Improved performance through load distribution
  - Support for larger datasets and higher throughput
  - Foundation for multi-region deployments

## Technical Approach

### Option 1: Custom Implementation with Consistent Hashing
- Use existing consistent hashing from `ShardRouter`
- Add server membership tracking
- Implement gRPC communication between servers
- Pros: Full control, no external dependencies
- Cons: More implementation work

### Option 2: Use Existing Crates
- **faro_sharding**: Provides sharding that doesn't move data between existing destinations when adding new ones
- **whirlwind**: Provides `ShardMap` and `ShardSet` with sharding strategy
- **sharded**: Provides concurrent collections with sharding
- Pros: Less code to maintain, proven implementations
- Cons: May need adaptation for our use case

**Recommendation**: Start with Option 1 (custom implementation) using consistent hashing, but evaluate `faro_sharding` for shard placement logic. This gives us full control while potentially leveraging proven sharding algorithms.

## Dependencies

- Existing sharding implementation (`ShardRouter`, `ShardedCollection`)
- gRPC infrastructure (already in use for MCP)
- Configuration system for cluster settings
- Optional: `faro_sharding` crate for advanced shard placement

