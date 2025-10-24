# Replication System

**Status**: 🚧 **Initial Implementation** - Complete architecture, pending full integration  
**Version**: 1.0.3  
**Inspired by**: Synap Replication, Redis Replication  
**Last Updated**: October 22, 2025

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Configuration](#configuration)
4. [REST API](#rest-api)
5. [Guarantees & Consistency](#guarantees--consistency)
6. [Failure Handling](#failure-handling)
7. [Deployment Examples](#deployment-examples)
8. [Monitoring](#monitoring)

---

## Overview

Vectorizer implements **Master-Replica replication** inspired by Redis and Synap:

- **1 Master Node** - Accepts writes, replicates to replicas
- **N Replica Nodes** - Read-only, receive from master
- **Async Replication** - Non-blocking, eventual consistency
- **Manual Failover** - Promote replica to master when needed

### Key Features

✅ **TCP Communication** - Length-prefixed binary protocol (u32 + bincode)  
✅ **Full Sync** - Complete snapshot transfer with CRC32 verification  
✅ **Partial Sync** - Incremental updates from replication log offset  
✅ **Auto-Reconnect** - Replicas reconnect with intelligent full/partial resync  
✅ **Lag Monitoring** - Real-time offset tracking and lag calculation  
✅ **Write Protection** - Replicas are strictly read-only (enforced)  
✅ **Circular Replication Log** - 1M operations buffer (Redis-style)  
✅ **Eventual Consistency** - System optimized for low latency  

---

## Architecture

### Master Node

```
┌─────────────────────────────────────┐
│         Master Node                  │
│                                      │
│  ┌──────────────────────────────┐  │
│  │   VectorStore (collections)  │  │
│  └──────────────┬───────────────┘  │
│                 │                    │
│  ┌──────────────▼──────────────────┐│
│  │   Replication Log (circular)   ││
│  │   [Op1, Op2, Op3, ..., OpN]    ││
│  └──────────────┬──────────────────┘│
│                 │                    │
│  ┌──────────────▼──────────────────┐│
│  │   TCP Server (bind_address)    ││
│  └──────┬──────┬──────────┬────────┘│
└─────────┼──────┼──────────┼─────────┘
          │      │          │
          ▼      ▼          ▼
     Replica1 Replica2  Replica3
```

**Responsibilities:**
- Maintain replication log of all operations
- Send operations to all connected replicas
- Track replica offsets and lag
- Provide full sync (snapshot) or partial sync

### Replica Node

```
┌─────────────────────────────────────┐
│         Replica Node                 │
│                                      │
│  ┌──────────────────────────────┐  │
│  │   TCP Client (connects to    │  │
│  │   master_address)            │  │
│  └──────────────┬───────────────┘  │
│                 │                    │
│  ┌──────────────▼──────────────────┐│
│  │   Apply Operations              ││
│  │   (CreateCollection, Insert,    ││
│  │    Update, Delete)              ││
│  └──────────────┬──────────────────┘│
│                 │                    │
│  ┌──────────────▼──────────────────┐│
│  │   VectorStore (read-only)       ││
│  └────────────────────────────────┘ │
└─────────────────────────────────────┘
```

**Responsibilities:**
- Connect to master and maintain connection
- Receive and apply operations
- Track local offset
- Auto-reconnect on disconnect

---

## Configuration

Replication can be configured via:
1. **YAML configuration file** (`config.yml`)
2. **Environment variables**
3. **REST API** (`POST /replication/configure`)

### Configuration via YAML

**Master Node** (`config.yml`):
```yaml
replication:
  enabled: true
  role: "master"
  bind_address: "0.0.0.0:7001"
  heartbeat_interval_secs: 5
  replica_timeout_secs: 30
  log_size: 1000000
```

**Replica Node** (`config.yml`):
```yaml
replication:
  enabled: true
  role: "replica"
  master_address: "192.168.1.10:7001"
  reconnect_interval_secs: 5
```

### Configuration via Environment Variables

```bash
# Master Node
export REPLICATION_ROLE=master
export REPLICATION_BIND_ADDRESS=0.0.0.0:7001
export REPLICATION_HEARTBEAT_INTERVAL=5
export REPLICATION_LOG_SIZE=1000000

# Replica Node
export REPLICATION_ROLE=replica
export REPLICATION_MASTER_ADDRESS=192.168.1.10:7001
export REPLICATION_RECONNECT_INTERVAL=5
```

### Configuration Parameters

| Parameter                    | Type   | Default     | Description                               |
|-----------------------------|--------|-------------|-------------------------------------------|
| `enabled`                   | bool   | false       | Enable/disable replication                |
| `role`                      | string | standalone  | Node role: master, replica, standalone    |
| `bind_address`              | string | -           | TCP address for master to listen (master) |
| `master_address`            | string | -           | Master address to connect (replica)       |
| `heartbeat_interval_secs`   | u64    | 5           | Heartbeat interval in seconds             |
| `replica_timeout_secs`      | u64    | 30          | Replica timeout in seconds                |
| `log_size`                  | usize  | 1000000     | Replication log size (operations)         |
| `reconnect_interval_secs`   | u64    | 5           | Auto-reconnect interval in seconds        |

### Quick Start Examples

**Example 1: Development Environment**

```bash
# Terminal 1: Master
cp config.development.yml config.yml
# Edit config.yml:
# replication:
#   enabled: true
#   role: "master"
#   bind_address: "127.0.0.1:7001"
./vectorizer

# Terminal 2: Replica
cp config.development.yml config-replica.yml
# Edit config-replica.yml:
# replication:
#   enabled: true
#   role: "replica"
#   master_address: "127.0.0.1:7001"
./vectorizer --config config-replica.yml
```

**Example 2: Production Environment**

See `config.production.yml` for production-optimized settings.

---

## REST API

### Get Replication Status

```bash
GET /replication/status
```

**Response (v1.2.0+):**
```json
{
  "role": "Master",
  "enabled": true,
  "stats": {
    "role": "master",
    "lag_ms": 0,
    "bytes_sent": 1048576,
    "bytes_received": 0,
    "last_sync": 1729612800,
    "operations_pending": 0,
    "snapshot_size": 524288,
    "connected_replicas": 2,
    "master_offset": 1523,
    "replica_offset": 0,
    "lag_operations": 0,
    "total_replicated": 1523
  },
  "replicas": []
}
```

**Note**: New fields added in v1.2.0:
- `role`: Node role (master/replica)
- `bytes_sent`: Total bytes sent (master only)
- `bytes_received`: Total bytes received  
- `last_sync`: Last sync timestamp (Unix seconds)
- `operations_pending`: Operations waiting to replicate
- `snapshot_size`: Size of last snapshot in bytes
- `connected_replicas`: Number of connected replicas (master only)

Legacy fields maintained for backwards compatibility.

### Configure Replication

```bash
POST /replication/configure
Content-Type: application/json

{
  "role": "master",
  "bind_address": "0.0.0.0:7001",
  "heartbeat_interval": 5,
  "log_size": 1000000
}
```

**Response:**
```json
{
  "success": true,
  "role": "Master",
  "message": "Replication configured successfully. Server restart required."
}
```

### Get Replication Statistics

```bash
GET /replication/stats
```

### List Connected Replicas (Master only)

```bash
GET /replication/replicas
```

**Response (v1.2.0+):**
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
  "count": 1,
  "message": "Replica list will populate when replication is actively running"
}
```

**Replica Status Values:**
- `Connected`: Replica is healthy and syncing normally
- `Syncing`: Replica is performing initial sync
- `Lagging`: Replica lag exceeds 1000ms threshold
- `Disconnected`: No heartbeat received for 60+ seconds

**Note**: New fields added in v1.2.0:
- `host`: Replica IP address
- `port`: Replica port number
- `status`: Health status enum
- `operations_synced`: Total operations successfully synced
- `last_heartbeat`: Unix timestamp of last heartbeat

Legacy fields maintained for backwards compatibility.

---

## Guarantees & Consistency

### Replication Mode

**Async Replication (Default)**
- Master does NOT wait for replica acknowledgment
- Writes complete immediately on master
- Eventual consistency (typically <10ms lag)
- Maximum throughput and low latency

### Data Consistency

✅ **Eventual Consistency**: Replicas eventually reflect master state  
✅ **Operation Ordering**: Operations applied in same order on all replicas  
✅ **CRC32 Verification**: Snapshots verified with checksums  
⚠️ **No Strict Consistency**: Replicas may lag behind master  
⚠️ **No Automatic Failover**: Manual promotion required

---

## Failure Handling

### Replica Disconnect

1. Master detects timeout (default: 30s)
2. Master removes replica from active list
3. Replica auto-reconnects after interval (default: 5s)
4. Master decides: Full sync or Partial sync
   - **Full Sync**: Offset too old (outside log window)
   - **Partial Sync**: Offset available in log

### Master Failure

**Manual Failover Process:**

1. **Stop Master Node**
   ```bash
   # Stop the master process
   systemctl stop vectorizer-master
   ```

2. **Promote Replica to Master**
   ```bash
   # On chosen replica
   POST /replication/configure
   {
     "role": "master",
     "bind_address": "0.0.0.0:7001"
   }
   
   # Restart server
   systemctl restart vectorizer
   ```

3. **Reconfigure Other Replicas**
   ```bash
   # Point to new master
   POST /replication/configure
   {
     "role": "replica",
     "master_address": "new-master-ip:7001"
   }
   ```

---

## Deployment Examples

### Single Master + 2 Replicas

```yaml
# docker-compose.yml
version: '3.8'

services:
  vectorizer-master:
    image: vectorizer:latest
    ports:
      - "15002:15002"  # REST API
      - "7001:7001"    # Replication
    environment:
      - REPLICATION_ROLE=master
      - REPLICATION_BIND_ADDRESS=0.0.0.0:7001
    volumes:
      - master-data:/app/data

  vectorizer-replica-1:
    image: vectorizer:latest
    ports:
      - "15003:15002"
    environment:
      - REPLICATION_ROLE=replica
      - REPLICATION_MASTER_ADDRESS=vectorizer-master:7001
    volumes:
      - replica1-data:/app/data

  vectorizer-replica-2:
    image: vectorizer:latest
    ports:
      - "15004:15002"
    environment:
      - REPLICATION_ROLE=replica
      - REPLICATION_MASTER_ADDRESS=vectorizer-master:7001
    volumes:
      - replica2-data:/app/data

volumes:
  master-data:
  replica1-data:
  replica2-data:
```

---

## Monitoring

### Key Metrics

**Master Metrics:**
- `replication.master_offset` - Current replication offset
- `replication.replicas_connected` - Number of connected replicas
- `replication.log_size` - Current replication log size

**Replica Metrics:**
- `replication.replica_offset` - Current offset on replica
- `replication.lag_operations` - Operations behind master
- `replication.lag_ms` - Time since last heartbeat
- `replication.connected` - Connection status

### Health Checks

```bash
# Check replication health
curl http://master:15002/replication/status

# Check replica lag
curl http://replica:15002/replication/stats
```

---

## Implementation Status

### ✅ Completed

- Replication types and configuration
- Replication log (circular buffer)
- Snapshot creation and application
- Master node implementation
- Replica node implementation
- REST API endpoints
- Basic tests

### 🚧 Pending Integration

- Full server integration (requires restart with replication enabled)
- Real-time stats collection from MasterNode/ReplicaNode
- Prometheus metrics export
- Advanced monitoring dashboard

### 📋 Future Enhancements

- Automatic failover
- Multi-master replication
- Compression for snapshots
- Delta sync optimization
- Replica read load balancing

---

## References

- Synap Replication: `/synap/docs/REPLICATION.md`
- Redis Replication: https://redis.io/docs/manual/replication/
- Implementation: `/vectorizer/src/replication/`

---

**Note**: This is an initial implementation. Full integration requires server restart with replication configuration enabled.

