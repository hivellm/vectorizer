# GPU Scale Performance Benchmark

**Generated**: 2025-10-04 08:38:26 UTC

## Test Configuration

- **Dimension**: 512D
- **Test Sizes**: [1000, 5000, 10000, 25000, 50000, 100000] vectors
- **Backends**: ["Vulkan", "DirectX12"]
- **HNSW Config**: M=16, ef_construction=200
- **Distance**: Cosine
- **Quantization**: SQ-8bit

## Backend Performance Comparison

### Speedup vs CPU Baseline

- **Vulkan**: 1.00x faster than CPU
- **DirectX 12**: 1.00x faster than CPU
- **Vulkan vs DirectX 12**: 1.00x
- **Best Backend**: DirectX 12
- **Most Efficient**: DirectX 12

## Performance Results by Backend

### Vulkan Backend

| Size | Build Time | Memory | Search Latency | QPS | MAP | GPU Memory |
|------|-----------|--------|----------------|-----|-----|------------|
| 1K | 0.1s | 346.5MB | 2022μs | 500 | 0.015 | 277.2MB |
| 5K | 0.6s | 346.8MB | 1944μs | 500 | 0.003 | 277.5MB |
| 10K | 1.3s | 347.3MB | 2126μs | 435 | 0.002 | 277.8MB |
| 25K | 3.2s | 348.5MB | 2233μs | 370 | 0.001 | 278.8MB |
| 50K | 6.6s | 350.6MB | 2318μs | 294 | 0.000 | 280.5MB |
| 100K | 12.8s | 354.8MB | 3184μs | 182 | 0.000 | 283.8MB |

### DirectX12 Backend

| Size | Build Time | Memory | Search Latency | QPS | MAP | GPU Memory |
|------|-----------|--------|----------------|-----|-----|------------|
| 1K | 0.1s | 346.5MB | 2043μs | 500 | 0.015 | 277.2MB |
| 5K | 0.6s | 346.8MB | 2016μs | 500 | 0.003 | 277.5MB |
| 10K | 1.4s | 347.3MB | 2156μs | 435 | 0.002 | 277.8MB |
| 25K | 3.2s | 348.5MB | 2240μs | 370 | 0.001 | 278.8MB |
| 50K | 6.4s | 350.6MB | 2338μs | 303 | 0.000 | 280.5MB |
| 100K | 12.3s | 354.8MB | 3432μs | 175 | 0.000 | 283.8MB |

## GPU vs CPU Performance Analysis

| Size | Vulkan QPS | DirectX12 QPS | DirectX12 vs Vulkan |
|------|------------|---------------|--------------------|
| 1K | 500 | 500 | 1.00x |
| 5K | 500 | 500 | 1.00x |
| 10K | 435 | 435 | 1.00x |
| 25K | 370 | 370 | 1.00x |
| 50K | 294 | 303 | 1.03x |
| 100K | 182 | 175 | 0.96x |

## GPU vs GPU Comparison

### Optimal GPU Backend by Size

- **25K vectors**: vulkan
- **1K vectors**: vulkan
- **10K vectors**: vulkan
- **100K vectors**: vulkan
- **5K vectors**: vulkan
- **50K vectors**: directx12

### GPU Performance Analysis

- **Best Performance**: DirectX 12
- **Most Memory Efficient**: DirectX 12
- **Most Cost Effective**: DirectX 12

## GPU Implementation Guidelines

### GPU Backend Selection Strategy

1. **Small Datasets** (< 5K): Use Vulkan for cross-platform compatibility
2. **Medium Datasets** (5K-25K): Use DirectX 12 on Windows, Vulkan on Linux
3. **Large Datasets** (25K+): Use Vulkan for best cross-platform performance

### GPU Optimization Tips

- **Memory Management**: Monitor GPU memory usage to avoid OOM
- **Batch Operations**: Use batch insertions for better GPU utilization
- **Quantization**: Enable SQ-8bit for memory efficiency
- **Backend Selection**: Choose based on platform and dataset size

---

*GPU Scale benchmark report generated automatically*
