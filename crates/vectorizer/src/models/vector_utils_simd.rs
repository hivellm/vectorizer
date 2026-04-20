//! Compatibility shim — pre-phase7 entry point for SIMD primitives.
//!
//! Historical note: this file used to host the AVX2 implementation
//! directly. After phase7a it routes through [`crate::simd`], which
//! has the runtime dispatch + per-ISA backends. The functions here
//! are kept as thin `#[inline]` wrappers so any external crate or
//! older test that imported `models::vector_utils_simd::*` keeps
//! compiling without a path change.
//!
//! New code should call `crate::simd::dot_product` etc. directly.

/// Sum of pairwise products. Forwards to [`crate::simd::dot_product`].
#[inline]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    crate::simd::dot_product(a, b)
}

/// Euclidean distance (un-squared). Forwards to
/// [`crate::simd::euclidean_distance`].
#[inline]
pub fn euclidean_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    crate::simd::euclidean_distance(a, b)
}

/// Cosine similarity assuming pre-normalised inputs. Forwards to
/// [`crate::simd::cosine_similarity`].
#[inline]
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    crate::simd::cosine_similarity(a, b)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // The actual numerical correctness is pinned by the dispatched
    // backend's own tests + the scalar oracle integration test in
    // tests/simd/scalar_oracle.rs. These tests just make sure the
    // shim's call sites still compile + return the right shape.

    #[test]
    fn shim_dot_product_returns_finite() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let v = dot_product_simd(&a, &b);
        assert!(v.is_finite());
        assert!((v - 120.0).abs() < 1e-5); // 1·8+2·7+...+8·1 = 120
    }

    #[test]
    fn shim_euclidean_distance_handles_3_4_5_triangle() {
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];
        assert!((euclidean_distance_simd(&a, &b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn shim_cosine_similarity_clamps() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((cosine_similarity_simd(&a, &b) - 1.0).abs() < 1e-6);
    }
}
