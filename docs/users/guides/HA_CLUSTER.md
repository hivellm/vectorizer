# High Availability Cluster Guide

Complete guide to deploying Vectorizer in HA (High Availability) mode with automatic replication, write routing, and failover.

## Overview

Vectorizer v2.5.0 supports a master-replica HA architecture:

```
Your Application
      │
      ▼
┌─────────────┐
│  Service URL │  ← single connection point
│  :15002      │
└──────┬──────┘
       │  load balances across pods
       │
  ┌────┼────────────────┐
  │    │                │
┌─▼──┐ ┌──▼─┐ ┌───▼┐
│Lead│ │Fol.│ │Fol.│
│    │ │    │ │    │
│R/W │ │ R  │ │ R  │
└─┬──┘ └──▲─┘ └──▲─┘
  │        │      │
  └────────┴──────┘
    TCP Replication
      (port 7001)
```

- **Leader (master)**: Accepts reads and writes. Replicates data to followers.
- **Followers (replicas)**: Serve reads locally. Answer writes with HTTP 307 pointing at the leader — they do not forward them.
- **Reads anywhere, writes to the leader**: Reads are served by any node. Writes must reach the leader, either directly or by re-sending to the `leader_url` a follower returns (see [Write Routing](#write-routing-http-307)).

## Quick Start

### Docker Compose (recommended for local/dev)

**1. Create `.env` file with credentials:**

```bash
cat > .env << 'EOF'
VECTORIZER_ADMIN_PASSWORD=MySecurePassword123!
VECTORIZER_JWT_SECRET=my-jwt-secret-key-must-be-at-least-32-characters-long
EOF
```

**2. Create master config (`config.ha-master.yml`):**

Copy from `config/config.example.yml` and change these sections:

```yaml
# server section
server:
  host: "0.0.0.0"
  port: 15002
  mcp_port: 15002

# auth section - enable with shared JWT secret
auth:
  enabled: true
  jwt_secret: "${VECTORIZER_JWT_SECRET}"  # from .env

# replication section
replication:
  enabled: true
  role: "master"
  bind_address: "0.0.0.0:7001"
  heartbeat_interval_secs: 5
  log_size: 1000000
```

**3. Create replica config (`config.ha-replica.yml`):**

Same as master but with these differences:

```yaml
# replication section
replication:
  enabled: true
  role: "replica"
  master_address: "vz-ha-master:7001"  # Docker service name
  heartbeat_interval_secs: 5
```

**4. Start the cluster:**

```bash
docker compose --profile ha up -d
```

**5. Connect your application:**

```
http://localhost:15002
```

That's it. One URL for everything.

### Kubernetes (production)

Follow the [HA on Kubernetes — End-to-End Runbook](../../deployment/HA_KUBERNETES_RUNBOOK.md).
It is the single, validated install guide for 3.8.0: Secret, headless
Service (`publishNotReadyAddresses: true`), ConfigMap template with
`__NODE_ID__`, StatefulSet (`ghcr.io/hivellm/vectorizer:3.8.0`,
`podManagementPolicy: Parallel`, `RUST_LOG=info`, `/ready` readiness),
validation, failover test, rolling updates, the one-time all-pods restart
when upgrading from ≤ 3.7.2, and troubleshooting. The matching manifests
are `deploy/k8s/service-ha.yaml`, `deploy/k8s/configmap-ha.yaml` and
`deploy/k8s/statefulset-ha.yaml`.

## How It Works

### Write Routing (HTTP 307)

When your application sends a write request (POST, PUT, DELETE, PATCH) to a follower node, it receives:

```
HTTP/1.1 307 Temporary Redirect
Location: http://leader-address:15002/your/original/path
X-Vectorizer-Leader: http://leader-address:15002
X-Vectorizer-Role: follower

{"redirect":"write operations must go to leader","leader_url":"http://leader-address:15002"}
```

The follower never forwards the write itself. `leader_url` is the leader's
internal address (in Kubernetes an in-cluster DNS name), so only clients on
the same network can reach it.

Most HTTP clients follow a 307 automatically, but they drop the
`Authorization` header when the redirect changes host (curl with `-L`,
Python `requests`, Go `net/http`, `fetch`). With authentication enabled —
mandatory for HA — the redirected write then fails with 401. Handle the 307
explicitly: re-send the same request with the same headers and body to
`leader_url` (curl: `--location-trusted`), or send writes to the leader in
the first place, e.g. through a leader-routing layer.

Every write path replicates in 3.8.0 (REST, RPC, MCP, GraphQL, native
gRPC). Qdrant-compatible gRPC points writes are not replicated yet — use
REST or RPC for writes in HA mode.

### Replication Flow

```
1. Client writes to Leader
2. Leader stores data locally
3. Leader streams operation to all Followers via TCP (port 7001)
4. Followers apply the operation to their local store
5. Followers send ACK back to Leader
6. Leader tracks confirmed offset per replica
```

### Shared Authentication

All nodes must share the same `jwt_secret` in their config. This means:

- A JWT token obtained from any node works on all other nodes
- API keys created on the leader are replicated to followers
- You only need to authenticate once

## Configuration Reference

### Master Node

```yaml
replication:
  enabled: true
  role: "master"
  bind_address: "0.0.0.0:7001"     # TCP listen address for replicas
  heartbeat_interval_secs: 5        # Heartbeat frequency
  replica_timeout_secs: 30          # Disconnect replica after this silence
  log_size: 1000000                 # Replication log buffer (operations)
  reconnect_interval_secs: 5        # Unused on master
  wal_enabled: true                 # Persist replication log to disk
```

### Replica Node

```yaml
replication:
  enabled: true
  role: "replica"
  master_address: "master-host:7001"  # IP:port or hostname:port of master
  heartbeat_interval_secs: 5
  replica_timeout_secs: 30
  log_size: 1000000
  reconnect_interval_secs: 5          # Retry interval on disconnect
  wal_enabled: false                  # Replicas don't need WAL
```

### Cluster Mode (for Raft consensus + sharding)

```yaml
cluster:
  enabled: true                        # Enables Raft + cluster features
  node_id: "node-1"                    # Unique node identifier
  discovery: "static"                  # static | dns
  timeout_ms: 5000                     # gRPC timeout for inter-node ops
  retry_count: 3

  # Static node discovery
  servers:
    - id: "node-1"
      address: "192.168.1.10"
      grpc_port: 15003
    - id: "node-2"
      address: "192.168.1.11"
      grpc_port: 15003

  # DNS discovery (Kubernetes)
  # discovery: "dns"
  # dns_name: "vectorizer-headless.default.svc.cluster.local"
  # dns_resolve_interval: 30
```

## API Endpoints

### Cluster Status

`GET /api/v1/cluster/leader` and `GET /api/v1/cluster/role` currently return
a fixed `"standalone"` placeholder even in HA mode, so they cannot identify
the leader. Use the logs (`This node is now the LEADER` /
`This node is now FOLLOWER`, visible with `RUST_LOG=info`) or the
`leader_url` a follower returns with a 307.

```bash
# List cluster nodes
GET /api/v1/cluster/nodes

# Shard distribution
GET /api/v1/cluster/shard-distribution
```

### Health Check

```bash
GET /health   # liveness: 200 as soon as the HTTP server is up
GET /ready    # readiness: 503 until the startup collection load completes, 200 after
```

Returns:
```json
{
  "status": "healthy",
  "version": "2.5.1"
}
```

## Examples

### JavaScript / TypeScript

```javascript
const VECTORIZER_URL = 'http://vectorizer:15002';

// Login (once)
const loginRes = await fetch(`${VECTORIZER_URL}/auth/login`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ username: 'admin', password: 'your-password' })
});
const { access_token } = await loginRes.json();

// Create collection (write → send to the leader; a follower answers 307)
await fetch(`${VECTORIZER_URL}/collections`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${access_token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ name: 'products', dimension: 384, metric: 'cosine' })
});

// Insert vectors (write → leader)
await fetch(`${VECTORIZER_URL}/qdrant/collections/products/points`, {
  method: 'PUT',
  headers: {
    'Authorization': `Bearer ${access_token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    points: [
      { id: 'prod-1', vector: [0.1, 0.2, ...], payload: { name: 'Widget' } },
      { id: 'prod-2', vector: [0.3, 0.4, ...], payload: { name: 'Gadget' } }
    ]
  })
});

// Search (read → served by any node)
const searchRes = await fetch(`${VECTORIZER_URL}/qdrant/collections/products/points/search`, {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${access_token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ vector: [0.1, 0.2, ...], limit: 10 })
});
const { result } = await searchRes.json();
```

### Python

```python
import requests

URL = 'http://vectorizer:15002'

# Login
token = requests.post(f'{URL}/auth/login', json={
    'username': 'admin', 'password': 'your-password'
}).json()['access_token']

headers = {'Authorization': f'Bearer {token}'}

# Create collection
requests.post(f'{URL}/collections', json={
    'name': 'docs', 'dimension': 768, 'metric': 'cosine'
}, headers=headers)

# Insert
requests.put(f'{URL}/qdrant/collections/docs/points', json={
    'points': [{'id': 'doc-1', 'vector': [0.1]*768, 'payload': {'text': 'hello'}}]
}, headers=headers)

# Search
results = requests.post(f'{URL}/qdrant/collections/docs/points/search', json={
    'vector': [0.1]*768, 'limit': 5
}, headers=headers).json()
```

### curl

```bash
# Login
TOKEN=$(curl -s -X POST http://vectorizer:15002/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"your-password"}' \
  | jq -r '.access_token')

# Create collection (--location-trusted re-sends the token if a follower answers 307)
curl --location-trusted -X POST http://vectorizer:15002/collections \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"my-collection","dimension":128,"metric":"cosine"}'

# Insert vectors
curl --location-trusted -X PUT http://vectorizer:15002/qdrant/collections/my-collection/points \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"points":[{"id":"v1","vector":[0.1,0.2,...128 values...],"payload":{"key":"value"}}]}'

# Search
curl -X POST http://vectorizer:15002/qdrant/collections/my-collection/points/search \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"vector":[0.1,0.2,...128 values...],"limit":5}'
```

## Dashboard

The web dashboard is available at the leader URL:

```
http://vectorizer:15002
```

The **Cluster** page (sidebar → Cluster) shows:
- Node status (healthy/unhealthy)
- Node roles (Leader/Follower)
- Replication status and lag
- Connected replicas count

## Failover Behavior

### Automatic failover with Raft (recommended for production)

When `cluster.enabled: true`, Raft consensus handles leader election automatically:

1. Leader node goes down
2. Raft detects missing heartbeat (~1-3 seconds)
3. Remaining followers start a new election
4. One follower is elected as the new leader
5. New leader starts a `MasterNode` (accepts writes on port 7001)
6. Other followers connect to the new leader as replicas
7. Followers answer writes with HTTP 307 pointing at the new leader
8. When the old leader recovers, it rejoins as a follower

**This is the recommended mode for Kubernetes deployments.** All pods use the same config; Raft decides roles dynamically.

### Manual failover (static master/replica roles)

When using static `replication.role: "master"` / `"replica"` without Raft:

1. Followers lose the TCP replication connection
2. Followers continue serving **reads** with their existing data
3. Followers still **redirect writes** (307) to the (dead) leader URL
4. **No automatic promotion** — an operator must manually reconfigure a replica as master
5. When the old leader recovers, followers automatically reconnect

This mode is simpler but has **no automatic failover**. Use only for development or when you have an external orchestrator managing roles.

### When a follower dies:

1. Leader detects missing heartbeat after `replica_timeout_secs`
2. Leader continues serving reads and writes normally
3. When the follower recovers, it reconnects and receives missed data

## Troubleshooting

### Writes returning 307 but not reaching the leader

Your HTTP client may not be following redirects, or follows them without the
`Authorization` header (the leader then answers 401). Solutions:
- `curl`: use `--location-trusted`; other clients: re-send the request to `leader_url` with the same headers
- Check that the leader URL in the 307 response is reachable from the client
- In Docker/K8s, the redirect uses internal network addresses

### Replica not connecting to master ("No route to host")

**In Kubernetes (v2.5.4+):** The replica re-resolves DNS on every reconnect attempt, so it will automatically find the master after a pod restart. If you still see this error:

1. Check that the headless service exists and includes port 7001:
   ```bash
   kubectl get svc <name>-headless -o jsonpath='{.spec.ports[*]}'
   ```
2. Verify DNS resolution inside the namespace (the default image has no shell, so use a throwaway pod):
   ```bash
   kubectl run dnscheck -n <ns> --rm -it --restart=Never --image=busybox:1.36 -- \
     nslookup <master-pod>.<headless-svc>.<ns>.svc.cluster.local
   ```
3. Check that port 7001 is declared in both the StatefulSet container ports and the headless service.

**Versions before v2.5.4:** The master address was resolved to an IP once at startup and cached forever. If the master pod restarted with a new IP, replicas would be stuck connecting to the old IP. **Upgrade to v2.5.4** to fix this.

### Pods crashing with "Path segments must not start with `:`"

**Fixed in v2.5.4.** The cluster API router used Axum 0.6 path syntax (`:node_id`) instead of Axum 0.7+ syntax (`{node_id}`). This caused a panic at startup when `cluster.enabled: true`. Upgrade to v2.5.4.

### No automatic failover when master dies

If replicas keep trying to reconnect to the dead master instead of electing a new leader, check your config:

- `cluster.enabled` must be `true` (enables Raft consensus)
- Do NOT set a static `replication.role: "master"` or `"replica"` — Raft manages roles automatically
- All pods must use the same config with `cluster.enabled: true` and `replication.enabled: true`
- The `servers` list must include all nodes with correct addresses and gRPC ports

### File watcher running in cluster mode

If you see `🔍 File watcher is still running...` in logs, set `file_watcher.enabled: false` in your config. The file watcher is incompatible with cluster mode and wastes resources.

### Data not replicating

Check master logs for:
```
Replica <id> connected from <addr> with offset <n>
Performing full sync for replica <id>
```

If you don't see these, the TCP connection is not established.

### Different passwords on each node

All nodes must share the same `jwt_secret`. Set it via:
- Config file: `auth.jwt_secret`
- Environment variable: `VECTORIZER_JWT_SECRET`

## Related Documentation

- [HA on Kubernetes — End-to-End Runbook](../../deployment/HA_KUBERNETES_RUNBOOK.md)
- [Cluster Deployment Guide](../../deployment/CLUSTER.md)
- [Master-Replica Routing](MASTER_REPLICA_ROUTING.md)
- [Configuration Reference](../configuration/CLUSTER.md)
