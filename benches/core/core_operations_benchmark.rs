//! Core Operations Performance Benchmark
//!
//! Comprehensive performance testing for all vectorizer core operations:
//! - Insert (individual & batch)
//! - Search (various k values)
//! - Update (re-indexing)
//! - Delete (individual & batch)
//!
//! Usage:
//!   cargo bench --bench core_operations_benchmark

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use vectorizer::db::{OptimizedHnswConfig, OptimizedHnswIndex};
use vectorizer::models::DistanceMetric;

/// Generate test vectors for benchmarking
fn generate_test_vectors(num_vectors: usize, dimension: usize) -> Vec<(String, Vec<f32>)> {
    let mut vectors = Vec::new();

    for i in 0..num_vectors {
        let id = format!("vec_{i}");

        // Generate pseudo-random but deterministic vector
        let mut vec = Vec::with_capacity(dimension);
        for j in 0..dimension {
            let val = ((i * 13 + j * 17) % 1000) as f32 / 1000.0;
            vec.push(val);
        }

        vectors.push((id, vec));
    }

    vectors
}

/// Benchmark single insertions
fn bench_insert_single(c: &mut Criterion) {
    let dimension = 512;
    let test_vectors = generate_test_vectors(1000, dimension);

    let mut group = c.benchmark_group("insert_single");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_insert", |b| {
        b.iter(|| {
            let hnsw_config = OptimizedHnswConfig {
                max_connections: 16,
                max_connections_0: 32,
                ef_construction: 200,
                distance_metric: DistanceMetric::Cosine,
                parallel: false,
                initial_capacity: test_vectors.len(),
                batch_size: 1,
                ..Default::default()
            };

            let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();

            for (id, vec) in test_vectors.iter().take(100) {
                let _ = index.add(black_box(id.clone()), black_box(vec.clone()));
            }
        })
    });

    group.finish();
}

/// Benchmark batch insertions
fn bench_insert_batch(c: &mut Criterion) {
    let dimension = 512;
    let test_vectors = generate_test_vectors(1000, dimension);
    let batch_sizes = [10, 50, 100];

    let mut group = c.benchmark_group("insert_batch");

    for &batch_size in &batch_sizes {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("batch_insert", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let hnsw_config = OptimizedHnswConfig {
                        max_connections: 16,
                        max_connections_0: 32,
                        ef_construction: 200,
                        distance_metric: DistanceMetric::Cosine,
                        parallel: true,
                        initial_capacity: test_vectors.len(),
                        batch_size,
                        ..Default::default()
                    };

                    let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();

                    for batch in test_vectors.chunks(batch_size).take(5) {
                        let _ = index.batch_add(black_box(batch.to_vec()));
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark search operations
fn bench_search(c: &mut Criterion) {
    let dimension = 512;
    let test_vectors = generate_test_vectors(1000, dimension);
    let k_values = [1, 10, 100];

    let mut group = c.benchmark_group("search");

    for &k in &k_values {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(BenchmarkId::new("search", k), &k, |b, &k| {
            // Build index once
            let hnsw_config = OptimizedHnswConfig {
                max_connections: 16,
                max_connections_0: 32,
                ef_construction: 200,
                distance_metric: DistanceMetric::Cosine,
                parallel: true,
                initial_capacity: test_vectors.len(),
                batch_size: 1000,
                ..Default::default()
            };

            let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();
            index.batch_add(test_vectors.clone()).unwrap();
            index.optimize().unwrap();

            b.iter(|| {
                for i in 0..100 {
                    let query_idx = i % test_vectors.len();
                    let query_vec = &test_vectors[query_idx].1;
                    let _ = index.search(black_box(query_vec), black_box(k));
                }
            })
        });
    }

    group.finish();
}

/// Benchmark update operations
fn bench_update(c: &mut Criterion) {
    let dimension = 512;
    let test_vectors = generate_test_vectors(1000, dimension);

    let mut group = c.benchmark_group("update");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_update", |b| {
        // Build initial index
        let hnsw_config = OptimizedHnswConfig {
            max_connections: 16,
            max_connections_0: 32,
            ef_construction: 200,
            distance_metric: DistanceMetric::Cosine,
            parallel: true,
            initial_capacity: test_vectors.len(),
            batch_size: 1000,
            ..Default::default()
        };

        let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();
        index.batch_add(test_vectors.clone()).unwrap();

        b.iter(|| {
            for i in 0..100 {
                let idx = i % test_vectors.len();
                let (id, original_vec) = &test_vectors[idx];

                // Create modified vector
                let mut modified_vec = original_vec.clone();
                for v in &mut modified_vec {
                    *v *= 1.01;
                }

                let _ = index.update(black_box(id), black_box(&modified_vec));
            }
        })
    });

    group.finish();
}

/// Benchmark delete operations
fn bench_delete(c: &mut Criterion) {
    let dimension = 512;
    let test_vectors = generate_test_vectors(1000, dimension);

    let mut group = c.benchmark_group("delete");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_delete", |b| {
        // Build initial index
        let hnsw_config = OptimizedHnswConfig {
            max_connections: 16,
            max_connections_0: 32,
            ef_construction: 200,
            distance_metric: DistanceMetric::Cosine,
            parallel: true,
            initial_capacity: test_vectors.len(),
            batch_size: 1000,
            ..Default::default()
        };

        let index = OptimizedHnswIndex::new(dimension, hnsw_config).unwrap();
        index.batch_add(test_vectors.clone()).unwrap();

        b.iter(|| {
            for i in 0..100 {
                let idx = i % test_vectors.len();
                let id = &test_vectors[idx].0;
                let _ = index.remove(black_box(id));
            }
        })
    });

    group.finish();
}

/// Benchmark concurrent mixed operations
fn bench_concurrent_mixed(c: &mut Criterion) {
    let dimension = 512;
    let test_vectors = generate_test_vectors(1000, dimension);

    let mut group = c.benchmark_group("concurrent_mixed");
    group.throughput(Throughput::Elements(100));

    group.bench_function("mixed_operations", |b| {
        use std::sync::Arc;

        // Build initial index
        let hnsw_config = OptimizedHnswConfig {
            max_connections: 16,
            max_connections_0: 32,
            ef_construction: 200,
            distance_metric: DistanceMetric::Cosine,
            parallel: true,
            initial_capacity: test_vectors.len() + 1000,
            batch_size: 1000,
            ..Default::default()
        };

        let index = Arc::new(OptimizedHnswIndex::new(dimension, hnsw_config).unwrap());
        index.batch_add(test_vectors.clone()).unwrap();

        b.iter(|| {
            for i in 0..100 {
                let idx = i % test_vectors.len();
                let operation = i % 10;

                match operation {
                    0..=6 => {
                        // 70% search
                        let _ = index.search(black_box(&test_vectors[idx].1), 10);
                    }
                    7..=8 => {
                        // 20% insert
                        let id = format!("new_{i}");
                        let vec = test_vectors[idx].1.clone();
                        let _ = index.add(black_box(id), black_box(vec));
                    }
                    9 => {
                        // 10% delete
                        if i < test_vectors.len() {
                            let _ = index.remove(black_box(&test_vectors[idx].0));
                        }
                    }
                    _ => {}
                }
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_insert_single,
    bench_insert_batch,
    bench_search,
    bench_update,
    bench_delete,
    bench_concurrent_mixed
);
criterion_main!(benches);
