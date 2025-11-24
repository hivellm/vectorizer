# Runbook: Slow Searches

## Symptoms

- Query latency > 1 second
- Timeout errors
- High p95/p99 latency
- User complaints about slow searches

## Diagnosis

### Check Query Latency

```bash
# Check current latency
curl http://localhost:15002/prometheus/metrics | grep vectorizer_query_duration

# Check p95 latency
curl http://localhost:15002/prometheus/metrics | grep "quantile=\"0.95\""

# Check query rate
curl http://localhost:15002/prometheus/metrics | grep vectorizer_queries_per_second
```

### Identify Slow Operations

```bash
# Check search operations
curl http://localhost:15002/prometheus/metrics | grep vectorizer_search_duration

# Check collection sizes
curl http://localhost:15002/api/collections | jq '.[] | {name, vector_count}'
```

## Immediate Actions

### 1. Increase ef_search (Temporary)

```yaml
# config.yml - Quick fix
collections:
  default:
    hnsw_config:
      ef_search: 200 # Increase from default 50-100
```

**Warning**: This increases latency but improves recall. Use temporarily.

### 2. Reduce Query Load

```bash
# Enable rate limiting
# Update config.yml
rate_limit:
  enabled: true
  requests_per_minute: 500  # Reduce load
```

### 3. Enable Query Caching

```yaml
cache:
  enabled: true
  max_size_mb: 2048
  ttl_seconds: 3600
```

## Root Cause Analysis

### Common Causes

1. **Large Collections**

   - Collections too large for single node
   - Need sharding or replication

2. **Low ef_search**

   - Too low ef_search parameter
   - Need to balance speed vs quality

3. **High Query Load**

   - Too many concurrent queries
   - System overloaded

4. **Resource Constraints**

   - Insufficient CPU
   - Memory pressure
   - I/O bottlenecks

5. **Inefficient Index**
   - HNSW index not optimized
   - Need to rebuild index

## Resolution Steps

### Step 1: Optimize HNSW Parameters

```yaml
# config.yml
collections:
  default:
    hnsw_config:
      m: 32 # Higher for better recall
      ef_construction: 200 # Higher for better quality
      ef_search: 100 # Balance speed vs quality
```

### Step 2: Enable Quantization

```yaml
collections:
  default:
    quantization:
      enabled: true
      type: "scalar"
      bits: 8 # Faster searches with quantized vectors
```

### Step 3: Optimize Collections

```bash
# Rebuild index for better performance
curl -X POST http://localhost:15002/api/collections/{name}/rebuild-index
```

### Step 4: Scale Resources

```bash
# Increase CPU (Kubernetes)
kubectl edit statefulset vectorizer -n vectorizer
# Update resources.limits.cpu

# Or scale horizontally
kubectl scale statefulset vectorizer --replicas=3 -n vectorizer
```

### Step 5: Use Query Optimization

```python
# Use optimized search config
config = {
    "max_results": 10,  # Reduce from default
    "ef_search": 50,    # Lower for speed
    "use_cache": True   # Enable caching
}
```

## Prevention

1. **Monitor Latency**: Set alerts at p95 > 500ms
2. **Load Testing**: Test before production
3. **Optimize Indexes**: Regular index optimization
4. **Capacity Planning**: Plan for query growth
5. **Query Optimization**: Use appropriate parameters

## Verification

```bash
# Verify latency improved
curl http://localhost:15002/prometheus/metrics | grep vectorizer_query_duration

# Test query performance
time curl -X POST http://localhost:15002/api/collections/{name}/search \
  -H "Content-Type: application/json" \
  -d '{"query": [0.1]*384, "limit": 10}'
```

## Escalation

If searches remain slow:

1. Review collection sizes
2. Consider sharding large collections
3. Scale resources or horizontally
4. Contact SRE team
