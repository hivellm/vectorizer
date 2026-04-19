//! Numerical-parity oracle: every SIMD-dispatched primitive must
//! agree with the [`vectorizer::simd::scalar::ScalarBackend`] within
//! f32 rounding on a battery of random vectors.
//!
//! If this breaks, either:
//!
//! - A backend introduced a divergence (most likely after a refactor
//!   in `simd::x86::avx2` / `simd::aarch64::neon` / etc.). Fix the
//!   backend; do NOT loosen the tolerance.
//! - The trait contract changed (cosine assumption, normalisation,
//!   etc.). Update [`vectorizer::simd::scalar::ScalarBackend`] AND
//!   every per-ISA backend in lock-step, then re-run this test.
//!
//! The fixed seed keeps the test deterministic across runs.

use vectorizer::simd::backend::SimdBackend;
use vectorizer::simd::scalar::ScalarBackend;
use vectorizer::simd::{cosine_similarity, dot_product, euclidean_distance, l2_norm};

/// Linear congruential generator — deterministic, no `rand` dep
/// needed and reproducible across platforms.
fn lcg(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // Map the top 24 bits to a finite f32 in [-1.0, 1.0].
    let bits = (*state >> 40) as u32;
    let unit = (bits as f32) / ((1u32 << 24) as f32);
    unit * 2.0 - 1.0
}

fn random_vector(seed: u64, len: usize) -> Vec<f32> {
    let mut state = seed;
    (0..len).map(|_| lcg(&mut state)).collect()
}

/// Tolerance derived from accumulated rounding error: `eps * len.sqrt()`
/// is the standard worst-case bound for an `n`-element f32 reduction.
fn tolerance(len: usize) -> f32 {
    let eps = f32::EPSILON;
    eps * (len as f32).sqrt() * 8.0
}

#[test]
fn dot_product_matches_oracle_on_assorted_lengths() {
    let scalar = ScalarBackend;
    // Lengths chosen to exercise: aligned to SIMD lanes (128, 256,
    // 1024), non-aligned (5, 13, 999), and the boundary case (8 = one
    // AVX2 lane).
    for &len in &[5usize, 8, 13, 128, 256, 999, 1024] {
        let a = random_vector(0xCAFE_BABE ^ len as u64, len);
        let b = random_vector(0xDEAD_BEEF ^ len as u64, len);
        let got = dot_product(&a, &b);
        let want = scalar.dot_product(&a, &b);
        let tol = tolerance(len);
        assert!(
            (got - want).abs() <= tol,
            "len={len} got={got} want={want} tol={tol}"
        );
    }
}

#[test]
fn euclidean_distance_matches_oracle() {
    let scalar = ScalarBackend;
    for &len in &[5usize, 8, 128, 1024] {
        let a = random_vector(0x1111_AAAA ^ len as u64, len);
        let b = random_vector(0x2222_BBBB ^ len as u64, len);
        let got = euclidean_distance(&a, &b);
        let want = scalar.euclidean_distance_squared(&a, &b).sqrt();
        let tol = tolerance(len) * 0.5; // sqrt halves rounding error
        assert!(
            (got - want).abs() <= tol,
            "len={len} got={got} want={want} tol={tol}"
        );
    }
}

#[test]
fn cosine_similarity_matches_oracle_on_normalised_inputs() {
    let scalar = ScalarBackend;
    for &len in &[8usize, 64, 384, 1024] {
        let mut a = random_vector(0x3333_CCCC ^ len as u64, len);
        let mut b = random_vector(0x4444_DDDD ^ len as u64, len);
        // Normalise so the cosine assumption holds.
        let norm_a = scalar.l2_norm(&a);
        let norm_b = scalar.l2_norm(&b);
        for x in a.iter_mut() {
            *x /= norm_a;
        }
        for x in b.iter_mut() {
            *x /= norm_b;
        }
        let got = cosine_similarity(&a, &b);
        let want = scalar.cosine_similarity(&a, &b);
        let tol = tolerance(len);
        assert!(
            (got - want).abs() <= tol,
            "len={len} got={got} want={want} tol={tol}"
        );
        // Cosine of two unit vectors is in [-1, 1] — verify the clamp.
        assert!((-1.0..=1.0).contains(&got), "cosine out of range: {got}");
    }
}

#[test]
fn l2_norm_matches_oracle() {
    let scalar = ScalarBackend;
    for &len in &[5usize, 8, 256, 1024] {
        let a = random_vector(0x5555_EEEE ^ len as u64, len);
        let got = l2_norm(&a);
        let want = scalar.l2_norm(&a);
        let tol = tolerance(len) * 0.5;
        assert!(
            (got - want).abs() <= tol,
            "len={len} got={got} want={want} tol={tol}"
        );
    }
}

#[test]
fn dispatch_picks_a_known_backend() {
    let name = vectorizer::simd::selected_backend_name();
    assert!(
        ["avx512", "avx2", "sse2", "neon", "sve", "wasm128", "scalar"].contains(&name),
        "unexpected backend name: {name}"
    );
}
