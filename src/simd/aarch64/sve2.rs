//! SVE2 backend — adds INT8 dot product on top of the SVE f32 path.
//!
//! SVE2 ships on Neoverse V2, AWS Graviton4, Apple's A17/M3 (NOT
//! M-series Mac SoCs — those are still NEON-only), and every newer
//! ARM v9 silicon. The headline primitive for our quantization work
//! is `svdot_s32`, which computes the four-way dot product of i8
//! byte pairs into i32 accumulators in one instruction — analogous
//! to AVX-512 VNNI's `vpdpbusd`.
//!
//! For the f32 primitives this backend forwards to
//! [`super::sve::SveBackend`] verbatim — there's no SVE2 gain on
//! float math, only on byte/word integer math. Callers that only
//! exercise f32 stay on the SVE path; the only reason to pick this
//! backend over plain SVE is when the workload includes the
//! quantized INT8 distance kernel from phase 7f.

use std::arch::aarch64::*;

use crate::simd::aarch64::sve::SveBackend;
use crate::simd::backend::SimdBackend;

/// Marker type for the SVE2 backend.
pub struct Sve2Backend;

impl SimdBackend for Sve2Backend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        SveBackend.dot_product(a, b)
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        SveBackend.euclidean_distance_squared(a, b)
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        SveBackend.cosine_similarity(a, b)
    }

    fn l2_norm(&self, a: &[f32]) -> f32 {
        SveBackend.l2_norm(a)
    }

    fn int8_dot_product(&self, a: &[i8], b: &[i8]) -> i32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::arch::is_aarch64_feature_detected!("sve2") {
            // SAFETY: SVE2 verified by the runtime detector
            // immediately above; equal-length precondition debug-
            // asserted on entry.
            unsafe { int8_dot_product_sve2(a, b) }
        } else {
            crate::simd::scalar::ScalarBackend.int8_dot_product(a, b)
        }
    }

    fn name(&self) -> &'static str {
        "sve2"
    }
}

/// # Safety
///
/// Caller must ensure SVE2 is available on the running CPU. `svdot`
/// requires SVE2 even though `svdot_s32` is sometimes treated as an
/// SVE-base instruction in older docs. Equal-length precondition is
/// enforced by the public wrapper. Predicated loads stay in-bounds
/// via `svwhilelt_b8`.
#[target_feature(enable = "sve2")]
#[inline]
unsafe fn int8_dot_product_sve2(a: &[i8], b: &[i8]) -> i32 {
    let len = a.len();

    // SAFETY: SVE2 gated by `#[target_feature]`. `svwhilelt_b8(i, len)`
    // builds a predicate over byte-lanes; `svld1_s8(pred, ptr)` reads
    // only enabled lanes. `svdot_s32` accumulates the four-way dot
    // products of consecutive i8 lanes into i32 accumulators.
    unsafe {
        // i32 accumulator vector — width determined by the CPU.
        let mut acc = svdup_n_s32(0);
        let mut i: u64 = 0;
        let len_u = len as u64;
        // Byte-lane count of the running CPU's vector length.
        let byte_lanes = svcntb();

        while i < len_u {
            let pred = svwhilelt_b8(i as u32, len_u as u32);
            let va = svld1_s8(pred, a.as_ptr().add(i as usize));
            let vb = svld1_s8(pred, b.as_ptr().add(i as usize));
            // Four-way dot: each i32 accumulator lane gets the sum
            // of four pairwise i8×i8 products from the corresponding
            // bytes of va/vb. Disabled byte lanes contribute zero.
            acc = svdot_s32(acc, va, vb);
            i += byte_lanes;
        }

        // Reduce all i32 lanes to a scalar.
        svaddv_s32(svptrue_b32(), acc)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn skip_unless_sve2() -> bool {
        if std::arch::is_aarch64_feature_detected!("sve2") {
            return false;
        }
        eprintln!("sve2 not available on this CPU; skipping SVE2 backend test");
        true
    }

    #[test]
    fn int8_dot_product_aligned() {
        if skip_unless_sve2() {
            return;
        }
        // 64 elements ≥ one SVE-256 byte vector; a longer vector
        // forces at least one full iteration even on SVE-512.
        let a: Vec<i8> = (0..128).map(|i| (i % 100) as i8).collect();
        let b: Vec<i8> = (0..128).map(|i| ((50 - i) % 100) as i8).collect();
        let got = Sve2Backend.int8_dot_product(&a, &b);
        let want: i32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as i32) * (*y as i32))
            .sum();
        assert_eq!(got, want);
    }

    #[test]
    fn int8_with_partial_tail() {
        if skip_unless_sve2() {
            return;
        }
        // Odd length — predicated tail handles the leftover.
        let a: Vec<i8> = (0..70).map(|i| (i % 50) as i8).collect();
        let b: Vec<i8> = (0..70).map(|i| ((30 - i) % 50) as i8).collect();
        let got = Sve2Backend.int8_dot_product(&a, &b);
        let want: i32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as i32) * (*y as i32))
            .sum();
        assert_eq!(got, want);
    }

    #[test]
    fn int8_zero_input_returns_zero() {
        if skip_unless_sve2() {
            return;
        }
        let a = vec![0i8; 128];
        let b = vec![42i8; 128];
        assert_eq!(Sve2Backend.int8_dot_product(&a, &b), 0);
    }

    #[test]
    fn name_is_sve2() {
        assert_eq!(Sve2Backend.name(), "sve2");
    }
}
