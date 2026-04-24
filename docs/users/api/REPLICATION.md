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
- **Async Replication** - Non-blocking, eventual consistency (default)
- **Optional Sync Replication** - `WriteConcern::{Count(n), All}` for strong consistency
- **Operator-Initiated Failover** - In this legacy mode, promotion is driven by config/ops; there is no `/promote` REST handler

### Deployment Modes: Legacy Master-Replica vs. HA Cluster

Vectorizer ships **two distinct replication topologies**. The one documented on this page is the legacy master-replica flavor. Pick the mode that matches your requirements:

| Mode                     | Consistency / Writes                             | Failover                                                       | Where to read                                |
| ------------------------ | ------------------------------------------------ | -------------------------------------------------------------- | -------------------------------------------- |
| Legacy Master-Replica    | Async by default; optional sync via `WriteConcern` | Operator-initiated: promote a replica by reconfiguring it as master and restarting | This document                                |
| HA / Cluster (Raft)      | Strong consistency across the quorum             | **Automatic** — Raft detects a missing leader heartbeat (~1-3s), elects a new leader, and redirects writes via HTTP 307 | [Cluster Deployment Guide](../../deployment/CLUSTER.md) |

If you need automatic failover without human intervention, use HA/Cluster mode (Raft). The master-replica endpoints on this page do **not** expose `promote`/`demote` handlers — promotion in this mode is a configuration-level operation.

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
- `Syncing` - Replica is performing initial sync (offset still 0)
- `Lagging` - Replica lag exceeds the **1000ms** threshold (hard-coded in `ReplicaInfo::update_status`, `crates/vectorizer/src/replication/types.rs`)
- `Disconnected` - No heartbeat received for 60+ seconds

These thresholds are evaluated by `ReplicaInfo::update_status()` on every
heartbeat tick; the lag threshold is currently a compile-time constant and
is not configurable via YAML or environment variables.

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

### Sync Replication (Available, v3.0+)

Sync replication is implemented and tested. The master exposes the
`replicate_with_concern(operation, concern, timeout)` API
(see `crates/vectorizer/src/replication/master.rs`), which blocks until the
requested number of replica acknowledgements has arrived or the timeout
expires.

The write concern is modeled by the `WriteConcern` enum
(`crates/vectorizer/src/replication/types.rs`):

| Variant         | Semantics                                                   |
| --------------- | ----------------------------------------------------------- |
| `None`          | Fire-and-forget. Returns as soon as the op is logged locally. This is the default. |
| `Count(n)`      | Wait for at least `n` replicas to ACK the op before returning. |
| `All`           | Wait for every currently connected replica to ACK.          |

If the configured number of ACKs is not reached before `timeout`, the call
returns `ReplicationError::WriteConcernTimeout { required, confirmed, offset }`
and the caller can retry or surface the error — the operation itself is still
durably recorded in the master's WAL.

**Rust example:**

```rust
use std::time::Duration;
use vectorizer::replication::types::WriteConcern;

// Wait for at least 2 replicas to ACK, or fail after 500ms
let offset = master
    .replicate_with_concern(op, WriteConcern::Count(2), Duration::from_millis(500))
    .await?;
```

- Best for: critical data consistency, quorum writes, or fencing before
  surfacing a successful write to the client.
- Trade-off: each sync write incurs at least one network round-trip to the
  slowest acknowledged replica.

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

