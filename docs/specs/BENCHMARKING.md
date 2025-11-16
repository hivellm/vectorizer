# Performance Benchmarking Guide

## Overview

Vectorizer includes comprehensive performance benchmarks to ensure consistent performance and detect regressions. We have **18 active benchmarks** covering all critical paths.

## Quick Start

### Running All Benchmarks

```bash
# Run all benchmarks with criterion
cargo +nightly bench --features benchmarks

# Run specific benchmark
cargo +nightly bench --features benchmarks --bench core_operations_bench

# Run with baseline comparison
cargo +nightly bench --features benchmarks -- --save-baseline my-baseline
```

### Running Specific Categories

```bash
# Core operations
cargo +nightly bench --features benchmarks --bench core_operations_bench
cargo +nightly bench --features benchmarks --bench cache_bench
cargo +nightly bench --features benchmarks --bench query_cache_bench

# Storage
cargo +nightly bench --features benchmarks --bench storage_bench

# Search
cargo +nightly bench --features benchmarks --bench search_bench

# GPU (requires GPU features)
cargo +nightly bench --features benchmarks,hive-gpu-cuda --bench cuda_bench
cargo +nightly bench --features benchmarks,hive-gpu-metal --bench metal_hnsw_search_bench

# Performance at scale
cargo +nightly bench --features benchmarks --bench scale_bench
cargo +nightly bench --features benchmarks --bench large_scale_bench
```

## Available Benchmarks

### Core Operations (4 benchmarks)

1. **core_operations_bench** - Basic vector operations (insert, search, delete)
2. **cache_bench** - Cache performance and hit rates
3. **query_cache_bench** - Query caching effectiveness
4. **update_bench** - Vector update operations

### Storage (1 benchmark)

5. **storage_bench** - Persistence and storage I/O performance

### Search (1 benchmark)

6. **search_bench** - Vector similarity search performance

### GPU (3 benchmarks)

7. **gpu_bench** - General GPU acceleration performance
8. **cuda_bench** - CUDA-specific operations (requires NVIDIA GPU)
9. **metal_hnsw_search_bench** - Metal-accelerated HNSW search (macOS)

### Quantization (1 benchmark)

10. **quantization_bench** - Vector quantization performance and quality

### Embeddings (1 benchmark)

11. **embeddings_bench** - Text embedding generation (requires fastembed)

### Performance at Scale (3 benchmarks)

12. **scale_bench** - Performance scaling with dataset size
13. **large_scale_bench** - Large dataset operations (10K-100K vectors)
14. **combined_optimization_bench** - Combined optimization techniques

### Replication (1 benchmark)

15. **replication_bench** - Data replication performance

### Examples (3 benchmarks)

16. **example_benchmark** - Example benchmark template
17. **simple_test** - Simple sanity check
18. **minimal_benchmark** - Minimal overhead test

## Performance Budgets

We enforce strict performance budgets to maintain quality:

### Search Operations
- **Budget**: < 5ms per query (10K vectors, 384 dimensions)
- **Current**: ~2-3ms (PASS ✅)
- **Measurement**: Average latency over 1000 queries

### Vector Indexing
- **Budget**: > 1000 vectors/second
- **Current**: ~2500 vectors/s (PASS ✅)
- **Measurement**: Bulk insertion throughput

### Memory Usage
- **Budget**: < 500MB for 10K vectors (384 dimensions)
- **Current**: ~350MB (PASS ✅)
- **Measurement**: Peak RSS memory

### Search Accuracy
- **Budget**: > 95% recall@10
- **Current**: ~98% (PASS ✅)
- **Measurement**: HNSW vs brute force comparison

## Benchmark Organization

```
benchmark/
├── core/               # Core operations
│   ├── cache_benchmark.rs
│   ├── core_operations_benchmark.rs
│   ├── query_cache_bench.rs
│   └── update_bench.rs
├── embeddings/         # Embedding generation
│   └── benchmark_embeddings.rs
├── gpu/                # GPU-accelerated operations
│   ├── cuda_benchmark.rs
│   ├── gpu_benchmark.rs
│   └── metal_hnsw_search_benchmark.rs
├── performance/        # Large-scale performance
│   ├── combined_optimization_benchmark.rs
│   ├── large_scale_benchmark.rs
│   └── scale_benchmark.rs
├── quantization/       # Quantization methods
│   └── quantization_benchmark.rs
├── replication/        # Data replication
│   └── replication_bench.rs
├── search/             # Search algorithms
│   └── search_bench.rs
├── storage/            # Persistence layer
│   └── storage_benchmark.rs
├── example_benchmark.rs
├── minimal_benchmark.rs
└── simple_test.rs
```

## CI/CD Integration

Benchmarks run automatically on:
- ✅ Push to `main` branch
- ✅ Pull requests to `main`
- ✅ Manual workflow dispatch

### Workflow Steps

1. **Run benchmarks** with baseline creation
2. **Compare with previous** baseline (PR only)
3. **Check budgets** against defined thresholds
4. **Upload artifacts** for historical tracking
5. **Comment on PR** with results summary

### Performance Regression Detection

PRs are **blocked** if:
- Any benchmark shows > 10% regression
- Performance budgets are violated
- Memory usage increases > 20%

## Historical Results

Benchmark results are stored in two places:

1. **`benchmark/reports/`** - Manual benchmark reports (30+ files)
   - JSON format for programmatic analysis
   - Markdown format for human reading
   - Timestamped for historical tracking

2. **GitHub Artifacts** - CI benchmark results
   - Criterion HTML reports
   - Baseline comparisons
   - 30-day retention

## Writing New Benchmarks

### Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use vectorizer::db::VectorStore;

fn bench_my_operation(c: &mut Criterion) {
    let store = VectorStore::new();
    
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            // Your operation here
            black_box(store.some_operation())
        });
    });
}

criterion_group!(benches, bench_my_operation);
criterion_main!(benches);
```

### Best Practices

1. **Use `black_box`** to prevent compiler optimizations
2. **Warm up** before measurement (Criterion does this automatically)
3. **Measure what matters** - focus on user-facing operations
4. **Isolate benchmarks** - avoid interference between tests
5. **Document expectations** - include expected performance in comments

### Adding to Cargo.toml

```toml
[[bench]]
name = "my_bench"
path = "benchmark/my_category/my_bench.rs"
harness = false
required-features = ["benchmarks"]
```

## Troubleshooting

### Benchmarks Won't Run

```bash
# Ensure you're using nightly
rustup default nightly

# Enable benchmarks feature
cargo +nightly bench --features benchmarks

# Check if specific benchmark exists
cargo +nightly bench --features benchmarks --bench nonexistent
```

### Inconsistent Results

- **CPU frequency scaling**: Pin CPU frequency for consistent results
- **Background processes**: Close unnecessary applications
- **Thermal throttling**: Ensure adequate cooling
- **System load**: Run on idle system

### GPU Benchmarks Fail

```bash
# CUDA (Linux/Windows with NVIDIA GPU)
cargo +nightly bench --features benchmarks,hive-gpu-cuda --bench cuda_bench

# Metal (macOS with Apple Silicon)
cargo +nightly bench --features benchmarks,hive-gpu-metal --bench metal_hnsw_search_bench

# Check GPU availability
nvidia-smi  # CUDA
system_profiler SPDisplaysDataType  # Metal
```

## Performance Tips

### For Users

1. **Enable release mode**: Always benchmark in release mode (Criterion default)
2. **Use appropriate features**: Enable SIMD, GPU if available
3. **Tune HNSW parameters**: Adjust `m` and `ef_construction` for your needs
4. **Consider quantization**: 8-bit quantization saves 75% memory

### For Developers

1. **Profile first**: Use `cargo flamegraph` before optimizing
2. **Measure impact**: Run benchmarks before and after changes
3. **Check all platforms**: Performance may vary (x86 vs ARM, Linux vs Windows)
4. **Document trade-offs**: Note any accuracy vs speed compromises

## Additional Resources

- **Criterion Documentation**: https://bheisler.github.io/criterion.rs/book/
- **Rust Performance Book**: https://nnethercote.github.io/perf-book/
- **HNSW Paper**: https://arxiv.org/abs/1603.09320
- **Project Benchmarks**: `benchmark/reports/` for historical data

## Support

For benchmark-related issues:
1. Check existing reports in `benchmark/reports/`
2. Review CI benchmark logs in GitHub Actions
3. Open an issue with benchmark results attached
4. Tag with `performance` label

