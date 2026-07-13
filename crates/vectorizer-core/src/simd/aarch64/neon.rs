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

    fn manhattan_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        // SAFETY: NEON is in the aarch64 psABI baseline; no runtime
        // check needed.
        unsafe { manhattan_distance_neon(a, b) }
    }

    fn quantize_f32_to_u8(
        &self,
        src: &[f32],
        dst: &mut [u8],
        scale: f32,
        offset: f32,
        levels: u32,
    ) {
        debug_assert_eq!(dst.len(), src.len(), "Buffers must have same length");
        debug_assert!(levels > 0, "levels must be positive");
        if scale == 0.0 {
            // Constant-input short circuit; matches the trait default
            // documented on `SimdBackend::quantize_f32_to_u8`.
            for d in dst.iter_mut() {
                *d = 0;
            }
            return;
        }
        // SAFETY: NEON is in the aarch64 psABI baseline; no runtime
        // check needed. Precondition (equal length, levels > 0) is
        // debug-asserted above.
        unsafe { quantize_f32_to_u8_neon(src, dst, scale, offset, levels) };
    }

    fn dequantize_u8_to_f32(&self, src: &[u8], dst: &mut [f32], scale: f32, offset: f32) {
        debug_assert_eq!(dst.len(), src.len(), "Buffers must have same length");
        // SAFETY: NEON is in the aarch64 psABI baseline; no runtime
        // check needed.
        unsafe { dequantize_u8_to_f32_neon(src, dst, scale, offset) };
    }

    fn int8_dot_product(&self, a: &[i8], b: &[i8]) -> i32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        // SAFETY: NEON is in the aarch64 psABI baseline; no runtime
        // check needed.
        unsafe { int8_dot_product_neon(a, b) }
    }

    fn name(&self) -> &'static str {
        "neon"
    }
}

/// # Safety
///
/// NEON is guaranteed on every aarch64 target (psABI baseline).
/// `a` and `b` must have equal length; reading past the end of
/// either slice is UB. The public wrapper enforces both.
#[target_feature(enable = "neon")]
#[inline]
unsafe fn manhattan_distance_neon(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: NEON gated by `#[target_feature]`. Loop bound
    // `i + SIMD_LANES <= simd_len <= len` keeps every load inside
    // the slice's allocation. `vabsq_f32` is a pure register op.
    unsafe {
        let mut sum = vdupq_n_f32(0.0);
        let mut i = 0;
        while i < simd_len {
            let va = vld1q_f32(a.as_ptr().add(i));
            let vb = vld1q_f32(b.as_ptr().add(i));
            let diff = vsubq_f32(va, vb);
            // `vabsq_f32` is the single-instruction absolute value.
            let abs_diff = vabsq_f32(diff);
            sum = vaddq_f32(sum, abs_diff);
            i += SIMD_LANES;
        }
        let mut result = vaddvq_f32(sum);
        for idx in simd_len..len {
            result += (a[idx] - b[idx]).abs();
        }
        result
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

/// # Safety
///
/// NEON is guaranteed on every aarch64 target. Caller must ensure
/// `dst.len() == src.len()`, `levels > 0`, and `scale != 0.0` (the
/// zero-scale short circuit lives in the public wrapper). Loop bound
/// `i + SIMD_LANES <= simd_len <= len` keeps every load/write inside
/// both allocations.
///
/// Mirrors `crate::simd::x86::avx2`'s `quantize_f32_to_u8_avx2`
/// design: vectorised sub/mul/clamp/round, scalar-narrow to `u8`.
/// Order of operations (`sub` then `mul`, not fma; round via
/// `floor(x + 0.5)`; saturate to `[0, 255]` before narrowing) matches
/// the scalar reference bit-for-bit for the same reasons documented
/// on the AVX2 sibling.
#[target_feature(enable = "neon")]
#[inline]
unsafe fn quantize_f32_to_u8_neon(
    src: &[f32],
    dst: &mut [u8],
    scale: f32,
    offset: f32,
    levels: u32,
) {
    let len = src.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: NEON gated by `#[target_feature]`. Loop bound
    // `i + SIMD_LANES <= simd_len <= len` keeps every load/write
    // inside the slice's allocation.
    unsafe {
        let inv_scale = 1.0 / scale;
        let max_level = (levels - 1) as f32;
        let v_offset = vdupq_n_f32(offset);
        let v_inv_scale = vdupq_n_f32(inv_scale);
        let v_zero = vdupq_n_f32(0.0);
        let v_max = vdupq_n_f32(max_level);
        let v_half = vdupq_n_f32(0.5);
        let v_u8_max = vdupq_n_u32(255);

        let mut i = 0;
        while i < simd_len {
            let vs = vld1q_f32(src.as_ptr().add(i));
            // Scalar order: `(s - offset) * inv_scale` — sub then
            // mul, NOT fma, so the rounding matches the scalar
            // reference bit-for-bit.
            let normalised = vmulq_f32(vsubq_f32(vs, v_offset), v_inv_scale);
            // NaN handling: NEON's vector `FMAX`/`FMIN` (`vmaxq_f32`/
            // `vminq_f32`) propagate NaN rather than resolving to the
            // non-NaN operand (unlike x86's asymmetric "second
            // operand wins" rule used in the AVX2 sibling). A NaN
            // `normalised` therefore stays NaN through `clamped`;
            // `vrndmq_f32` (floor) on NaN also propagates NaN;
            // `vcvtq_u32_f32` (float -> unsigned int) then maps NaN to
            // `0` per the aarch64 FCVTZU "invalid operand" rule —
            // landing on the same final `0` the scalar path produces
            // via `NaN.round() as u8` (Rust's saturating float->int
            // cast maps NaN to 0).
            let clamped = vminq_f32(vmaxq_f32(normalised, v_zero), v_max);
            // Round half away from zero: `clamped` is always >= 0.0
            // here, so `floor(x + 0.5)` matches `f32::round()` exactly.
            let rounded = vrndmq_f32(vaddq_f32(clamped, v_half));
            let as_u32 = vcvtq_u32_f32(rounded);
            // Saturate to the u8 range before narrowing — mirrors
            // Rust's saturating `f32 as u8` cast for `levels > 256`
            // callers, where `max_level` can exceed 255.
            let saturated = vminq_u32(as_u32, v_u8_max);

            let mut tmp = [0u32; SIMD_LANES];
            vst1q_u32(tmp.as_mut_ptr(), saturated);
            for (k, &v) in tmp.iter().enumerate() {
                dst[i + k] = v as u8;
            }
            i += SIMD_LANES;
        }

        // Tail loop for the leftover (len % SIMD_LANES) elements —
        // literally the scalar reference's body, so it is bit-exact
        // by construction.
        for idx in simd_len..len {
            let normalised = (src[idx] - offset) * inv_scale;
            let clamped = normalised.clamp(0.0, max_level);
            dst[idx] = clamped.round() as u8;
        }
    }
}

/// # Safety
///
/// NEON is guaranteed on every aarch64 target. Caller must ensure
/// `dst.len() == src.len()`. Loop bound `i + LANES <= simd_len <=
/// len` keeps every load/write inside both allocations.
///
/// Computed as `offset + (src[i] as f32) * scale` — mul then add, NOT
/// fma — to match the scalar reference's two separate rounding steps
/// bit-for-bit. Widening `u8 -> u16 -> u32 -> f32` (via `vmovl_u8` /
/// `vmovl_u16` / `vcvtq_f32_u32`) is exact for the full `0..=255`
/// range.
#[target_feature(enable = "neon")]
#[inline]
unsafe fn dequantize_u8_to_f32_neon(src: &[u8], dst: &mut [f32], scale: f32, offset: f32) {
    const LANES: usize = 8; // `vld1_u8` loads 8 x u8 (64-bit register) per call.

    let len = src.len();
    let simd_len = len - (len % LANES);

    // SAFETY: NEON gated by `#[target_feature]`. `vld1_u8` reads
    // exactly 8 bytes per call; `i + LANES <= simd_len <= len` keeps
    // every read/write inside both buffers' allocations.
    unsafe {
        let v_scale = vdupq_n_f32(scale);
        let v_offset = vdupq_n_f32(offset);

        let mut i = 0;
        while i < simd_len {
            let bytes = vld1_u8(src.as_ptr().add(i));
            let widened16 = vmovl_u8(bytes);
            let lo32 = vmovl_u16(vget_low_u16(widened16));
            let hi32 = vmovl_u16(vget_high_u16(widened16));
            let lo_f32 = vcvtq_f32_u32(lo32);
            let hi_f32 = vcvtq_f32_u32(hi32);
            let lo_result = vaddq_f32(v_offset, vmulq_f32(lo_f32, v_scale));
            let hi_result = vaddq_f32(v_offset, vmulq_f32(hi_f32, v_scale));
            vst1q_f32(dst.as_mut_ptr().add(i), lo_result);
            vst1q_f32(dst.as_mut_ptr().add(i + 4), hi_result);
            i += LANES;
        }

        for idx in simd_len..len {
            dst[idx] = offset + (src[idx] as f32) * scale;
        }
    }
}

/// # Safety
///
/// NEON is guaranteed on every aarch64 target. `a` and `b` must have
/// equal length; reading past the end of either slice is UB. Loop
/// bound `i + LANES <= simd_len <= len` keeps every load in-bounds.
///
/// Sign-extends 8 x i8 -> 8 x i16 via `vmovl_s8` (exact — both `a` and
/// `b` are true SIGNED i8, so no unsigned/signed bias trick is needed
/// here), then widens each 4-lane half into an `i32` accumulator via
/// `vmlal_s16` (a genuine widening multiply-add: `i16 * i16 -> i32`,
/// added to the accumulator). Every step is exact — no saturation
/// anywhere in the pipeline, unlike an `i8`-native `vmull_s8` +
/// `vpadal` sequence would risk — so the reduction is bit-exact
/// regardless of tree shape (integer addition is associative).
#[target_feature(enable = "neon")]
#[inline]
unsafe fn int8_dot_product_neon(a: &[i8], b: &[i8]) -> i32 {
    const LANES: usize = 8; // `vld1_s8` loads 8 x i8 (64-bit register) per call.

    let len = a.len();
    let simd_len = len - (len % LANES);

    // SAFETY: NEON gated by `#[target_feature]`. `vld1_s8` reads
    // exactly 8 bytes per call; `i + LANES <= simd_len <= len` keeps
    // every read inside both slices' allocations.
    unsafe {
        let mut acc_lo = vdupq_n_s32(0);
        let mut acc_hi = vdupq_n_s32(0);

        let mut i = 0;
        while i < simd_len {
            let va8 = vld1_s8(a.as_ptr().add(i));
            let vb8 = vld1_s8(b.as_ptr().add(i));
            let va16 = vmovl_s8(va8);
            let vb16 = vmovl_s8(vb8);
            acc_lo = vmlal_s16(acc_lo, vget_low_s16(va16), vget_low_s16(vb16));
            acc_hi = vmlal_s16(acc_hi, vget_high_s16(va16), vget_high_s16(vb16));
            i += LANES;
        }

        let combined = vaddq_s32(acc_lo, acc_hi);
        let mut result = vaddvq_s32(combined);
        for idx in simd_len..len {
            result += (a[idx] as i32) * (b[idx] as i32);
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

    // ── Oracle tests: NEON quantize/dequantize/int8_dot_product vs
    // `ScalarBackend`. Elementwise kernels (quantize, dequantize) are
    // asserted bit-exact (no cross-lane reduction, so no
    // rounding-order divergence is possible); `int8_dot_product`
    // reduces via integer addition, which is exact regardless of
    // summation order, so it is also asserted bit-exact. These tests
    // only compile/run on aarch64 (this file's parent module gate);
    // see the phase38 §3 task report for the x86_64-host compile
    // status. ────────────────────────────────────────────────────

    use crate::simd::scalar::ScalarBackend;

    /// Linear congruential generator — deterministic, no `rand` dep.
    /// Mirrors the generator in `simd::x86::avx2`'s oracle tests and
    /// `tests/simd/scalar_oracle.rs`.
    fn lcg(state: &mut u64) -> u32 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*state >> 32) as u32
    }

    /// Deterministic f32 vector in `[-3.0, 3.0]` — the amplitude
    /// intentionally exceeds a typical `[-1, 1]` quantization range so
    /// the clamp path is exercised, not just the interior.
    fn random_f32_vector(seed: u64, len: usize) -> Vec<f32> {
        let mut state = seed;
        (0..len)
            .map(|_| {
                let bits = lcg(&mut state) >> 8; // top 24 bits → exact f32 unit range
                let unit = (bits as f32) / ((1u32 << 24) as f32);
                (unit * 2.0 - 1.0) * 3.0
            })
            .collect()
    }

    /// Deterministic full-range `u8` vector.
    fn random_u8_vector(seed: u64, len: usize) -> Vec<u8> {
        let mut state = seed;
        (0..len).map(|_| (lcg(&mut state) % 256) as u8).collect()
    }

    /// Deterministic full-range `i8` vector (including the `-128`
    /// edge).
    fn random_i8_vector(seed: u64, len: usize) -> Vec<i8> {
        let mut state = seed;
        (0..len)
            .map(|_| (lcg(&mut state) % 256) as u8 as i8)
            .collect()
    }

    const ORACLE_LENGTHS: [usize; 11] = [0, 1, 7, 8, 9, 31, 32, 33, 255, 256, 1000];

    #[test]
    fn quantize_f32_to_u8_matches_scalar_oracle() {
        let scale = 2.0 / 255.0;
        let offset = -1.0;
        let levels = 256u32;
        for &len in &ORACLE_LENGTHS {
            let src = random_f32_vector(0xA5A5_0000 ^ len as u64, len);
            let mut got = vec![0u8; len];
            let mut want = vec![0u8; len];
            NeonBackend.quantize_f32_to_u8(&src, &mut got, scale, offset, levels);
            ScalarBackend.quantize_f32_to_u8(&src, &mut want, scale, offset, levels);
            assert_eq!(got, want, "len={len}");
        }
    }

    #[test]
    fn quantize_f32_to_u8_zero_scale_matches_scalar() {
        for &len in &ORACLE_LENGTHS {
            let src = random_f32_vector(0xB6B6_0000 ^ len as u64, len);
            let mut got = vec![0xFFu8; len];
            let mut want = vec![0xFFu8; len];
            NeonBackend.quantize_f32_to_u8(&src, &mut got, 0.0, 0.0, 256);
            ScalarBackend.quantize_f32_to_u8(&src, &mut want, 0.0, 0.0, 256);
            assert_eq!(got, want, "len={len}");
        }
    }

    #[test]
    fn dequantize_u8_to_f32_matches_scalar_oracle() {
        let scale = 2.0 / 255.0;
        let offset = -1.0;
        for &len in &ORACLE_LENGTHS {
            let src = random_u8_vector(0xC7C7_0000 ^ len as u64, len);
            let mut got = vec![0.0f32; len];
            let mut want = vec![0.0f32; len];
            NeonBackend.dequantize_u8_to_f32(&src, &mut got, scale, offset);
            ScalarBackend.dequantize_u8_to_f32(&src, &mut want, scale, offset);
            assert_eq!(got, want, "len={len}");
        }
    }

    #[test]
    fn int8_dot_product_matches_scalar_oracle() {
        for &len in &ORACLE_LENGTHS {
            let a = random_i8_vector(0xD8D8_0000 ^ len as u64, len);
            let b = random_i8_vector(0xE9E9_0000 ^ len as u64, len);
            let got = NeonBackend.int8_dot_product(&a, &b);
            let want = ScalarBackend.int8_dot_product(&a, &b);
            assert_eq!(got, want, "len={len}");
        }
    }

    #[test]
    fn int8_dot_product_extremes_do_not_overflow() {
        // All-(-128) against all-(-128) — the widening multiply-add
        // must not saturate or overflow.
        let a = vec![-128i8; 64];
        let b = vec![-128i8; 64];
        let got = NeonBackend.int8_dot_product(&a, &b);
        let want = ScalarBackend.int8_dot_product(&a, &b);
        assert_eq!(got, want);
        assert_eq!(want, 128 * 128 * 64);
    }
}
