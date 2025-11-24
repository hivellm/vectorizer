# Vectorizer vs Qdrant Comprehensive Benchmark Report

Generated: 2025-11-24 20:45:29 UTC

## Executive Summary

### Overall Performance Comparison

| Scenario | Dimension | Vectors | Insert Winner | Search Winner |
|----------|-----------|---------|---------------|---------------|
| Small dataset (1K vectors) | 384 | 1000 | Qdrant | Vectorizer |
| Medium dataset (5K vectors) | 384 | 5000 | Qdrant | Vectorizer |
| Large dataset (10K vectors) | 384 | 10000 | Qdrant | Vectorizer |
| Medium dataset (512 dim) | 512 | 5000 | Qdrant | Vectorizer |
| Medium dataset (768 dim) | 768 | 5000 | Qdrant | Vectorizer |

## Detailed Results by Scenario

### Scenario 1: Small dataset (1K vectors)

- **Dimension**: 384
- **Vectors**: 1000
- **Queries**: 200
- **Batch Size**: 50

#### Insertion Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 0.23ms | 0.09ms | Qdrant | 2.42x |
| Throughput | 4416.02 vec/s | 10675.07 vec/s | Qdrant | 2.42x |

#### Search Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 0.16ms | 0.84ms | Vectorizer | 5.31x |
| Throughput | 6285.18 q/s | 1183.73 q/s | Vectorizer | 5.31x |

#### Search Quality

| Metric | Vectorizer | Qdrant | Winner |
|--------|------------|--------|--------|
| Precision@10 | 0.00% | 100.00% | Qdrant |
| Recall@10 | 0.00% | 100.00% | Qdrant |
| F1-Score | 0.00% | 100.00% | Qdrant |

#### Detailed Results

**Vectorizer:**
```json
{
  "system_name": "Vectorizer",
  "insertion": {
    "total_vectors": 1000,
    "total_time_ms": 226.4485,
    "avg_latency_ms": 0.22644850000000055,
    "p50_latency_ms": 0.22438199999999997,
    "p95_latency_ms": 0.243658,
    "p99_latency_ms": 0.244068,
    "throughput_vectors_per_sec": 4416.015120435773
  },
  "search": {
    "total_queries": 200,
    "total_time_ms": 31.820899999999998,
    "avg_latency_ms": 0.15910449999999998,
    "p50_latency_ms": 0.1575,
    "p95_latency_ms": 0.18359999999999999,
    "p99_latency_ms": 0.2477,
    "throughput_queries_per_sec": 6285.17735199193
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

**Qdrant:**
```json
{
  "system_name": "Qdrant",
  "insertion": {
    "total_vectors": 1000,
    "total_time_ms": 93.6762,
    "avg_latency_ms": 0.09367619999999968,
    "p50_latency_ms": 0.084694,
    "p95_latency_ms": 0.125422,
    "p99_latency_ms": 0.20629,
    "throughput_vectors_per_sec": 10675.070081835087
  },
  "search": {
    "total_queries": 200,
    "total_time_ms": 168.9575,
    "avg_latency_ms": 0.8447874999999999,
    "p50_latency_ms": 0.8146,
    "p95_latency_ms": 0.9851000000000001,
    "p99_latency_ms": 1.125,
    "throughput_queries_per_sec": 1183.729636151103
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

### Scenario 2: Medium dataset (5K vectors)

- **Dimension**: 384
- **Vectors**: 5000
- **Queries**: 500
- **Batch Size**: 100

#### Insertion Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 1.11ms | 0.08ms | Qdrant | 14.28x |
| Throughput | 903.51 vec/s | 12899.46 vec/s | Qdrant | 14.28x |

#### Search Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 0.16ms | 0.80ms | Vectorizer | 4.93x |
| Throughput | 6171.13 q/s | 1251.69 q/s | Vectorizer | 4.93x |

#### Search Quality

| Metric | Vectorizer | Qdrant | Winner |
|--------|------------|--------|--------|
| Precision@10 | 0.00% | 100.00% | Qdrant |
| Recall@10 | 0.00% | 100.00% | Qdrant |
| F1-Score | 0.00% | 100.00% | Qdrant |

#### Detailed Results

**Vectorizer:**
```json
{
  "system_name": "Vectorizer",
  "insertion": {
    "total_vectors": 5000,
    "total_time_ms": 5533.9872000000005,
    "avg_latency_ms": 1.1067974400000085,
    "p50_latency_ms": 0.208855,
    "p95_latency_ms": 9.19637,
    "p99_latency_ms": 18.062729,
    "throughput_vectors_per_sec": 903.5076915248376
  },
  "search": {
    "total_queries": 500,
    "total_time_ms": 81.0225,
    "avg_latency_ms": 0.1620450000000001,
    "p50_latency_ms": 0.1604,
    "p95_latency_ms": 0.188,
    "p99_latency_ms": 0.2186,
    "throughput_queries_per_sec": 6171.125304699312
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

**Qdrant:**
```json
{
  "system_name": "Qdrant",
  "insertion": {
    "total_vectors": 5000,
    "total_time_ms": 387.6132,
    "avg_latency_ms": 0.07752263999999995,
    "p50_latency_ms": 0.075544,
    "p95_latency_ms": 0.098308,
    "p99_latency_ms": 0.104299,
    "throughput_vectors_per_sec": 12899.457500415363
  },
  "search": {
    "total_queries": 500,
    "total_time_ms": 399.4601,
    "avg_latency_ms": 0.7989201999999999,
    "p50_latency_ms": 0.7774,
    "p95_latency_ms": 0.9173,
    "p99_latency_ms": 1.0151,
    "throughput_queries_per_sec": 1251.689467859243
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

### Scenario 3: Large dataset (10K vectors)

- **Dimension**: 384
- **Vectors**: 10000
- **Queries**: 1000
- **Batch Size**: 200

#### Insertion Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 2.28ms | 0.11ms | Qdrant | 20.99x |
| Throughput | 438.76 vec/s | 9209.13 vec/s | Qdrant | 20.99x |

#### Search Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 0.17ms | 0.86ms | Vectorizer | 5.16x |
| Throughput | 6014.91 q/s | 1166.44 q/s | Vectorizer | 5.16x |

#### Search Quality

| Metric | Vectorizer | Qdrant | Winner |
|--------|------------|--------|--------|
| Precision@10 | 0.00% | 100.00% | Qdrant |
| Recall@10 | 0.00% | 100.00% | Qdrant |
| F1-Score | 0.00% | 100.00% | Qdrant |

#### Detailed Results

**Vectorizer:**
```json
{
  "system_name": "Vectorizer",
  "insertion": {
    "total_vectors": 10000,
    "total_time_ms": 22791.304,
    "avg_latency_ms": 2.279130400000026,
    "p50_latency_ms": 0.7211960000000001,
    "p95_latency_ms": 12.6674555,
    "p99_latency_ms": 15.933512,
    "throughput_vectors_per_sec": 438.7638372951368
  },
  "search": {
    "total_queries": 1000,
    "total_time_ms": 166.2534,
    "avg_latency_ms": 0.1662534,
    "p50_latency_ms": 0.1639,
    "p95_latency_ms": 0.19010000000000002,
    "p99_latency_ms": 0.2314,
    "throughput_queries_per_sec": 6014.914582198018
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

**Qdrant:**
```json
{
  "system_name": "Qdrant",
  "insertion": {
    "total_vectors": 10000,
    "total_time_ms": 1085.8783999999998,
    "avg_latency_ms": 0.10858784000000186,
    "p50_latency_ms": 0.0963785,
    "p95_latency_ms": 0.1219035,
    "p99_latency_ms": 0.6065045,
    "throughput_vectors_per_sec": 9209.134282438992
  },
  "search": {
    "total_queries": 1000,
    "total_time_ms": 857.3102,
    "avg_latency_ms": 0.8573102000000009,
    "p50_latency_ms": 0.8276,
    "p95_latency_ms": 1.034,
    "p99_latency_ms": 1.2550000000000001,
    "throughput_queries_per_sec": 1166.4389389044945
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

### Scenario 4: Medium dataset (512 dim)

- **Dimension**: 512
- **Vectors**: 5000
- **Queries**: 500
- **Batch Size**: 100

#### Insertion Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 1.47ms | 0.11ms | Qdrant | 13.77x |
| Throughput | 681.29 vec/s | 9379.75 vec/s | Qdrant | 13.77x |

#### Search Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 0.18ms | 0.81ms | Vectorizer | 4.39x |
| Throughput | 5434.00 q/s | 1238.07 q/s | Vectorizer | 4.39x |

#### Search Quality

| Metric | Vectorizer | Qdrant | Winner |
|--------|------------|--------|--------|
| Precision@10 | 0.00% | 100.00% | Qdrant |
| Recall@10 | 0.00% | 100.00% | Qdrant |
| F1-Score | 0.00% | 100.00% | Qdrant |

#### Detailed Results

**Vectorizer:**
```json
{
  "system_name": "Vectorizer",
  "insertion": {
    "total_vectors": 5000,
    "total_time_ms": 7339.0063,
    "avg_latency_ms": 1.4678012600000157,
    "p50_latency_ms": 0.222083,
    "p95_latency_ms": 13.804455,
    "p99_latency_ms": 23.073902999999994,
    "throughput_vectors_per_sec": 681.2911442793011
  },
  "search": {
    "total_queries": 500,
    "total_time_ms": 92.0132,
    "avg_latency_ms": 0.1840263999999998,
    "p50_latency_ms": 0.18230000000000002,
    "p95_latency_ms": 0.22,
    "p99_latency_ms": 0.28709999999999997,
    "throughput_queries_per_sec": 5434.002947403198
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

**Qdrant:**
```json
{
  "system_name": "Qdrant",
  "insertion": {
    "total_vectors": 5000,
    "total_time_ms": 533.0635,
    "avg_latency_ms": 0.10661270000000023,
    "p50_latency_ms": 0.10258899999999999,
    "p95_latency_ms": 0.136054,
    "p99_latency_ms": 0.149925,
    "throughput_vectors_per_sec": 9379.745565021803
  },
  "search": {
    "total_queries": 500,
    "total_time_ms": 403.85490000000004,
    "avg_latency_ms": 0.8077098000000009,
    "p50_latency_ms": 0.7866000000000001,
    "p95_latency_ms": 0.9178,
    "p99_latency_ms": 1.031,
    "throughput_queries_per_sec": 1238.0684250704894
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

### Scenario 5: Medium dataset (768 dim)

- **Dimension**: 768
- **Vectors**: 5000
- **Queries**: 500
- **Batch Size**: 100

#### Insertion Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 1.82ms | 0.16ms | Qdrant | 11.65x |
| Throughput | 548.61 vec/s | 6391.29 vec/s | Qdrant | 11.65x |

#### Search Performance

| Metric | Vectorizer | Qdrant | Winner | Speedup |
|--------|------------|--------|--------|----------|
| Avg Latency | 0.23ms | 0.87ms | Vectorizer | 3.86x |
| Throughput | 4427.02 q/s | 1145.52 q/s | Vectorizer | 3.86x |

#### Search Quality

| Metric | Vectorizer | Qdrant | Winner |
|--------|------------|--------|--------|
| Precision@10 | 0.00% | 100.00% | Qdrant |
| Recall@10 | 0.00% | 100.00% | Qdrant |
| F1-Score | 0.00% | 100.00% | Qdrant |

#### Detailed Results

**Vectorizer:**
```json
{
  "system_name": "Vectorizer",
  "insertion": {
    "total_vectors": 5000,
    "total_time_ms": 9114.0167,
    "avg_latency_ms": 1.8228033399999946,
    "p50_latency_ms": 0.252059,
    "p95_latency_ms": 19.784895,
    "p99_latency_ms": 28.703853,
    "throughput_vectors_per_sec": 548.6055341548803
  },
  "search": {
    "total_queries": 500,
    "total_time_ms": 112.94269999999999,
    "avg_latency_ms": 0.2258854,
    "p50_latency_ms": 0.2256,
    "p95_latency_ms": 0.2744,
    "p99_latency_ms": 0.3387,
    "throughput_queries_per_sec": 4427.0236146293655
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

**Qdrant:**
```json
{
  "system_name": "Qdrant",
  "insertion": {
    "total_vectors": 5000,
    "total_time_ms": 782.3147,
    "avg_latency_ms": 0.15646293999999494,
    "p50_latency_ms": 0.150333,
    "p95_latency_ms": 0.182147,
    "p99_latency_ms": 0.259853,
    "throughput_vectors_per_sec": 6391.289847934597
  },
  "search": {
    "total_queries": 500,
    "total_time_ms": 436.4847,
    "avg_latency_ms": 0.8729693999999997,
    "p50_latency_ms": 0.8611,
    "p95_latency_ms": 0.9939,
    "p99_latency_ms": 1.1235,
    "throughput_queries_per_sec": 1145.515524370041
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

## Summary Statistics

- **Average Search Speedup (Vectorizer vs Qdrant)**: 4.73x
- **Average Insert Speedup (Qdrant vs Vectorizer)**: 0.14x

## Test Configuration

- Total test scenarios: 5
- Test environments: Docker containers
- Both systems running on same hardware
- All tests use cosine similarity metric

