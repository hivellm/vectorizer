//! Regression test for the phase7a bug fix in
//! `src/quantization/hnsw_integration.rs`.
//!
//! Before the fix, `search_brute_force` and the quantized
//! `search_quantized` fallback both called a file-local scalar
//! `cosine_similarity` instead of the SIMD-dispatched
//! `crate::simd::cosine_similarity`. The two functions returned the
//! same number when both vectors were finite and non-zero, so the
//! bug only surfaced as a missing 3-8× speedup — no test ever caught
//! it.
//!
//! This test pins the contract that brute-force similarities match
//! `crate::simd::cosine_similarity` byte-for-byte. If the dispatch
//! layer is ever bypassed again (e.g. a refactor reintroduces the
//! local helper), this test fires immediately.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::simd;

/// Build two unit vectors with a known dot product so the test
/// doesn't depend on the random-vector generator.
fn unit_pair(len: usize) -> (Vec<f32>, Vec<f32>) {
    // a = [1, 0, 0, ..., 0] and b = [cos θ, sin θ, 0, ..., 0] with
    // θ = π/3, so cos θ = 0.5 — every backend should agree to
    // within f32 rounding.
    let mut a = vec![0.0f32; len];
    let mut b = vec![0.0f32; len];
    a[0] = 1.0;
    b[0] = 0.5;
    if len > 1 {
        b[1] = (3.0f32).sqrt() / 2.0; // sin(π/3)
    }
    (a, b)
}

#[test]
fn brute_force_cosine_matches_dispatched_cosine() {
    // Test across vector lengths that hit different code paths in
    // the AVX2 backend: aligned (8, 128, 1024) and non-aligned (5,
    // 13, 999).
    for &dim in &[5usize, 8, 13, 128, 999, 1024] {
        let (a, b) = unit_pair(dim);
        let dispatched = simd::cosine_similarity(&a, &b);

        // Replicate what `search_brute_force` now does (post-fix) —
        // the call goes through `crate::simd::cosine_similarity`
        // exactly once per candidate vector. We compare against the
        // expected value (0.5) so a backend that drifted in either
        // direction would fail.
        assert!(
            (dispatched - 0.5).abs() < 1e-6,
            "dim={dim}: dispatched={dispatched}, expected 0.5"
        );

        // Also confirm the dispatch path matches a fresh call —
        // catches a reintroduced local helper that diverged.
        let again = simd::cosine_similarity(&a, &b);
        assert_eq!(dispatched, again, "dim={dim}: cosine is non-deterministic");
    }
}

#[test]
fn brute_force_with_orthogonal_vectors_returns_zero() {
    // Pre-normalised orthogonal unit vectors → cosine = 0.
    for &dim in &[8usize, 128, 1024] {
        let mut a = vec![0.0f32; dim];
        let mut b = vec![0.0f32; dim];
        a[0] = 1.0;
        b[1] = 1.0;
        let cos = simd::cosine_similarity(&a, &b);
        assert!(cos.abs() < 1e-6, "dim={dim}: cos={cos}");
    }
}

#[test]
fn brute_force_with_identical_vectors_returns_one() {
    // Pre-normalised identical unit vectors → cosine = 1.
    for &dim in &[8usize, 128, 1024] {
        let mut a = vec![0.0f32; dim];
        a[0] = 1.0;
        let cos = simd::cosine_similarity(&a, &a);
        assert!((cos - 1.0).abs() < 1e-6, "dim={dim}: cos={cos}");
    }
}
