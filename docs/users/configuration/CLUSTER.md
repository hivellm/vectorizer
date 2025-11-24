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

Vectorizer supports distributed sharding, allowing you to distribute collections and vectors across multiple server instances for horizontal scalability. This enables:

- **Horizontal Scaling**: Add more servers to handle increased load
- **Fault Tolerance**: Continue operating if some servers fail
- **Load Distribution**: Automatically distribute shards across cluster nodes
- **Automatic Routing**: Operations are automatically routed to the correct server

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
| `enabled` | boolean | `false` | Enable cluster mode |
| `node_id` | string | auto-generated | Unique identifier for this node |
| `discovery` | string | `"static"` | Discovery method: `"static"` or `"dns"` |
| `timeout_ms` | integer | `5000` | gRPC request timeout in milliseconds |
| `retry_count` | integer | `3` | Number of retries for failed requests |
| `servers` | array | `[]` | List of cluster nodes |

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

#### DNS Discovery (Future)

```yaml
cluster:
  enabled: true
  discovery: "dns"
  dns_name: "vectorizer-cluster.local"
  dns_port: 15003
```

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

- **REST API Port**: Default `15002` (configurable via `server.port`)
- **gRPC Port**: Default `15003` (configurable via `cluster.servers[].grpc_port`)

### Firewall Rules

Ensure the following ports are open:

- REST API port (default: 15002) - for client connections
- gRPC port (default: 15003) - for inter-server communication

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

## Related Documentation

- [Sharding Guide](../collections/SHARDING.md) - Detailed sharding documentation
- [Server Configuration](./SERVER.md) - Network and server settings
- [Performance Tuning](./PERFORMANCE_TUNING.md) - Optimization tips
- [API Documentation](../../api/README.md) - Cluster API endpoints

