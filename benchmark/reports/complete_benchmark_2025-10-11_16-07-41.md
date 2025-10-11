# Complete Normalization & Quantization Benchmark

**Date**: 2025-10-11 16:07:41 UTC
**Documents**: 50
**Embedding**: BM25 (real)
**Index**: HNSW (real)
**Scenarios**: 6

---

## 💾 Storage Impact

| Scenario | Text | Vectors | Total | Saved | Reduction |
|----------|------|---------|-------|-------|-----------|
| Baseline (No Norm, No Quant) | 475 KB | 100 KB | 575 KB | 0 KB | 0.0% |
| Quantization Only (SQ-8) | 475 KB | 25 KB | 500 KB | 75 KB | 13.0% |
| Normalization Conservative | 469 KB | 100 KB | 569 KB | 6 KB | 1.0% |
| Normalization Moderate | 469 KB | 100 KB | 569 KB | 6 KB | 1.0% |
| Normalization Aggressive | 469 KB | 100 KB | 569 KB | 6 KB | 1.0% |
| Moderate + SQ-8 (DEFAULT) | 469 KB | 25 KB | 494 KB | 81 KB | 14.1% |

## ⚡ Performance

| Scenario | Preprocessing | Search | Total |
|----------|---------------|--------|-------|
| Baseline (No Norm, No Quant) | 5.6995ms | 756.95µs | 5.6995ms |
| Quantization Only (SQ-8) | 5.4947ms | 734.8µs | 5.4947ms |
| Normalization Conservative | 11.2262ms | 982.75µs | 11.2262ms |
| Normalization Moderate | 12.4062ms | 694.55µs | 12.4062ms |
| Normalization Aggressive | 10.2922ms | 743.5µs | 10.2922ms |
| Moderate + SQ-8 (DEFAULT) | 10.5461ms | 737.8µs | 10.5461ms |

## 🎯 Search Quality

| Scenario | Precision | Recall | F1-Score |
|----------|-----------|--------|----------|
| Baseline (No Norm, No Quant) | 40.0% | 9.8% | 15.7% |
| Quantization Only (SQ-8) | 40.0% | 9.8% | 15.7% |
| Normalization Conservative | 40.0% | 9.8% | 15.7% |
| Normalization Moderate | 40.0% | 9.8% | 15.7% |
| Normalization Aggressive | 40.0% | 9.8% | 15.7% |
| Moderate + SQ-8 (DEFAULT) | 40.0% | 9.8% | 15.7% |

## ✅ Key Findings

**Moderate + SQ-8 (Default Configuration)**:
- Storage: -81 KB (14.1% reduction)
- Quality: 15.7% F1 (+0.0% vs baseline)
- Latency: 737.8µs (-2.5% vs baseline)

---
**Generated**: 2025-10-11 16:07:41 UTC
