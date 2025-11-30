# Cluster Quick Start Guide

Quick guide to setting up a Vectorizer cluster for distributed sharding.

## Prerequisites

- 2+ servers with Vectorizer installed
- Network connectivity between servers
- Same Vectorizer version on all nodes

## Step 1: Configure Each Node

### Node 1 (`config-node1.yml`)

```yaml
server:
  host: "0.0.0.0"
  port: 15002

cluster:
  enabled: true
  node_id: "node-1"
  discovery: "static"
  timeout_ms: 5000
  retry_count: 3
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

### Node 2 (`config-node2.yml`)

```yaml
server:
  host: "0.0.0.0"
  port: 15002

cluster:
  enabled: true
  node_id: "node-2"  # Different node_id
  discovery: "static"
  timeout_ms: 5000
  retry_count: 3
  servers:
    # Same servers list as Node 1
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

### Node 3 (`config-node3.yml`)

```yaml
server:
  host: "0.0.0.0"
  port: 15002

cluster:
  enabled: true
  node_id: "node-3"  # Different node_id
  discovery: "static"
  timeout_ms: 5000
  retry_count: 3
  servers:
    # Same servers list as Node 1 and 2
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

## Step 2: Start Servers

```bash
# Node 1
vectorizer --config config-node1.yml

# Node 2
vectorizer --config config-node2.yml

# Node 3
vectorizer --config config-node3.yml
```

## Step 3: Verify Cluster

```bash
curl http://192.168.1.10:15002/api/v1/cluster/nodes
```

Expected response:
```json
{
  "nodes": [
    {"id": "node-1", "status": "active", ...},
    {"id": "node-2", "status": "active", ...},
    {"id": "node-3", "status": "active", ...}
  ]
}
```

## Step 4: Create Distributed Collection

```bash
curl -X POST "http://192.168.1.10:15002/api/v1/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "distributed-collection",
    "config": {
      "dimension": 128,
      "metric": "cosine",
      "sharding": {
        "shard_count": 6,
        "virtual_nodes_per_shard": 100,
        "rebalance_threshold": 0.2
      }
    }
  }'
```

## Step 5: Verify Shard Distribution

```bash
curl http://192.168.1.10:15002/api/v1/cluster/shard-distribution
```

## Common Issues

### Node Not Joining Cluster

- Verify `cluster.enabled: true` in config
- Check network connectivity between nodes
- Ensure gRPC ports are open (default: 15003)
- Verify all nodes have the same `servers` list

### Shards Not Distributed

- Trigger rebalancing: `curl -X POST http://localhost:15002/api/v1/cluster/rebalance`
- Check node health: `curl http://localhost:15002/api/v1/cluster/nodes`
- Verify all nodes are active

## Next Steps

- [Full Cluster Configuration Guide](./CLUSTER.md)
- [Sharding Guide](../collections/SHARDING.md)
- [Cluster Deployment Guide](../../deployment/CLUSTER.md)
- [API Reference](../api/API_REFERENCE.md) - Complete API documentation

