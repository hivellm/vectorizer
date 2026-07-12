//! Insert pipeline benchmark (phase38 §5)
//!
//! Benchmarks `Collection::insert_batch` as exercised through the public
//! `VectorStore::insert` entry point: create a collection (dimension 256),
//! then insert batches of 100 / 1000 vectors and measure throughput.
//!
//! Usage:
//!   cargo bench --bench insert_pipeline

// Benchmark binary: unwrap is idiomatic for the harness setup, the
// `unwrap_used` / `expect_used` workspace lints apply only to library code.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector,
};

const DIMENSION: usize = 256;

/// Generate deterministic pseudo-random test vectors (no `rand` dependency,
/// matching the pattern already used by `benches/core/core_operations_benchmark.rs`).
fn generate_test_vectors(count: usize, offset: usize) -> Vec<Vector> {
    (0..count)
        .map(|i| {
            let idx = offset + i;
            let data: Vec<f32> = (0..DIMENSION)
                .map(|j| ((idx * 13 + j * 17) % 1000) as f32 / 1000.0)
                .collect();
            Vector::new(format!("vec_{idx}"), data)
        })
        .collect()
}

fn new_test_store() -> (VectorStore, &'static str) {
    let store = VectorStore::new();
    let collection_name = "insert_pipeline_bench";

    let config = CollectionConfig {
        dimension: DIMENSION,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        ..Default::default()
    };

    store.create_collection(collection_name, config).unwrap();
    (store, collection_name)
}

/// Benchmark batch inserts (Collection::insert_batch via VectorStore::insert)
/// at 100 and 1000 vectors per batch.
fn bench_insert_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_pipeline");

    for &batch_size in &[100usize, 1000usize] {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("insert_batch", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let (store, collection_name) = new_test_store();
                    let vectors = generate_test_vectors(batch_size, 0);
                    store.insert(collection_name, black_box(vectors)).unwrap();
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_insert_batch);
criterion_main!(benches);
