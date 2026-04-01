---
title: Cluster Configuration Guide
module: configuration
id: cluster-configuration
order: 5
description: Configuring Vectorizer cluster for distributed sharding
tags: [cluster, distributed, sharding, configuration]
---

# Cluster Configuration Guide

Complete guide to configuring Vectorizer cluster for distributed sharding across multiple server instances.

## Overview

Vectorizer supports two cluster modes:

### 1. Distributed Sharding (gRPC cluster)
Distribute collections and vectors across multiple servers for horizontal scalability.

### 2. High Availability with Raft (v2.5.4+, recommended for production)
Automatic leader election and failover using Raft consensus. One leader accepts writes; followers serve reads and replicate data via TCP.

Both modes use the same `cluster` config section. When `cluster.enabled: true`, both sharding and Raft are active.

**Key features:**
- **Horizontal Scaling**: Add more servers to handle increased load
- **Automatic Failover**: Raft elects a new leader if the current one dies (~1-3s)
- **Load Distribution**: Automatically distribute shards across cluster nodes
- **Automatic Routing**: Writes are redirected to the leader via HTTP 307
- **DNS Re-resolution**: Replicas reconnect to the master at its new IP after pod restarts (v2.5.4)

## Quick Start

### 1. Enable Cluster Mode

Edit your `config.yml`:

```yaml
cluster:
  enabled: true
  node_id: "node-1"  # Unique ID for this node
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

### 2. Start Multiple Servers

**Node 1:**
```bash
vectorizer --config config-node1.yml --port 15002
```

**Node 2:**
```bash
vectorizer --config config-node2.yml --port 15003
```

**Node 3:**
```bash
vectorizer --config config-node3.yml --port 15004
```

### 3. Create Sharded Collection

```bash
curl -X POST "http://localhost:15002/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "distributed-collection",
    "dimension": 512,
    "metric": "cosine",
    "sharding": {
      "shard_count": 6,
      "virtual_nodes_per_shard": 100
    }
  }'
```

## Configuration Options

### Cluster Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `false` | Enable cluster mode (Raft + sharding) |
| `node_id` | string | auto-generated | Unique identifier for this node |
| `discovery` | string | `"static"` | Discovery method: `"static"` or `"dns"` |
| `timeout_ms` | integer | `5000` | gRPC request timeout in milliseconds |
| `retry_count` | integer | `3` | Number of retries for failed requests |
| `servers` | array | `[]` | List of cluster nodes (all must be listed for Raft) |
| `dns_name` | string | `""` | Kubernetes headless service FQDN (for DNS discovery) |
| `dns_resolve_interval` | integer | `30` | DNS re-resolution interval in seconds |
| `dns_grpc_port` | integer | `15003` | gRPC port for DNS-discovered nodes |
| `raft_node_id` | integer | auto | Explicit Raft node ID (auto-derived from `node_id` hash) |
| `memory.enforce_mmap_storage` | boolean | `true` | Reject Memory storage in cluster mode |
| `memory.disable_file_watcher` | boolean | `true` | File watcher incompatible with clusters |
| `memory.max_cache_memory_bytes` | integer | `1073741824` | Global cache limit (1GB default) |
| `memory.strict_validation` | boolean | `true` | Fail startup on config errors |

### Server Configuration

Each server in the `servers` array requires:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique node identifier |
| `address` | string | Yes | Server IP address or hostname |
| `grpc_port` | integer | Yes | gRPC port for inter-server communication |

### Discovery Methods

#### Static Discovery

Configure all nodes manually in the configuration file:

```yaml
cluster:
  enabled: true
  discovery: "static"
  servers:
    - id: "node-1"
      address: "192.168.1.10"
      grpc_port: 15003
    - id: "node-2"
      address: "192.168.1.11"
      grpc_port: 15003
    - id: "node-3"
      address: "192.168.1.12"
      grpc_port: 15003
```

#### DNS Discovery (Kubernetes)

Used with Kubernetes headless services for automatic pod discovery:

```yaml
cluster:
  enabled: true
  discovery: "dns"
  dns_name: "vectorizer-headless.<namespace>.svc.cluster.local"
  dns_resolve_interval: 30   # Re-resolve DNS every 30s
  dns_grpc_port: 15003
```

> **v2.5.4:** Stale nodes that remain unavailable for >5 minutes are automatically garbage collected.

## HA / Raft Configuration (v2.5.4+)

When `cluster.enabled: true`, Raft consensus handles leader election automatically. Combined with replication, this provides full HA with automatic failover.

### Required settings

```yaml
cluster:
  enabled: true
  node_id: "node-1"        # Unique per node (in K8s, use pod hostname)
  servers:                  # ALL nodes must be listed
    - id: "node-1"
      address: "node-1.headless.svc.cluster.local"
      grpc_port: 15003
    - id: "node-2"
      address: "node-2.headless.svc.cluster.local"
      grpc_port: 15003
    - id: "node-3"
      address: "node-3.headless.svc.cluster.local"
      grpc_port: 15003
  memory:
    enforce_mmap_storage: true
    disable_file_watcher: true

replication:
  enabled: true
  bind_address: "0.0.0.0:7001"
  # Do NOT set role — Raft manages it automatically

file_watcher:
  enabled: false             # MUST be false in cluster mode

auth:
  enabled: true
  jwt_secret: "min-32-chars"  # Required field — config parse fails without it
  jwt_expiration: 3600
  api_key_length: 32
```

### How it works

1. All nodes start and bootstrap Raft with the configured members
2. Raft elects a leader (~1-3s after all nodes are up)
3. Leader starts `MasterNode` on port 7001 (TCP replication)
4. Followers resolve the leader address and start `ReplicaNode`
5. Replicas connect to leader, receive full sync, then heartbeats
6. If leader dies, Raft elects a new leader automatically

### Kubernetes-specific

In Kubernetes, use an init container to inject the pod hostname as `node_id`:

```yaml
initContainers:
  - name: config-selector
    image: busybox:1.36
    command: ["sh", "-c"]
    args:
      - sed "s/__NODE_ID__/$HOSTNAME/g" /configs/config-template.yml > /active-config/config.yml
```

Required environment variables:
- `HOSTNAME` — from `fieldRef: metadata.name`
- `POD_IP` — from `fieldRef: status.podIP`
- `VECTORIZER_SERVICE_NAME` — headless service FQDN

See [HA Cluster Guide](../guides/HA_CLUSTER.md) for complete Kubernetes deployment instructions.

### Common pitfalls (fixed in v2.5.4)

| Problem | Cause | Fix |
|---|---|---|
| Replicas stuck on stale IP | DNS resolved once at startup | Re-resolved on every reconnect |
| Panic on startup | Axum 0.6 path syntax `:node_id` | Fixed to `{node_id}` |
| No automatic failover | Static `role: "master"` | Use Raft (`cluster.enabled: true`) |
| `Address already in use` on 7001 | Fast leader transitions | `SO_REUSEADDR` on TCP listener |
| File watcher in cluster mode | `file_watcher.enabled: true` | Must be `false` |

## Sharding Configuration

When creating a collection with distributed sharding, configure the sharding parameters:

```yaml
sharding:
  shard_count: 6          # Number of shards (distributed across nodes)
  virtual_nodes_per_shard: 100  # Virtual nodes for consistent hashing
```

### Shard Distribution

Shards are automatically distributed across cluster nodes using consistent hashing:

- Each shard is assigned to a node based on consistent hashing
- Virtual nodes ensure even distribution
- Shards can be rebalanced when nodes are added or removed

### Rebalancing

Shards are automatically rebalanced when:

- A new node is added to the cluster
- A node is removed from the cluster
- Manual rebalancing is triggered via API

## Network Configuration

### Ports

Each node requires:

- **REST API Port**: Default `15002` (configurable via `server.port`) — client connections
- **gRPC Port**: Default `15003` (configurable via `cluster.servers[].grpc_port`) — Raft consensus + cluster ops
- **Replication Port**: Default `7001` (configurable via `replication.bind_address`) — TCP data replication

### Firewall Rules

Ensure the following ports are open between all cluster nodes:

- REST API port (default: 15002) — client connections + write redirects
- gRPC port (default: 15003) — Raft consensus, cluster state sync
- Replication port (default: 7001) — master-replica TCP data streaming

### Example Firewall Configuration

**UFW (Ubuntu):**
```bash
sudo ufw allow 15002/tcp  # REST API
sudo ufw allow 15003/tcp  # gRPC
```

**FirewallD (CentOS/RHEL):**
```bash
sudo firewall-cmd --add-port=15002/tcp --permanent  # REST API
sudo firewall-cmd --add-port=15003/tcp --permanent  # gRPC
sudo firewall-cmd --reload
```

## Health Checks

### Node Health

Each node periodically checks the health of other nodes:

- Health check interval: 5 seconds (configurable)
- Node marked as unhealthy after 3 consecutive failures
- Unhealthy nodes are excluded from shard routing

### Monitoring

Use the cluster API to monitor node health:

```bash
# List all nodes
curl "http://localhost:15002/api/v1/cluster/nodes"

# Get node info
curl "http://localhost:15002/api/v1/cluster/nodes/node-1"

# Get shard distribution
curl "http://localhost:15002/api/v1/cluster/shard-distribution"
```

## Best Practices

### 1. Node IDs

- Use descriptive, unique node IDs (e.g., `node-1`, `node-2`, `node-dc1-1`)
- Avoid changing node IDs after deployment
- Use consistent naming conventions

### 2. Network Configuration

- Keep nodes in the same data center for low latency
- Use private networks for inter-server communication
- Configure load balancers for client connections

### 3. Shard Count

- Start with 2-4 shards per node
- Increase shard count as data grows
- Use `shard_count = number_of_nodes * 2` as a rule of thumb

### 4. Monitoring

- Monitor node health regularly
- Set up alerts for node failures
- Track shard distribution and rebalancing

### 5. Backup Strategy

- Backup data from all nodes
- Ensure backups are consistent across nodes
- Test backup restoration procedures

## Troubleshooting

### Node Not Joining Cluster

**Problem**: Node doesn't appear in cluster node list

**Solutions**:
1. Verify `cluster.enabled: true` in config
2. Check network connectivity between nodes
3. Verify gRPC ports are open
4. Check server logs for connection errors

### Shards Not Distributed Evenly

**Problem**: Some nodes have more shards than others

**Solutions**:
1. Trigger manual rebalancing:
   ```bash
   curl -X POST "http://localhost:15002/api/v1/cluster/rebalance"
   ```
2. Check node health - unhealthy nodes are excluded
3. Verify all nodes are in the cluster

### High Latency on Remote Operations

**Problem**: Operations on remote shards are slow

**Solutions**:
1. Check network latency between nodes
2. Verify nodes are in the same data center
3. Consider increasing `timeout_ms` if needed
4. Check for network congestion

### Node Marked as Unhealthy

**Problem**: Node appears as unhealthy in cluster status

**Solutions**:
1. Check node logs for errors
2. Verify gRPC port is accessible
3. Check network connectivity
4. Restart the node if needed

## Example Configurations

### Single Data Center

```yaml
cluster:
  enabled: true
  node_id: "node-dc1-1"
  discovery: "static"
  servers:
    - id: "node-dc1-1"
      address: "10.0.1.10"
      grpc_port: 15003
    - id: "node-dc1-2"
      address: "10.0.1.11"
      grpc_port: 15003
    - id: "node-dc1-3"
      address: "10.0.1.12"
      grpc_port: 15003
```

### Multi-Data Center

```yaml
cluster:
  enabled: true
  node_id: "node-dc1-1"
  discovery: "static"
  servers:
    # Data Center 1
    - id: "node-dc1-1"
      address: "10.0.1.10"
      grpc_port: 15003
    - id: "node-dc1-2"
      address: "10.0.1.11"
      grpc_port: 15003
    # Data Center 2
    - id: "node-dc2-1"
      address: "10.0.2.10"
      grpc_port: 15003
    - id: "node-dc2-2"
      address: "10.0.2.11"
      grpc_port: 15003
```

## Distributed Collection Features

### Shard Router

The shard router provides automatic routing of operations to the correct cluster node:

```rust
// Get all shards across the cluster
let all_shards = shard_router.get_all_shards();

// Route a vector operation to the correct node
let node_id = shard_router.route_vector(&vector_id);
```

### Document Count

Track document counts across distributed collections:

```bash
# Get collection info including document count
curl "http://localhost:15002/collections/my_collection"
```

Response includes distributed counts:
```json
{
  "name": "my_collection",
  "vector_count": 1000000,
  "document_count": 50000,
  "shards": {
    "total": 6,
    "active": 6
  }
}
```

### Remote Operations

The cluster service supports remote operations via gRPC:

| Operation | Status | Description |
|-----------|--------|-------------|
| RemoteInsertVector | Implemented | Insert vector on remote node |
| RemoteUpdateVector | Implemented | Update vector on remote node |
| RemoteDeleteVector | Implemented | Delete vector on remote node |
| RemoteSearchVectors | Implemented | Search across remote shards |
| RemoteGetCollectionInfo | Implemented | Get collection info from remote node |
| RemoteCreateCollection | Planned | Create collection on remote node |
| RemoteDeleteCollection | Planned | Delete collection on remote node |

### Multi-Tenant Cluster Operations

All cluster operations support tenant context for multi-tenant deployments:

```protobuf
message TenantContext {
    string tenant_id = 1;          // Tenant/user ID
    optional string username = 2;  // For logging
    repeated string permissions = 3; // read, write, admin
    optional string trace_id = 4;  // Distributed tracing
}
```

### Quota Management

The cluster service includes distributed quota checking:

```bash
# Check quota across cluster
curl "http://localhost:15002/api/v1/cluster/quota?tenant_id=<uuid>&type=vectors"
```

Response:
```json
{
  "allowed": true,
  "current_usage": 50000,
  "limit": 100000,
  "remaining": 50000
}
```

## Related Documentation

- [Sharding Guide](../collections/SHARDING.md) - Detailed sharding documentation
- [Server Configuration](./SERVER.md) - Network and server settings
- [Performance Tuning](./PERFORMANCE_TUNING.md) - Optimization tips
- [API Documentation](../api/API_REFERENCE.md) - Cluster API endpoints
- [gRPC API](../api/GRPC.md) - Cluster gRPC service documentation

