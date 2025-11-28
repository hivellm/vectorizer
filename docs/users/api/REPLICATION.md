---
title: Replication API
module: api
id: replication-api
order: 7
description: Master-replica replication for high availability
tags: [api, replication, high-availability, master-replica]
---

# Replication API

The Replication API enables master-replica replication for high availability and read scaling.

## Overview

Vectorizer implements **Master-Replica replication**:
- **1 Master Node** - Accepts writes, replicates to replicas
- **N Replica Nodes** - Read-only, receive from master
- **Async Replication** - Non-blocking, eventual consistency
- **Manual Failover** - Promote replica to master when needed

### Key Features

- TCP Communication - Length-prefixed binary protocol
- Full Sync - Complete snapshot transfer with CRC32 verification
- Partial Sync - Incremental updates from replication log offset
- Auto-Reconnect - Replicas reconnect with intelligent full/partial resync
- Lag Monitoring - Real-time offset tracking and lag calculation
- Write Protection - Replicas are strictly read-only (enforced)
- Circular Replication Log - 1M operations buffer

## Configuration

### Master Node Configuration

**YAML Configuration:**

```yaml
replication:
  enabled: true
  role: "master"
  bind_address: "0.0.0.0:7001"
  heartbeat_interval_secs: 5
  replica_timeout_secs: 30
  log_size: 1000000
```

**Environment Variables:**

```bash
export REPLICATION_ROLE=master
export REPLICATION_BIND_ADDRESS=0.0.0.0:7001
export REPLICATION_HEARTBEAT_INTERVAL=5
export REPLICATION_LOG_SIZE=1000000
```

### Replica Node Configuration

**YAML Configuration:**

```yaml
replication:
  enabled: true
  role: "replica"
  master_address: "192.168.1.10:7001"
  reconnect_interval_secs: 5
```

**Environment Variables:**

```bash
export REPLICATION_ROLE=replica
export REPLICATION_MASTER_ADDRESS=192.168.1.10:7001
export REPLICATION_RECONNECT_INTERVAL=5
```

## API Endpoints

### Get Replication Status

Get current replication status and configuration.

**Endpoint:** `GET /replication/status`

**Response:**

```json
{
  "enabled": true,
  "role": "master",
  "bind_address": "0.0.0.0:7001",
  "connected_replicas": 2,
  "replication_log_size": 1000000,
  "current_offset": 15234
}
```

**Example:**

```bash
curl http://localhost:15002/replication/status
```

**Python SDK:**

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

status = await client.get_replication_status()
print(f"Role: {status['role']}")
print(f"Connected replicas: {status['connected_replicas']}")
```

### Configure Replication

Configure replication settings (requires server restart).

**Endpoint:** `POST /replication/configure`

**Request Body:**

```json
{
  "role": "master",
  "bind_address": "0.0.0.0:7001",
  "heartbeat_interval": 5,
  "log_size": 1000000
}
```

**Parameters:**

| Parameter            | Type   | Required      | Description                                    |
| -------------------- | ------ | ------------- | ---------------------------------------------- |
| `role`               | string | Yes           | Node role: `master` or `replica`               |
| `bind_address`       | string | No (master)   | Bind address for master (e.g., `0.0.0.0:7001`) |
| `master_address`     | string | No (replica)  | Master address for replica (e.g., `192.168.1.10:7001`) |
| `heartbeat_interval` | number | No            | Heartbeat interval in seconds (default: 5)    |
| `log_size`           | number | No            | Replication log size (default: 1000000)       |
| `reconnect_interval` | number | No            | Reconnect interval for replicas (default: 5)  |

**Response:**

```json
{
  "success": true,
  "role": "Master",
  "message": "Replication configured successfully. Server restart required."
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/replication/configure \
  -H "Content-Type: application/json" \
  -d '{
    "role": "master",
    "bind_address": "0.0.0.0:7001",
    "heartbeat_interval": 5
  }'
```

### Get Replication Statistics

Get detailed replication statistics.

**Endpoint:** `GET /replication/stats`

**Response:**

```json
{
  "role": "master",
  "bytes_sent": 104857600,
  "bytes_received": 0,
  "last_sync": 1729612800,
  "operations_pending": 0,
  "snapshot_size": 52428800,
  "connected_replicas": 2,
  "current_offset": 15234,
  "replication_log_size": 1000000
}
```

**Example:**

```bash
curl http://localhost:15002/replication/stats
```

### List Connected Replicas

List all connected replica nodes (master only).

**Endpoint:** `GET /replication/replicas`

**Response:**

```json
{
  "replicas": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "host": "192.168.1.10",
      "port": 6381,
      "status": "Connected",
      "lag_ms": 150,
      "last_heartbeat": 1729612800,
      "operations_synced": 1500,
      "address": "192.168.1.10:6381",
      "offset": 1500,
      "lag_operations": 23
    }
  ],
  "count": 1
}
```

**Replica Status Values:**

- `Connected` - Replica is healthy and syncing normally
- `Syncing` - Replica is performing initial sync
- `Lagging` - Replica lag exceeds 1000ms threshold
- `Disconnected` - No heartbeat received for 60+ seconds

**Example:**

```bash
curl http://localhost:15002/replication/replicas
```

**Python SDK:**

```python
replicas = await client.list_replicas()

for replica in replicas["replicas"]:
    print(f"Replica {replica['id']}: {replica['status']}")
    print(f"  Lag: {replica['lag_ms']}ms")
    print(f"  Operations synced: {replica['operations_synced']}")
```

## Replication Modes

### Async Replication (Default)

- Master does NOT wait for replica acknowledgment
- Low latency, high throughput
- Eventual consistency
- Best for: High-performance applications

### Sync Replication (Future)

- Master waits for replica acknowledgment
- Strong consistency
- Higher latency
- Best for: Critical data consistency

## Use Cases

### High Availability Setup

Set up master-replica for high availability:

```python
# Master node configuration
master_config = {
    "role": "master",
    "bind_address": "0.0.0.0:7001",
    "heartbeat_interval": 5
}

await client.configure_replication(master_config)

# Replica node configuration
replica_config = {
    "role": "replica",
    "master_address": "192.168.1.10:7001",
    "reconnect_interval": 5
}

await client.configure_replication(replica_config)
```

### Read Scaling

Use replicas for read-heavy workloads:

```python
# Master handles writes
master_client = VectorizerClient("http://master:15002")
await master_client.insert_text("collection", "New document")

# Replicas handle reads
replica_client = VectorizerClient("http://replica1:15002")
results = await replica_client.search("collection", "query")
```

### Monitoring Replication

Monitor replication health:

```python
# Check replication status
status = await client.get_replication_status()
print(f"Role: {status['role']}")
print(f"Connected replicas: {status['connected_replicas']}")

# Get detailed statistics
stats = await client.get_replication_stats()
print(f"Bytes sent: {stats['bytes_sent']}")
print(f"Current offset: {stats['current_offset']}")

# List replicas
replicas = await client.list_replicas()
for replica in replicas["replicas"]:
    if replica["status"] != "Connected":
        print(f"Warning: Replica {replica['id']} is {replica['status']}")
```

## Best Practices

1. **Use async replication for performance**: Default async mode provides best performance
2. **Monitor replica lag**: Check `lag_ms` and `lag_operations` regularly
3. **Set appropriate log size**: Larger logs allow longer disconnections
4. **Use multiple replicas**: Distribute read load across replicas
5. **Monitor heartbeat**: Ensure replicas stay connected
6. **Plan for failover**: Have a process to promote replica to master

## Related Topics

- [Operations Guide](../operations/MONITORING.md) - Monitoring and health checks
- [Configuration Guide](../configuration/CONFIGURATION.md) - Server configuration
- [Backup and Restore](./BACKUP_RESTORE.md) - Data protection

