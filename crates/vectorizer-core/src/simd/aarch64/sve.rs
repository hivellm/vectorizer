//! SVE backend — vector-length-agnostic (VLA) f32 kernels.
//!
//! The Scalable Vector Extension lets the same compiled binary run
//! on 128-bit, 256-bit, or 512-bit implementations: the CPU decides
//! the vector length, and the kernel queries it at entry via
//! `svcntw()`. Graviton3+ ships at 256 bits, Neoverse V1+ at 256,
//! Fujitsu A64FX at 512 — writing the kernel once buys all of them.
//!
//! The other SVE win is **predication**: every load + arithmetic op
//! takes a per-lane mask, so the standard "load, compute, reduce,
//! tail loop" pattern collapses into a single loop driven by
//! `svwhilelt_b32`, which produces a predicate enabling exactly the
//! lanes that fit before `len`. No scalar tail loop, no boundary
//! gymnastics.
//!
//! Apple Silicon (M-series) does NOT implement SVE; M1/M2/M3/M4 are
//! NEON-only. The dispatcher correctly falls back to
//! [`super::neon::NeonBackend`] on those CPUs. Documented in
//! `docs/architecture/simd.md` § "Apple Silicon caveat".

use std::arch::aarch64::*;

use crate::simd::backend::SimdBackend;

/// Marker type for the SVE backend.
pub struct SveBackend;

impl SimdBackend for SveBackend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::arch::is_aarch64_feature_detected!("sve") {
            // SAFETY: SVE verified by the runtime detector
            // immediately above; equal-length precondition is debug-
            // asserted on entry.
            unsafe { dot_product_sve(a, b) }
        } else {
            // Defensive fallback for callers that construct
            // `SveBackend` directly on a non-SVE CPU. The dispatcher
            // never picks this backend in that case.
            crate::simd::scalar::ScalarBackend.dot_product(a, b)
        }
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::arch::is_aarch64_feature_detected!("sve") {
            // SAFETY: SVE verified above.
            unsafe { euclidean_distance_squared_sve(a, b) }
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
        "sve"
    }
}

/// # Safety
///
/// Caller must ensure SVE is available on the running CPU. The
/// public `SveBackend::dot_product` enforces this. The predicated
/// load reads at most `svcntw()` lanes per iteration and the
/// `svwhilelt_b32` predicate disables out-of-bounds lanes, so even
/// the final partial-vector iteration stays inside the slice.
#[target_feature(enable = "sve")]
#[inline]
unsafe fn dot_product_sve(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();

    // SAFETY: SVE gated by `#[target_feature]`. `svwhilelt_b32(i, len)`
    // returns a predicate whose lane `j` is true iff `i + j < len`,
    // so `svld1_f32(pred, ptr+i)` reads exactly the in-bounds lanes
    // — predicated loads ignore disabled lanes (they don't fault
    // and they don't observe the value, see ARM ARM § B1.2). The
    // scaled increment `svcntw()` returns the f32-lane count of the
    // running CPU's vector length.
    unsafe {
        let mut acc = svdup_n_f32(0.0);
        let mut i: u64 = 0;
        let len_u = len as u64;
        let lanes = svcntw();

        while i < len_u {
            let pred = svwhilelt_b32(i as u32, len_u as u32);
            let va = svld1_f32(pred, a.as_ptr().add(i as usize));
            let vb = svld1_f32(pred, b.as_ptr().add(i as usize));
            // Predicated multiply-accumulate: `acc = a * b + acc`
            // for enabled lanes, unchanged for disabled lanes.
            acc = svmla_f32_x(pred, acc, va, vb);
            i += lanes;
        }

        // Reduce all lanes (predicated true) to a single scalar.
        svaddv_f32(svptrue_b32(), acc)
    }
}

/// # Safety
///
/// Same preconditions as [`dot_product_sve`]. Returns the SQUARED
/// distance.
#[target_feature(enable = "sve")]
#[inline]
unsafe fn euclidean_distance_squared_sve(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();

    // SAFETY: same as `dot_product_sve` — SVE gated by
    // `#[target_feature]`, predicated loads stay in-bounds.
    unsafe {
        let mut acc = svdup_n_f32(0.0);
        let mut i: u64 = 0;
        let len_u = len as u64;
        let lanes = svcntw();

        while i < len_u {
            let pred = svwhilelt_b32(i as u32, len_u as u32);
            let va = svld1_f32(pred, a.as_ptr().add(i as usize));
            let vb = svld1_f32(pred, b.as_ptr().add(i as usize));
            let diff = svsub_f32_x(pred, va, vb);
            acc = svmla_f32_x(pred, acc, diff, diff);
            i += lanes;
        }

        svaddv_f32(svptrue_b32(), acc)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn skip_unless_sve() -> bool {
        if std::arch::is_aarch64_feature_detected!("sve") {
            return false;
        }
        eprintln!("sve not available on this CPU; skipping SVE backend test");
        true
    }

    #[test]
    fn dot_product_matches_scalar() {
        if skip_unless_sve() {
            return;
        }
        let a: Vec<f32> = (1..=64).map(|i| i as f32 * 0.1).collect();
        let b: Vec<f32> = (1..=64).map(|i| i as f32 * 0.2).collect();
        let got = SveBackend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-3, "got={got} want={want}");
    }

    #[test]
    fn dot_product_partial_vector() {
        if skip_unless_sve() {
            return;
        }
        // 7 elements — guaranteed to leave a partial-vector iteration
        // on every supported SVE width (128, 256, 512 bit).
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let b = vec![7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let got = SveBackend.dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-5);
    }

    #[test]
    fn euclidean_squared_345_triangle() {
        if skip_unless_sve() {
            return;
        }
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        assert!((SveBackend.euclidean_distance_squared(&a, &b) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn name_is_sve() {
        assert_eq!(SveBackend.name(), "sve");
    }
}
