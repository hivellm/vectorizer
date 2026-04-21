//! Criterion bench for `crate::simd::l2_norm`. Single-vector
//! reduction; no second input.

use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use vectorizer::simd::backend::SimdBackend;
use vectorizer::simd::scalar::ScalarBackend;

#[path = "util.rs"]
mod util;

fn bench_l2_norm(c: &mut criterion::Criterion) {
    let backend_name = util::dispatched_backend_name();
    let mut group = c.benchmark_group("simd::l2_norm");

    for &dim in util::STANDARD_DIMS {
        let a = util::random_vector(0x5555_EEEE ^ dim as u64, dim);

        group.throughput(Throughput::Elements(dim as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("dispatched/{backend_name}"), dim),
            &dim,
            |bencher, _| {
                bencher.iter(|| black_box(vectorizer::simd::l2_norm(black_box(&a))));
            },
        );

        let scalar = ScalarBackend;
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| black_box(scalar.l2_norm(black_box(&a))));
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
    targets = bench_l2_norm
}
criterion_main!(benches);
