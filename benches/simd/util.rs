//! Shared helpers for the phase-7g SIMD benchmarks.
//!
//! Provides:
//!
//! - Seeded random-vector generators so every backend sees the same
//!   inputs across runs (and across machines).
//! - The standard set of dimensions every per-primitive bench
//!   sweeps over (covers the common embedding sizes from BERT-base
//!   384 / 768 to GPT-3 12288, plus power-of-two boundaries).
//! - A Criterion configuration helper that pins warmup/measurement
//!   so the comparison numbers across primitives are apples-to-apples.

#![allow(dead_code)] // each bench file picks the helpers it needs

use std::time::Duration;

/// Dimensions every per-primitive bench sweeps. Picks chosen to hit
/// both SIMD-aligned (64, 128, 256, 384, 512, 768, 1024) and
/// non-aligned (1536, 3072) boundaries plus typical embedding sizes.
pub const STANDARD_DIMS: &[usize] = &[64, 128, 256, 384, 512, 768, 1024, 1536, 3072];

/// Linear-congruential PRNG identical to the one in
/// `tests/simd/scalar_oracle.rs` so bench inputs match the test
/// inputs at a given seed.
fn lcg(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let bits = (*state >> 40) as u32;
    let unit = (bits as f32) / ((1u32 << 24) as f32);
    unit * 2.0 - 1.0
}

/// Seeded random vector in `[-1.0, 1.0]`. Same seed → same vector
/// across runs, machines, and backends.
pub fn random_vector(seed: u64, len: usize) -> Vec<f32> {
    let mut state = seed;
    (0..len).map(|_| lcg(&mut state)).collect()
}

/// Seeded random vector pre-normalised to unit L2 norm. Useful for
/// the cosine-similarity bench because the contract assumes
/// pre-normalised inputs.
pub fn random_unit_vector(seed: u64, len: usize) -> Vec<f32> {
    let mut v = random_vector(seed, len);
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        let inv = 1.0 / norm;
        for x in v.iter_mut() {
            *x *= inv;
        }
    }
    v
}

/// Apply the project's standard Criterion settings so every bench
/// produces directly-comparable numbers.
///
/// - Warm-up 3s: long enough for CPUs to ramp clocks but short
///   enough that the bench fits in CI budget.
/// - Measurement 5s: gives ~50k samples on a typical 100-200ns
///   primitive, enough for stable median estimates.
/// - Sample size 100: Criterion's default; explicit here so a
///   future change to the global default doesn't drift bench
///   numbers vs. the committed baseline.
pub fn standard_criterion() -> criterion::Criterion {
    criterion::Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .sample_size(100)
}

/// Returns the dispatched backend's diagnostic name. Useful for
/// labelling Criterion benches so the report shows which path the
/// kernel actually took on the running CPU.
pub fn dispatched_backend_name() -> &'static str {
    vectorizer::simd::selected_backend_name()
}
