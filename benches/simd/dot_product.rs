//! Criterion bench for `crate::simd::dot_product` across the
//! standard dimensions sweep.
//!
//! Compares the dispatched SIMD backend against the
//! `ScalarBackend` oracle so the report shows the speedup factor
//! the running CPU is delivering. Run with::
//!
//!     cargo bench --bench simd_dot_product
//!
//! The backend label in the bench id (`avx2+fma`, `neon`, ...)
//! comes from `crate::simd::selected_backend_name()`, so the same
//! command on different hosts produces directly comparable JSON
//! baselines that `scripts/simd/check-regression.sh` can diff.

use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use vectorizer::simd::backend::SimdBackend;
use vectorizer::simd::scalar::ScalarBackend;

#[path = "util.rs"]
mod util;

fn bench_dot_product(c: &mut criterion::Criterion) {
    let backend_name = util::dispatched_backend_name();
    let mut group = c.benchmark_group("simd::dot_product");

    for &dim in util::STANDARD_DIMS {
        let a = util::random_vector(0xCAFE_BABE ^ dim as u64, dim);
        let b = util::random_vector(0xDEAD_BEEF ^ dim as u64, dim);

        // Throughput in elements per second so the report shows the
        // backend's per-lane rate directly.
        group.throughput(Throughput::Elements(dim as u64));

        // Dispatched backend (the one selected for this CPU).
        group.bench_with_input(
            BenchmarkId::new(format!("dispatched/{backend_name}"), dim),
            &dim,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(vectorizer::simd::dot_product(black_box(&a), black_box(&b)))
                });
            },
        );

        // Scalar oracle — the speedup-vs-this is the headline metric.
        let scalar = ScalarBackend;
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| black_box(scalar.dot_product(black_box(&a), black_box(&b))));
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
    targets = bench_dot_product
}
criterion_main!(benches);
