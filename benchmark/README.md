# Vectorizer Benchmarks

This directory contains comprehensive benchmarks for the Vectorizer vector database, organized by category.

## Directory Structure

```
benchmark/
├── core/                    # Core database operations
│   ├── query_cache_bench.rs
│   ├── update_bench.rs
│   ├── core_operations_benchmark.rs
│   └── cache_benchmark.rs
├── search/                  # Search and query performance
├── gpu/                     # GPU acceleration benchmarks
│   ├── metal_hnsw_search_benchmark.rs
│   ├── metal_native_search_benchmark.rs
│   └── cuda_benchmark.rs
├── replication/             # Replication system benchmarks
│   └── replication_bench.rs
├── storage/                 # Storage and persistence benchmarks
│   └── storage_benchmark.rs
├── quantization/            # Vector quantization benchmarks
│   └── quantization_benchmark.rs
├── embeddings/              # Embedding generation benchmarks
│   └── benchmark_embeddings.rs
├── integration/             # End-to-end integration benchmarks
├── performance/             # Large-scale performance tests
│   ├── scale_benchmark.rs
│   ├── large_scale_benchmark.rs
│   ├── dimension_comparison_benchmark.rs
│   └── combined_optimization_benchmark.rs
└── README.md               # This file
```

## Running Benchmarks

### Run All Benchmarks
```bash
cargo bench
```

### Run Specific Category
```bash
# Core operations (comprehensive CRUD and search performance)
cargo bench --bench core_operations_bench

# GPU benchmarks
cargo bench --bench metal_hnsw_search_bench

# Replication benchmarks
cargo bench --bench replication_bench

# Dimension comparison benchmark
cargo run --bin dimension_comparison_benchmark --release

# Combined optimization benchmark
cargo bench --bench combined_optimization_bench
```

### Core Operations Benchmark Details
The `core_operations_bench` provides comprehensive performance testing for all vectorizer core operations:

- **Insert Performance**: Tests single inserts vs batch inserts (10, 50, 100 vectors per batch)
- **Search Performance**: Tests search with k=1, k=10, and k=100 for different use cases
- **Update Performance**: Tests individual vector updates with modified data
- **Delete Performance**: Tests individual vector deletion operations
- **Mixed Workload**: Simulates realistic usage with 70% search, 20% insert, 10% delete

**Sample Results** (on typical hardware):
- Single insert: ~105 elem/s
- Batch insert (100): ~800 elem/s (7.6x faster than single)
- Search k=1: ~80 elem/s
- Search k=100: ~46 elem/s (1.7x slower than k=1)
- Update: ~12.7K elem/s
- Delete: ~487K elem/s
- Mixed workload: ~7.2K elem/s

### Dimension Comparison Benchmark Details
The `dimension_comparison_benchmark` tests performance characteristics across different vector dimensions:

- **Test Dimensions**: 64D, 128D, 256D, 512D, 768D, 1024D, 1536D
- **Metrics Measured**: Build time, search latency, memory usage, quality (MAP, Recall@10)
- **Analysis**: Memory efficiency, speed efficiency, quality efficiency
- **Recommendations**: Optimal dimensions for different use cases

**Sample Results** (10K vectors):
- **64D**: 408μs latency, 2500 QPS, 2.4MB memory, 4096 vectors/MB
- **128D**: 665μs latency, 1429 QPS, 4.9MB memory, 2048 vectors/MB
- **512D**: 830μs latency, 1111 QPS, 19.5MB memory, 512 vectors/MB
- **1536D**: 1055μs latency, 833 QPS, 58.6MB memory, 171 vectors/MB

**Use Case Guidelines**:
- **Low Latency**: 64D-128D (< 1ms search)
- **Balanced**: 256D-512D (good quality/speed trade-off)
- **High Quality**: 768D-1536D (maximum accuracy)
- **Memory Constrained**: 64D-256D (efficient memory usage)

### Combined Optimization Benchmark Details
The `combined_optimization_bench` tests all combinations of dimensions and quantization methods to find optimal configurations:

- **Test Dimensions**: 256D, 384D, 512D, 768D, 1024D
- **Quantization Methods**: None, SQ-8bit, PQ (8x256), Binary (1-bit)
- **Total Configurations**: 20 combinations (5 dimensions × 4 quantization methods)
- **Metrics Measured**: Memory usage, compression ratio, search latency, quality (MAP, Recall@10)
- **Analysis**: Quality vs memory trade-offs, efficiency scores, overall recommendations

**Sample Results** (1000 documents):
- **512D + SQ-8bit**: 2.1MB memory, 4.0x compression, 850μs latency, 0.8234 MAP
- **256D + Binary**: 0.3MB memory, 32.0x compression, 420μs latency, 0.7891 MAP
- **1024D + None**: 3.9MB memory, 1.0x compression, 1200μs latency, 0.9012 MAP

**Use Case Recommendations**:
- **Production Default**: 512D + SQ-8bit (balanced quality/memory)
- **Maximum Quality**: 1024D + None (when accuracy critical)
- **Memory Constrained**: 256D + Binary (< 1MB target)
- **Low Latency**: 256D + SQ-8bit (< 500μs target)

### Run with Specific Features
```bash
# With GPU acceleration
cargo bench --features hive-gpu

# With all features
cargo bench --features full
```

## Benchmark Categories

### Core Operations
- **Vector insertion**: Single and batch insert performance
- **Vector updates**: Individual and batch update operations
- **Vector deletion**: Single and batch delete performance
- **Search operations**: Various k values (1, 10, 100) for similarity search
- **Concurrent mixed workload**: 70% search, 20% insert, 10% delete operations
- **Query caching**: Cache hit/miss performance
- **Collection management**: Basic CRUD operations

### Search Performance
- Vector similarity search
- HNSW index performance
- Query optimization
- Search latency measurements

### GPU Acceleration
- Metal (macOS) GPU benchmarks
- CUDA (NVIDIA) GPU benchmarks
- GPU vs CPU performance comparison
- Memory bandwidth tests

### Replication
- Replication log performance
- Snapshot creation and application
- Master-replica synchronization
- Concurrent replication operations

### Storage & Persistence
- Disk I/O performance
- Memory-mapped file operations
- Serialization/deserialization
- Data compression

### Quantization
- Scalar quantization performance
- Product quantization benchmarks
- Memory usage optimization
- Quality vs performance trade-offs

### Embeddings
- Embedding model performance
- Batch processing efficiency
- Model loading and inference
- Cross-encoder reranking

### Performance & Scale
- Large-scale vector operations
- Memory usage under load
- Concurrent access patterns
- System resource utilization
- Combined dimension and quantization optimization
- Quality vs memory trade-off analysis

## Benchmark Configuration

Each benchmark uses the Criterion framework for statistical analysis and HTML reporting. Results are saved to `target/criterion/` with detailed performance metrics.

### Key Metrics
- **Throughput**: Operations per second
- **Latency**: Time per operation
- **Memory Usage**: Peak and average memory consumption
- **CPU Utilization**: Processor usage patterns

## Best Practices

1. **Consistent Setup**: Use the same test data and configuration across related benchmarks
2. **Statistical Significance**: Run benchmarks multiple times for reliable results
3. **Resource Monitoring**: Track memory and CPU usage during benchmarks
4. **Documentation**: Document any special setup requirements or known limitations
5. **Version Control**: Commit benchmark results for performance regression tracking

## Troubleshooting

### Common Issues
- **Out of Memory**: Reduce vector count or dimension in large-scale benchmarks
- **GPU Not Found**: Ensure proper GPU drivers and feature flags are enabled
- **Slow Benchmarks**: Check system load and close unnecessary applications

### Performance Tips
- Run benchmarks on dedicated hardware when possible
- Use release builds for accurate performance measurements
- Warm up the system before running critical benchmarks
- Monitor system resources during benchmark execution
