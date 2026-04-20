# Combined Dimension + Quantization Optimization Benchmark

**Generated**: 2025-10-26 02:59:51 UTC

**Dataset**: 1000 documents

## Executive Summary

| Config | Memory | Compress | Search (μs) | QPS | MAP | Recall@10 | Score |
|--------|--------|----------|-------------|-----|-----|-----------|-------|
| 🥇 384D+Binary | 0.05 MB | 32.0x | 110 | 9102 | 0.1312 | 0.2500 | 1.262 |
| 🥈 256D+Binary | 0.03 MB | 32.0x | 107 | 9303 | 0.0727 | 0.1600 | 1.205 |
| 🥉 384D+SQ-8bit | 0.37 MB | 4.0x | 111 | 9004 | 0.1573 | 0.2500 | 1.022 |
|    256D+SQ-8bit | 0.24 MB | 4.0x | 108 | 9290 | 0.0707 | 0.1700 | 0.993 |
|    256D+PQ | 0.26 MB | 3.8x | 115 | 8728 | 0.1013 | 0.2200 | 0.963 |
|    256D+None | 0.98 MB | 1.0x | 113 | 8877 | 0.0540 | 0.1000 | 0.920 |
|    384D+PQ | 0.38 MB | 3.8x | 142 | 7059 | 0.0917 | 0.2400 | 0.776 |
|    512D+SQ-8bit | 0.49 MB | 4.0x | 153 | 6552 | 0.1083 | 0.2400 | 0.732 |
|    384D+None | 1.46 MB | 1.0x | 149 | 6721 | 0.0713 | 0.1700 | 0.713 |
|    512D+None | 1.95 MB | 1.0x | 158 | 6317 | 0.1146 | 0.2400 | 0.695 |

## Best Configurations by Category

### 🏆 Best Quality: 384D + SQ-8bit
- MAP: 0.1573
- Recall@10: 0.2500
- Memory: 0.37 MB
- Search: 111 μs

### 💾 Most Memory Efficient: 256D + Binary
- Memory: 0.03 MB
- Compression: 32.0x
- Quality (MAP): 0.0727
- Search: 107 μs

### ⚡ Fastest Search: 256D + Binary
- Latency: 107 μs
- QPS: 9303
- Quality (MAP): 0.0727
- Memory: 0.03 MB

### 🎯 Best Efficiency (Quality/Memory): 384D + Binary
- Efficiency: 2.8651 MAP/MB
- Quality: 0.1312 MAP
- Memory: 0.05 MB
- Compression: 32.0x

## Detailed Results by Dimension

### 256D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 0.98 MB | 1.0x | 113 | 0.0540 | 0.1000 |
| SQ-8bit | 0.24 MB | 4.0x | 108 | 0.0707 | 0.1700 |
| PQ | 0.26 MB | 3.8x | 115 | 0.1013 | 0.2200 |
| Binary | 0.03 MB | 32.0x | 107 | 0.0727 | 0.1600 |

### 384D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 1.46 MB | 1.0x | 149 | 0.0713 | 0.1700 |
| SQ-8bit | 0.37 MB | 4.0x | 111 | 0.1573 | 0.2500 |
| PQ | 0.38 MB | 3.8x | 142 | 0.0917 | 0.2400 |
| Binary | 0.05 MB | 32.0x | 110 | 0.1312 | 0.2500 |

### 512D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 1.95 MB | 1.0x | 158 | 0.1146 | 0.2400 |
| SQ-8bit | 0.49 MB | 4.0x | 153 | 0.1083 | 0.2400 |
| PQ | 0.51 MB | 3.8x | 169 | 0.0825 | 0.2000 |
| Binary | 0.06 MB | 32.0x | 179 | 0.0520 | 0.1400 |

### 768D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 2.93 MB | 1.0x | 185 | 0.0340 | 0.1400 |
| SQ-8bit | 0.73 MB | 4.0x | 172 | 0.0426 | 0.1000 |
| PQ | 0.76 MB | 3.9x | 225 | 0.0300 | 0.0800 |
| Binary | 0.09 MB | 32.0x | 178 | 0.0757 | 0.1800 |

### 1024D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 3.91 MB | 1.0x | 221 | 0.0413 | 0.1500 |
| SQ-8bit | 0.98 MB | 4.0x | 232 | 0.0351 | 0.1200 |
| PQ | 1.01 MB | 3.9x | 296 | 0.0320 | 0.0500 |
| Binary | 0.12 MB | 32.0x | 247 | 0.0897 | 0.1300 |

## Key Trade-offs Analysis

### Quality Loss by Quantization

**256D**:
- SQ-8bit: 130.8% quality retention, 4.0x compression
- PQ: 187.5% quality retention, 3.8x compression
- Binary: 134.6% quality retention, 32.0x compression

**384D**:
- SQ-8bit: 220.6% quality retention, 4.0x compression
- PQ: 128.5% quality retention, 3.8x compression
- Binary: 183.9% quality retention, 32.0x compression

**512D**:
- SQ-8bit: 94.5% quality retention, 4.0x compression
- PQ: 72.0% quality retention, 3.8x compression
- Binary: 45.4% quality retention, 32.0x compression

**768D**:
- SQ-8bit: 125.1% quality retention, 4.0x compression
- PQ: 88.1% quality retention, 3.9x compression
- Binary: 222.4% quality retention, 32.0x compression

**1024D**:
- SQ-8bit: 85.1% quality retention, 4.0x compression
- PQ: 77.5% quality retention, 3.9x compression
- Binary: 217.3% quality retention, 32.0x compression

### Memory Savings Matrix

Comparison to 512D + No Quantization baseline:

| Config | Memory | vs Baseline | Quality | vs Baseline |
|--------|--------|-------------|---------|-------------|
| 384D+Binary | 0.05 MB | +97.7% | 0.1312 | +14.5% |
| 256D+Binary | 0.03 MB | +98.4% | 0.0727 | -36.6% |
| 384D+SQ-8bit | 0.37 MB | +81.2% | 0.1573 | +37.3% |
| 256D+SQ-8bit | 0.24 MB | +87.5% | 0.0707 | -38.3% |
| 256D+PQ | 0.26 MB | +86.8% | 0.1013 | -11.6% |
| 256D+None | 0.98 MB | +50.0% | 0.0540 | -52.9% |
| 384D+PQ | 0.38 MB | +80.4% | 0.0917 | -20.0% |
| 512D+SQ-8bit | 0.49 MB | +75.0% | 0.1083 | -5.5% |
| 384D+None | 1.46 MB | +25.0% | 0.0713 | -37.8% |
| 512D+None | 1.95 MB | +0.0% | 0.1146 | +0.0% |
| 768D+Binary | 0.09 MB | +95.3% | 0.0757 | -34.0% |
| 512D+Binary | 0.06 MB | +96.9% | 0.0520 | -54.6% |
| 512D+PQ | 0.51 MB | +74.0% | 0.0825 | -28.0% |
| 768D+SQ-8bit | 0.73 MB | +62.5% | 0.0426 | -62.9% |
| 768D+None | 2.93 MB | -50.0% | 0.0340 | -70.3% |
| 1024D+Binary | 0.12 MB | +93.7% | 0.0897 | -21.7% |
| 1024D+None | 3.91 MB | -100.0% | 0.0413 | -64.0% |
| 768D+PQ | 0.76 MB | +61.2% | 0.0300 | -73.8% |
| 1024D+SQ-8bit | 0.98 MB | +50.0% | 0.0351 | -69.3% |
| 1024D+PQ | 1.01 MB | +48.4% | 0.0320 | -72.1% |

## Recommendations

### 🥇 Overall Winner

**384D + Binary** (Score: 1.262)

Reasons:
- Quality: 0.1312 MAP (best balance)
- Memory: 0.05 MB (32.0x compression)
- Performance: 110 μs (9102 QPS)
- Efficiency: 2.8651 MAP/MB

### Use Case Recommendations

1. **Production Default** (balanced):
   - 384D + Binary
   - 0.05 MB memory, 0.1312 MAP

2. **Maximum Quality** (when accuracy critical):
   - 384D + SQ-8bit
   - 0.37 MB memory, 0.1573 MAP

3. **Memory Constrained** (< 2 MB target):
   - 384D + SQ-8bit
   - 0.37 MB memory, 0.1573 MAP

4. **Low Latency** (< 500 μs target):
   - 384D + SQ-8bit
   - 111 μs latency, 0.1573 MAP

---

*Report generated by Vectorizer Combined Optimization Benchmark*
