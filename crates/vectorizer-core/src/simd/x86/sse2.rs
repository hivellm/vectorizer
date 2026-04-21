//! SSE2 backend — 128-bit registers, 4 f32 lanes per cycle.
//!
//! SSE2 is in the x86_64 psABI baseline (every x86_64 CPU since 2003
//! has it), so this backend exists primarily as a safety net for
//! pre-AVX2 cloud base tiers — without it those CPUs would fall all
//! the way to the scalar path. The 4×-narrower vectors mean SSE2
//! gives a ~3-4× speedup over scalar instead of AVX2's ~6-8×, but
//! either is better than a plain loop.

use std::arch::x86_64::*;

use crate::simd::backend::SimdBackend;

const SIMD_LANES: usize = 4;

/// Marker type for the SSE2 backend.
pub struct Sse2Backend;

impl SimdBackend for Sse2Backend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        // SAFETY: SSE2 is guaranteed on every x86_64 target by the
        // psABI; no runtime check needed. Equal-length precondition
        // is debug-asserted above.
        unsafe { dot_product_sse2(a, b) }
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        // SAFETY: same as `dot_product` — SSE2 always available.
        unsafe { euclidean_distance_squared_sse2(a, b) }
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        // Trait contract: clamped dot product on pre-normalised inputs.
        self.dot_product(a, b).clamp(-1.0, 1.0)
    }

    fn l2_norm(&self, a: &[f32]) -> f32 {
        // L2 norm is `sqrt(dot(a, a))`.
        self.dot_product(a, a).sqrt()
    }

    fn name(&self) -> &'static str {
        "sse2"
    }
}

/// # Safety
///
/// SSE2 must be available on the running CPU (x86_64 psABI guarantee).
/// `a` and `b` must have equal length; reading past the end of either
/// slice is UB. The public wrapper enforces both.
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn dot_product_sse2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: SSE2 gated by `#[target_feature]`. Loop bound
    // `i + SIMD_LANES <= simd_len <= len` keeps every load inside
    // the slice's allocation.
    unsafe {
        let mut sum = _mm_setzero_ps();
        let mut i = 0;
        while i < simd_len {
            let va = _mm_loadu_ps(a.as_ptr().add(i));
            let vb = _mm_loadu_ps(b.as_ptr().add(i));
            let prod = _mm_mul_ps(va, vb);
            sum = _mm_add_ps(sum, prod);
            i += SIMD_LANES;
        }

        let mut result = horizontal_sum_sse2(sum);

        // Tail loop for the leftover (len % 4) elements.
        for idx in simd_len..len {
            result += a[idx] * b[idx];
        }
        result
    }
}

/// # Safety
///
/// Same preconditions as [`dot_product_sse2`]. Returns the SQUARED
/// distance.
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn euclidean_distance_squared_sse2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: same as `dot_product_sse2`.
    unsafe {
        let mut sum_sq = _mm_setzero_ps();
        let mut i = 0;
        while i < simd_len {
            let va = _mm_loadu_ps(a.as_ptr().add(i));
            let vb = _mm_loadu_ps(b.as_ptr().add(i));
            let diff = _mm_sub_ps(va, vb);
            let sq = _mm_mul_ps(diff, diff);
            sum_sq = _mm_add_ps(sum_sq, sq);
            i += SIMD_LANES;
        }
        let mut result = horizontal_sum_sse2(sum_sq);
        for idx in simd_len..len {
            let d = a[idx] - b[idx];
            result += d * d;
        }
        result
    }
}

/// # Safety
///
/// SSE2 must be available. Pure shuffle/add intrinsics — only called
/// from the other SSE2 helpers in this file.
#[target_feature(enable = "sse2")]
#[inline]
unsafe fn horizontal_sum_sse2(v: __m128) -> f32 {
    // SAFETY: SSE2 gated by `#[target_feature]`. Pure register
    // shuffles + adds with no memory access.
    unsafe {
        // Reduce 4 lanes → 2 lanes → 1 lane.
        let shuf = _mm_shuffle_ps(v, v, 0b1011_0001); // [b, a, d, c]
        let sums = _mm_add_ps(v, shuf); // [a+b, b+a, c+d, d+c]
        let shuf = _mm_movehl_ps(sums, sums); // [c+d, d+c, c+d, d+c]
        let sums = _mm_add_ss(sums, shuf); // bottom = (a+b)+(c+d)
        _mm_cvtss_f32(sums)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn dot_product_aligned() {
        // 8 elements = 2 SIMD chunks, no tail.
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let got = Sse2Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5, "got={got} want={want}");
    }

    #[test]
    fn dot_product_with_tail() {
        // 5 elements = 1 SIMD chunk + 1 tail element.
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let got = Sse2Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5);
    }

    #[test]
    fn euclidean_squared_simple() {
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];
        // 9 + 16 = 25
        assert!((Sse2Backend.euclidean_distance_squared(&a, &b) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_clamps_to_one() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        assert!((Sse2Backend.cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn l2_norm_345_triangle() {
        let a = vec![3.0, 4.0, 0.0, 0.0]; // sqrt(9+16) = 5
        assert!((Sse2Backend.l2_norm(&a) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn name_is_sse2() {
        assert_eq!(Sse2Backend.name(), "sse2");
    }
}
