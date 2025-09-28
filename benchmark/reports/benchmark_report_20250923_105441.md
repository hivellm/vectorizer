# Vectorizer Embedding Benchmark Report

**Generated**: 2025-09-23 10:54:41 UTC

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
| TF-IDF | 0.0067 | 0.9375 | 0.8750 | 0.9583 | 0.9750 | 0.0006 | 0.0020 | 0.0034 |
| BM25 | 0.0067 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0.0007 | 0.0021 | 0.0034 |
| BERT(768D) | 0.0019 | 0.8438 | 0.7500 | 0.7917 | 0.7875 | 0.0006 | 0.0012 | 0.0016 |
| MiniLM(384D) | 0.0015 | 0.4345 | 0.2500 | 0.4167 | 0.4292 | 0.0001 | 0.0007 | 0.0012 |

## Detailed Results

### TF-IDF

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0067
- **Mean Reciprocal Rank**: 0.9375

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.8750 |
| 2 | 0.9375 |
| 3 | 0.9583 |
| 4 | 0.9688 |
| 5 | 0.9750 |
| 6 | 0.9792 |
| 7 | 0.9821 |
| 8 | 0.9844 |
| 9 | 0.9861 |
| 10 | 0.9875 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0006 |
| 2 | 0.0013 |
| 3 | 0.0020 |
| 4 | 0.0027 |
| 5 | 0.0034 |
| 6 | 0.0040 |
| 7 | 0.0047 |
| 8 | 0.0054 |
| 9 | 0.0061 |
| 10 | 0.0068 |

### BM25

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
| 7 | 0.9821 |
| 8 | 0.9844 |
| 9 | 0.9861 |
| 10 | 0.9861 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0007 |
| 2 | 0.0014 |
| 3 | 0.0021 |
| 4 | 0.0027 |
| 5 | 0.0034 |
| 6 | 0.0040 |
| 7 | 0.0047 |
| 8 | 0.0054 |
| 9 | 0.0061 |
| 10 | 0.0067 |

### BERT(768D)

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0019
- **Mean Reciprocal Rank**: 0.8438

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.7500 |
| 2 | 0.8125 |
| 3 | 0.7917 |
| 4 | 0.8021 |
| 5 | 0.7875 |
| 6 | 0.8083 |
| 7 | 0.7589 |
| 8 | 0.7701 |
| 9 | 0.7788 |
| 10 | 0.6812 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0006 |
| 2 | 0.0010 |
| 3 | 0.0012 |
| 4 | 0.0014 |
| 5 | 0.0016 |
| 6 | 0.0017 |
| 7 | 0.0020 |
| 8 | 0.0022 |
| 9 | 0.0023 |
| 10 | 0.0025 |

### MiniLM(384D)

- **Queries Evaluated**: 8
- **Mean Average Precision**: 0.0015
- **Mean Reciprocal Rank**: 0.4345

#### Precision@K

| K | Precision |
|---|-----------|
| 1 | 0.2500 |
| 2 | 0.2500 |
| 3 | 0.4167 |
| 4 | 0.4062 |
| 5 | 0.4292 |
| 6 | 0.4333 |
| 7 | 0.4631 |
| 8 | 0.4435 |
| 9 | 0.4395 |
| 10 | 0.4534 |

#### Recall@K

| K | Recall |
|---|--------|
| 1 | 0.0001 |
| 2 | 0.0003 |
| 3 | 0.0007 |
| 4 | 0.0009 |
| 5 | 0.0012 |
| 6 | 0.0015 |
| 7 | 0.0018 |
| 8 | 0.0020 |
| 9 | 0.0023 |
| 10 | 0.0025 |

## Analysis & Insights

### Best Performers

- **Highest MAP**: BM25 (0.0067)
- **Highest MRR**: BM25 (1.0000)

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