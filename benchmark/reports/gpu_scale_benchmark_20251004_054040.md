# GPU Scale Performance Benchmark

**Generated**: 2025-10-04 05:40:40 UTC

## Test Configuration

- **Dimension**: 512D
- **Test Sizes**: [1000, 5000, 10000, 25000, 50000, 100000] vectors
- **Backends**: ["Cpu", "Vulkan", "DirectX12"]
- **HNSW Config**: M=16, ef_construction=200
- **Distance**: Cosine
- **Quantization**: SQ-8bit

## Backend Performance Comparison

### Speedup vs CPU Baseline

- **Vulkan**: 0.94x faster than CPU
- **DirectX 12**: 0.85x faster than CPU
- **Vulkan vs DirectX 12**: 1.11x
- **Best Backend**: Vulkan
- **Most Efficient**: DirectX 12

## Performance Results by Backend

### DirectX12 Backend

| Size | Build Time | Memory | Search Latency | QPS | MAP | GPU Memory |
|------|-----------|--------|----------------|-----|-----|------------|
| 1K | 0.6s | 2.1MB | 500μs | 2000 | 0.013 | 1.6MB |
| 5K | 8.9s | 10.3MB | 1511μs | 667 | 0.003 | 8.2MB |
| 10K | 27.9s | 20.6MB | 3479μs | 278 | 0.001 | 16.5MB |
| 25K | 103.6s | 51.5MB | 6949μs | 137 | 0.000 | 41.2MB |
| 50K | 249.7s | 103.0MB | 9503μs | 97 | 0.000 | 82.4MB |
| 100K | 617.7s | 206.0MB | 35726μs | 26 | 0.000 | 164.8MB |

### Cpu Backend

| Size | Build Time | Memory | Search Latency | QPS | MAP | GPU Memory |
|------|-----------|--------|----------------|-----|-----|------------|
| 1K | 0.6s | 2.1MB | 479μs | 2500 | 0.010 | 0.0MB |
| 5K | 8.4s | 10.3MB | 1392μs | 714 | 0.002 | 0.0MB |
| 10K | 28.2s | 20.5MB | 3079μs | 312 | 0.001 | 0.0MB |
| 25K | 108.1s | 51.3MB | 7660μs | 123 | 0.001 | 0.0MB |
| 50K | 253.5s | 102.6MB | 11335μs | 82 | 0.000 | 0.0MB |
| 100K | 586.6s | 205.2MB | 23430μs | 40 | 0.000 | 0.0MB |

### Vulkan Backend

| Size | Build Time | Memory | Search Latency | QPS | MAP | GPU Memory |
|------|-----------|--------|----------------|-----|-----|------------|
| 1K | 0.6s | 2.1MB | 420μs | 2500 | 0.010 | 1.6MB |
| 5K | 8.8s | 10.3MB | 2156μs | 455 | 0.002 | 8.2MB |
| 10K | 27.9s | 20.6MB | 2730μs | 357 | 0.001 | 16.5MB |
| 25K | 103.9s | 51.5MB | 7539μs | 127 | 0.001 | 41.2MB |
| 50K | 253.0s | 103.0MB | 12736μs | 73 | 0.000 | 82.4MB |
| 100K | 560.1s | 206.0MB | 19238μs | 48 | 0.000 | 164.8MB |

## GPU vs CPU Performance Analysis

| Size | Vulkan QPS | DirectX12 QPS | CPU QPS | Vulkan Speedup | DirectX12 Speedup |
|------|------------|---------------|---------|----------------|-------------------|
| 1K | 2500 | 2000 | 2500 | 1.00x | 0.80x |
| 5K | 455 | 667 | 714 | 0.64x | 0.93x |
| 10K | 357 | 278 | 312 | 1.14x | 0.89x |
| 25K | 127 | 137 | 123 | 1.03x | 1.11x |
| 50K | 73 | 97 | 82 | 0.89x | 1.18x |
| 100K | 48 | 26 | 40 | 1.20x | 0.66x |

## GPU Recommendations

### Optimal Backend by Size

- **50K vectors**: directx12
- **100K vectors**: vulkan
- **1K vectors**: vulkan
- **10K vectors**: vulkan
- **25K vectors**: cpu
- **5K vectors**: directx12

### GPU Threshold: **100K vectors**

### Most Efficient Backends

- **Memory Efficient**: DirectX 12
- **Performance Efficient**: Vulkan
- **Cost Effective**: Vulkan

## Implementation Guidelines

### Backend Selection Strategy

1. **Small Datasets** (< 5K): Use CPU for simplicity
2. **Medium Datasets** (5K-25K): Use DirectX 12 on Windows, Vulkan on Linux
3. **Large Datasets** (25K+): Use Vulkan for best cross-platform performance

### GPU Optimization Tips

- **Memory Management**: Monitor GPU memory usage to avoid OOM
- **Batch Operations**: Use batch insertions for better GPU utilization
- **Quantization**: Enable SQ-8bit for memory efficiency
- **Backend Selection**: Choose based on platform and dataset size

---

*GPU Scale benchmark report generated automatically*
