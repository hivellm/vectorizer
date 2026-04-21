//! Criterion bench for `crate::simd::manhattan_distance`. Phase 7e
//! primitive; AVX2 and NEON backends ship explicit overrides
//! (sign-bit-mask trick on x86, `vabsq_f32` on aarch64) that this
//! bench compares against the scalar `.abs()` loop.

use std::hint::black_box;

use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use vectorizer::simd::backend::SimdBackend;
use vectorizer::simd::scalar::ScalarBackend;

#[path = "util.rs"]
mod util;

fn bench_manhattan(c: &mut criterion::Criterion) {
    let backend_name = util::dispatched_backend_name();
    let mut group = c.benchmark_group("simd::manhattan_distance");

    for &dim in util::STANDARD_DIMS {
        let a = util::random_vector(0x1234_5678 ^ dim as u64, dim);
        let b = util::random_vector(0x8765_4321 ^ dim as u64, dim);

        group.throughput(Throughput::Elements(dim as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("dispatched/{backend_name}"), dim),
            &dim,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(vectorizer::simd::manhattan_distance(
                        black_box(&a),
                        black_box(&b),
                    ))
                });
            },
        );

        let scalar = ScalarBackend;
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| black_box(scalar.manhattan_distance(black_box(&a), black_box(&b))));
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
    targets = bench_manhattan
}
criterion_main!(benches);
