# Metrics Reference

**Version**: 1.2.0  
**Total Metrics**: 15  
**Last Updated**: October 25, 2025

---

## Metric Categories

| Category | Count | Description |
|----------|-------|-------------|
| Search | 3 | Search requests, latency, results |
| Indexing | 4 | Vector/collection counts, insert operations |
| Replication | 4 | Lag, bytes transferred, pending operations |
| System | 3 | Memory, cache, errors |
| **Total** | **14** | All instrumented metrics |

---

## Complete Metrics List

### Search Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `vectorizer_search_requests_total` | Counter | `collection`, `search_type`, `status` | Total search requests |
| `vectorizer_search_latency_seconds` | Histogram | `collection`, `search_type` | Search latency distribution |
| `vectorizer_search_results_count` | Histogram | `collection`, `search_type` | Results returned per search |

**Search Types**:
- `text` - Text-based search
- `intelligent` - Intelligent semantic search
- `semantic` - Semantic search with reranking
- `contextual` - Context-aware search
- `multi_collection` - Cross-collection search

**Status Values**:
- `success` - Request succeeded
- `error` - Request failed

---

### Indexing Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `vectorizer_vectors_total` | Gauge | None | Total vectors stored |
| `vectorizer_collections_total` | Gauge | None | Total collections |
| `vectorizer_insert_requests_total` | Counter | `collection`, `status` | Total insert requests |
| `vectorizer_insert_latency_seconds` | Histogram | None | Insert operation latency |

---

### Replication Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `vectorizer_replication_lag_ms` | Gauge | None | Replication lag in milliseconds |
| `vectorizer_replication_bytes_sent_total` | Counter | None | Bytes sent via replication |
| `vectorizer_replication_bytes_received_total` | Counter | None | Bytes received (replica only) |
| `vectorizer_replication_operations_pending` | Gauge | None | Operations pending replication |

---

### System Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `vectorizer_memory_usage_bytes` | Gauge | None | Process memory usage |
| `vectorizer_cache_requests_total` | Counter | `cache_type`, `result` | Cache hit/miss tracking |
| `vectorizer_api_errors_total` | Counter | `endpoint`, `error_type`, `status_code` | API errors |

**Cache Types**:
- `normalization` - Text normalization cache
- `embedding` - Embedding cache
- `query` - Query result cache (future)

**Cache Results**:
- `hit` - Cache hit
- `miss` - Cache miss

---

## Histogram Buckets

### Search Latency Buckets

```
1ms, 3ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s
```

**Rationale**: Captures sub-3ms target (p95) with granular detail

### Insert Latency Buckets

```
1ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s
```

**Rationale**: Inserts can be slower than searches, broader range needed

### Results Count Buckets

```
0, 1, 5, 10, 25, 50, 100, 250, 500, 1000
```

**Rationale**: Typical search limits are 10-100, captures distribution

---

## PromQL Query Examples

### Search Performance

```promql
# Average search latency (all collections)
rate(vectorizer_search_latency_seconds_sum[5m]) /
rate(vectorizer_search_latency_seconds_count[5m])

# P50 latency
histogram_quantile(0.50, rate(vectorizer_search_latency_seconds_bucket[5m]))

# P95 latency
histogram_quantile(0.95, rate(vectorizer_search_latency_seconds_bucket[5m]))

# P99 latency
histogram_quantile(0.99, rate(vectorizer_search_latency_seconds_bucket[5m]))

# Searches per second by type
sum by (search_type) (rate(vectorizer_search_requests_total[5m]))

# Success rate percentage
100 * sum(rate(vectorizer_search_requests_total{status="success"}[5m])) /
sum(rate(vectorizer_search_requests_total[5m]))
```

### Indexing Metrics

```promql
# Vector growth rate (vectors per hour)
rate(vectorizer_vectors_total[1h]) * 3600

# Inserts per second
rate(vectorizer_insert_requests_total[5m])

# Failed insert rate
rate(vectorizer_insert_requests_total{status="error"}[5m])

# Average vectors per collection
vectorizer_vectors_total / vectorizer_collections_total
```

### Replication Health

```promql
# Replication throughput (operations/sec)
rate(vectorizer_replication_bytes_sent_total[5m]) / 100  # Approximate

# Bytes sent per second
rate(vectorizer_replication_bytes_sent_total[5m])

# Bytes received per second
rate(vectorizer_replication_bytes_received_total[5m])

# Operations backlog
vectorizer_replication_operations_pending
```

### System Resources

```promql
# Memory usage in GB
vectorizer_memory_usage_bytes / 1024 / 1024 / 1024

# Memory growth rate (MB per hour)
rate(vectorizer_memory_usage_bytes[1h]) * 3600 / 1024 / 1024

# Cache hit rate
sum(rate(vectorizer_cache_requests_total{result="hit"}[5m])) /
sum(rate(vectorizer_cache_requests_total[5m]))

# Cache miss rate
sum(rate(vectorizer_cache_requests_total{result="miss"}[5m])) /
sum(rate(vectorizer_cache_requests_total[5m]))

# Error rate by endpoint
sum by (endpoint) (rate(vectorizer_api_errors_total[5m]))

# Errors per minute
rate(vectorizer_api_errors_total[1m]) * 60
```

---

## Metric Label Cardinality

### Low Cardinality (Safe) ✅

- `collection`: Typically 10-1000 collections
- `search_type`: 5 types (text, intelligent, semantic, contextual, multi)
- `status`: 2 values (success, error)
- `cache_type`: 3-5 types
- `result`: 2 values (hit, miss)

### Medium Cardinality (Monitor) ⚠️

- `endpoint`: ~50 API endpoints
- `error_type`: ~20 error types
- `status_code`: ~15 HTTP status codes

### High Cardinality (Avoid) ❌

- ❌ User IDs
- ❌ Request IDs / Correlation IDs
- ❌ Timestamps
- ❌ IP addresses
- ❌ Vector IDs

**Best Practice**: Use correlation IDs in logs, not metrics.

---

## Retention Policies

### Recommended Settings

| Data Type | Retention | Storage |
|-----------|-----------|---------|
| Raw metrics | 15 days | ~50MB/day |
| Aggregated (5m) | 90 days | ~10MB/day |
| Aggregated (1h) | 1 year | ~5MB/day |

### Prometheus Configuration

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

storage:
  tsdb:
    retention.time: 15d
    retention.size: 50GB
```

---

## Cardinality Estimation

### Current Metrics

```
Total unique series = 
  (Search metrics × cardinality) +
  (Indexing metrics × cardinality) +
  (Replication metrics × cardinality) +
  (System metrics × cardinality)

= (3 × ~100 collections × 5 types × 2 status) +
  (4 × 1) +
  (4 × 1) +
  (3 × ~5)

≈ 3,000 + 4 + 4 + 15
≈ 3,023 time series
```

**Storage**: ~3,000 series × 1KB/series × 15 days ≈ 45MB

---

## Integration with External Systems

### DataDog

```yaml
# datadog-agent.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: datadog-config
data:
  openmetrics.yaml: |
    init_config:
    instances:
      - prometheus_url: http://vectorizer:15002/prometheus/metrics
        namespace: vectorizer
        metrics:
          - vectorizer_*
```

### New Relic

```yaml
# newrelic-prometheus-configurator.yaml
integrations:
  - name: nri-prometheus
    config:
      targets:
        - description: Vectorizer
          urls: ["http://vectorizer:15002/prometheus/metrics"]
      transformations:
        - description: Add environment tag
          add_attributes:
            - metric_prefix: vectorizer_
              attributes:
                environment: production
```

### AWS CloudWatch

Use AWS CloudWatch Agent with Prometheus scraping:

```json
{
  "metrics": {
    "namespace": "Vectorizer",
    "metrics_collected": {
      "prometheus": {
        "prometheus_config_path": "/etc/prometheus/prometheus.yml",
        "emf_processor": {
          "metric_namespace": "Vectorizer",
          "metric_unit": {
            "vectorizer_search_latency_seconds": "Seconds",
            "vectorizer_memory_usage_bytes": "Bytes"
          }
        }
      }
    }
  }
}
```

---

## Troubleshooting Guide

### Missing Metrics

**Symptom**: Specific metrics not appearing in Prometheus

**Check**:
1. Verify metric is implemented in code
2. Check if operation has been triggered (e.g., no searches = no search metrics)
3. Verify labels match exactly

### High Cardinality Warning

**Symptom**: Prometheus warning about high cardinality

**Solution**:
1. Review labels for high-cardinality values
2. Use `topk` or `bottomk` to limit results
3. Consider aggregating before export

### Scrape Failures

**Symptom**: Prometheus shows scrape errors

**Check**:
1. Vectorizer server is running
2. Port 15002 is accessible
3. `/prometheus/metrics` endpoint responds

```bash
curl -v http://localhost:15002/prometheus/metrics
```

---

## Future Enhancements

Planned metrics for future releases:

- [ ] Query cache hit/miss rate
- [ ] File watcher events processed
- [ ] Embedding generation latency
- [ ] Disk I/O metrics
- [ ] Network I/O metrics
- [ ] HNSW index build time
- [ ] Background task queue depth
- [ ] WebSocket connection count

---

**For detailed monitoring setup, see**: `MONITORING.md`

