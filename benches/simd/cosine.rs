//! Criterion bench for `crate::simd::cosine_similarity`. Inputs are
//! pre-normalised so the trait contract holds; the bench exercises
//! the same clamped-dot path the production code uses.

use std::hint::black_box;

use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use vectorizer::simd::backend::SimdBackend;
use vectorizer::simd::scalar::ScalarBackend;

#[path = "util.rs"]
mod util;

fn bench_cosine(c: &mut criterion::Criterion) {
    let backend_name = util::dispatched_backend_name();
    let mut group = c.benchmark_group("simd::cosine_similarity");

    for &dim in util::STANDARD_DIMS {
        // Pre-normalised inputs: cosine_similarity assumes unit-norm
        // inputs (see SimdBackend trait docs).
        let a = util::random_unit_vector(0x3333_CCCC ^ dim as u64, dim);
        let b = util::random_unit_vector(0x4444_DDDD ^ dim as u64, dim);

        group.throughput(Throughput::Elements(dim as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("dispatched/{backend_name}"), dim),
            &dim,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(vectorizer::simd::cosine_similarity(
                        black_box(&a),
                        black_box(&b),
                    ))
                });
            },
        );

        let scalar = ScalarBackend;
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| black_box(scalar.cosine_similarity(black_box(&a), black_box(&b))));
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
    targets = bench_cosine
}
criterion_main!(benches);
