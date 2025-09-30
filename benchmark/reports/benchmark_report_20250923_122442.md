# Vectorizer Embedding Benchmark Report

**Generated**: 2025-09-23 12:24:42 UTC

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
| TF-IDF+SVD | Sparse Reduced | 300D | TF-IDF with dimensionality reduction |
| TF-IDF+SVD | Sparse Reduced | 768D | TF-IDF with dimensionality reduction |
| BERT | Dense | 768D | Contextual embeddings (placeholder implementation) |
| MiniLM | Dense | 384D | Efficient sentence embeddings (placeholder implementation) |

### Evaluation Metrics

- **MAP (Mean Average Precision)**: Average precision across all relevant documents
- **MRR (Mean Reciprocal Rank)**: Average of reciprocal ranks of first relevant document
- **Precision@K**: Fraction of relevant documents in top-K results
- **Recall@K**: Fraction of relevant documents retrieved in top-K results

## Results Summary

| Method | MAP | MRR | P@1 | P@3 | P@5 | R@1 | R@3 | R@5 |
|--------|-----|-----|-----|-----|-----|-----|-----|-----|
| TF-IDF | 0.0005 | 0.3420 | 0.2500 | 0.3125 | 0.3125 | 0.0001 | 0.0003 | 0.0003 |
| BM25 | 0.0009 | 0.5000 | 0.5000 | 0.4167 | 0.4271 | 0.0003 | 0.0005 | 0.0006 |
| BERT(768D Placeholder) | 0.0014 | 0.5521 | 0.2500 | 0.4792 | 0.5188 | 0.0002 | 0.0007 | 0.0012 |
| MiniLM(384D Placeholder) | 0.0014 | 0.5087 | 0.3750 | 0.4167 | 0.4000 | 0.0002 | 0.0005 | 0.0011 |

## Detailed Results

### TF-IDF

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0005
- **Mean Reciprocal Rank**: 0.3420

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.2500 |
| 2 | 0.3125 |
| 3 | 0.3125 |
| 4 | 0.3125 |
| 5 | 0.3125 |
| 6 | 0.1667 |
| 7 | 0.1667 |
| 8 | 0.1823 |
| 9 | 0.2153 |
| 10 | 0.2208 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0001 |
| 2 | 0.0003 |
| 3 | 0.0003 |
| 4 | 0.0003 |
| 5 | 0.0003 |
| 6 | 0.0005 |
| 7 | 0.0005 |
| 8 | 0.0006 |
| 9 | 0.0009 |
| 10 | 0.0009 |

### BM25

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0009
- **Mean Reciprocal Rank**: 0.5000

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.5000 |
| 2 | 0.5000 |
| 3 | 0.4167 |
| 4 | 0.4271 |
| 5 | 0.4271 |
| 6 | 0.3958 |
| 7 | 0.4018 |
| 8 | 0.3125 |
| 9 | 0.3160 |
| 10 | 0.2285 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0003 |
| 2 | 0.0004 |
| 3 | 0.0005 |
| 4 | 0.0006 |
| 5 | 0.0006 |
| 6 | 0.0007 |
| 7 | 0.0008 |
| 8 | 0.0010 |
| 9 | 0.0011 |
| 10 | 0.0012 |

### BERT(768D Placeholder)

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0014
- **Mean Reciprocal Rank**: 0.5521

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.2500 |
| 2 | 0.5000 |
| 3 | 0.4792 |
| 4 | 0.5312 |
| 5 | 0.5188 |
| 6 | 0.5479 |
| 7 | 0.5327 |
| 8 | 0.5521 |
| 9 | 0.5625 |
| 10 | 0.4583 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0002 |
| 2 | 0.0006 |
| 3 | 0.0007 |
| 4 | 0.0010 |
| 5 | 0.0012 |
| 6 | 0.0016 |
| 7 | 0.0018 |
| 8 | 0.0021 |
| 9 | 0.0022 |
| 10 | 0.0025 |

### MiniLM(384D Placeholder)

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0014
- **Mean Reciprocal Rank**: 0.5087

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.3750 |
| 2 | 0.3750 |
| 3 | 0.4167 |
| 4 | 0.4062 |
| 5 | 0.4000 |
| 6 | 0.4042 |
| 7 | 0.4042 |
| 8 | 0.4198 |
| 9 | 0.4337 |
| 10 | 0.4309 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0002 |
| 2 | 0.0004 |
| 3 | 0.0005 |
| 4 | 0.0009 |
| 5 | 0.0011 |
| 6 | 0.0015 |
| 7 | 0.0015 |
| 8 | 0.0018 |
| 9 | 0.0022 |
| 10 | 0.0026 |

## Analysis & Insights

### Best Performers

- **Highest MAP**: MiniLM(384D Placeholder) (0.0014)
- **Highest MRR**: BERT(768D Placeholder) (0.5521)

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