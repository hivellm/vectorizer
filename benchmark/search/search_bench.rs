//! Search Performance Benchmarks
//!
//! Measures performance characteristics of vector search operations:
//! - HNSW search performance across different dimensions
//! - Query latency under various loads
//! - Search accuracy vs speed trade-offs
//! - Concurrent search performance

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector,
};

fn create_test_collection(store: &VectorStore, name: &str, dimension: usize, vector_count: usize) {
    let config = CollectionConfig {
        dimension,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };

    store.create_collection(name, config).unwrap();

    // Insert vectors in batches
    let batch_size = 100;
    for batch in 0..(vector_count / batch_size) {
        let mut vectors = Vec::new();
        for i in 0..batch_size {
            let idx = batch * batch_size + i;
            let data: Vec<f32> = (0..dimension)
                .map(|j| ((idx + j) as f32) / dimension as f32)
                .collect();
            vectors.push(Vector::new(format!("vec_{idx}"), data));
        }
        store.insert(name, vectors).unwrap();
    }
}

fn bench_search_different_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_dimensions");
    group.measurement_time(std::time::Duration::from_secs(10));

    let store = VectorStore::new();
    let vector_count = 1000;

    for dimension in [64, 128, 256, 512, 768, 1024] {
        let collection_name = format!("search_dim_{dimension}");
        create_test_collection(&store, &collection_name, dimension, vector_count);

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("hnsw_search", dimension),
            &dimension,
            |b, &dim| {
                let query_vector: Vec<f32> = (0..dim).map(|i| (i as f32) / dim as f32).collect();

                b.iter(|| {
                    let results = store.search(&collection_name, &query_vector, 10).unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_search_different_limits(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_limits");
    group.measurement_time(std::time::Duration::from_secs(10));

    let store = VectorStore::new();
    let dimension = 128;
    let vector_count = 10000;
    let collection_name = "search_limits_test";

    create_test_collection(&store, collection_name, dimension, vector_count);

    for limit in [1, 5, 10, 50, 100, 500] {
        group.throughput(Throughput::Elements(limit));
        group.bench_with_input(
            BenchmarkId::new("search_limit", limit),
            &limit,
            |b, &limit| {
                let query_vector: Vec<f32> = (0..dimension)
                    .map(|i| (i as f32) / dimension as f32)
                    .collect();

                b.iter(|| {
                    let results = store
                        .search(collection_name, &query_vector, limit.try_into().unwrap())
                        .unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_search_with_quantization(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_quantization");
    group.measurement_time(std::time::Duration::from_secs(10));

    let dimension = 128;
    let vector_count = 5000;

    for quantization in [
        ("none", QuantizationConfig::None),
        ("sq8", QuantizationConfig::SQ { bits: 8 }),
        ("sq4", QuantizationConfig::SQ { bits: 4 }),
    ] {
        let store = VectorStore::new();
        let collection_name = format!("search_quant_{}", quantization.0);

        let config = CollectionConfig {
            dimension,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: quantization.1,
            compression: Default::default(),
            normalization: None,
        };

        store.create_collection(&collection_name, config).unwrap();

        // Insert vectors
        let batch_size = 100;
        for batch in 0..(vector_count / batch_size) {
            let mut vectors = Vec::new();
            for i in 0..batch_size {
                let idx = batch * batch_size + i;
                let data: Vec<f32> = (0..dimension)
                    .map(|j| ((idx + j) as f32) / dimension as f32)
                    .collect();
                vectors.push(Vector::new(format!("vec_{idx}"), data));
            }
            store.insert(&collection_name, vectors).unwrap();
        }

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("quantized_search", quantization.0),
            &quantization.0,
            |b, &_| {
                let query_vector: Vec<f32> = (0..dimension)
                    .map(|i| (i as f32) / dimension as f32)
                    .collect();

                b.iter(|| {
                    let results = store.search(&collection_name, &query_vector, 10).unwrap();
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_search_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_concurrent");
    group.measurement_time(std::time::Duration::from_secs(15));

    let store = std::sync::Arc::new(VectorStore::new());
    let dimension = 128;
    let vector_count = 10000;
    let collection_name = "concurrent_search_test";

    create_test_collection(&store, collection_name, dimension, vector_count);

    for num_threads in [1, 2, 4, 8] {
        group.throughput(Throughput::Elements(num_threads));
        group.bench_with_input(
            BenchmarkId::new("concurrent_threads", num_threads),
            &num_threads,
            |b, &threads| {
                b.iter(|| {
                    let handles: Vec<_> = (0..threads)
                        .map(|thread_id| {
                            let store = std::sync::Arc::clone(&store);
                            std::thread::spawn(move || {
                                let query_vector: Vec<f32> = (0..dimension)
                                    .map(|i| ((i + thread_id as usize) as f32) / dimension as f32)
                                    .collect();

                                for _ in 0..100 {
                                    let results =
                                        store.search(collection_name, &query_vector, 10).unwrap();
                                    black_box(results);
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_search_different_dimensions,
    bench_search_different_limits,
    bench_search_with_quantization,
    bench_search_concurrent
);

criterion_main!(benches);
