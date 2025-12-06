# Vectorizer Scaling Guide

This guide covers horizontal and vertical scaling strategies for Vectorizer in production environments.

## Overview

Vectorizer supports multiple scaling strategies:
- **Vertical Scaling**: Increase resources on a single node
- **Horizontal Scaling**: Add more nodes with replication or sharding
- **Hybrid Scaling**: Combine both approaches

## Vertical Scaling

### CPU Scaling

Vectorizer automatically uses all available CPU cores for:
- Parallel vector operations
- Concurrent search queries
- HNSW index building

**Recommendations:**
- Minimum: 4 cores for production
- Recommended: 8-16 cores for moderate workloads
- High performance: 32+ cores for large-scale deployments

```yaml
# config.yml
performance:
  threads: 0  # 0 = auto-detect (all cores)
  # Or specify explicitly:
  # threads: 16
```

### Memory Scaling

Memory requirements depend on:
- Number of vectors
- Vector dimensions
- Index type (HNSW uses more memory)
- Quantization settings

**Memory Formula:**
```
Base memory = vectors × dimensions × 4 bytes (float32)
HNSW overhead = vectors × (M × 2 × 4 + 16) bytes
Total ≈ Base memory × 2.5 (with HNSW)
```

**Example:**
```
1M vectors × 384 dimensions:
- Base: 1.5 GB
- With HNSW (M=16): ~3.8 GB
- Recommended allocation: 6-8 GB
```

**Configuration:**
```yaml
# config.yml
memory:
  max_heap_mb: 8192
  cache_size_mb: 2048
  mmap_enabled: true  # Use memory-mapped files for large datasets
```

### Storage Scaling

For large datasets, optimize storage:

```yaml
# config.yml
storage:
  data_dir: /data/vectorizer
  compression: zstd  # Options: none, lz4, zstd
  compression_level: 3  # 1-22 for zstd

quantization:
  enabled: true
  type: scalar  # scalar (8-bit) or product
  # Reduces memory by ~75% with minimal quality loss
```

## Horizontal Scaling

### Master-Replica Architecture

For read-heavy workloads, deploy replicas:

```
                    ┌─────────────┐
                    │   Clients   │
                    └──────┬──────┘
                           │
                    ┌──────┴──────┐
                    │ Load Balancer│
                    └──────┬──────┘
              ┌────────────┼────────────┐
              │            │            │
        ┌─────┴─────┐ ┌────┴────┐ ┌─────┴─────┐
        │  Master   │ │ Replica1│ │ Replica2 │
        │  (writes) │ │ (reads) │ │ (reads)  │
        └───────────┘ └─────────┘ └──────────┘
```

**Master Configuration:**
```yaml
# config.yml (master)
replication:
  enabled: true
  role: master
  bind_address: 0.0.0.0:15003
  sync_interval_ms: 100
  max_lag_ms: 5000
```

**Replica Configuration:**
```yaml
# config.yml (replica)
replication:
  enabled: true
  role: replica
  master_address: master.internal:15003
  sync_interval_ms: 100
```

**SDK Configuration:**
```typescript
const client = new VectorizerClient({
  hosts: {
    master: 'https://master.example.com',
    replicas: [
      'https://replica1.example.com',
      'https://replica2.example.com',
    ],
  },
  readPreference: 'replica',
});
```

### Sharding Architecture

For large datasets that don't fit on a single node:

```
                    ┌─────────────┐
                    │   Clients   │
                    └──────┬──────┘
                           │
                    ┌──────┴──────┐
                    │ Shard Router│
                    └──────┬──────┘
              ┌────────────┼────────────┐
              │            │            │
        ┌─────┴─────┐ ┌────┴────┐ ┌─────┴─────┐
        │  Shard 0  │ │ Shard 1 │ │  Shard 2  │
        │ (0-33%)   │ │(33-66%) │ │ (66-100%) │
        └───────────┘ └─────────┘ └──────────┘
```

**Cluster Configuration:**
```yaml
# config.yml (shard router)
cluster:
  enabled: true
  mode: distributed
  shards:
    - id: 0
      address: shard0.internal:15002
      weight: 1
    - id: 1
      address: shard1.internal:15002
      weight: 1
    - id: 2
      address: shard2.internal:15002
      weight: 1

  sharding:
    strategy: consistent_hash  # consistent_hash or range
    replication_factor: 2  # Each vector on 2 shards
```

**Shard Node Configuration:**
```yaml
# config.yml (shard node)
cluster:
  enabled: true
  mode: shard
  shard_id: 0
  router_address: router.internal:15010
```

## Kubernetes Deployment

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: vectorizer-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: vectorizer
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: vectorizer_search_latency_p99
      target:
        type: AverageValue
        averageValue: 10m  # 10ms
```

### StatefulSet for Sharding

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: vectorizer-shard
spec:
  serviceName: vectorizer-shard
  replicas: 3
  selector:
    matchLabels:
      app: vectorizer-shard
  template:
    metadata:
      labels:
        app: vectorizer-shard
    spec:
      containers:
      - name: vectorizer
        image: hivellm/vectorizer:latest
        resources:
          requests:
            memory: "8Gi"
            cpu: "4"
          limits:
            memory: "16Gi"
            cpu: "8"
        env:
        - name: SHARD_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        volumeMounts:
        - name: data
          mountPath: /data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 100Gi
```

## Multi-Tenant Scaling

### Per-Tenant Resource Limits

```yaml
# config.yml
hub:
  enabled: true
  tenant_limits:
    default:
      max_collections: 100
      max_vectors_per_collection: 10000000
      max_storage_bytes: 107374182400  # 100 GB
      max_requests_per_minute: 1000

    enterprise:
      max_collections: 1000
      max_vectors_per_collection: 100000000
      max_storage_bytes: 1099511627776  # 1 TB
      max_requests_per_minute: 10000
```

### Tenant Isolation

For complete isolation, use storage-level isolation:

```yaml
hub:
  enabled: true
  tenant_isolation: storage  # none | collection | storage
```

This creates separate data directories per tenant:
```
/data/tenants/
├── tenant_abc123/
│   ├── collections/
│   └── indexes/
├── tenant_def456/
│   ├── collections/
│   └── indexes/
```

## Performance Tuning

### HNSW Parameters

```yaml
index:
  type: hnsw
  hnsw:
    m: 16  # Connections per node (higher = better quality, more memory)
    ef_construction: 200  # Build-time accuracy (higher = better, slower build)
    ef_search: 100  # Search-time accuracy (higher = better, slower search)
```

**Guidelines:**
| Workload | M | ef_construction | ef_search |
|----------|---|-----------------|-----------|
| Fast search | 8 | 100 | 50 |
| Balanced | 16 | 200 | 100 |
| High recall | 32 | 400 | 200 |
| Maximum recall | 64 | 800 | 400 |

### Connection Pooling

```yaml
server:
  max_connections: 10000
  connection_timeout_ms: 30000
  keep_alive_ms: 60000

  # Thread pool for async operations
  thread_pool:
    core_size: 16
    max_size: 64
    queue_size: 1000
```

### Caching

```yaml
cache:
  enabled: true

  # Query result cache
  query_cache:
    size_mb: 512
    ttl_seconds: 300

  # Vector cache (hot vectors)
  vector_cache:
    size_mb: 2048
    eviction: lru
```

## Monitoring Scaling

### Key Metrics to Watch

1. **CPU Utilization**
   - Scale up if consistently > 70%
   - Scale down if < 30%

2. **Memory Usage**
   - Scale up if > 80%
   - Consider quantization if memory-bound

3. **Search Latency (P99)**
   - Scale out if > 50ms
   - Tune HNSW parameters if scaling doesn't help

4. **Queue Depth**
   - Scale out if growing
   - Indicates processing can't keep up

5. **Disk I/O**
   - Move to SSD if high
   - Enable mmap for read-heavy workloads

### Prometheus Queries

```promql
# CPU-based scaling decision
avg(vectorizer_cpu_usage) > 0.7

# Memory pressure
vectorizer_memory_usage / vectorizer_memory_limit > 0.8

# Search latency
histogram_quantile(0.99, vectorizer_search_latency_seconds_bucket) > 0.05

# Request queue depth
vectorizer_request_queue_depth > 100
```

## Capacity Planning

### Estimation Worksheet

```
1. Vector Count: _______ vectors
2. Dimensions: _______ dims
3. Expected QPS: _______ queries/sec
4. Latency SLA: _______ ms (P99)

Memory Required:
  Base = vectors × dims × 4 bytes = _______ GB
  With HNSW = Base × 2.5 = _______ GB
  Safety margin = _______ × 1.5 = _______ GB

CPU Required:
  Search: 1 core per 100 QPS ≈ _______ cores
  Indexing: 2-4 cores additional = _______ cores
  Total = _______ cores

Storage Required:
  Vectors = Base memory = _______ GB
  Indexes = Base × 0.5 = _______ GB
  Logs/temp = 20 GB
  Total = _______ GB
```

### Example Configurations

**Small (1M vectors, 100 QPS)**
- 1 node, 8 cores, 16 GB RAM, 50 GB SSD

**Medium (10M vectors, 500 QPS)**
- 1 master + 2 replicas
- Each: 8 cores, 32 GB RAM, 200 GB SSD

**Large (100M vectors, 2000 QPS)**
- 3 shards, each with 1 master + 2 replicas
- Each node: 16 cores, 64 GB RAM, 500 GB NVMe

**Enterprise (1B+ vectors, 10000+ QPS)**
- 10+ shards with replication
- GPU acceleration for search
- Dedicated index builder nodes
- Contact HiveLLM for architecture review
