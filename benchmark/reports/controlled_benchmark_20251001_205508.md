# 🔬 Controlled FLAT vs HNSW Benchmark Report

**Generated:** 2025-10-01 20:55:08 UTC

## 🖥️ System Information

- **CPU:** 32 x AMD Ryzen 9 7950X3D 16-Core Processor
- **NUMA:** NUMA (multi-socket)
- **Build Flags:** target-cpu=native;lto=thin;opt=3

## 📊 Summary

- **Datasets Tested:** 1
- **Total Benchmarks:** 56
- **Configurations:** FLAT + HNSW with multiple k/ef_search values

## ⚠️ Anomaly Detection

- **Total Anomalies:** 50

### Anomalies Found:

- **recall_implausible** (dataset: 10000, mode: FLAT, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: FLAT, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 1): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 10): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 50): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline
- **recall_implausible** (dataset: 10000, mode: HNSW, k: 100): MAP too low for f32 baseline

## 📈 Performance Overview

### Dataset Size: 10000

**FLAT Search:**
- QPS: 262
- Mean Average Precision: 0.4012
- Recall@10: 0.4275

**HNSW Search:**
- QPS: 1673
- Mean Average Precision: 0.0770
- Recall@10: 0.2398
- Avg Nodes Visited: 134

**Performance Comparison:**
- HNSW Speedup: 6.39x vs FLAT
- ✅ HNSW is faster

## 📋 Detailed Results

| Dataset | Mode | Quant | k | ef_search | Phase | QPS | Latency P50 | Latency P95 | MAP | Recall@10 | Nodes Visited | Anomaly |
|---------|------|-------|---|-----------|-------|-----|-------------|-------------|-----|-----------|---------------|---------|
| 10000 | FLAT | f32 | 1 | 0 | cold | 267 | 3756μs | 4013μs | 0.0747 | 0.0000 | 10000 | recall_implausible |
| 10000 | FLAT | f32 | 1 | 0 | warm | 267 | 3736μs | 3976μs | 0.0747 | 0.0000 | 10000 | recall_implausible |
| 10000 | FLAT | f32 | 10 | 0 | cold | 266 | 3765μs | 4061μs | 0.5100 | 0.5700 | 10000 | none |
| 10000 | FLAT | f32 | 10 | 0 | warm | 251 | 4040μs | 4294μs | 0.5100 | 0.5700 | 10000 | none |
| 10000 | FLAT | f32 | 50 | 0 | cold | 254 | 3993μs | 4273μs | 0.5100 | 0.5700 | 10000 | none |
| 10000 | FLAT | f32 | 50 | 0 | warm | 265 | 3770μs | 4030μs | 0.5100 | 0.5700 | 10000 | none |
| 10000 | FLAT | f32 | 100 | 0 | cold | 270 | 3722μs | 3986μs | 0.5100 | 0.5700 | 10000 | none |
| 10000 | FLAT | f32 | 100 | 0 | warm | 254 | 3965μs | 4192μs | 0.5100 | 0.5700 | 10000 | none |
| 10000 | HNSW | f32 | 1 | 64 | cold | 1786 | 546μs | 671μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 64 | warm | 1852 | 535μs | 661μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 72 | cold | 1667 | 596μs | 710μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 72 | cold | 1786 | 547μs | 667μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 72 | cold | 1667 | 565μs | 680μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 72 | warm | 1852 | 531μs | 636μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 72 | warm | 1852 | 535μs | 655μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 72 | warm | 1852 | 540μs | 649μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 128 | cold | 1724 | 564μs | 700μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 128 | warm | 1852 | 532μs | 643μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 256 | cold | 1667 | 578μs | 687μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 1 | 256 | warm | 1786 | 545μs | 645μs | 0.0507 | 0.0000 | 16 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 64 | cold | 1724 | 562μs | 704μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 64 | warm | 1852 | 530μs | 632μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 128 | cold | 1667 | 555μs | 755μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 128 | warm | 1852 | 537μs | 653μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 144 | cold | 1667 | 573μs | 692μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 144 | cold | 1724 | 555μs | 681μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 144 | cold | 1724 | 565μs | 676μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 144 | warm | 1852 | 537μs | 643μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 144 | warm | 1852 | 535μs | 645μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 144 | warm | 1786 | 550μs | 665μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 256 | cold | 1667 | 574μs | 710μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 10 | 256 | warm | 1852 | 537μs | 631μs | 0.1059 | 0.2991 | 43 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 64 | cold | 1667 | 566μs | 718μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 64 | warm | 1786 | 550μs | 693μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 128 | cold | 1667 | 564μs | 710μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 128 | warm | 1724 | 570μs | 693μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 256 | cold | 1667 | 589μs | 739μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 256 | warm | 1724 | 564μs | 682μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 464 | cold | 1562 | 620μs | 779μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 464 | cold | 1667 | 577μs | 722μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 464 | cold | 1667 | 579μs | 713μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 464 | warm | 1786 | 552μs | 661μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 464 | warm | 1786 | 547μs | 691μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 50 | 464 | warm | 1724 | 561μs | 695μs | 0.0823 | 0.2660 | 163 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 64 | cold | 1389 | 697μs | 992μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 64 | warm | 1515 | 636μs | 830μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 128 | cold | 1316 | 714μs | 1038μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 128 | warm | 1471 | 648μs | 843μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 256 | cold | 1429 | 665μs | 999μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 256 | warm | 1515 | 645μs | 840μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 864 | cold | 1389 | 695μs | 1028μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 864 | cold | 1351 | 706μs | 968μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 864 | cold | 1389 | 689μs | 1017μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 864 | warm | 1562 | 630μs | 801μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 864 | warm | 1515 | 655μs | 802μs | 0.0691 | 0.3940 | 313 | recall_implausible |
| 10000 | HNSW | f32 | 100 | 864 | warm | 1471 | 658μs | 902μs | 0.0691 | 0.3940 | 313 | recall_implausible |

## 💡 Recommendations

### ⚠️ Critical Issues

Several anomalies were detected in the benchmark results:

- **Low Quality Search Results**: Many configurations show unexpectedly low MAP and recall scores. This suggests potential issues with:
  - Ground truth generation
  - Embedding quality
  - Index implementation

### 🔧 Next Steps

1. **Investigate HNSW Quality Issues**: The low MAP scores suggest fundamental problems with approximate search
2. **Verify Ground Truth**: Ensure semantic similarity calculations are working correctly
3. **Performance Optimization**: Focus on improving HNSW search quality while maintaining speed advantages
4. **Extended Testing**: Test with larger datasets and more diverse queries

---

*Report generated by HiveLLM Vectorizer Benchmark Suite*
