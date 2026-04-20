//! Embedding Performance Benchmark
//!
//! Comprehensive performance testing for different embedding approaches:
//! - TF-IDF and BM25 sparse embeddings
//! - BERT and MiniLM dense embeddings
//! - SVD dimensionality reduction
//! - Hybrid search approaches
//!
//! Usage:
//!   cargo bench --bench embeddings_bench

// Removed unused import: std::time::Instant

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use vectorizer::db::{OptimizedHnswConfig, OptimizedHnswIndex};
use vectorizer::embedding::{
    BertEmbedding, Bm25Embedding, EmbeddingProvider, MiniLmEmbedding, SvdEmbedding, TfIdfEmbedding,
};

/// Generate test documents for benchmarking
fn generate_test_documents(count: usize) -> Vec<String> {
    let mut documents = Vec::new();

    for i in 0..count {
        let doc = format!(
            "This is test document number {i} about machine learning, artificial intelligence, and vector databases. \
             It contains various technical terms and concepts that are commonly used in the field of information retrieval \
             and natural language processing. Document {i} discusses topics such as embeddings, similarity search, \
             and semantic understanding of text content."
        );
        documents.push(doc);
    }

    documents
}

/// Generate test queries for benchmarking
fn generate_test_queries() -> Vec<String> {
    vec![
        "machine learning algorithms".to_string(),
        "vector similarity search".to_string(),
        "natural language processing".to_string(),
        "artificial intelligence applications".to_string(),
        "information retrieval systems".to_string(),
        "semantic text understanding".to_string(),
        "embedding models and techniques".to_string(),
        "database performance optimization".to_string(),
    ]
}

/// Benchmark TF-IDF embedding generation
fn bench_tfidf_embedding(c: &mut Criterion) {
    let documents = generate_test_documents(1000);
    let dimension = 128;

    let mut group = c.benchmark_group("tfidf_embedding");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_document", |b| {
        let mut tfidf = TfIdfEmbedding::new(dimension);
        tfidf.build_vocabulary(&documents.iter().map(|s| s.as_str()).collect::<Vec<_>>());

        b.iter(|| black_box(tfidf.embed(black_box(&documents[0])).unwrap()));
    });

    group.bench_function("batch_100", |b| {
        let mut tfidf = TfIdfEmbedding::new(dimension);
        tfidf.build_vocabulary(&documents.iter().map(|s| s.as_str()).collect::<Vec<_>>());

        b.iter(|| {
            for doc in documents.iter().take(100) {
                black_box(tfidf.embed(black_box(doc)).unwrap());
            }
        });
    });

    group.finish();
}

/// Benchmark BM25 embedding generation
fn bench_bm25_embedding(c: &mut Criterion) {
    let documents = generate_test_documents(1000);
    let dimension = 128;

    let mut group = c.benchmark_group("bm25_embedding");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_document", |b| {
        let mut bm25 = Bm25Embedding::new(dimension);
        bm25.build_vocabulary(&documents);

        b.iter(|| black_box(bm25.embed(black_box(&documents[0])).unwrap()));
    });

    group.bench_function("batch_100", |b| {
        let mut bm25 = Bm25Embedding::new(dimension);
        bm25.build_vocabulary(&documents);

        b.iter(|| {
            for doc in documents.iter().take(100) {
                black_box(bm25.embed(black_box(doc)).unwrap());
            }
        });
    });

    group.finish();
}

/// Benchmark BERT embedding generation
fn bench_bert_embedding(c: &mut Criterion) {
    let documents = generate_test_documents(100);
    let dimension = 768;

    let mut group = c.benchmark_group("bert_embedding");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_document", |b| {
        let mut bert = BertEmbedding::new(dimension);
        bert.load_model().unwrap();

        b.iter(|| black_box(bert.embed(black_box(&documents[0])).unwrap()));
    });

    group.bench_function("batch_10", |b| {
        let mut bert = BertEmbedding::new(dimension);
        bert.load_model().unwrap();

        b.iter(|| {
            for doc in documents.iter().take(10) {
                black_box(bert.embed(black_box(doc)).unwrap());
            }
        });
    });

    group.finish();
}

/// Benchmark MiniLM embedding generation
fn bench_minilm_embedding(c: &mut Criterion) {
    let documents = generate_test_documents(100);
    let dimension = 384;

    let mut group = c.benchmark_group("minilm_embedding");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_document", |b| {
        let mut minilm = MiniLmEmbedding::new(dimension);
        minilm.load_model().unwrap();

        b.iter(|| black_box(minilm.embed(black_box(&documents[0])).unwrap()));
    });

    group.bench_function("batch_10", |b| {
        let mut minilm = MiniLmEmbedding::new(dimension);
        minilm.load_model().unwrap();

        b.iter(|| {
            for doc in documents.iter().take(10) {
                black_box(minilm.embed(black_box(doc)).unwrap());
            }
        });
    });

    group.finish();
}

/// Benchmark SVD embedding generation
fn bench_svd_embedding(c: &mut Criterion) {
    let documents = generate_test_documents(500);
    let svd_dimension = 300;
    let vocab_size = 1000;

    let mut group = c.benchmark_group("svd_embedding");
    group.throughput(Throughput::Elements(1));

    group.bench_function("fit_and_embed", |b| {
        b.iter(|| {
            let mut svd = SvdEmbedding::new(svd_dimension, vocab_size);
            let doc_refs: Vec<&str> = documents.iter().map(|s| s.as_str()).collect();
            svd.fit_svd(&doc_refs).unwrap();
            black_box(svd.embed(&documents[0]).unwrap())
        });
    });

    group.finish();
}

/// Benchmark HNSW index building with different embedding methods
fn bench_hnsw_indexing(c: &mut Criterion) {
    let documents = generate_test_documents(1000);
    let dimension = 128;

    let mut group = c.benchmark_group("hnsw_indexing");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("tfidf_indexing", |b| {
        b.iter(|| {
            let mut tfidf = TfIdfEmbedding::new(dimension);
            tfidf.build_vocabulary(&documents.iter().map(|s| s.as_str()).collect::<Vec<_>>());

            let hnsw_config = OptimizedHnswConfig {
                batch_size: 100,
                parallel: true,
                initial_capacity: documents.len(),
                ..Default::default()
            };
            let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();

            let mut batch_vectors = Vec::new();
            for (i, doc) in documents.iter().enumerate() {
                let embedding = tfidf.embed(doc).unwrap();
                batch_vectors.push((format!("doc_{i}"), embedding));

                if batch_vectors.len() >= 100 {
                    index.batch_add(batch_vectors.clone()).unwrap();
                    batch_vectors.clear();
                }
            }
            if !batch_vectors.is_empty() {
                index.batch_add(batch_vectors).unwrap();
            }

            index.optimize().unwrap();
            black_box(index)
        });
    });

    group.bench_function("bm25_indexing", |b| {
        b.iter(|| {
            let mut bm25 = Bm25Embedding::new(dimension);
            bm25.build_vocabulary(&documents);

            let hnsw_config = OptimizedHnswConfig {
                batch_size: 100,
                parallel: true,
                initial_capacity: documents.len(),
                ..Default::default()
            };
            let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();

            let mut batch_vectors = Vec::new();
            for (i, doc) in documents.iter().enumerate() {
                let embedding = bm25.embed(doc).unwrap();
                batch_vectors.push((format!("doc_{i}"), embedding));

                if batch_vectors.len() >= 100 {
                    index.batch_add(batch_vectors.clone()).unwrap();
                    batch_vectors.clear();
                }
            }
            if !batch_vectors.is_empty() {
                index.batch_add(batch_vectors).unwrap();
            }

            index.optimize().unwrap();
            black_box(index)
        });
    });

    group.finish();
}

/// Benchmark search performance with different embedding methods
fn bench_search_performance(c: &mut Criterion) {
    let documents = generate_test_documents(1000);
    let queries = generate_test_queries();
    let dimension = 128;

    // Pre-build indexes for search benchmarks
    let mut tfidf = TfIdfEmbedding::new(dimension);
    tfidf.build_vocabulary(&documents.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let mut bm25 = Bm25Embedding::new(dimension);
    bm25.build_vocabulary(&documents);

    let mut group = c.benchmark_group("search_performance");
    group.throughput(Throughput::Elements(1));

    group.bench_function("tfidf_search", |b| {
        let hnsw_config = OptimizedHnswConfig {
            batch_size: 100,
            parallel: true,
            initial_capacity: documents.len(),
            ..Default::default()
        };
        let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();

        let mut batch_vectors = Vec::new();
        for (i, doc) in documents.iter().enumerate() {
            let embedding = tfidf.embed(doc).unwrap();
            batch_vectors.push((format!("doc_{i}"), embedding));
        }
        index.batch_add(batch_vectors).unwrap();
        index.optimize().unwrap();

        b.iter(|| {
            let query_embedding = tfidf.embed(black_box(&queries[0])).unwrap();
            black_box(index.search(&query_embedding, 10).unwrap())
        });
    });

    group.bench_function("bm25_search", |b| {
        let hnsw_config = OptimizedHnswConfig {
            batch_size: 100,
            parallel: true,
            initial_capacity: documents.len(),
            ..Default::default()
        };
        let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();

        let mut batch_vectors = Vec::new();
        for (i, doc) in documents.iter().enumerate() {
            let embedding = bm25.embed(doc).unwrap();
            batch_vectors.push((format!("doc_{i}"), embedding));
        }
        index.batch_add(batch_vectors).unwrap();
        index.optimize().unwrap();

        b.iter(|| {
            let query_embedding = bm25.embed(black_box(&queries[0])).unwrap();
            black_box(index.search(&query_embedding, 10).unwrap())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tfidf_embedding,
    bench_bm25_embedding,
    bench_bert_embedding,
    bench_minilm_embedding,
    bench_svd_embedding,
    bench_hnsw_indexing,
    bench_search_performance
);

criterion_main!(benches);
