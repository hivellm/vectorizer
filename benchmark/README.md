# Vectorizer Benchmarks

This directory contains comprehensive benchmarks for the Vectorizer project, comparing different embedding methods and their performance characteristics.

## Latest Results (2025-09-23)

Tested with 3931 real documents from the gov/ directory:

### Performance Summary

| Method | MAP | MRR | Indexing Speed | Use Case |
|--------|-----|-----|----------------|----------|
| TF-IDF | 0.0006 | 0.3021 | 3.5k docs/sec | Fast but low quality |
| BM25 | 0.0003 | 0.2240 | 3.2k docs/sec | Sparse retrieval baseline |
| TF-IDF+SVD(768D) | **0.0294** | 0.9375 | 650 docs/sec | Best balance |
| Hybrid BM25→BERT | 0.0067 | **1.0000** | 100 queries/sec | Highest quality |

### Key Findings

1. **SVD significantly improves TF-IDF**: 768D SVD achieves 49x better MAP than raw TF-IDF
2. **Hybrid search excels at precision**: Perfect MRR (1.0) for finding the most relevant result
3. **Optimized HNSW delivers**: 3.5k docs/sec indexing with batch operations
4. **Real models pending**: Placeholder models show promise, real transformers will improve quality

## Running Benchmarks

### Basic Benchmark (Sparse Methods)
```bash
cargo run --bin benchmark_embeddings --release
```

### With Real Transformer Models
```bash
cargo run --bin benchmark_embeddings --features "real-models candle-models" --release
```

### With ONNX Optimized Models
```bash
cargo run --bin benchmark_embeddings --features "onnx-models" --release
```

### Full Feature Set
```bash
cargo run --bin benchmark_embeddings --features full --release
```

## Benchmark Scripts

- `scripts/benchmark_embeddings.rs`: Main benchmark comparing all embedding methods
- Reports are saved to `reports/` with timestamps

## Interpreting Results

### Metrics Explained

- **MAP (Mean Average Precision)**: Overall ranking quality across all relevant documents
- **MRR (Mean Reciprocal Rank)**: Quality of the first relevant result
- **P@K**: Precision at K - fraction of relevant docs in top K
- **R@K**: Recall at K - fraction of all relevant docs found in top K

### Choosing the Right Method

1. **For Speed**: Use BM25 or TF-IDF
2. **For Quality**: Use TF-IDF+SVD(768D) or Hybrid Search
3. **For Production**: Consider ONNX models when available
4. **For Research**: Test real transformer models with candle-models feature

## Throughput Benchmarks

Additional throughput benchmarks can be run with:

```bash
cargo bench --bench throughput_benchmark
```

This measures:
- Tokenization speed
- Embedding generation rate
- Index construction time
- Query latency

## Contributing

When adding new embedding methods:
1. Implement the `EmbeddingProvider` trait
2. Add evaluation in `benchmark_embeddings.rs`
3. Update this README with results
4. Consider adding to the throughput benchmark