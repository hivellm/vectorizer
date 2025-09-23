//! Benchmarks for VectorStore operations

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vectorizer::{
    CollectionConfig, Vector, VectorStore,
    models::{DistanceMetric, HnswConfig},
};

fn create_random_vector(id: String, dimension: usize) -> Vector {
    let data: Vec<f32> = (0..dimension).map(|i| (i as f32).sin()).collect();
    Vector::new(id, data)
}

fn benchmark_insert(c: &mut Criterion) {
    let store = VectorStore::new();
    let config = CollectionConfig {
        dimension: 768,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };

    store.create_collection("bench", config).unwrap();

    c.bench_function("insert_single_vector", |b| {
        let mut counter = 0;
        b.iter(|| {
            let vector = create_random_vector(format!("v{}", counter), 768);
            store.insert("bench", vec![black_box(vector)]).unwrap();
            counter += 1;
        });
    });
}

fn benchmark_search(c: &mut Criterion) {
    let store = VectorStore::new();
    let config = CollectionConfig {
        dimension: 768,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };

    store.create_collection("bench", config).unwrap();

    // Insert 1000 vectors
    let vectors: Vec<Vector> = (0..1000)
        .map(|i| create_random_vector(format!("v{}", i), 768))
        .collect();

    store.insert("bench", vectors).unwrap();

    // Benchmark search
    let query = vec![0.5f32; 768];

    c.bench_function("search_top_10", |b| {
        b.iter(|| {
            store.search("bench", black_box(&query), 10).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_insert, benchmark_search);
criterion_main!(benches);
