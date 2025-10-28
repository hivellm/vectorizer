use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector,
};

fn create_test_store_with_vectors(count: usize) -> (VectorStore, String) {
    let store = VectorStore::new();
    let collection_name = "bench_collection";

    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };

    store.create_collection(collection_name, config).unwrap();

    // Pre-populate with vectors
    let vectors: Vec<Vector> = (0..count)
        .map(|i| {
            let data: Vec<f32> = (0..128).map(|j| ((i + j) as f32) / 128.0).collect();
            Vector::new(format!("vec_{i}"), data)
        })
        .collect();

    store.insert(collection_name, vectors).unwrap();

    (store, collection_name.to_string())
}

fn bench_atomic_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_update");

    for size in [100, 1000, 10000] {
        let (store, collection_name) = create_test_store_with_vectors(size);

        group.bench_with_input(
            BenchmarkId::new("atomic_update", size),
            &size,
            |b, &_size| {
                let mut counter = 0;
                b.iter(|| {
                    let idx = counter % size;
                    let data: Vec<f32> = (0..128).map(|j| ((idx + j + 1) as f32) / 128.0).collect();
                    let vector = Vector::new(format!("vec_{idx}"), data);

                    store.update(&collection_name, black_box(vector)).unwrap();
                    counter += 1;
                });
            },
        );
    }

    group.finish();
}

fn bench_update_single_vector(c: &mut Criterion) {
    let (store, collection_name) = create_test_store_with_vectors(1000);

    c.bench_function("update_single_vector", |b| {
        b.iter(|| {
            let data: Vec<f32> = (0..128).map(|i| (i as f32) / 128.0).collect();
            let vector = Vector::new("vec_500".to_string(), data);
            store.update(&collection_name, black_box(vector)).unwrap();
        });
    });
}

fn bench_update_with_different_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_dimensions");

    for dim in [64, 128, 256, 512, 1024] {
        let store = VectorStore::new();
        let collection_name = "bench_dim";

        let config = CollectionConfig {
            dimension: dim,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };

        store.create_collection(collection_name, config).unwrap();

        // Insert initial vector
        let data: Vec<f32> = (0..dim).map(|i| (i as f32) / dim as f32).collect();
        let vector = Vector::new("vec_0".to_string(), data);
        store.insert(collection_name, vec![vector]).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(dim), &dim, |b, &dimension| {
            b.iter(|| {
                let data: Vec<f32> = (0..dimension)
                    .map(|i| ((i + 1) as f32) / dimension as f32)
                    .collect();
                let vector = Vector::new("vec_0".to_string(), data);
                store.update(collection_name, black_box(vector)).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_atomic_update,
    bench_update_single_vector,
    bench_update_with_different_dimensions
);
criterion_main!(benches);
