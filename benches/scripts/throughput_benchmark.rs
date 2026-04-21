//! Performance benchmarks for throughput measurement
//!
//! This benchmark suite measures:
//! - Tokenization throughput (tokens/sec)
//! - Embedding throughput (docs/sec)
//! - Indexing throughput (vectors/sec)
//! - End-to-end pipeline performance

use criterion::{Criterion, Throughput, criterion_group};
use std::time::Duration;
use vectorizer::{
    db::{HnswIndex, OptimizedHnswConfig, OptimizedHnswIndex, VectorStore},
    embedding::{CacheConfig, EmbeddingCache, EmbeddingManager, TfIdfEmbedding},
    models::{CollectionConfig, DistanceMetric, HnswConfig},
    parallel::{ParallelConfig, init_parallel_env},
};

#[cfg(feature = "tokenizers")]
use vectorizer::embedding::{FastTokenizer, FastTokenizerConfig};

#[cfg(feature = "onnx-models")]
use vectorizer::embedding::{OnnxConfig, OnnxEmbedder, OnnxModelType};

/// Generate synthetic documents for benchmarking
fn generate_documents(count: usize, avg_length: usize) -> Vec<String> {
    (0..count)
        .map(|i| {
            let words: Vec<String> = (0..avg_length)
                .map(|j| format!("word{}_{}", i, j))
                .collect();
            words.join(" ")
        })
        .collect()
}

/// Benchmark tokenization throughput
#[cfg(feature = "tokenizers")]
fn bench_tokenization(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenization");
    group.measurement_time(Duration::from_secs(10));

    let docs = generate_documents(1000, 200); // 1000 docs, ~200 words each
    let doc_refs: Vec<&str> = docs.iter().map(|s| s.as_str()).collect();

    // Configure tokenizer
    let config = FastTokenizerConfig {
        max_length: 384,
        batch_size: 128,
        ..Default::default()
    };

    let tokenizer = FastTokenizer::from_pretrained(
        "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
        config,
    )
    .expect("Failed to load tokenizer");

    // Single document tokenization
    group.throughput(Throughput::Elements(1));
    group.bench_function("single", |b| {
        b.iter(|| {
            for doc in &doc_refs[..10] {
                black_box(tokenizer.encode(doc).unwrap());
            }
        })
    });

    // Batch tokenization
    for batch_size in [32, 64, 128, 256] {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let batch = &doc_refs[..size];
                    black_box(tokenizer.encode_batch(batch).unwrap());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark embedding throughput
fn bench_embedding_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("embedding");
    group.measurement_time(Duration::from_secs(10));

    let docs = generate_documents(100, 200);

    // TF-IDF baseline
    {
        let mut manager = EmbeddingManager::new();
        let mut tfidf = TfIdfEmbedding::new(384);
        tfidf.build_vocabulary(&docs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        manager.set_default_provider("tfidf").unwrap();

        group.throughput(Throughput::Elements(10));
        group.bench_function("tfidf", |b| {
            b.iter(|| {
                for doc in &docs[..10] {
                    black_box(manager.embed(doc).unwrap());
                }
            })
        });
    }

    // ONNX model benchmark
    #[cfg(feature = "onnx-models")]
    {
        let config = OnnxConfig {
            model_type: OnnxModelType::MiniLMMultilingual384,
            batch_size: 32,
            use_int8: true,
            ..Default::default()
        };

        if let Ok(embedder) = OnnxEmbedder::new(config) {
            let batch_sizes = [1, 16, 32, 64];

            for &batch_size in &batch_sizes {
                group.throughput(Throughput::Elements(batch_size as u64));
                group.bench_with_input(
                    BenchmarkId::new("onnx_minilm", batch_size),
                    &batch_size,
                    |b, &size| {
                        let batch_docs: Vec<&str> =
                            docs[..size].iter().map(|s| s.as_str()).collect();
                        b.iter(|| {
                            black_box(embedder.embed_batch(&batch_docs).unwrap());
                        })
                    },
                );
            }
        }
    }

    group.finish();
}

/// Benchmark HNSW indexing throughput
fn bench_indexing_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("indexing");
    group.measurement_time(Duration::from_secs(10));

    let dimension = 384;
    let num_vectors = 10_000;

    // Generate random vectors
    let vectors: Vec<(String, Vec<f32>)> = (0..num_vectors)
        .map(|i| {
            let vec: Vec<f32> = (0..dimension)
                .map(|_| rand::random::<f32>() * 2.0 - 1.0)
                .collect();
            (format!("vec_{}", i), vec)
        })
        .collect();

    // Standard HNSW
    {
        let config = HnswConfig {
            m: 16,
            ef_construction: 200,
            ..Default::default()
        };

        group.throughput(Throughput::Elements(1000));
        group.bench_function("hnsw_standard", |b| {
            b.iter_with_setup(
                || {
                    let index = HnswIndex::new(config.clone(), DistanceMetric::Cosine, dimension);
                    (index, vectors[..1000].to_vec())
                },
                |(mut index, vecs)| {
                    for (id, data) in vecs {
                        index.add(&id, &data).unwrap();
                    }
                },
            );
        });
    }

    // Optimized HNSW with batching
    {
        let config = OptimizedHnswConfig {
            batch_size: 100,
            parallel: true,
            ..Default::default()
        };

        for &batch_size in &[100, 500, 1000] {
            group.throughput(Throughput::Elements(batch_size as u64));
            group.bench_with_input(
                BenchmarkId::new("hnsw_optimized", batch_size),
                &batch_size,
                |b, &size| {
                    b.iter_with_setup(
                        || {
                            let index = OptimizedHnswIndex::new(dimension, config.clone()).unwrap();
                            (index, vectors[..size].to_vec())
                        },
                        |(index, vecs)| {
                            index.batch_add(vecs).unwrap();
                        },
                    );
                },
            );
        }
    }

    group.finish();
}

/// Benchmark end-to-end pipeline
fn bench_pipeline_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline");
    group.measurement_time(Duration::from_secs(20));

    // Initialize parallel environment
    let parallel_config = ParallelConfig::default();
    init_parallel_env(&parallel_config).unwrap();

    // Generate documents
    let docs = generate_documents(1000, 200);

    // Configure components
    let cache_config = CacheConfig::default();
    let cache = EmbeddingCache::new(cache_config).unwrap();

    let store = VectorStore::new();
    store
        .create_collection(
            "bench_collection",
            CollectionConfig {
                dimension: 384,
                metric: DistanceMetric::Cosine,
                hnsw_config: HnswConfig::default(),
                quantization: None,
                compression: Default::default(),
            },
        )
        .unwrap();

    // Measure end-to-end throughput
    group.throughput(Throughput::Elements(100));
    group.bench_function("full_pipeline", |b| {
        b.iter(|| {
            // Process 100 documents through the full pipeline
            for doc in &docs[..100] {
                // Check cache
                if let Some(embedding) = cache.get(doc) {
                    // Insert to store
                    store
                        .insert(
                            "bench_collection",
                            vec![models::Vector {
                                id: format!("doc_{}", doc.len()),
                                data: embedding,
                                payload: None,
                            }],
                        )
                        .unwrap();
                } else {
                    // Compute embedding (using simple hash for benchmark)
                    let embedding: Vec<f32> =
                        (0..384).map(|i| ((doc.len() + i) as f32).sin()).collect();

                    // Cache it
                    cache.put(doc, &embedding).unwrap();

                    // Insert to store
                    store
                        .insert(
                            "bench_collection",
                            vec![models::Vector {
                                id: format!("doc_{}", doc.len()),
                                data: embedding,
                                payload: None,
                            }],
                        )
                        .unwrap();
                }
            }
        });
    });

    group.finish();
}

/// Benchmark search performance
fn bench_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");

    let dimension = 384;
    let num_vectors = 10_000;

    // Setup index with vectors
    let index = OptimizedHnswIndex::new(dimension, OptimizedHnswConfig::default()).unwrap();

    let vectors: Vec<(String, Vec<f32>)> = (0..num_vectors)
        .map(|i| {
            let vec: Vec<f32> = (0..dimension)
                .map(|_| rand::random::<f32>() * 2.0 - 1.0)
                .collect();
            (format!("vec_{}", i), vec)
        })
        .collect();

    index.batch_add(vectors.clone()).unwrap();
    index.optimize().unwrap();

    // Generate query vectors
    let queries: Vec<Vec<f32>> = (0..100)
        .map(|_| {
            (0..dimension)
                .map(|_| rand::random::<f32>() * 2.0 - 1.0)
                .collect()
        })
        .collect();

    // Benchmark different k values
    for &k in &[1, 10, 50, 100] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("k", k), &k, |b, &k| {
            let query = &queries[0];
            b.iter(|| {
                index.search(query, k).unwrap();
            });
        });
    }

    group.finish();
}

// Add rand dependency for benchmarks
use criterion::{BenchmarkId, black_box};
use vectorizer::models;

criterion_group!(
    benches,
    bench_embedding_throughput,
    bench_indexing_throughput,
    bench_pipeline_throughput,
    bench_search_performance
);

#[cfg(feature = "tokenizers")]
criterion_group!(tokenizer_benches, bench_tokenization);

#[cfg(not(feature = "tokenizers"))]
fn main() {
    benches();
}

#[cfg(feature = "tokenizers")]
fn main() {
    benches();
    tokenizer_benches();
}
