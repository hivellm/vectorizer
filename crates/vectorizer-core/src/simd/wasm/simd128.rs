//! WASM SIMD128 backend — 128-bit `v128` registers, 4 f32 lanes per
//! cycle.
//!
//! Compile-time gated on `target_arch = "wasm32"` AND
//! `target_feature = "simd128"`. Browsers / WASM engines that don't
//! support SIMD128 fail the module instantiation, which is the
//! correct behaviour: a WASM build either has SIMD128 throughout or
//! falls all the way back to scalar at compile time.
//!
//! Build flag: `RUSTFLAGS="-C target-feature=+simd128"` for
//! `cargo build --target wasm32-unknown-unknown`. The repo's
//! `.cargo/config.toml` carries an opt-in snippet for WASM consumers.

use std::arch::wasm32::*;

use crate::simd::backend::SimdBackend;

const SIMD_LANES: usize = 4;

/// Marker type for the WASM SIMD128 backend.
pub struct Wasm128Backend;

impl SimdBackend for Wasm128Backend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        // SAFETY: SIMD128 is a compile-time contract on this target;
        // the module won't load on a runtime that lacks it. Equal-
        // length precondition debug-asserted on entry.
        unsafe { dot_product_simd128(a, b) }
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        // SAFETY: same as `dot_product` — SIMD128 is a compile-time
        // contract.
        unsafe { euclidean_distance_squared_simd128(a, b) }
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
        "wasm128"
    }
}

/// # Safety
///
/// SIMD128 must be enabled at compile time (the `cfg` gate on the
/// module enforces this). `a` and `b` must have equal length;
/// reading past the end of either slice is UB. The public wrapper
/// enforces both.
#[target_feature(enable = "simd128")]
#[inline]
unsafe fn dot_product_simd128(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: SIMD128 gated by `#[target_feature]` + the module's
    // `cfg`. Loop bound `i + SIMD_LANES <= simd_len <= len` keeps
    // every load inside the slice's allocation.
    unsafe {
        let mut sum = f32x4_splat(0.0);
        let mut i = 0;
        while i < simd_len {
            let va = v128_load(a.as_ptr().add(i) as *const v128);
            let vb = v128_load(b.as_ptr().add(i) as *const v128);
            let prod = f32x4_mul(va, vb);
            sum = f32x4_add(sum, prod);
            i += SIMD_LANES;
        }

        let mut result = horizontal_sum_simd128(sum);

        // Tail loop for the leftover (len % 4) elements.
        for idx in simd_len..len {
            result += a[idx] * b[idx];
        }
        result
    }
}

/// # Safety
///
/// Same preconditions as [`dot_product_simd128`]. Returns the
/// SQUARED distance.
#[target_feature(enable = "simd128")]
#[inline]
unsafe fn euclidean_distance_squared_simd128(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: same as `dot_product_simd128`.
    unsafe {
        let mut sum_sq = f32x4_splat(0.0);
        let mut i = 0;
        while i < simd_len {
            let va = v128_load(a.as_ptr().add(i) as *const v128);
            let vb = v128_load(b.as_ptr().add(i) as *const v128);
            let diff = f32x4_sub(va, vb);
            let sq = f32x4_mul(diff, diff);
            sum_sq = f32x4_add(sum_sq, sq);
            i += SIMD_LANES;
        }
        let mut result = horizontal_sum_simd128(sum_sq);
        for idx in simd_len..len {
            let d = a[idx] - b[idx];
            result += d * d;
        }
        result
    }
}

/// # Safety
///
/// SIMD128 must be enabled at compile time. WASM has no single-
/// instruction horizontal reduction, so we extract each lane and
/// scalar-add. The engine vectorises this pattern well in practice.
#[target_feature(enable = "simd128")]
#[inline]
unsafe fn horizontal_sum_simd128(v: v128) -> f32 {
    // SAFETY: SIMD128 gated by `#[target_feature]`. `f32x4_extract_lane`
    // is a pure register operation with no memory access.
    unsafe {
        f32x4_extract_lane::<0>(v)
            + f32x4_extract_lane::<1>(v)
            + f32x4_extract_lane::<2>(v)
            + f32x4_extract_lane::<3>(v)
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
        let got = Wasm128Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5);
    }

    #[test]
    fn dot_product_with_tail() {
        // 5 elements = 1 SIMD chunk + 1 tail element.
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let got = Wasm128Backend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5);
    }

    #[test]
    fn euclidean_squared_345_triangle() {
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];
        // 9 + 16 = 25
        assert!((Wasm128Backend.euclidean_distance_squared(&a, &b) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_clamps_to_one() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0, 0.0];
        assert!((Wasm128Backend.cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn l2_norm_345_triangle() {
        let a = vec![3.0, 4.0, 0.0, 0.0]; // sqrt(9+16) = 5
        assert!((Wasm128Backend.l2_norm(&a) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn name_is_wasm128() {
        assert_eq!(Wasm128Backend.name(), "wasm128");
    }
}
