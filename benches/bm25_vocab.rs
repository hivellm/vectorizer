//! BM25 vocabulary benchmark (phase38 §5)
//!
//! Benchmarks `Bm25Embedding::build_vocabulary` on a synthetic corpus
//! (1,000 documents x ~50 words) plus `embed()` on a fitted provider.
//!
//! Usage:
//!   cargo bench --bench bm25_vocab

// Benchmark binary: unwrap is idiomatic for the harness setup, the
// `unwrap_used` / `expect_used` workspace lints apply only to library code.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use vectorizer::embedding::{Bm25Embedding, EmbeddingProvider};

const DOC_COUNT: usize = 1000;
const WORDS_PER_DOC: usize = 50;
const DIMENSION: usize = 512;

/// A small, fixed vocabulary used to build deterministic synthetic
/// documents (no `rand` dependency).
const WORD_POOL: &[&str] = &[
    "vector",
    "search",
    "index",
    "embedding",
    "query",
    "database",
    "semantic",
    "similarity",
    "hnsw",
    "quantization",
    "collection",
    "document",
    "token",
    "vocabulary",
    "score",
    "rank",
    "cluster",
    "graph",
    "storage",
    "cache",
    "batch",
    "insert",
    "update",
    "delete",
    "payload",
    "distance",
    "cosine",
    "metric",
    "dimension",
    "model",
];

/// Generate a synthetic corpus of `DOC_COUNT` documents, each with
/// `WORDS_PER_DOC` words drawn deterministically from `WORD_POOL`.
fn generate_corpus() -> Vec<String> {
    (0..DOC_COUNT)
        .map(|doc_idx| {
            (0..WORDS_PER_DOC)
                .map(|word_idx| WORD_POOL[(doc_idx * 7 + word_idx * 3) % WORD_POOL.len()])
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// Benchmark building the BM25 vocabulary from the synthetic corpus.
fn bench_build_vocabulary(c: &mut Criterion) {
    let corpus = generate_corpus();

    c.bench_function("bm25_build_vocabulary", |b| {
        b.iter(|| {
            let mut bm25 = Bm25Embedding::new(DIMENSION);
            bm25.build_vocabulary(black_box(&corpus));
            black_box(bm25)
        })
    });
}

/// Benchmark `embed()` on a provider that has already had its
/// vocabulary fitted once (steady-state query embedding cost).
fn bench_embed_fitted(c: &mut Criterion) {
    let corpus = generate_corpus();
    let mut bm25 = Bm25Embedding::new(DIMENSION);
    bm25.build_vocabulary(&corpus);

    c.bench_function("bm25_embed_fitted", |b| {
        b.iter(|| black_box(bm25.embed(black_box(&corpus[0])).unwrap()));
    });
}

criterion_group!(benches, bench_build_vocabulary, bench_embed_fitted);
criterion_main!(benches);
