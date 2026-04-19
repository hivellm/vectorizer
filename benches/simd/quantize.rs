//! Criterion bench for the phase-7f quantize/dequantize primitives.
//! Sweeps the standard dims so the result table directly answers
//! "how much faster does the SIMD path quantize a 768-dim vector".

use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use vectorizer::simd::backend::SimdBackend;
use vectorizer::simd::scalar::ScalarBackend;

#[path = "util.rs"]
mod util;

fn bench_quantize(c: &mut criterion::Criterion) {
    let backend_name = util::dispatched_backend_name();

    // Quantize: f32 → u8.
    let mut group = c.benchmark_group("simd::quantize_f32_to_u8");
    let scale = 2.0 / 255.0;
    let offset = -1.0;
    for &dim in util::STANDARD_DIMS {
        let src = util::random_vector(0xABCD_0000 ^ dim as u64, dim);
        let mut dst_disp = vec![0u8; dim];
        let mut dst_scal = vec![0u8; dim];

        group.throughput(Throughput::Elements(dim as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("dispatched/{backend_name}"), dim),
            &dim,
            |bencher, _| {
                bencher.iter(|| {
                    vectorizer::simd::quantize_f32_to_u8(
                        black_box(&src),
                        black_box(&mut dst_disp),
                        scale,
                        offset,
                        256,
                    );
                });
            },
        );

        let scalar = ScalarBackend;
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| {
                scalar.quantize_f32_to_u8(
                    black_box(&src),
                    black_box(&mut dst_scal),
                    scale,
                    offset,
                    256,
                );
            });
        });
    }
    group.finish();

    // Dequantize: u8 → f32.
    let mut group = c.benchmark_group("simd::dequantize_u8_to_f32");
    for &dim in util::STANDARD_DIMS {
        let src: Vec<u8> = (0..dim).map(|i| (i % 256) as u8).collect();
        let mut dst_disp = vec![0.0f32; dim];
        let mut dst_scal = vec![0.0f32; dim];

        group.throughput(Throughput::Elements(dim as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("dispatched/{backend_name}"), dim),
            &dim,
            |bencher, _| {
                bencher.iter(|| {
                    vectorizer::simd::dequantize_u8_to_f32(
                        black_box(&src),
                        black_box(&mut dst_disp),
                        scale,
                        offset,
                    );
                });
            },
        );

        let scalar = ScalarBackend;
        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |bencher, _| {
            bencher.iter(|| {
                scalar.dequantize_u8_to_f32(
                    black_box(&src),
                    black_box(&mut dst_scal),
                    scale,
                    offset,
                );
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
    targets = bench_quantize
}
criterion_main!(benches);
