# Cluster Deployment Guide

Complete guide to deploying Vectorizer in a distributed cluster configuration.

## Overview

This guide covers deploying Vectorizer across multiple servers for horizontal scalability and high availability.

## Prerequisites

- Multiple servers (physical or virtual machines)
- Network connectivity between servers
- Same Vectorizer version on all nodes
- Sufficient resources (CPU, RAM, disk) on each node

## Architecture

### Single-Server vs Cluster

**Single-Server:**
```
┌─────────────────┐
│  Vectorizer     │
│  (All Shards)   │
└─────────────────┘
```

**Cluster (3 Nodes):**
```
┌──────────┐  ┌──────────┐  ┌──────────┐
│ Node 1   │  │ Node 2   │  │ Node 3   │
│ Shard 0  │  │ Shard 1  │  │ Shard 2  │
│ Shard 3  │  │ Shard 4  │  │ Shard 5  │
└──────────┘  └──────────┘  └──────────┘
      │             │             │
      └─────────────┴─────────────┘
              gRPC Network
```

## Deployment Steps

### Step 1: Prepare Servers

On each server:

1. **Install Vectorizer:**
   ```bash
   curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
   ```

2. **Create data directory:**
   ```bash
   sudo mkdir -p /var/lib/vectorizer/data
   sudo chown vectorizer:vectorizer /var/lib/vectorizer/data
   ```

3. **Configure firewall:**
   ```bash
   # REST API port
   sudo ufw allow 15002/tcp
   
   # gRPC port
   sudo ufw allow 15003/tcp
   ```

### Step 2: Configure Each Node

**Node 1 (`/etc/vectorizer/config-node1.yml`):**
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

**Node 2 (`/etc/vectorizer/config-node2.yml`):**
```yaml
server:
  host: "0.0.0.0"
  port: 15002

cluster:
  enabled: true
  node_id: "node-2"
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

**Node 3 (`/etc/vectorizer/config-node3.yml`):**
```yaml
server:
  host: "0.0.0.0"
  port: 15002

cluster:
  enabled: true
  node_id: "node-3"
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

### Step 3: Start Nodes

Start nodes in order (recommended):

**Node 1:**
```bash
sudo systemctl start vectorizer --config /etc/vectorizer/config-node1.yml
```

**Node 2:**
```bash
sudo systemctl start vectorizer --config /etc/vectorizer/config-node2.yml
```

**Node 3:**
```bash
sudo systemctl start vectorizer --config /etc/vectorizer/config-node3.yml
```

### Step 4: Verify Cluster

Check cluster status:

```bash
curl "http://192.168.1.10:15002/api/v1/cluster/nodes"
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

### Step 5: Create Distributed Collection

Create a collection with sharding enabled:

```bash
curl -X POST "http://192.168.1.10:15002/collections" \
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

### Step 6: Verify Shard Distribution

Check shard distribution:

```bash
curl "http://192.168.1.10:15002/api/v1/cluster/shard-distribution"
```

## Docker Deployment

### Docker Compose Example

**`docker-compose.yml`:**
```yaml
version: '3.8'

services:
  vectorizer-node1:
    image: ghcr.io/hivellm/vectorizer:latest
    ports:
      - "15002:15002"
      - "15003:15003"
    volumes:
      - ./config-node1.yml:/etc/vectorizer/config.yml
      - node1-data:/var/lib/vectorizer/data
    environment:
      - VECTORIZER_CONFIG=/etc/vectorizer/config.yml
    networks:
      - vectorizer-cluster

  vectorizer-node2:
    image: ghcr.io/hivellm/vectorizer:latest
    ports:
      - "15004:15002"
      - "15005:15003"
    volumes:
      - ./config-node2.yml:/etc/vectorizer/config.yml
      - node2-data:/var/lib/vectorizer/data
    environment:
      - VECTORIZER_CONFIG=/etc/vectorizer/config.yml
    networks:
      - vectorizer-cluster

  vectorizer-node3:
    image: ghcr.io/hivellm/vectorizer:latest
    ports:
      - "15006:15002"
      - "15007:15003"
    volumes:
      - ./config-node3.yml:/etc/vectorizer/config.yml
      - node3-data:/var/lib/vectorizer/data
    environment:
      - VECTORIZER_CONFIG=/etc/vectorizer/config.yml
    networks:
      - vectorizer-cluster

volumes:
  node1-data:
  node2-data:
  node3-data:

networks:
  vectorizer-cluster:
    driver: bridge
```

**Start cluster:**
```bash
docker-compose up -d
```

## Kubernetes Deployment

### Kubernetes Example

**`vectorizer-cluster.yaml`:**
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: vectorizer-cluster
spec:
  serviceName: vectorizer
  replicas: 3
  selector:
    matchLabels:
      app: vectorizer
  template:
    metadata:
      labels:
        app: vectorizer
    spec:
      containers:
      - name: vectorizer
        image: ghcr.io/hivellm/vectorizer:latest
        ports:
        - containerPort: 15002
          name: rest
        - containerPort: 15003
          name: grpc
        volumeMounts:
        - name: data
          mountPath: /var/lib/vectorizer/data
        - name: config
          mountPath: /etc/vectorizer/config.yml
          subPath: config.yml
        env:
        - name: VECTORIZER_CONFIG
          value: /etc/vectorizer/config.yml
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 10Gi
---
apiVersion: v1
kind: Service
metadata:
  name: vectorizer
spec:
  clusterIP: None
  selector:
    app: vectorizer
  ports:
  - port: 15002
    name: rest
  - port: 15003
    name: grpc
```

**Deploy:**
```bash
kubectl apply -f vectorizer-cluster.yaml
```

## Load Balancer Configuration

### Nginx Example

**`/etc/nginx/sites-available/vectorizer`:**
```nginx
upstream vectorizer {
    least_conn;
    server 192.168.1.10:15002;
    server 192.168.1.11:15002;
    server 192.168.1.12:15002;
}

server {
    listen 80;
    server_name vectorizer.example.com;

    location / {
        proxy_pass http://vectorizer;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## Monitoring

### Health Checks

Set up health checks for each node:

```bash
# Health check script
#!/bin/bash
NODE=$1
curl -f "http://${NODE}:15002/health" || exit 1
```

### Prometheus Metrics

Each node exposes Prometheus metrics:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'vectorizer-cluster'
    static_configs:
      - targets:
        - '192.168.1.10:15002'
        - '192.168.1.11:15002'
        - '192.168.1.12:15002'
    metrics_path: '/prometheus/metrics'
```

## Backup and Recovery

### Backup Strategy

1. **Backup all nodes:**
   ```bash
   # On each node
   tar -czf backup-$(date +%Y%m%d).tar.gz /var/lib/vectorizer/data
   ```

2. **Backup cluster configuration:**
   ```bash
   # Save cluster configs
   cp /etc/vectorizer/config-*.yml /backup/
   ```

### Recovery

1. **Restore data on each node**
2. **Restore cluster configuration**
3. **Start nodes in order**
4. **Verify cluster status**

## Scaling

### Adding Nodes

1. **Add node to all configurations:**
   ```yaml
   cluster:
     servers:
       # ... existing nodes ...
       - id: "node-4"
         address: "192.168.1.13"
         grpc_port: 15003
   ```

2. **Start new node**
3. **Trigger rebalancing:**
   ```bash
   curl -X POST "http://192.168.1.10:15002/api/v1/cluster/rebalance"
   ```

### Removing Nodes

1. **Remove node via API:**
   ```bash
   curl -X DELETE "http://192.168.1.10:15002/api/v1/cluster/nodes/node-3"
   ```

2. **Wait for shard migration**
3. **Stop node process**
4. **Update configurations on remaining nodes**

## Testing Cluster Deployment

### Integration Tests

Vectorizer includes comprehensive tests for cluster functionality:

```bash
# Test cluster management
cargo test --lib integration::cluster

# Test distributed sharding
cargo test --lib integration::distributed_sharding

# Test failure scenarios
cargo test --lib integration::cluster_failures

# Test scaling operations
cargo test --lib integration::cluster_scale

# Test performance
cargo test --lib integration::cluster_performance

# Test fault tolerance
cargo test --lib integration::cluster_fault_tolerance

# Test end-to-end workflows
cargo test --lib integration::cluster_e2e

# Test feature integration
cargo test --lib integration::cluster_integration
```

### Manual Testing Checklist

When testing with real servers:

1. **Basic Operations**:
   - [ ] Create distributed collection
   - [ ] Insert vectors (verify distribution)
   - [ ] Search vectors (verify results merged)
   - [ ] Update vectors
   - [ ] Delete vectors

2. **Node Management**:
   - [ ] Add node to cluster
   - [ ] Remove node from cluster
   - [ ] Verify shard rebalancing
   - [ ] Check cluster state synchronization

3. **Failure Scenarios**:
   - [ ] Node failure during insert
   - [ ] Node failure during search
   - [ ] Network partition
   - [ ] Node recovery

4. **Performance**:
   - [ ] Concurrent inserts
   - [ ] Concurrent searches
   - [ ] Throughput measurement
   - [ ] Latency measurement

### Common Test Issues

**Issue**: Tests fail with "No active cluster nodes available"
- **Solution**: Ensure at least 2 nodes are added to cluster before creating distributed collection

**Issue**: gRPC connection errors in tests
- **Solution**: Tests use mock connections. For real server tests, ensure servers are running and ports are accessible

**Issue**: Shard assignment inconsistent
- **Solution**: Consistent hashing may assign shards differently. Use `rebalance` API to force rebalancing

## Troubleshooting

### Node Not Joining

**Symptoms**: Node doesn't appear in cluster, operations fail

**Solutions**:
1. Check network connectivity between nodes
2. Verify firewall rules allow gRPC port (default: 15012+)
3. Check node logs: `journalctl -u vectorizer -f`
4. Verify node ID is unique
5. Check cluster configuration matches on all nodes
6. Ensure discovery method is correct (static/dns)

**Debug Commands**:
```bash
# Check if node is listening
netstat -tuln | grep 15012

# Test gRPC connectivity
grpcurl -plaintext localhost:15012 list

# Check cluster state
curl http://localhost:15002/api/v1/cluster/nodes
```

### Uneven Shard Distribution

**Symptoms**: Some nodes have more shards than others, uneven load

**Solutions**:
1. Trigger manual rebalancing via API:
   ```bash
   curl -X POST http://localhost:15002/api/v1/cluster/rebalance
   ```
2. Check node health: `curl http://localhost:15002/api/v1/cluster/nodes`
3. Verify all nodes are active
4. Check shard distribution: `curl http://localhost:15002/api/v1/cluster/shard-distribution`
5. Increase `virtual_nodes_per_shard` for better distribution

### High Latency

**Symptoms**: Slow operations, timeouts

**Solutions**:
1. Check network latency between nodes: `ping <node-ip>`
2. Verify nodes are in same data center
3. Check for network congestion
4. Increase timeout in cluster config: `timeout_ms: 10000`
5. Check node CPU/memory usage
6. Verify gRPC connection pool settings

### gRPC Connection Errors

**Symptoms**: `Failed to get client for node`, connection refused

**Solutions**:
1. Verify gRPC port is correct in config
2. Check firewall rules
3. Ensure node is running and healthy
4. Check network connectivity
5. Verify TLS settings if using secure connections
6. Check connection pool timeout settings

### Search Returns Incomplete Results

**Symptoms**: Search returns fewer results than expected

**Solutions**:
1. Check if remote nodes are accessible
2. Verify all shards are assigned to active nodes
3. Check for node failures: `curl http://localhost:15002/api/v1/cluster/nodes`
4. Review search logs for errors
5. Ensure all nodes have the collection

### Vector Count Inconsistency

**Symptoms**: `vector_count()` returns incorrect total

**Solutions**:
1. Cache may be stale - wait 5 seconds and retry
2. Check if remote nodes are accessible
3. Verify collection exists on all nodes
4. Check for node failures
5. Manually invalidate cache (restart server)

### State Synchronization Issues

**Symptoms**: Nodes have different cluster state

**Solutions**:
1. Check state synchronization interval
2. Manually trigger sync: `curl -X POST http://localhost:15002/api/v1/cluster/sync`
3. Verify all nodes can communicate
4. Check logs for sync errors
5. Restart nodes if state is corrupted

### Performance Issues

**Symptoms**: Slow inserts/searches, high CPU usage

**Solutions**:
1. Check number of shards (too many = overhead)
2. Verify shard distribution is even
3. Check network latency between nodes
4. Reduce `virtual_nodes_per_shard` if memory constrained
5. Enable quantization for memory savings
6. Use MMAP storage for large collections
7. Check for retry storms (too many failed operations)

### Common Error Messages

**"No active cluster nodes available"**:
- Add at least one remote node to cluster
- Verify node is marked as active
- Check node health

**"Failed to get client for node"**:
- Node is unreachable
- Check network connectivity
- Verify gRPC port is correct
- Check firewall rules

**"Shard not found or not assigned"**:
- Shard assignment may be inconsistent
- Trigger rebalancing
- Verify all nodes are active

## Best Practices

1. **Start with 3 nodes** for high availability
2. **Use same data center** for low latency
3. **Monitor node health** regularly
4. **Backup all nodes** consistently
5. **Test failover** scenarios
6. **Use load balancer** for client connections
7. **Set up alerts** for node failures

## Related Documentation

- [Cluster Configuration](../users/configuration/CLUSTER.md)
- [Cluster API](../api/CLUSTER.md)
- [Sharding Guide](../users/collections/SHARDING.md)

