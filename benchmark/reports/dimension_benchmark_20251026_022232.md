# Dimension Comparison Performance Benchmark

**Generated**: 2025-10-26 02:22:32 UTC

## Test Configuration

- **Dataset Size**: 10000 vectors
- **Test Dimensions**: [64, 128, 256, 512, 768, 1024, 1536]D
- **HNSW Config**: M=16, ef_construction=200
- **Distance**: Cosine

## Performance Results

| Dimension | Build Time | Memory | Search Latency | QPS | MAP | Recall@10 | Memory Eff |
|-----------|-----------|--------|----------------|-----|-----|-----------|------------|
| 64D | 0.6s | 2.4MB | 408μs | 2500 | 0.029 | 0.077 | 4096/MB |
| 128D | 1.1s | 4.9MB | 665μs | 1429 | 0.038 | 0.157 | 2048/MB |
| 256D | 2.3s | 9.8MB | 642μs | 1429 | 0.017 | 0.053 | 1024/MB |
| 512D | 4.1s | 19.5MB | 830μs | 1111 | 0.053 | 0.234 | 512/MB |
| 768D | 6.1s | 29.3MB | 895μs | 1000 | 0.014 | 0.139 | 341/MB |
| 1024D | 7.3s | 39.1MB | 876μs | 1000 | 0.012 | 0.117 | 256/MB |
| 1536D | 7.7s | 58.6MB | 1055μs | 833 | 0.107 | 0.292 | 171/MB |

## Dimension Analysis

### Performance vs Dimension

| Dimension | Latency | QPS | Memory | Quality |
|-----------|---------|-----|--------|----------|
| 64D | 1.0x | 1.0x | 2.4MB | 1.0x |
| 128D | 1.6x | 0.6x | 4.9MB | 1.3x |
| 256D | 1.6x | 0.6x | 9.8MB | 0.6x |
| 512D | 2.0x | 0.4x | 19.5MB | 1.8x |
| 768D | 2.2x | 0.4x | 29.3MB | 0.5x |
| 1024D | 2.1x | 0.4x | 39.1MB | 0.4x |
| 1536D | 2.6x | 0.3x | 58.6MB | 3.6x |

## Recommendations

### Optimal Dimension: **1536D**

### Memory Efficient: **64D**

### Speed Efficient: **64D**

### Quality Efficient: **64D**

### Balanced: **64D**

## Use Case Guidelines

### Low Latency Applications

**Recommended**: 64D

**Reasoning**: Lowest search latency: 408μs

**Trade-offs**: May sacrifice some quality for speed

### High Quality Applications

**Recommended**: 1536D

**Reasoning**: Highest MAP score: 0.1068

**Trade-offs**: May require more memory and computation

### Memory Constrained Applications

**Recommended**: 64D

**Reasoning**: Highest memory efficiency: 4096 vectors/MB

**Trade-offs**: May sacrifice quality for memory efficiency

### High Throughput Applications

**Recommended**: 64D

**Reasoning**: Highest throughput: 2500 QPS

**Trade-offs**: May require more memory for optimal performance

## Implementation Guidelines

### Dimension Selection Strategy

1. **Low Latency** (< 1ms): Use 64D-128D
2. **Balanced Performance**: Use 256D-512D
3. **High Quality**: Use 768D-1024D
4. **Memory Constrained**: Use 64D-256D
5. **High Throughput**: Use 128D-512D

### Monitoring Thresholds

- **Latency Alert**: > 5ms average search
- **Quality Alert**: < 0.8 MAP score
- **Memory Alert**: > 1GB per collection
- **Throughput Alert**: < 100 QPS

---

*Dimension comparison benchmark report generated automatically*
