# Cluster API Documentation

Complete reference for Vectorizer cluster management API endpoints.

**Version:** 1.4.0  
**Base Path:** `/api/v1/cluster`  
**Authentication:** Not required (consider adding in production)

## Overview

The Cluster API provides endpoints for managing distributed sharding across multiple Vectorizer server instances. Use these endpoints to:

- Monitor cluster nodes and their status
- View shard distribution across nodes
- Trigger shard rebalancing
- Add or remove nodes from the cluster

## Endpoints

### List Cluster Nodes

Get a list of all nodes in the cluster.

**Endpoint:** `GET /api/v1/cluster/nodes`

**Response:**
```json
{
  "nodes": [
    {
      "id": "node-1",
      "address": "127.0.0.1",
      "grpc_port": 15003,
      "status": "active",
      "shard_count": 2,
      "last_seen": "2025-01-15T10:30:00Z"
    },
    {
      "id": "node-2",
      "address": "127.0.0.1",
      "grpc_port": 15004,
      "status": "active",
      "shard_count": 2,
      "last_seen": "2025-01-15T10:30:00Z"
    }
  ]
}
```

**Example:**
```bash
curl "http://localhost:15002/api/v1/cluster/nodes"
```

### Get Node Information

Get detailed information about a specific node.

**Endpoint:** `GET /api/v1/cluster/nodes/:node_id`

**Parameters:**
- `node_id` (path) - Node identifier

**Response:**
```json
{
  "id": "node-1",
  "address": "127.0.0.1",
  "grpc_port": 15003,
  "status": "active",
  "shard_count": 2,
  "shards": [0, 1],
  "last_seen": "2025-01-15T10:30:00Z",
  "uptime_seconds": 3600
}
```

**Example:**
```bash
curl "http://localhost:15002/api/v1/cluster/nodes/node-1"
```

**Error Responses:**
- `404 Not Found` - Node not found

### Get Shard Distribution

Get the distribution of shards across cluster nodes.

**Endpoint:** `GET /api/v1/cluster/shard-distribution`

**Response:**
```json
{
  "distribution": {
    "node-1": [0, 1, 2],
    "node-2": [3, 4, 5]
  },
  "total_shards": 6,
  "nodes": 2,
  "shards_per_node": {
    "node-1": 3,
    "node-2": 3
  }
}
```

**Example:**
```bash
curl "http://localhost:15002/api/v1/cluster/shard-distribution"
```

### Trigger Rebalancing

Manually trigger shard rebalancing across cluster nodes.

**Endpoint:** `POST /api/v1/cluster/rebalance`

**Request Body:**
```json
{
  "force": false
}
```

**Parameters:**
- `force` (boolean, optional) - Force rebalancing even if distribution is balanced

**Response:**
```json
{
  "status": "success",
  "message": "Rebalancing triggered",
  "migrations": [
    {
      "shard_id": 2,
      "from_node": "node-1",
      "to_node": "node-2"
    }
  ]
}
```

**Example:**
```bash
curl -X POST "http://localhost:15002/api/v1/cluster/rebalance" \
  -H "Content-Type: application/json" \
  -d '{"force": false}'
```

### Add Node to Cluster

Add a new node to the cluster.

**Endpoint:** `POST /api/v1/cluster/nodes`

**Request Body:**
```json
{
  "id": "node-3",
  "address": "127.0.0.1",
  "grpc_port": 15005
}
```

**Parameters:**
- `id` (string, required) - Unique node identifier
- `address` (string, required) - Node IP address or hostname
- `grpc_port` (integer, required) - gRPC port for inter-server communication

**Response:**
```json
{
  "status": "success",
  "message": "Node added to cluster",
  "node": {
    "id": "node-3",
    "address": "127.0.0.1",
    "grpc_port": 15005,
    "status": "active"
  }
}
```

**Example:**
```bash
curl -X POST "http://localhost:15002/api/v1/cluster/nodes" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "node-3",
    "address": "127.0.0.1",
    "grpc_port": 15005
  }'
```

**Error Responses:**
- `400 Bad Request` - Invalid node configuration
- `409 Conflict` - Node already exists

### Remove Node from Cluster

Remove a node from the cluster. Shards on the removed node will be redistributed.

**Endpoint:** `DELETE /api/v1/cluster/nodes/:node_id`

**Parameters:**
- `node_id` (path) - Node identifier to remove

**Response:**
```json
{
  "status": "success",
  "message": "Node removed from cluster",
  "migrations": [
    {
      "shard_id": 0,
      "from_node": "node-1",
      "to_node": "node-2"
    }
  ]
}
```

**Example:**
```bash
curl -X DELETE "http://localhost:15002/api/v1/cluster/nodes/node-1"
```

**Error Responses:**
- `404 Not Found` - Node not found
- `400 Bad Request` - Cannot remove last node in cluster

## Status Codes

| Code | Description |
|------|-------------|
| `200 OK` | Request successful |
| `400 Bad Request` | Invalid request parameters |
| `404 Not Found` | Resource not found |
| `409 Conflict` | Resource conflict (e.g., node already exists) |
| `500 Internal Server Error` | Server error |

## Node Status

Nodes can have the following statuses:

- `active` - Node is healthy and operational
- `unhealthy` - Node is not responding to health checks
- `removed` - Node has been removed from the cluster

## Shard Migration

When nodes are added or removed, shards are automatically migrated:

1. **Adding a Node**: Shards are redistributed to include the new node
2. **Removing a Node**: Shards on the removed node are migrated to remaining nodes
3. **Rebalancing**: Shards are redistributed for even load

Migration happens automatically in the background. Use the rebalance endpoint to trigger manual rebalancing.

## Best Practices

### Monitoring

Regularly check cluster status:

```bash
# Check node health
curl "http://localhost:15002/api/v1/cluster/nodes"

# Monitor shard distribution
curl "http://localhost:15002/api/v1/cluster/shard-distribution"
```

### Adding Nodes

1. Add the node to all cluster configurations
2. Start the new node
3. Verify it appears in the node list
4. Trigger rebalancing if needed

### Removing Nodes

1. Verify other nodes can handle the load
2. Remove the node via API
3. Wait for shard migration to complete
4. Stop the node process

### Rebalancing

- Rebalancing is automatic when nodes are added/removed
- Use manual rebalancing for fine-tuning
- Monitor shard distribution after rebalancing

## Related Documentation

- [Cluster Configuration Guide](../users/configuration/CLUSTER.md) - Configuration details
- [Sharding Guide](../users/collections/SHARDING.md) - Sharding concepts
- [REST API Overview](./README.md) - General API documentation

