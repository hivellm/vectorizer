//! AVX-512 backend — 512-bit registers, 16 f32 lanes per cycle.
//!
//! Available on Ice Lake server, Tiger Lake / Sapphire Rapids,
//! Zen4+, and Intel 12th-gen+ client CPUs (with caveats — see § the
//! downclock note in `docs/architecture/simd.md`). When available
//! and not vetoed by the `VECTORIZER_SIMD_BACKEND` env override,
//! this backend wins the dispatch race over AVX2+FMA.
//!
//! Uses AVX-512F for the basic floating-point ops; AVX-512DQ /
//! AVX-512BW will land in phase7f for the byte-mask quantization
//! paths. The horizontal reduction uses `_mm512_reduce_add_ps` (a
//! library helper that compiles down to a chain of shuffle+add
//! intrinsics) to avoid hand-writing the lane reduction.
//!
//! ## Tail handling
//!
//! For the leftover `len % 16` elements we use `_mm512_mask_loadu_ps`
//! with a load-mask derived from the remaining count instead of a
//! scalar tail loop. The masked load reads zeros for any disabled
//! lane, so the full reduction stays inside the AVX-512 path.
//!
//! ## Build requirements
//!
//! AVX-512 intrinsics are still nightly-only on stable Rust pre-1.89,
//! but the `_mm512_*` family used here has been stabilised. Gated by
//! `cfg(feature = "simd-avx512")`; the compile fails with a clear
//! diagnostic on toolchains that lack the intrinsics.

use std::arch::x86_64::*;

use crate::simd::backend::SimdBackend;

const SIMD_LANES: usize = 16;

/// Marker type for the AVX-512F backend.
pub struct Avx512Backend;

impl SimdBackend for Avx512Backend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx512f") {
            // SAFETY: AVX-512F verified by the runtime detector.
            unsafe { dot_product_avx512(a, b) }
        } else {
            // The dispatcher should never pick this backend without
            // AVX-512F, so this branch is paranoia for the case where
            // a caller constructs Avx512Backend manually for testing
            // on a non-512 CPU.
            crate::simd::scalar::ScalarBackend.dot_product(a, b)
        }
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx512f") {
            // SAFETY: AVX-512F verified above.
            unsafe { euclidean_distance_squared_avx512(a, b) }
        } else {
            crate::simd::scalar::ScalarBackend.euclidean_distance_squared(a, b)
        }
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        self.dot_product(a, b).clamp(-1.0, 1.0)
    }

    fn l2_norm(&self, a: &[f32]) -> f32 {
        self.dot_product(a, a).sqrt()
    }

    fn name(&self) -> &'static str {
        "avx512"
    }
}

/// # Safety
///
/// Caller must ensure AVX-512F is available on the running CPU. The
/// public `Avx512Backend::dot_product` wrapper enforces this. Slices
/// must have equal length; the masked-load tail reads at most
/// `len - simd_len < 16` extra lanes, all gated by the load mask.
#[target_feature(enable = "avx512f")]
#[inline]
unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: AVX-512F gated by `#[target_feature]`. Aligned-load
    // bounds `i + SIMD_LANES <= simd_len <= len` keep every load
    // inside the slice. The tail uses a masked load whose mask only
    // enables the actually-present lanes.
    unsafe {
        let mut sum = _mm512_setzero_ps();

        // FMA-folded body: each iteration is one fused-multiply-add
        // on 16 lanes (vs. AVX2's 8). 2× the lane count plus FMA
        // halves the total instruction count vs. AVX2 without FMA.
        let mut i = 0;
        while i < simd_len {
            let va = _mm512_loadu_ps(a.as_ptr().add(i));
            let vb = _mm512_loadu_ps(b.as_ptr().add(i));
            sum = _mm512_fmadd_ps(va, vb, sum);
            i += SIMD_LANES;
        }

        // Masked tail load — mask bit `j` enables lane `j`.
        let tail = (len - simd_len) as u32;
        if tail > 0 {
            let mask = ((1u32 << tail) - 1) as u16;
            let va = _mm512_maskz_loadu_ps(mask, a.as_ptr().add(simd_len));
            let vb = _mm512_maskz_loadu_ps(mask, b.as_ptr().add(simd_len));
            sum = _mm512_fmadd_ps(va, vb, sum);
        }

        _mm512_reduce_add_ps(sum)
    }
}

/// # Safety
///
/// Same preconditions as [`dot_product_avx512`]. Returns the SQUARED
/// distance.
#[target_feature(enable = "avx512f")]
#[inline]
unsafe fn euclidean_distance_squared_avx512(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: same as `dot_product_avx512`.
    unsafe {
        let mut sum_sq = _mm512_setzero_ps();
        let mut i = 0;
        while i < simd_len {
            let va = _mm512_loadu_ps(a.as_ptr().add(i));
            let vb = _mm512_loadu_ps(b.as_ptr().add(i));
            let diff = _mm512_sub_ps(va, vb);
            sum_sq = _mm512_fmadd_ps(diff, diff, sum_sq);
            i += SIMD_LANES;
        }

        let tail = (len - simd_len) as u32;
        if tail > 0 {
            let mask = ((1u32 << tail) - 1) as u16;
            let va = _mm512_maskz_loadu_ps(mask, a.as_ptr().add(simd_len));
            let vb = _mm512_maskz_loadu_ps(mask, b.as_ptr().add(simd_len));
            let diff = _mm512_sub_ps(va, vb);
            sum_sq = _mm512_fmadd_ps(diff, diff, sum_sq);
        }

        _mm512_reduce_add_ps(sum_sq)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn skip_unless_avx512() -> bool {
        if std::is_x86_feature_detected!("avx512f") {
            return false;
        }
        eprintln!("avx512f not available on this CPU; skipping AVX-512 backend test");
        true
    }

    #[test]
    fn dot_product_aligned_to_lane_count() {
        if skip_unless_avx512() {
            return;
        }
        // 16 elements = 1 SIMD chunk, no tail.
        let a: Vec<f32> = (1..=16).map(|i| i as f32).collect();
        let b: Vec<f32> = (16..=31).map(|i| i as f32).collect();
        let got = Avx512Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-4, "got={got} want={want}");
    }

    #[test]
    fn dot_product_masked_tail() {
        if skip_unless_avx512() {
            return;
        }
        // 17 elements = 1 SIMD chunk + 1 masked-load lane.
        let a: Vec<f32> = (1..=17).map(|i| i as f32).collect();
        let b: Vec<f32> = (17..=33).map(|i| i as f32).collect();
        let got = Avx512Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-3, "got={got} want={want}");
    }

    #[test]
    fn euclidean_squared_345_triangle() {
        if skip_unless_avx512() {
            return;
        }
        let mut a = vec![0.0f32; 16];
        let mut b = vec![0.0f32; 16];
        b[0] = 3.0;
        b[1] = 4.0;
        a.push(0.0);
        b.push(0.0);
        let got = Avx512Backend.euclidean_distance_squared(&a, &b);
        // 9 + 16 = 25
        assert!((got - 25.0).abs() < 1e-4);
    }

    #[test]
    fn name_is_avx512() {
        assert_eq!(Avx512Backend.name(), "avx512");
    }
}
