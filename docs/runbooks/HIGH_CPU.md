# Runbook: High CPU Usage

## Symptoms

- CPU usage consistently above 80%
- Slow query responses
- High load average
- System unresponsive

## Diagnosis

### Check Current CPU Usage

```bash
# Check CPU usage
top -p $(pgrep vectorizer)

# Or using metrics
curl http://localhost:15002/prometheus/metrics | grep process_cpu
```

### Identify High CPU Operations

```bash
# Check query rate
curl http://localhost:15002/prometheus/metrics | grep vectorizer_queries_per_second

# Check search operations
curl http://localhost:15002/prometheus/metrics | grep vectorizer_search
```

## Immediate Actions

### 1. Reduce Load

```bash
# Enable rate limiting (if not already enabled)
# Update config.yml
rate_limit:
  enabled: true
  requests_per_minute: 500  # Reduce from default

# Restart service
systemctl restart vectorizer
```

### 2. Scale Horizontally

```bash
# Add more replicas (Kubernetes)
kubectl scale statefulset vectorizer --replicas=3 -n vectorizer

# Or add more instances (Docker Compose)
docker-compose scale vectorizer=3
```

### 3. Optimize Queries

- Reduce `ef_search` parameter
- Use quantization to reduce memory operations
- Enable query caching
- Batch multiple queries

## Root Cause Analysis

### Common Causes

1. **High Query Rate**

   - Too many concurrent queries
   - Inefficient query patterns
   - Missing rate limiting

2. **Large Collections**

   - Collections too large for single node
   - Need sharding or replication

3. **Inefficient Index Configuration**

   - `ef_search` too high
   - `ef_construction` too high
   - Need to optimize HNSW parameters

4. **Resource Constraints**
   - Insufficient CPU cores
   - CPU throttling
   - Need more resources

## Resolution Steps

### Step 1: Optimize Configuration

```yaml
# config.yml
performance:
  cpu:
    max_threads: 8 # Match CPU cores
    enable_simd: true

collections:
  default:
    hnsw_config:
      ef_search: 50 # Reduce from 100
      ef_construction: 100 # Reduce from 200
```

### Step 2: Enable Caching

```yaml
cache:
  enabled: true
  max_size_mb: 1024
  ttl_seconds: 3600
```

### Step 3: Use Quantization

```yaml
collections:
  default:
    quantization:
      enabled: true
      type: "scalar"
      bits: 8
```

### Step 4: Scale Resources

```bash
# Increase CPU limits (Kubernetes)
kubectl edit statefulset vectorizer -n vectorizer
# Update resources.limits.cpu to "8" or higher
```

## Prevention

1. **Monitor CPU Usage**: Set up alerts at 70%
2. **Load Testing**: Test before production
3. **Capacity Planning**: Plan for peak loads
4. **Rate Limiting**: Always enable rate limiting
5. **Resource Monitoring**: Monitor and alert on resource usage

## Verification

```bash
# Verify CPU usage reduced
top -p $(pgrep vectorizer)

# Verify query performance
curl http://localhost:15002/api/status

# Check metrics
curl http://localhost:15002/prometheus/metrics | grep process_cpu
```

## Escalation

If CPU usage remains high after optimization:

1. Contact SRE team
2. Consider horizontal scaling
3. Review application architecture
4. Consider dedicated hardware
