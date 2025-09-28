# Vectorizer Embedding Benchmark Report

**Generated**: 2025-09-23 12:40:39 UTC

## Dataset Overview

- **Documents**: 3931 total
- **Queries**: 8 test queries
- **Ground Truth**: Manually annotated relevant documents per query

## Benchmark Configuration

### Embedding Methods Tested

| Method | Type | Dimensions | Description |
|--------|------|------------|-------------|
| TF-IDF | Sparse | Variable | Traditional term frequency-inverse document frequency |
| BM25 | Sparse | Variable | Advanced sparse retrieval with k1=1.5, b=0.75 |
| TF-IDF+SVD | Sparse Reduced | 300D/768D | TF-IDF with dimensionality reduction |
| BERT | Dense | 768D | Contextual embeddings (placeholder/real) |
| MiniLM | Dense | 384D | Efficient sentence embeddings (placeholder/real) |
| ONNX Models | Dense | 384D/768D | Optimized inference with INT8 quantization |
| Hybrid Search | Two-stage | Variable | BM25 retrieval + dense re-ranking |

### Evaluation Metrics

- **MAP (Mean Average Precision)**: Average precision across all relevant documents
- **MRR (Mean Reciprocal Rank)**: Average of reciprocal ranks of first relevant document
- **Precision@K**: Fraction of relevant documents in top-K results
- **Recall@K**: Fraction of relevant documents retrieved in top-K results

## Results Summary

| Method | MAP | MRR | P@1 | P@3 | P@5 | R@1 | R@3 | R@5 |
|--------|-----|-----|-----|-----|-----|-----|-----|-----|
| TF-IDF | 0.0003 | 0.2292 | 0.1250 | 0.2292 | 0.1750 | 0.0001 | 0.0002 | 0.0004 |
| BM25 | 0.0006 | 0.3264 | 0.2500 | 0.3125 | 0.2875 | 0.0001 | 0.0004 | 0.0005 |
| TF-IDF+SVD(300D) | 0.0195 | 0.8542 | 0.7500 | 0.8125 | 0.7292 | 0.0028 | 0.0077 | 0.0123 |
| TF-IDF+SVD(768D) | 0.0301 | 1.0000 | 1.0000 | 0.9583 | 0.9250 | 0.0036 | 0.0102 | 0.0166 |
| BERT(768D Placeholder) | 0.0016 | 0.8333 | 0.7500 | 0.7500 | 0.6125 | 0.0005 | 0.0008 | 0.0012 |
| MiniLM(384D Placeholder) | 0.0017 | 0.6951 | 0.6250 | 0.5833 | 0.6021 | 0.0004 | 0.0008 | 0.0013 |
| Hybrid: BM25->BERT | 0.0067 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0.0007 | 0.0021 | 0.0034 |
| Hybrid: BM25->MiniLM | 0.0065 | 1.0000 | 1.0000 | 1.0000 | 0.9750 | 0.0007 | 0.0020 | 0.0033 |

## Detailed Results

### TF-IDF

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0003
- **Mean Reciprocal Rank**: 0.2292

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.1250 |
| 2 | 0.1875 |
| 3 | 0.2292 |
| 4 | 0.2500 |
| 5 | 0.1750 |
| 6 | 0.1750 |
| 7 | 0.1786 |
| 8 | 0.1786 |
| 9 | 0.1577 |
| 10 | 0.1542 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0001 |
| 2 | 0.0001 |
| 3 | 0.0002 |
| 4 | 0.0004 |
| 5 | 0.0004 |
| 6 | 0.0004 |
| 7 | 0.0005 |
| 8 | 0.0005 |
| 9 | 0.0006 |
| 10 | 0.0006 |

### BM25

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0006
- **Mean Reciprocal Rank**: 0.3264

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.2500 |
| 2 | 0.3125 |
| 3 | 0.3125 |
| 4 | 0.3125 |
| 5 | 0.2875 |
| 6 | 0.2875 |
| 7 | 0.2875 |
| 8 | 0.1875 |
| 9 | 0.2014 |
| 10 | 0.2014 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0001 |
| 2 | 0.0003 |
| 3 | 0.0004 |
| 4 | 0.0004 |
| 5 | 0.0005 |
| 6 | 0.0005 |
| 7 | 0.0005 |
| 8 | 0.0007 |
| 9 | 0.0008 |
| 10 | 0.0008 |

### TF-IDF+SVD(300D)

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0195
- **Mean Reciprocal Rank**: 0.8542

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.7500 |
| 2 | 0.8125 |
| 3 | 0.8125 |
| 4 | 0.7604 |
| 5 | 0.7292 |
| 6 | 0.7167 |
| 7 | 0.7256 |
| 8 | 0.7390 |
| 9 | 0.7407 |
| 10 | 0.7207 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0028 |
| 2 | 0.0051 |
| 3 | 0.0077 |
| 4 | 0.0099 |
| 5 | 0.0123 |
| 6 | 0.0151 |
| 7 | 0.0179 |
| 8 | 0.0195 |
| 9 | 0.0207 |
| 10 | 0.0226 |

### TF-IDF+SVD(768D)

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0301
- **Mean Reciprocal Rank**: 1.0000

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 1.0000 |
| 2 | 1.0000 |
| 3 | 0.9583 |
| 4 | 0.9583 |
| 5 | 0.9250 |
| 6 | 0.9375 |
| 7 | 0.9226 |
| 8 | 0.9219 |
| 9 | 0.9306 |
| 10 | 0.9333 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0036 |
| 2 | 0.0067 |
| 3 | 0.0102 |
| 4 | 0.0130 |
| 5 | 0.0166 |
| 6 | 0.0198 |
| 7 | 0.0229 |
| 8 | 0.0264 |
| 9 | 0.0294 |
| 10 | 0.0314 |

### BERT(768D Placeholder)

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0016
- **Mean Reciprocal Rank**: 0.8333

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.7500 |
| 2 | 0.8125 |
| 3 | 0.7500 |
| 4 | 0.7708 |
| 5 | 0.6125 |
| 6 | 0.5479 |
| 7 | 0.5446 |
| 8 | 0.5551 |
| 9 | 0.5675 |
| 10 | 0.4675 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0005 |
| 2 | 0.0006 |
| 3 | 0.0008 |
| 4 | 0.0010 |
| 5 | 0.0012 |
| 6 | 0.0016 |
| 7 | 0.0018 |
| 8 | 0.0021 |
| 9 | 0.0023 |
| 10 | 0.0026 |

### MiniLM(384D Placeholder)

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0017
- **Mean Reciprocal Rank**: 0.6951

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.6250 |
| 2 | 0.6250 |
| 3 | 0.5833 |
| 4 | 0.5833 |
| 5 | 0.6021 |
| 6 | 0.4667 |
| 7 | 0.4786 |
| 8 | 0.4875 |
| 9 | 0.4667 |
| 10 | 0.4944 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0004 |
| 2 | 0.0006 |
| 3 | 0.0008 |
| 4 | 0.0010 |
| 5 | 0.0013 |
| 6 | 0.0016 |
| 7 | 0.0017 |
| 8 | 0.0020 |
| 9 | 0.0025 |
| 10 | 0.0028 |

### Hybrid: BM25->BERT

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0067
- **Mean Reciprocal Rank**: 1.0000

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 1.0000 |
| 2 | 1.0000 |
| 3 | 1.0000 |
| 4 | 1.0000 |
| 5 | 1.0000 |
| 6 | 1.0000 |
| 7 | 1.0000 |
| 8 | 1.0000 |
| 9 | 1.0000 |
| 10 | 1.0000 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0007 |
| 2 | 0.0014 |
| 3 | 0.0021 |
| 4 | 0.0027 |
| 5 | 0.0034 |
| 6 | 0.0041 |
| 7 | 0.0048 |
| 8 | 0.0055 |
| 9 | 0.0062 |
| 10 | 0.0067 |

### Hybrid: BM25->MiniLM

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0065
- **Mean Reciprocal Rank**: 1.0000

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 1.0000 |
| 2 | 1.0000 |
| 3 | 1.0000 |
| 4 | 0.9688 |
| 5 | 0.9750 |
| 6 | 0.9792 |
| 7 | 0.9821 |
| 8 | 0.9844 |
| 9 | 0.9844 |
| 10 | 0.9750 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0007 |
| 2 | 0.0014 |
| 3 | 0.0020 |
| 4 | 0.0026 |
| 5 | 0.0033 |
| 6 | 0.0040 |
| 7 | 0.0047 |
| 8 | 0.0054 |
| 9 | 0.0060 |
| 10 | 0.0067 |

## Analysis & Insights

### Best Performers

- **Highest MAP**: TF-IDF+SVD(768D) (0.0301)
- **Highest MRR**: Hybrid: BM25->MiniLM (1.0000)

### Observations

- **Sparse vs Dense**: Compare TF-IDF/BM25 (efficient) vs BERT/MiniLM (semantic)
- **SVD Impact**: Evaluate dimensionality reduction effects on TF-IDF
- **Hybrid Benefits**: Assess if BM25 + dense re-ranking improves quality
- **Dataset Characteristics**: Small dataset may favor exact matching methods

### Recommendations

1. **For Efficiency**: Use BM25 or TF-IDF+SVD for fast retrieval
2. **For Quality**: Consider hybrid approaches when compute allows
3. **For Scale**: Test with larger, more diverse datasets
4. **Real Models**: Replace placeholders with actual BERT/MiniLM implementations

## Technical Details

### Implementation Notes

- **TF-IDF+SVD**: Pseudo-orthogonal transformation using Gram-Schmidt orthogonalization
- **BERT/MiniLM**: Placeholder implementations using seeded hashing
- **BM25**: Standard parameters (k1=1.5, b=0.75)
- **Sparse Methods**: TF-IDF and BM25 use variable vocabulary sizes
- **Dense Methods**: BERT/MiniLM use fixed dimensions with reproducible embeddings

### Dependencies

- `ndarray`: Linear algebra operations
- `ndarray-linalg`: SVD decomposition
- Custom evaluation framework

---

*Report generated by Vectorizer benchmark suite*