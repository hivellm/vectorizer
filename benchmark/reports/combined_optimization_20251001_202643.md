# Combined Dimension + Quantization Optimization Benchmark

**Generated**: 2025-10-01 20:26:43 UTC

**Dataset**: 3000 documents

## Executive Summary

| Config | Memory | Compress | Search (μs) | QPS | MAP | Recall@10 | Score |
|--------|--------|----------|-------------|-----|-----|-----------|-------|
| 🥇 512D+SQ-8bit | 1.46 MB | 4.0x | 473 | 2115 | 0.9900 | 0.9900 | 0.774 |
| 🥈 512D+None | 5.86 MB | 1.0x | 480 | 2085 | 1.0000 | 1.0000 | 0.726 |
| 🥉 384D+SQ-8bit | 1.10 MB | 4.0x | 421 | 2375 | 0.6602 | 0.7100 | 0.628 |
|    512D+Binary | 0.18 MB | 32.0x | 256 | 3912 | 0.2020 | 0.2500 | 0.602 |
|    768D+SQ-8bit | 2.20 MB | 4.0x | 625 | 1599 | 0.7479 | 0.7800 | 0.568 |
|    384D+None | 4.39 MB | 1.0x | 456 | 2195 | 0.6501 | 0.7000 | 0.559 |
|    256D+Binary | 0.09 MB | 32.0x | 234 | 4271 | 0.0793 | 0.1500 | 0.553 |
|    768D+None | 8.79 MB | 1.0x | 622 | 1608 | 0.7410 | 0.7700 | 0.540 |
|    1024D+SQ-8bit | 2.93 MB | 4.0x | 845 | 1183 | 0.7373 | 0.7500 | 0.512 |
|    512D+PQ | 0.52 MB | 11.2x | 461 | 2171 | 0.4178 | 0.4800 | 0.506 |

## Best Configurations by Category

### 🏆 Best Quality: 512D + None
- MAP: 1.0000
- Recall@10: 1.0000
- Memory: 5.86 MB
- Search: 480 μs

### 💾 Most Memory Efficient: 256D + Binary
- Memory: 0.09 MB
- Compression: 32.0x
- Quality (MAP): 0.0793
- Search: 234 μs

### ⚡ Fastest Search: 256D + Binary
- Latency: 234 μs
- QPS: 4271
- Quality (MAP): 0.0793
- Memory: 0.09 MB

### 🎯 Best Efficiency (Quality/Memory): 512D + Binary
- Efficiency: 1.1030 MAP/MB
- Quality: 0.2020 MAP
- Memory: 0.18 MB
- Compression: 32.0x

## Detailed Results by Dimension

### 256D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 2.93 MB | 1.0x | 347 | 0.2951 | 0.3800 |
| SQ-8bit | 0.73 MB | 4.0x | 360 | 0.3025 | 0.3900 |
| PQ | 0.27 MB | 10.7x | 332 | 0.2051 | 0.2700 |
| Binary | 0.09 MB | 32.0x | 234 | 0.0793 | 0.1500 |

### 384D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 4.39 MB | 1.0x | 456 | 0.6501 | 0.7000 |
| SQ-8bit | 1.10 MB | 4.0x | 421 | 0.6602 | 0.7100 |
| PQ | 0.40 MB | 11.0x | 381 | 0.2475 | 0.3600 |
| Binary | 0.14 MB | 32.0x | 261 | 0.0880 | 0.1800 |

### 512D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 5.86 MB | 1.0x | 480 | 1.0000 | 1.0000 |
| SQ-8bit | 1.46 MB | 4.0x | 473 | 0.9900 | 0.9900 |
| PQ | 0.52 MB | 11.2x | 461 | 0.4178 | 0.4800 |
| Binary | 0.18 MB | 32.0x | 256 | 0.2020 | 0.2500 |

### 768D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 8.79 MB | 1.0x | 622 | 0.7410 | 0.7700 |
| SQ-8bit | 2.20 MB | 4.0x | 625 | 0.7479 | 0.7800 |
| PQ | 0.77 MB | 11.4x | 678 | 0.2980 | 0.3800 |
| Binary | 0.27 MB | 32.0x | 321 | 0.2170 | 0.3000 |

### 1024D Embeddings

| Quantization | Memory | Compression | Search (μs) | MAP | Recall@10 |
|--------------|--------|-------------|-------------|-----|----------|
| None | 11.72 MB | 1.0x | 822 | 0.7486 | 0.7600 |
| SQ-8bit | 2.93 MB | 4.0x | 845 | 0.7373 | 0.7500 |
| PQ | 1.02 MB | 11.5x | 691 | 0.2362 | 0.3000 |
| Binary | 0.37 MB | 32.0x | 394 | 0.1715 | 0.2400 |

## Key Trade-offs Analysis

### Quality Loss by Quantization

**256D**:
- SQ-8bit: 102.5% quality retention, 4.0x compression
- PQ: 69.5% quality retention, 10.7x compression
- Binary: 26.9% quality retention, 32.0x compression

**384D**:
- SQ-8bit: 101.6% quality retention, 4.0x compression
- PQ: 38.1% quality retention, 11.0x compression
- Binary: 13.5% quality retention, 32.0x compression

**512D**:
- SQ-8bit: 99.0% quality retention, 4.0x compression
- PQ: 41.8% quality retention, 11.2x compression
- Binary: 20.2% quality retention, 32.0x compression

**768D**:
- SQ-8bit: 100.9% quality retention, 4.0x compression
- PQ: 40.2% quality retention, 11.4x compression
- Binary: 29.3% quality retention, 32.0x compression

**1024D**:
- SQ-8bit: 98.5% quality retention, 4.0x compression
- PQ: 31.6% quality retention, 11.5x compression
- Binary: 22.9% quality retention, 32.0x compression

### Memory Savings Matrix

Comparison to 512D + No Quantization baseline:

| Config | Memory | vs Baseline | Quality | vs Baseline |
|--------|--------|-------------|---------|-------------|
| 512D+SQ-8bit | 1.46 MB | +75.0% | 0.9900 | -1.0% |
| 512D+None | 5.86 MB | +0.0% | 1.0000 | +0.0% |
| 384D+SQ-8bit | 1.10 MB | +81.2% | 0.6602 | -34.0% |
| 512D+Binary | 0.18 MB | +96.9% | 0.2020 | -79.8% |
| 768D+SQ-8bit | 2.20 MB | +62.5% | 0.7479 | -25.2% |
| 384D+None | 4.39 MB | +25.0% | 0.6501 | -35.0% |
| 256D+Binary | 0.09 MB | +98.4% | 0.0793 | -92.1% |
| 768D+None | 8.79 MB | -50.0% | 0.7410 | -25.9% |
| 1024D+SQ-8bit | 2.93 MB | +50.0% | 0.7373 | -26.3% |
| 512D+PQ | 0.52 MB | +91.1% | 0.4178 | -58.2% |
| 1024D+None | 11.72 MB | -100.0% | 0.7486 | -25.1% |
| 768D+Binary | 0.27 MB | +95.3% | 0.2170 | -78.3% |
| 384D+Binary | 0.14 MB | +97.7% | 0.0880 | -91.2% |
| 256D+PQ | 0.27 MB | +95.3% | 0.2051 | -79.5% |
| 256D+SQ-8bit | 0.73 MB | +87.5% | 0.3025 | -69.7% |
| 384D+PQ | 0.40 MB | +93.2% | 0.2475 | -75.3% |
| 256D+None | 2.93 MB | +50.0% | 0.2951 | -70.5% |
| 1024D+Binary | 0.37 MB | +93.7% | 0.1715 | -82.8% |
| 768D+PQ | 0.77 MB | +86.8% | 0.2980 | -70.2% |
| 1024D+PQ | 1.02 MB | +82.5% | 0.2362 | -76.4% |

## Recommendations

### 🥇 Overall Winner

**512D + SQ-8bit** (Score: 0.774)

Reasons:
- Quality: 0.9900 MAP (best balance)
- Memory: 1.46 MB (4.0x compression)
- Performance: 473 μs (2115 QPS)
- Efficiency: 0.6758 MAP/MB

### Use Case Recommendations

1. **Production Default** (balanced):
   - 512D + SQ-8bit
   - 1.46 MB memory, 0.9900 MAP

2. **Maximum Quality** (when accuracy critical):
   - 512D + None
   - 5.86 MB memory, 1.0000 MAP

3. **Memory Constrained** (< 2 MB target):
   - 512D + SQ-8bit
   - 1.46 MB memory, 0.9900 MAP

4. **Low Latency** (< 500 μs target):
   - 512D + None
   - 480 μs latency, 1.0000 MAP

---

*Report generated by Vectorizer Combined Optimization Benchmark*
