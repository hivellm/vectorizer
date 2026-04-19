//! AVX2 backend — 256-bit registers, 8 f32 lanes per cycle.
//!
//! Ported from the original `models::vector_utils_simd` file with
//! the same intrinsics and the same horizontal-sum sequence; the
//! only behavioural change is that `cosine_similarity` now matches
//! the trait contract (clamped dot product on pre-normalised inputs)
//! instead of being a one-line wrapper around `dot_product_simd`.
//!
//! ## Safety
//!
//! Every `unsafe fn` carrying `#[target_feature(enable = "avx2")]`
//! requires the caller to have verified AVX2 is available on the
//! running CPU. The `Avx2Backend` impl below performs that check
//! exactly once per call (the `is_x86_feature_detected!` macro is
//! cheap — backed by a CPUID cache); the dispatch layer also checks
//! once at startup, so the runtime cost is amortised to a single
//! branch per primitive.
//!
//! Each AVX2 helper wraps its full body in one `unsafe { ... }`
//! block with a single `// SAFETY:` comment — the safety condition
//! is the same for every intrinsic in the body (AVX2 must be
//! available; the slices must have equal length; `i + SIMD_LANES <=
//! len` by loop construction).

use std::arch::x86_64::*;

use crate::simd::backend::SimdBackend;

const SIMD_LANES: usize = 8;

/// Marker type for the AVX2 backend. All state lives in the
/// processor; no fields needed.
pub struct Avx2Backend;

impl SimdBackend for Avx2Backend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 verified by the runtime detector
            // immediately above; equal-length precondition is debug-
            // asserted on entry.
            unsafe { dot_product_avx2(a, b) }
        } else {
            crate::simd::scalar::ScalarBackend.dot_product(a, b)
        }
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: same as `dot_product` — AVX2 verified above,
            // equal-length precondition debug-asserted on entry.
            unsafe { euclidean_distance_squared_avx2(a, b) }
        } else {
            crate::simd::scalar::ScalarBackend.euclidean_distance_squared(a, b)
        }
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        // Trait contract: clamped dot product on pre-normalised inputs.
        self.dot_product(a, b).clamp(-1.0, 1.0)
    }

    fn l2_norm(&self, a: &[f32]) -> f32 {
        // L2 norm is `sqrt(dot(a, a))`. Re-using the AVX2 dot keeps
        // the SIMD path warm without a separate intrinsic.
        self.dot_product(a, a).sqrt()
    }

    fn name(&self) -> &'static str {
        "avx2"
    }
}

/// # Safety
///
/// Caller must ensure AVX2 is available on the running CPU. `a` and
/// `b` must have equal length; reading past the end of either slice
/// is UB. The public `Avx2Backend::dot_product` enforces both.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: AVX2 is gated by the function's `#[target_feature]`
    // attribute and verified by the caller. All `_mm256_*` intrinsics
    // are pure register operations; the only memory reads are
    // `_mm256_loadu_ps(a.as_ptr().add(i))` / same on `b`, where
    // `i + SIMD_LANES <= simd_len <= len` by loop construction, so
    // the load stays within the slice's allocation.
    unsafe {
        let mut sum = _mm256_setzero_ps();

        // Process 8 floats at a time.
        let mut i = 0;
        while i < simd_len {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));
            let prod = _mm256_mul_ps(va, vb);
            sum = _mm256_add_ps(sum, prod);
            i += SIMD_LANES;
        }

        // Horizontal sum reduces 8 lanes → 1 scalar.
        let mut result = horizontal_sum_avx2(sum);

        // Tail loop for the leftover (len % 8) elements.
        for idx in simd_len..len {
            result += a[idx] * b[idx];
        }

        result
    }
}

/// # Safety
///
/// Same preconditions as [`dot_product_avx2`]. Returns the SQUARED
/// distance — the public wrapper takes `sqrt` only when the caller
/// asks for the un-squared form.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn euclidean_distance_squared_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: same as `dot_product_avx2` — AVX2 gated by
    // `#[target_feature]`, equal-length precondition enforced by the
    // public wrapper, and `i + SIMD_LANES <= simd_len <= len` keeps
    // every load inside the slice.
    unsafe {
        let mut sum_sq = _mm256_setzero_ps();

        let mut i = 0;
        while i < simd_len {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));
            let diff = _mm256_sub_ps(va, vb);
            let sq = _mm256_mul_ps(diff, diff);
            sum_sq = _mm256_add_ps(sum_sq, sq);
            i += SIMD_LANES;
        }

        let mut result = horizontal_sum_avx2(sum_sq);

        for idx in simd_len..len {
            let diff = a[idx] - b[idx];
            result += diff * diff;
        }

        result
    }
}

/// # Safety
///
/// AVX2 must be available. Pure shuffle/add intrinsics — only called
/// from the other AVX2 helpers in this file, which already enforce
/// the precondition.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn horizontal_sum_avx2(v: __m256) -> f32 {
    // SAFETY: AVX2 gated by `#[target_feature]`. All operations are
    // pure register shuffles + adds with no memory access — no
    // allocation invariants involved.
    unsafe {
        // Horizontal add within a 256-bit vector: combine the two halves.
        let hi = _mm256_extractf128_ps(v, 1);
        let lo = _mm256_castps256_ps128(v);
        let sum128 = _mm_add_ps(hi, lo);

        // Horizontal add within the resulting 128-bit vector.
        let shuf = _mm_movehdup_ps(sum128);
        let sums = _mm_add_ps(sum128, shuf);
        let shuf = _mm_movehl_ps(shuf, sums);
        let sums = _mm_add_ss(sums, shuf);

        _mm_cvtss_f32(sums)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn skip_unless_avx2() -> bool {
        if std::is_x86_feature_detected!("avx2") {
            return false;
        }
        eprintln!("avx2 not available on this CPU; skipping AVX2 backend test");
        true
    }

    #[test]
    fn dot_product_matches_scalar_on_aligned_input() {
        if skip_unless_avx2() {
            return;
        }
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let got = Avx2Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5, "got={got} want={want}");
    }

    #[test]
    fn dot_product_handles_tail_loop() {
        if skip_unless_avx2() {
            return;
        }
        // 5 elements → 0 SIMD chunks, 5 tail elements.
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let got = Avx2Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5);
    }

    #[test]
    fn euclidean_squared_matches_scalar() {
        if skip_unless_avx2() {
            return;
        }
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];
        let got = Avx2Backend.euclidean_distance_squared(&a, &b);
        // 9 + 16 + 0 + 0 = 25
        assert!((got - 25.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_clamps_to_unit_interval() {
        if skip_unless_avx2() {
            return;
        }
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        let got = Avx2Backend.cosine_similarity(&a, &b);
        assert!((got - 1.0).abs() < 1e-6);
    }

    #[test]
    fn l2_norm_matches_scalar_sqrt() {
        if skip_unless_avx2() {
            return;
        }
        let a = vec![3.0, 4.0, 0.0, 0.0]; // sqrt(9+16) = 5
        let got = Avx2Backend.l2_norm(&a);
        assert!((got - 5.0).abs() < 1e-5);
    }

    #[test]
    fn large_vector_relative_error_under_1e_minus_4() {
        if skip_unless_avx2() {
            return;
        }
        let a: Vec<f32> = (0..1000).map(|i| i as f32 * 0.1).collect();
        let b: Vec<f32> = (0..1000).map(|i| i as f32 * 0.2).collect();
        let got = Avx2Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let rel = (got - want).abs() / want.abs().max(1e-6);
        assert!(rel < 1e-4, "rel={rel} got={got} want={want}");
    }

    #[test]
    fn name_is_avx2() {
        assert_eq!(Avx2Backend.name(), "avx2");
    }
}
