# Comprehensive Benchmarking System Specification

**Status**: Specification  
**Priority**: 🟢 **P2 - MEDIUM**  
**Complexity**: Medium  
**Created**: October 1, 2025  
**Updated**: October 1, 2025 - **PRIORITY CONFIRMED - ALREADY HAVE EXCELLENT BENCHMARKS**

## 🎯 **WHY P2 PRIORITY - BENCHMARK INSIGHTS**

**Priority confirmed** as P2 because **we already have excellent benchmarks**:
1. **Comprehensive coverage**: Quantization, dimension comparison, combined optimization
2. **Proven value**: Benchmarks revealed 4x compression + better quality (SQ-8bit)
3. **Quality metrics**: MAP, Recall@K, compression ratios all measured
4. **Performance tracking**: Search latency, memory usage, build times documented
5. **Focus on implementation**: Rather than more benchmarks, implement proven features

## Overview

Implement a complete benchmarking system to measure and track performance metrics across all vectorizer operations.

**Note**: We already have comprehensive benchmarks that proved quantization delivers exceptional value. This spec focuses on expanding the existing system.

## Required Metrics

### 1. Insertion Metrics

```rust
pub struct InsertionMetrics {
    // Time metrics
    pub avg_insert_time_us: f64,
    pub p50_insert_time_us: f64,
    pub p95_insert_time_us: f64,
    pub p99_insert_time_us: f64,
    pub max_insert_time_us: f64,
    
    // Throughput
    pub vectors_per_second: f64,
    pub batch_throughput: f64,
    
    // Resource usage
    pub memory_per_vector_bytes: usize,
    pub disk_per_vector_bytes: usize,
    
    // Index building
    pub index_build_time_ms: f64,
    pub index_update_time_us: f64,
}
```

### 2. Search Metrics

```rust
pub struct SearchMetrics {
    // Latency
    pub avg_search_time_ms: f64,
    pub p50_search_time_ms: f64,
    pub p95_search_time_ms: f64,
    pub p99_search_time_ms: f64,
    pub max_search_time_ms: f64,
    
    // Throughput
    pub queries_per_second: f64,
    pub concurrent_queries: usize,
    
    // Quality
    pub avg_recall_at_10: f32,
    pub avg_recall_at_100: f32,
    pub avg_precision: f32,
    
    // Breakdown
    pub embedding_time_ms: f64,
    pub index_search_time_ms: f64,
    pub result_assembly_time_ms: f64,
}
```

### 3. Summarization Metrics

```rust
pub struct SummarizationMetrics {
    // Time by method
    pub extractive_time_ms: HashMap<usize, f64>,  // text_length -> time
    pub keyword_time_ms: HashMap<usize, f64>,
    pub sentence_time_ms: HashMap<usize, f64>,
    pub abstractive_time_ms: HashMap<usize, f64>,
    
    // Quality
    pub compression_ratios: Vec<f32>,
    pub avg_compression_ratio: f32,
    
    // Throughput
    pub chars_per_second: f64,
    pub summaries_per_minute: f64,
}
```

### 4. System Metrics

```rust
pub struct SystemMetrics {
    // CPU
    pub cpu_usage_percent: f32,
    pub cpu_usage_per_core: Vec<f32>,
    
    // Memory
    pub memory_total_gb: f32,
    pub memory_used_gb: f32,
    pub memory_usage_percent: f32,
    
    // Per collection
    pub collection_metrics: HashMap<String, CollectionSystemMetrics>,
}

pub struct CollectionSystemMetrics {
    pub memory_mb: f32,
    pub disk_mb: f32,
    pub vector_count: usize,
    pub index_size_mb: f32,
    pub cache_size_mb: f32,
}
```

## Implementation

### 1. Benchmark Suite (Criterion)

```rust
// benches/insertion_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_insertions(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertions");
    
    // Different vector counts
    for vector_count in [100, 1_000, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::new("insert_sequential", vector_count),
            &vector_count,
            |b, &count| {
                let store = setup_vector_store();
                let vectors = generate_test_vectors(count, 384);
                
                b.iter(|| {
                    for vector in &vectors {
                        store.insert(black_box(vector.clone()));
                    }
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("insert_batch", vector_count),
            &vector_count,
            |b, &count| {
                let store = setup_vector_store();
                let vectors = generate_test_vectors(count, 384);
                
                b.iter(|| {
                    store.insert_batch(black_box(vectors.clone()));
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_insertions, bench_search, bench_summarization);
criterion_main!(benches);
```

### 2. Real-time Metrics Collection

```rust
pub struct MetricsCollector {
    insertion_times: Arc<Mutex<Histogram>>,
    search_times: Arc<Mutex<Histogram>>,
    summarization_times: Arc<Mutex<Histogram>>,
    system_monitor: SystemMonitor,
}

impl MetricsCollector {
    pub async fn record_insertion(&self, duration: Duration, collection: &str) {
        self.insertion_times.lock().await.record(duration.as_micros() as u64);
        
        // Update per-collection metrics
        METRICS_REGISTRY.record_counter(
            "insertion_count",
            &[("collection", collection)]
        );
        
        METRICS_REGISTRY.record_histogram(
            "insertion_duration_us",
            duration.as_micros() as f64,
            &[("collection", collection)]
        );
    }
    
    pub async fn get_summary(&self) -> MetricsSummary {
        let insertion_hist = self.insertion_times.lock().await;
        
        MetricsSummary {
            insertions: InsertionMetrics {
                avg_insert_time_us: insertion_hist.mean(),
                p50_insert_time_us: insertion_hist.percentile(0.50),
                p95_insert_time_us: insertion_hist.percentile(0.95),
                p99_insert_time_us: insertion_hist.percentile(0.99),
                max_insert_time_us: insertion_hist.max(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
```

### 3. Automatic Benchmarking on Startup

```rust
pub struct StartupBenchmark {
    quick_mode: bool,
    comprehensive: bool,
}

impl StartupBenchmark {
    pub async fn run(&self) -> Result<BenchmarkReport> {
        let mut report = BenchmarkReport::default();
        
        if self.quick_mode {
            // Quick sanity check (~30 seconds)
            report.insertion = self.bench_insertion_quick().await?;
            report.search = self.bench_search_quick().await?;
        }
        
        if self.comprehensive {
            // Full benchmark suite (~10 minutes)
            report.insertion = self.bench_insertion_comprehensive().await?;
            report.search = self.bench_search_comprehensive().await?;
            report.summarization = self.bench_summarization().await?;
            report.system = self.bench_system_limits().await?;
        }
        
        // Save report
        self.save_report(&report).await?;
        
        // Compare with historical
        if let Some(previous) = self.load_previous_report().await? {
            report.regression_analysis = self.detect_regressions(&previous, &report)?;
        }
        
        Ok(report)
    }
}
```

## Dashboard Integration

### Real-time Metrics Display

```javascript
// Dashboard metrics view
class MetricsViewer {
    constructor() {
        this.ws = new WebSocket('ws://localhost:15001/metrics/stream');
        this.charts = this.initializeCharts();
    }
    
    initializeCharts() {
        return {
            insertionTime: new Chart('insertion-time-chart', {
                type: 'line',
                data: {
                    datasets: [{
                        label: 'Avg Insertion Time (μs)',
                        data: []
                    }]
                }
            }),
            
            searchTime: new Chart('search-time-chart', {
                type: 'line',
                data: {
                    datasets: [{
                        label: 'P95 Search Time (ms)',
                        data: []
                    }]
                }
            }),
            
            memoryUsage: new Chart('memory-chart', {
                type: 'bar',
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Memory per Collection (MB)',
                        data: []
                    }]
                }
            })
        };
    }
    
    handleMetricsUpdate(metrics) {
        // Update insertion chart
        this.charts.insertionTime.data.datasets[0].data.push({
            x: new Date(),
            y: metrics.avg_insert_time_us
        });
        
        // Update search chart
        this.charts.searchTime.data.datasets[0].data.push({
            x: new Date(),
            y: metrics.p95_search_time_ms
        });
        
        // Update memory chart
        this.charts.memoryUsage.data.labels = Object.keys(metrics.collection_metrics);
        this.charts.memoryUsage.data.datasets[0].data = Object.values(metrics.collection_metrics)
            .map(c => c.memory_mb);
        
        // Refresh charts
        Object.values(this.charts).forEach(chart => chart.update());
    }
}
```

## API Endpoints

```http
# Get current metrics
GET /api/metrics
Response: {
  "insertion": { ... },
  "search": { ... },
  "summarization": { ... },
  "system": { ... }
}

# Get historical metrics
GET /api/metrics/historical?hours=24
Response: {
  "timestamps": [...],
  "insertion_times": [...],
  "search_times": [...],
  "memory_usage": [...]
}

# Run benchmark
POST /api/benchmark
{
  "mode": "quick" | "comprehensive",
  "save_report": true
}

# Get benchmark reports
GET /api/benchmark/reports
Response: [
  {
    "id": "...",
    "timestamp": "...",
    "duration": "...",
    "summary": { ... }
  }
]

# Compare benchmarks
GET /api/benchmark/compare?baseline=...&current=...
```

## Benchmark Reports

```markdown
# Vectorizer Benchmark Report
**Date**: 2025-10-01 15:30:00  
**Version**: 0.21.0  
**Mode**: Comprehensive  
**Duration**: 8m 34s

## Insertion Performance
- Avg Time: 8.3 μs/vector
- P95 Time: 12.1 μs/vector
- Throughput: 120,000 vectors/second
- Memory: 1.2 bytes/vector overhead

## Search Performance
- Avg Time: 0.62 ms (top-10)
- P95 Time: 1.1 ms
- P99 Time: 2.3 ms
- QPS: 1,612 queries/second

## Summarization Performance
- Extractive: 45 ms (1000 chars)
- Keyword: 12 ms (1000 chars)
- Sentence: 38 ms (1000 chars)
- Abstractive: 230 ms (1000 chars)

## System Resources
- Peak Memory: 2.3 GB
- CPU Usage: 35% avg, 92% peak
- Disk I/O: 45 MB/s write, 120 MB/s read

## Regression Analysis
✅ No regressions detected
📈 Search 5% faster than v0.20.0
📈 Memory 3% lower than v0.20.0
```

## Configuration

```yaml
# config.yml
benchmarks:
  # Automatic benchmarking
  auto_run_on_startup: false
  startup_mode: "quick"  # quick | comprehensive
  
  # Continuous benchmarking
  continuous:
    enabled: false
    interval: 3600  # seconds
    mode: "quick"
  
  # Test data
  test_data:
    vector_counts: [100, 1000, 10000, 100000]
    dimensions: [384, 512, 768]
    query_counts: [100, 1000]
  
  # Reporting
  reports:
    save_directory: "./benchmark-reports"
    keep_last: 20
    export_formats: ["json", "markdown", "html"]
```

## Success Criteria

- ✅ All metrics tracked accurately
- ✅ Benchmarks complete in < 10 minutes
- ✅ Historical comparison working
- ✅ Regression detection accurate
- ✅ Dashboard shows real-time metrics
- ✅ Reports are actionable

---

**Estimated Effort**: 2-3 weeks  
**Dependencies**: Metrics infrastructure  
**Risk**: Low

