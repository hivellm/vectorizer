# Vectorizer vs Qdrant Benchmark Report

Generated: 2025-11-24 20:42:12 UTC

## Executive Summary

### Insertion Performance

| Metric | Vectorizer | Qdrant | Winner |
|--------|------------|--------|--------|
| Avg Latency | 0.21ms | 0.13ms | Qdrant |
| P95 Latency | 0.24ms | 0.18ms | Qdrant |
| Throughput | 4716.48 vec/s | 7680.59 vec/s | Qdrant |

### Search Performance

| Metric | Vectorizer | Qdrant | Winner |
|--------|------------|--------|--------|
| Avg Latency | 0.17ms | 1.06ms | Vectorizer |
| P95 Latency | 0.21ms | 1.21ms | Vectorizer |
| Throughput | 5943.40 q/s | 943.23 q/s | Vectorizer |

### Search Quality

| Metric | Vectorizer | Qdrant | Winner |
|--------|------------|--------|--------|
| Precision@10 | 0.00% | 100.00% | Qdrant |
| Recall@10 | 0.00% | 100.00% | Qdrant |
| F1-Score | 0.00% | 100.00% | Qdrant |

## Detailed Results

### Vectorizer Results

```json
{
  "system_name": "Vectorizer",
  "insertion": {
    "total_vectors": 500,
    "total_time_ms": 106.0112,
    "avg_latency_ms": 0.2120224000000005,
    "p50_latency_ms": 0.20875500000000002,
    "p95_latency_ms": 0.23695,
    "p99_latency_ms": 0.243255,
    "throughput_vectors_per_sec": 4716.482786724421
  },
  "search": {
    "total_queries": 100,
    "total_time_ms": 16.825400000000002,
    "avg_latency_ms": 0.16825400000000001,
    "p50_latency_ms": 0.1634,
    "p95_latency_ms": 0.2089,
    "p99_latency_ms": 0.23399999999999999,
    "throughput_queries_per_sec": 5943.395105019791
  },
  "memory": {
    "memory_usage_mb": 0.0,
    "memory_per_vector_bytes": 0.0
  },
  "quality": {
    "precision_at_10": 0.0,
    "recall_at_10": 0.0,
    "f1_score": 0.0
  }
}
```

### Qdrant Results

```json
{
  "system_name": "Qdrant",
  "insertion": {
    "total_vectors": 500,
    "total_time_ms": 65.0992,
    "avg_latency_ms": 0.13019839999999927,
    "p50_latency_ms": 0.11689999999999998,
    "p95_latency_ms": 0.177105,
    "p99_latency_ms": 0.42906999999999995,
    "throughput_vectors_per_sec": 7680.585936539927
  },
  "search": {
    "total_queries": 100,
    "total_time_ms": 106.01910000000001,
    "avg_latency_ms": 1.0601909999999999,
    "p50_latency_ms": 0.857,
    "p95_latency_ms": 1.214,
    "p99_latency_ms": 2.426,
    "throughput_queries_per_sec": 943.2262677196844
  },
  "memory": {
    "memory_usage_mb": 0.0,
    "memory_per_vector_bytes": 0.0
  },
  "quality": {
    "precision_at_10": 1.0,
    "recall_at_10": 1.0,
    "f1_score": 1.0
  }
}
```

## Test Configuration

- Vector dimension: 384
- Test vectors: 10,000
- Test queries: 1,000
- Batch size: 100
