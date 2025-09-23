# Vectorizer Benchmark Suite

This directory contains the complete performance testing and evaluation suite for Vectorizer, including automated benchmarks, evaluation reports, and regression tests.

## 📁 Directory Structure

```
benchmark/
├── README.md                    # Complete documentation
├── reports/                     # Automatically generated benchmark reports
│   └── benchmark_report_*.md    # Detailed Markdown reports
├── scripts/                     # Benchmark auxiliary scripts
│   ├── benchmark_embeddings.rs  # Main Rust benchmark script
│   ├── run_all_benchmarks.sh    # Runs all benchmarks
│   ├── compare_reports.py       # Compares reports from different versions
│   └── generate_plots.py        # Generates performance charts
├── datasets/                    # Structured test datasets
│   ├── mini_dataset.json        # Small dataset for quick tests
│   ├── evaluation_queries.json  # Evaluation queries with ground truth
│   └── ai_papers.json          # AI papers dataset for testing
├── results/                     # Structured results (JSON/CSV)
│   ├── latest_metrics.json      # Metrics from latest execution
│   ├── embedding_comparison.json # Detailed embedding comparison
│   └── historical/              # Performance history by version
└── tests/                       # Feature-specific tests
    ├── embedding_tests.rs       # Embedding unit tests
    ├── hybrid_search_tests.rs   # Hybrid search tests
    └── evaluation_tests.rs      # Evaluation metrics tests
```

## 🚀 Running Benchmarks

### Complete Embedding Benchmark

```bash
# Runs complete comparison of 8 embedding methods
# (from vectorizer/ directory)
cargo run --bin benchmark_embeddings

# With real transformer models (downloads ~500MB-2GB models)
cargo run --bin benchmark_embeddings --features real-models

# Results automatically saved to benchmark/reports/benchmark_report_TIMESTAMP.md
```

### Specific Performance Tests

```bash
# Test only sparse methods (fast)
cargo run --bin benchmark_embeddings -- --methods sparse

# Test only dense methods
cargo run --bin benchmark_embeddings -- --methods dense

# Test only hybrid search
cargo run --bin benchmark_embeddings -- --methods hybrid
```

### Stress Benchmarks

```bash
# Test with larger dataset
cargo run --bin benchmark_embeddings -- --dataset large

# Memory test
cargo run --bin benchmark_embeddings -- --memory-profile
```

## 📊 Evaluated Metrics

### Information Retrieval (IR) Metrics
- **MAP (Mean Average Precision)**: Average precision across all relevant documents
- **MRR (Mean Reciprocal Rank)**: Average reciprocal rank of first relevant document
- **Precision@K**: Fraction of relevant documents in top-K results
- **Recall@K**: Fraction of relevant documents retrieved in top-K results

### Performance Metrics
- **Indexing time**: Time to process documents
- **Query time**: Latency per query
- **Memory usage**: Peak memory during execution
- **Throughput**: Queries per second

## 🧮 Tested Embedding Methods

### Sparse Methods
| Method | Dimension | Characteristics |
|--------|-----------|----------------|
| TF-IDF | Variable | Baseline, exact matching |
| BM25 | Variable | Probabilistic ranking k1=1.5, b=0.75 |

### Reduced Methods
| Method | Dimension | Characteristics |
|--------|-----------|----------------|
| TF-IDF+SVD | 300D | Reduction with orthogonal transformation |
| TF-IDF+SVD | 768D | BERT-compatible |

### Dense Methods
| Method | Dimension | Characteristics |
|--------|-----------|----------------|
| BERT | 768D | Contextual embeddings (placeholder) |
| MiniLM | 384D | Efficient embeddings (placeholder) |

### Real Transformer Models (with --features real-models)
| Method | Dimension | Model | Characteristics |
|--------|-----------|-------|----------------|
| MiniLM-Multilingual | 384D | paraphrase-multilingual-MiniLM-L12-v2 | Fast, multilingual, excellent quality/cost |
| DistilUSE-Multilingual | 512D | distiluse-base-multilingual-cased-v2 | Efficient, slightly more accurate than MiniLM |
| MPNet-Multilingual | 768D | paraphrase-multilingual-mpnet-base-v2 | Strong baseline at 768D |
| E5-Small | 384D | multilingual-e5-small | Optimized retriever, prefix required |
| E5-Base | 768D | multilingual-e5-base | High-quality retriever, prefix required |
| GTE-Base | 768D | gte-multilingual-base | Alibaba's retriever, excellent performance |
| LaBSE | 768D | LaBSE | Stable multilingual baseline |

### Hybrid Methods
| Method | Pipeline | Characteristics |
|--------|----------|----------------|
| BM25→BERT | Sparse→Dense | Efficiency + Quality |
| BM25→MiniLM | Sparse→Dense | Optimal balance |
| TF-IDF+SVD→BERT | Reduced→Dense | Memory optimized |

## 📈 Interpreting Results

### Expected Results

**Sparse Methods (TF-IDF/BM25)**:
- Excellent performance on structured datasets
- MAP/MRR close to 1.0 on well-defined test data
- Ideal for keyword-based search

**Dense Methods (BERT/MiniLM)**:
- Current modest results due to placeholder implementation
- With real models: MAP 0.7-0.9 expected
- Superior for semantic queries and paraphrases

**Hybrid Methods**:
- Combine efficiency of sparse with quality of dense methods
- Best production trade-off
- Scalable with proper initial k configuration

### Performance Comparison (Real Data Results)

Based on benchmark with 3,931 chunks from HiveLLM Governance project:

```
Scenario              Best Method        MRR    P@1    Justification
--------------------- ------------------ ------ ------ ---------------
Exact search          BM25               1.000  100%   Perfect precision on keyword matches
FAQ/Documentation     BM25               1.000  100%   Finds exact matches reliably
Technical docs        BM25               1.000  100%   Handles structured content perfectly
Large datasets        BM25               1.000  100%   Scales excellently (3.9k docs tested)
Semantic queries      BM25→BERT          0.844   75%   BM25 retrieves, BERT reranks
Conversational        BM25→BERT          0.845   75%   Hybrid approach for understanding
Production ready      BM25               1.000  100%   Battle-tested, fast, reliable
```

## 🛠️ Benchmark Development

### Adding New Embedding Method

1. Implement method in `src/embedding/mod.rs`
2. Add to method enum in benchmark
3. Register in documentation table above
4. Run benchmark for baseline

### Adding New Metric

1. Implement calculation in `src/evaluation/mod.rs`
2. Add to `EvaluationMetrics` struct
3. Update `generate_markdown_report` function
4. Include in documentation

### Custom Test Dataset

```rust
// Example of custom dataset
let custom_dataset = BenchmarkDataset {
    documents: vec![
        "Your document 1".to_string(),
        "Your document 2".to_string(),
    ],
    queries: vec![
        "your test query".to_string(),
    ],
    ground_truth: vec![
        HashSet::from([0, 1]), // Relevant documents for query 0
    ],
};
```

## 🔍 Regression Analysis

### Comparing Versions

```bash
# Compare results from different executions
python3 benchmark/scripts/compare_reports.py \
    benchmark/reports/benchmark_report_20250101_000000.md \
    benchmark/reports/benchmark_report_20250102_000000.md
```

### Detecting Regressions

- Automatic alerts if MAP drops >5%
- Comparison with historical baseline
- Statistical consistency validation

## 📋 Quality Checklist

### Before Commit
- [ ] All embedding methods tested
- [ ] Metrics calculated correctly
- [ ] Markdown report generated
- [ ] Results saved to `reports/`
- [ ] Documentation updated

### Result Validation
- [ ] MAP/MRR make sense for the dataset
- [ ] Precision@K decreases monotonically
- [ ] Recall@K increases monotonically
- [ ] Comparison consistent with previous executions

## 🎯 Production Recommendations

### When to Use Each Method

#### 🚀 **Use BM25 For:**
- **FAQ Systems**: Perfect P@1 (100%) for exact question matching
- **Technical Documentation**: Handles structured content excellently
- **Log Search**: Precise pattern matching
- **Large Datasets**: Scales to thousands of documents reliably
- **Production Systems**: Battle-tested, fast, and reliable

#### 🤖 **Use BM25→BERT For:**
- **Semantic Search**: When users don't write exactly like documents
- **Conversational Queries**: Understanding intent and context
- **Question Answering**: Finding answers even with paraphrasing
- **Advanced Applications**: Where semantic understanding matters

#### 📊 **Benchmark Results Summary:**
- **BM25**: MRR 1.000, P@1 100% - Production ready for most use cases
- **TF-IDF**: MRR 0.938, P@1 87.5% - Good baseline, fast indexing
- **MiniLM-Multilingual (Real)**: Expected MRR 0.85-0.95 - Fast, excellent quality/cost
- **E5-Small (Real)**: Expected MRR 0.88-0.96 - Optimized for retrieval
- **E5-Base/MPNet (Real)**: Expected MRR 0.90-0.97 - High-quality semantic search
- **BERT/MiniLM (Placeholder)**: MRR 0.435-0.844 - Limited by placeholder implementation

### 💡 Key Insights

1. **BM25 is surprisingly good** for most real-world applications
2. **Hybrid search (BM25 + BERT)** provides best of both worlds
3. **Real model implementations** will significantly improve dense embeddings
4. **Dataset size doesn't hurt BM25** - tested with 3.9k chunks successfully

## 🤝 Contributing

1. Add new tests to `benchmark_embeddings.rs`
2. Document new methods in this README
3. Run complete benchmark before PR
4. Compare with historical baseline
5. Update expected metrics if necessary

---

*Vectorizer benchmark suite - Last update: September 2025*
