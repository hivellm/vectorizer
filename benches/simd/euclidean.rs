//! Criterion bench for `crate::simd::euclidean_distance`. Same
//! template as `dot_product.rs`; see that file for the rationale
//! behind the dimension sweep + Criterion settings.

use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use vectorizer::simd::backend::SimdBackend;
use vectorizer::simd::scalar::ScalarBackend;

#[path = "util.rs"]
mod util;

fn bench_euclidean(c: &mut criterion::Criterion) {
    let backend_name = util::dispatched_backend_name();
    let mut group = c.benchmark_group("simd::euclidean_distance");

    for &dim in util::STANDARD_DIMS {
        let a = util::random_vector(0xAAAA_1111 ^ dim as u64, dim);
        let b = util::random_vector(0xBBBB_2222 ^ dim as u64, dim);

        group.throughput(Throughput::Elements(dim as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("dispatched/{backend_name}"), dim),
            &dim,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(vectorizer::simd::euclidean_distance(
                        black_box(&a),
                        black_box(&b),
                    ))
                });
            },
        );

        let scalar = ScalarBackend;
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| {
                black_box(
                    scalar
                        .euclidean_distance_squared(black_box(&a), black_box(&b))
                        .sqrt(),
                )
            });
        });
    }

    group.finish();
}

fn config() -> criterion::Criterion {
    util::standard_criterion()
}

criterion_group! {
    name = benches;
    config = config();
    targets = bench_euclidean
}
criterion_main!(benches);
