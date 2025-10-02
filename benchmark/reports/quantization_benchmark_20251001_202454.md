# Vector Quantization Benchmark Report

**Generated**: 2025-10-01 20:24:54 UTC

## Dataset Information

- **Documents**: 19874
- **Vectors**: 19874 (dimension: 512)
- **Queries**: 20
- **Source**: HiveLLM Workspace (all projects)

## Summary Comparison

| Method | Memory (MB) | Compression | Build Time (ms) | Avg Search (μs) | MAP | Recall@10 | Quality Loss |
|--------|-------------|-------------|-----------------|-----------------|-----|-----------|-------------|
| Baseline | 38.98 | 1.00x | 16940 | 2514 | 0.8400 | 0.8400 | 0.0% |
| Scalar Quantization (SQ) | 9.70 | 4.00x | 16998 | 2709 | 0.9147 | 0.9200 | 8.9% |
| Scalar Quantization (SQ) | 9.70 | 4.00x | 17044 | 2551 | 0.7004 | 0.7450 | 16.6% |
| Product Quantization (PQ) | 0.65 | 59.57x | 21850 | 2529 | 0.2573 | 0.3350 | 69.4% |
| Product Quantization (PQ) | 0.80 | 48.32x | 22400 | 2726 | 0.3482 | 0.4300 | 58.6% |
| Product Quantization (PQ) | 1.15 | 33.71x | 33079 | 2805 | 0.0574 | 0.0900 | 93.2% |
| Binary Quantization | 1.21 | 32.00x | 11718 | 2444 | 0.0146 | 0.0350 | 98.3% |

## Detailed Results

### Baseline

**Configuration**: No quantization (full f32)

#### Memory & Performance

- **Memory Usage**: 38.98 MB (40869708 bytes)
- **Compression Ratio**: 1.00x
- **Index Build Time**: 16940.00 ms
- **Avg Search Time**: 2514 μs
- **P50 Search Time**: 2546 μs
- **P95 Search Time**: 2816 μs
- **P99 Search Time**: 2816 μs

#### Search Quality

- **MAP**: 0.8400
- **MRR**: 1.0000
- **Precision@1**: 1.0000
- **Precision@5**: 1.0000
- **Precision@10**: 1.0000
- **Recall@1**: 0.1000
- **Recall@5**: 0.4900
- **Recall@10**: 0.8400
- **NDCG@10**: 0.6772

### Scalar Quantization (SQ)

**Configuration**: bits=8

#### Memory & Performance

- **Memory Usage**: 9.70 MB (10175496 bytes)
- **Compression Ratio**: 4.00x
- **Index Build Time**: 16998.00 ms
- **Avg Search Time**: 2709 μs
- **P50 Search Time**: 2721 μs
- **P95 Search Time**: 2918 μs
- **P99 Search Time**: 2918 μs

#### Search Quality

- **MAP**: 0.9147
- **MRR**: 1.0000
- **Precision@1**: 1.0000
- **Precision@5**: 1.0000
- **Precision@10**: 0.9850
- **Recall@1**: 0.1000
- **Recall@5**: 0.4950
- **Recall@10**: 0.9200
- **NDCG@10**: 0.6932

### Scalar Quantization (SQ)

**Configuration**: bits=4

#### Memory & Performance

- **Memory Usage**: 9.70 MB (10175496 bytes)
- **Compression Ratio**: 4.00x
- **Index Build Time**: 17044.00 ms
- **Avg Search Time**: 2551 μs
- **P50 Search Time**: 2562 μs
- **P95 Search Time**: 2817 μs
- **P99 Search Time**: 2817 μs

#### Search Quality

- **MAP**: 0.7004
- **MRR**: 1.0000
- **Precision@1**: 1.0000
- **Precision@5**: 0.9600
- **Precision@10**: 0.8700
- **Recall@1**: 0.1000
- **Recall@5**: 0.4400
- **Recall@10**: 0.7450
- **NDCG@10**: 0.6114

### Product Quantization (PQ)

**Configuration**: subquantizers=8, centroids=256

#### Memory & Performance

- **Memory Usage**: 0.65 MB (683280 bytes)
- **Compression Ratio**: 59.57x
- **Index Build Time**: 21850.00 ms
- **Avg Search Time**: 2529 μs
- **P50 Search Time**: 2501 μs
- **P95 Search Time**: 2776 μs
- **P99 Search Time**: 2776 μs

#### Search Quality

- **MAP**: 0.2573
- **MRR**: 0.7250
- **Precision@1**: 0.6500
- **Precision@5**: 0.7025
- **Precision@10**: 0.6560
- **Recall@1**: 0.0650
- **Recall@5**: 0.2350
- **Recall@10**: 0.3350
- **NDCG@10**: 0.3451

### Product Quantization (PQ)

**Configuration**: subquantizers=16, centroids=256

#### Memory & Performance

- **Memory Usage**: 0.80 MB (842272 bytes)
- **Compression Ratio**: 48.32x
- **Index Build Time**: 22400.00 ms
- **Avg Search Time**: 2726 μs
- **P50 Search Time**: 2730 μs
- **P95 Search Time**: 3025 μs
- **P99 Search Time**: 3025 μs

#### Search Quality

- **MAP**: 0.3482
- **MRR**: 0.8367
- **Precision@1**: 0.8000
- **Precision@5**: 0.7850
- **Precision@10**: 0.6878
- **Recall@1**: 0.0800
- **Recall@5**: 0.2850
- **Recall@10**: 0.4300
- **NDCG@10**: 0.4132

### Product Quantization (PQ)

**Configuration**: subquantizers=8, centroids=512

#### Memory & Performance

- **Memory Usage**: 1.15 MB (1207568 bytes)
- **Compression Ratio**: 33.71x
- **Index Build Time**: 33079.00 ms
- **Avg Search Time**: 2805 μs
- **P50 Search Time**: 2789 μs
- **P95 Search Time**: 3017 μs
- **P99 Search Time**: 3017 μs

#### Search Quality

- **MAP**: 0.0574
- **MRR**: 0.2000
- **Precision@1**: 0.1500
- **Precision@5**: 0.2217
- **Precision@10**: 0.1980
- **Recall@1**: 0.0150
- **Recall@5**: 0.0600
- **Recall@10**: 0.0900
- **NDCG@10**: 0.0939

### Binary Quantization

**Configuration**: 1-bit per dimension

#### Memory & Performance

- **Memory Usage**: 1.21 MB (1271940 bytes)
- **Compression Ratio**: 32.00x
- **Index Build Time**: 11718.00 ms
- **Avg Search Time**: 2444 μs
- **P50 Search Time**: 2456 μs
- **P95 Search Time**: 2817 μs
- **P99 Search Time**: 2817 μs

#### Search Quality

- **MAP**: 0.0146
- **MRR**: 0.0458
- **Precision@1**: 0.0000
- **Precision@5**: 0.0592
- **Precision@10**: 0.0619
- **Recall@1**: 0.0000
- **Recall@5**: 0.0250
- **Recall@10**: 0.0350
- **NDCG@10**: 0.0302

## Analysis & Recommendations

### Best Compression: Product Quantization (PQ) (59.57x)
### Best Quality: Scalar Quantization (SQ) (MAP: 0.9147)
### Fastest Search: Binary Quantization (2444 μs avg)

### Quality vs Compression Trade-offs

- **Scalar Quantization (SQ)**: 4.00x compression, 108.9% quality retention (MAP)
- **Scalar Quantization (SQ)**: 4.00x compression, 83.4% quality retention (MAP)
- **Product Quantization (PQ)**: 59.57x compression, 30.6% quality retention (MAP)
- **Product Quantization (PQ)**: 48.32x compression, 41.4% quality retention (MAP)
- **Product Quantization (PQ)**: 33.71x compression, 6.8% quality retention (MAP)
- **Binary Quantization**: 32.00x compression, 1.7% quality retention (MAP)

### Recommendations

1. **For Maximum Quality** (≥95% retention): Use Scalar Quantization (8-bit)
2. **For Balanced Trade-off** (90-95% retention): Use Product Quantization (8 subquantizers, 256 centroids)
3. **For Maximum Compression** (memory-critical): Use Binary Quantization
4. **Auto-selection Strategy**: Start with SQ-8, fall back to PQ if memory still high

### Implementation Priority

Based on results, implement in this order:
1. ✅ Scalar Quantization (8-bit) - Best quality/compression balance
2. ✅ Product Quantization - Good for very large collections
3. ⚠️  Binary Quantization - Only if extreme compression needed

---

*Report generated by Vectorizer Quantization Benchmark*
