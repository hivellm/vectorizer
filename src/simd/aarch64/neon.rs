//! NEON backend — 128-bit registers, 4 f32 lanes per cycle.
//!
//! NEON is in the aarch64 psABI (every aarch64 CPU has it), so this
//! backend has no runtime gate beyond `cfg(target_arch = "aarch64")`.
//! The lane count matches SSE2 on x86_64, but NEON ships with two
//! advantages over SSE2 that make it the more pleasant kernel:
//!
//! - `vfmaq_f32` is the fused multiply-add primitive, available on
//!   every aarch64 CPU (no separate FMA feature gate). We use it
//!   unconditionally for `dot_product` and `euclidean_distance_squared`.
//! - `vaddvq_f32` reduces a 4-lane register to a scalar in ONE
//!   instruction — no horizontal shuffle chain like SSE2 needs.
//!
//! Apple Silicon (M1/M2/M3/M4), AWS Graviton (all generations),
//! Ampere Altra/A1, Azure Cobalt, and every aarch64 mobile chip lands
//! here. SVE-capable CPUs (Graviton3+, Neoverse V1+) skip past this
//! backend at dispatch time.

use std::arch::aarch64::*;

use crate::simd::backend::SimdBackend;

const SIMD_LANES: usize = 4;

/// Marker type for the NEON backend.
pub struct NeonBackend;

impl SimdBackend for NeonBackend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        // SAFETY: NEON is part of the aarch64 psABI baseline; no
        // runtime check needed. Equal-length precondition is debug-
        // asserted above.
        unsafe { dot_product_neon(a, b) }
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        // SAFETY: same as `dot_product` — NEON always available.
        unsafe { euclidean_distance_squared_neon(a, b) }
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
        "neon"
    }
}

/// # Safety
///
/// NEON is guaranteed on every aarch64 target (psABI baseline). `a`
/// and `b` must have equal length; reading past the end of either
/// slice is UB. The public wrapper enforces the length precondition.
#[target_feature(enable = "neon")]
#[inline]
unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: NEON gated by `#[target_feature]`. Loop bound
    // `i + SIMD_LANES <= simd_len <= len` keeps every load inside
    // the slice's allocation. `vaddvq_f32` is a pure register
    // reduction with no memory access.
    unsafe {
        let mut sum = vdupq_n_f32(0.0);
        let mut i = 0;
        while i < simd_len {
            let va = vld1q_f32(a.as_ptr().add(i));
            let vb = vld1q_f32(b.as_ptr().add(i));
            // Fused multiply-add: `sum = va * vb + sum` in one
            // instruction with one rounding step.
            sum = vfmaq_f32(sum, va, vb);
            i += SIMD_LANES;
        }

        // Single-instruction horizontal reduction.
        let mut result = vaddvq_f32(sum);

        // Tail loop for the leftover (len % 4) elements.
        for idx in simd_len..len {
            result += a[idx] * b[idx];
        }
        result
    }
}

/// # Safety
///
/// Same preconditions as [`dot_product_neon`]. Returns the SQUARED
/// distance.
#[target_feature(enable = "neon")]
#[inline]
unsafe fn euclidean_distance_squared_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: same as `dot_product_neon`.
    unsafe {
        let mut sum_sq = vdupq_n_f32(0.0);
        let mut i = 0;
        while i < simd_len {
            let va = vld1q_f32(a.as_ptr().add(i));
            let vb = vld1q_f32(b.as_ptr().add(i));
            let diff = vsubq_f32(va, vb);
            // `sum_sq = diff * diff + sum_sq`.
            sum_sq = vfmaq_f32(sum_sq, diff, diff);
            i += SIMD_LANES;
        }
        let mut result = vaddvq_f32(sum_sq);
        for idx in simd_len..len {
            let d = a[idx] - b[idx];
            result += d * d;
        }
        result
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn dot_product_aligned() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let got = NeonBackend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5, "got={got} want={want}");
    }

    #[test]
    fn dot_product_with_tail() {
        // 5 elements = 1 SIMD chunk + 1 tail element.
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let got = NeonBackend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5);
    }

    #[test]
    fn euclidean_squared_345_triangle() {
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];
        // 9 + 16 = 25
        assert!((NeonBackend.euclidean_distance_squared(&a, &b) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_clamps_to_one() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        assert!((NeonBackend.cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn l2_norm_345_triangle() {
        let a = vec![3.0, 4.0, 0.0, 0.0]; // sqrt(9+16) = 5
        assert!((NeonBackend.l2_norm(&a) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn name_is_neon() {
        assert_eq!(NeonBackend.name(), "neon");
    }
}
