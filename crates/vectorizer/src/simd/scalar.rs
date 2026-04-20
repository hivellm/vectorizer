//! Scalar fallback backend.
//!
//! Always available, on every target. Doubles as the **correctness
//! oracle**: integration tests in `tests/simd/scalar_oracle.rs` check
//! that the dispatched backend produces values within f32 rounding of
//! these straight-loop implementations. If you change a primitive
//! here, mirror the change everywhere — and if you can't (because the
//! ISA can't express it precisely), re-derive the tolerance and
//! document the divergence.

use super::backend::SimdBackend;

/// Plain-loop f32 implementations. No intrinsics, no parallelism.
/// Compiled into every binary even when SIMD is disabled.
pub struct ScalarBackend;

impl SimdBackend for ScalarBackend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let d = x - y;
                d * d
            })
            .sum()
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        // Pre-normalised assumption per trait docs — same shape as
        // the SIMD backends, so the oracle test compares like-for-like.
        self.dot_product(a, b).clamp(-1.0, 1.0)
    }

    fn l2_norm(&self, a: &[f32]) -> f32 {
        a.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    fn name(&self) -> &'static str {
        "scalar"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn dot_product_simple() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        // 1·4 + 2·5 + 3·6 = 4 + 10 + 18 = 32
        assert!((ScalarBackend.dot_product(&a, &b) - 32.0).abs() < 1e-6);
    }

    #[test]
    fn euclidean_squared_simple() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        // |b - a|² = 9 + 16 + 0 = 25
        assert!((ScalarBackend.euclidean_distance_squared(&a, &b) - 25.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_clamps_to_unit_interval() {
        // Normalised parallel vectors: dot product = 1.0 exactly.
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((ScalarBackend.cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_clamps_above_one() {
        // Dot product > 1 (caller violated pre-normalisation
        // contract); the clamp keeps the result in [-1, 1].
        let a = vec![2.0, 0.0];
        let b = vec![2.0, 0.0];
        assert!((ScalarBackend.cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn l2_norm_simple() {
        let a = vec![3.0, 4.0]; // 3² + 4² = 25 → sqrt = 5
        assert!((ScalarBackend.l2_norm(&a) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn name_is_scalar() {
        assert_eq!(ScalarBackend.name(), "scalar");
    }
}
