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

/// AVX2 backend, with optional FMA fusion in the inner loops.
///
/// Construct via [`Avx2Backend::auto_detect`] (the dispatcher uses
/// this) which sets `with_fma = true` when `is_x86_feature_detected!("fma")`
/// returns true. Most CPUs that have AVX2 also have FMA (Haswell+,
/// 2013+), so the dispatcher picks the FMA path almost always.
///
/// The two paths share the rest of the implementation; only the
/// inner-loop multiply-and-accumulate differs:
///
/// - `with_fma = false`: separate `_mm256_mul_ps` + `_mm256_add_ps`.
/// - `with_fma = true`:  fused `_mm256_fmadd_ps`.
///
/// FMA produces ~20% fewer instructions on `dot_product` /
/// `euclidean_distance_squared` and gives slightly better numerical
/// behaviour because the rounding step happens once per fused op
/// instead of twice.
pub struct Avx2Backend {
    with_fma: bool,
}

impl Avx2Backend {
    /// Construct an `Avx2Backend` with FMA detection: enables the
    /// FMA-folded inner loops if the running CPU advertises FMA.
    ///
    /// Used by the dispatcher; constructed once and cached in the
    /// `OnceLock`.
    pub fn auto_detect() -> Self {
        Self {
            with_fma: std::is_x86_feature_detected!("fma"),
        }
    }

    /// Construct without FMA — emits the legacy mul+add sequence
    /// even when the CPU has FMA. Used by tests that want to compare
    /// the two paths against the scalar oracle independently.
    pub fn without_fma() -> Self {
        Self { with_fma: false }
    }

    /// Returns true when the FMA path is in use.
    pub fn has_fma(&self) -> bool {
        self.with_fma
    }
}

impl SimdBackend for Avx2Backend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx2") {
            if self.with_fma {
                // SAFETY: AVX2+FMA verified — the constructor only
                // sets `with_fma=true` when the CPU advertises both
                // (FMA implies AVX support, and we just checked AVX2).
                unsafe { dot_product_avx2_fma(a, b) }
            } else {
                // SAFETY: AVX2 verified by the runtime detector;
                // equal-length precondition is debug-asserted on entry.
                unsafe { dot_product_avx2(a, b) }
            }
        } else {
            crate::simd::scalar::ScalarBackend.dot_product(a, b)
        }
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx2") {
            if self.with_fma {
                // SAFETY: AVX2+FMA verified — see `dot_product`.
                unsafe { euclidean_distance_squared_avx2_fma(a, b) }
            } else {
                // SAFETY: AVX2 verified — equal-length precondition
                // debug-asserted on entry.
                unsafe { euclidean_distance_squared_avx2(a, b) }
            }
        } else {
            crate::simd::scalar::ScalarBackend.euclidean_distance_squared(a, b)
        }
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        // Trait contract: clamped dot product on pre-normalised inputs.
        self.dot_product(a, b).clamp(-1.0, 1.0)
    }

    fn l2_norm(&self, a: &[f32]) -> f32 {
        // L2 norm is `sqrt(dot(a, a))`. Reusing the AVX2 dot keeps
        // the SIMD path warm without a separate intrinsic.
        self.dot_product(a, a).sqrt()
    }

    fn manhattan_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 verified by the runtime detector
            // immediately above.
            unsafe { manhattan_distance_avx2(a, b) }
        } else {
            crate::simd::scalar::ScalarBackend.manhattan_distance(a, b)
        }
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
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 verified by the runtime detector
            // immediately above; equal-length precondition is
            // debug-asserted on entry.
            unsafe { quantize_f32_to_u8_avx2(src, dst, scale, offset, levels) };
        } else {
            crate::simd::scalar::ScalarBackend.quantize_f32_to_u8(src, dst, scale, offset, levels);
        }
    }

    fn dequantize_u8_to_f32(&self, src: &[u8], dst: &mut [f32], scale: f32, offset: f32) {
        debug_assert_eq!(dst.len(), src.len(), "Buffers must have same length");
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 verified by the runtime detector
            // immediately above; equal-length precondition is
            // debug-asserted on entry.
            unsafe { dequantize_u8_to_f32_avx2(src, dst, scale, offset) };
        } else {
            crate::simd::scalar::ScalarBackend.dequantize_u8_to_f32(src, dst, scale, offset);
        }
    }

    fn int8_dot_product(&self, a: &[i8], b: &[i8]) -> i32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 verified by the runtime detector
            // immediately above; equal-length precondition is
            // debug-asserted on entry.
            unsafe { int8_dot_product_avx2(a, b) }
        } else {
            crate::simd::scalar::ScalarBackend.int8_dot_product(a, b)
        }
    }

    fn name(&self) -> &'static str {
        if self.with_fma { "avx2+fma" } else { "avx2" }
    }
}

/// # Safety
///
/// Caller must ensure AVX2 is available on the running CPU. `a` and
/// `b` must have equal length; reading past the end of either slice
/// is UB. The public wrapper enforces both.
///
/// The absolute-value trick: `|x|` for f32 clears the sign bit,
/// which we do with `_mm256_andnot_ps(sign_mask, x)` where
/// `sign_mask = _mm256_set1_ps(-0.0)` has only the sign bit set.
/// `andnot(sign_mask, x)` is `~sign_mask & x`, which clears x's
/// sign bit and leaves the rest alone — equivalent to `x.abs()`
/// for finite values.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn manhattan_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: AVX2 gated by `#[target_feature]`. Loop bound
    // `i + SIMD_LANES <= simd_len <= len` keeps every load inside
    // the slice's allocation.
    unsafe {
        let sign_mask = _mm256_set1_ps(-0.0);
        let mut sum = _mm256_setzero_ps();
        let mut i = 0;
        while i < simd_len {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));
            let diff = _mm256_sub_ps(va, vb);
            // Clear the sign bit lane-wise: `andnot(sign_mask, x)`.
            let abs_diff = _mm256_andnot_ps(sign_mask, diff);
            sum = _mm256_add_ps(sum, abs_diff);
            i += SIMD_LANES;
        }

        let mut result = horizontal_sum_avx2(sum);
        for idx in simd_len..len {
            result += (a[idx] - b[idx]).abs();
        }
        result
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
/// Caller must ensure AVX2 + FMA are both available on the running
/// CPU. Same equal-length / in-bounds preconditions as
/// [`dot_product_avx2`]. The fused multiply-add halves the
/// instruction count vs. the non-FMA variant.
#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn dot_product_avx2_fma(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: AVX2+FMA gated by `#[target_feature]`. Loop bound
    // `i + SIMD_LANES <= simd_len <= len` keeps every load inside
    // the slice's allocation.
    unsafe {
        let mut sum = _mm256_setzero_ps();
        let mut i = 0;
        while i < simd_len {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));
            // Fused multiply-add: `sum = va * vb + sum` in one
            // instruction with one rounding step.
            sum = _mm256_fmadd_ps(va, vb, sum);
            i += SIMD_LANES;
        }

        let mut result = horizontal_sum_avx2(sum);
        for idx in simd_len..len {
            result += a[idx] * b[idx];
        }
        result
    }
}

/// # Safety
///
/// Same preconditions as [`dot_product_avx2_fma`]. Returns the
/// SQUARED distance.
#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn euclidean_distance_squared_avx2_fma(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: AVX2+FMA gated by `#[target_feature]`. Same in-bounds
    // reasoning as the non-FMA variant.
    unsafe {
        let mut sum_sq = _mm256_setzero_ps();
        let mut i = 0;
        while i < simd_len {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));
            let diff = _mm256_sub_ps(va, vb);
            sum_sq = _mm256_fmadd_ps(diff, diff, sum_sq);
            i += SIMD_LANES;
        }
        let mut result = horizontal_sum_avx2(sum_sq);
        for idx in simd_len..len {
            let d = a[idx] - b[idx];
            result += d * d;
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

/// # Safety
///
/// Caller must ensure AVX2 is available on the running CPU, `dst.len()
/// == src.len()`, `levels > 0`, and `scale != 0.0` (the zero-scale
/// short circuit lives in the public wrapper). Reading/writing past
/// the end of either buffer is UB; loop bound `i + SIMD_LANES <=
/// simd_len <= len` keeps every access inside both allocations.
///
/// Every step is ordered to match [`crate::simd::scalar::ScalarBackend::quantize_f32_to_u8`]
/// exactly (same rounding at every intermediate): `(s - offset) *
/// inv_scale` (sub then mul, not fused), clamp to `[0, max_level]`,
/// round half-away-from-zero via `floor(x + 0.5)` (valid because the
/// clamped value is always non-negative and — for the realistic
/// `levels <= 256` domain this function targets — never large enough
/// for the `+ 0.5` to lose precision), then saturate to the u8 range
/// before narrowing (mirrors Rust's saturating `f32 as u8` cast for
/// `levels > 256` callers, where `max_level` can exceed 255).
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn quantize_f32_to_u8_avx2(
    src: &[f32],
    dst: &mut [u8],
    scale: f32,
    offset: f32,
    levels: u32,
) {
    let len = src.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: AVX2 gated by `#[target_feature]`. Loop bound
    // `i + SIMD_LANES <= simd_len <= len` keeps every load/write
    // inside the slice's allocation.
    unsafe {
        let inv_scale = 1.0 / scale;
        let max_level = (levels - 1) as f32;
        let v_offset = _mm256_set1_ps(offset);
        let v_inv_scale = _mm256_set1_ps(inv_scale);
        let v_zero = _mm256_setzero_ps();
        let v_max = _mm256_set1_ps(max_level);
        let v_half = _mm256_set1_ps(0.5);
        let v_u8_max = _mm256_set1_epi32(255);

        let mut i = 0;
        while i < simd_len {
            let vs = _mm256_loadu_ps(src.as_ptr().add(i));
            // Scalar order: `(s - offset) * inv_scale` — sub then
            // mul, NOT fma, so the rounding matches the scalar
            // reference bit-for-bit.
            let normalised = _mm256_mul_ps(_mm256_sub_ps(vs, v_offset), v_inv_scale);
            // NaN handling: `_mm256_max_ps(a, b)` returns `b` whenever
            // `a` is NaN (Intel's "second operand wins" rule, not
            // IEEE `maxNum`). `normalised` can only be NaN in the `a`
            // position here, so a NaN input resolves to `0.0` before
            // it ever reaches `_mm256_cvttps_epi32` — matching the
            // scalar path's `NaN.clamp(..).round() as u8 == 0`
            // outcome (Rust's saturating float→int cast maps NaN to
            // 0) without hitting `cvttps_epi32`'s NaN→`i32::MIN`
            // "integer indefinite" behaviour.
            let clamped = _mm256_min_ps(_mm256_max_ps(normalised, v_zero), v_max);
            // Round half away from zero: `clamped` is always >= 0.0
            // here, so `floor(x + 0.5)` matches `f32::round()` exactly.
            let rounded = _mm256_floor_ps(_mm256_add_ps(clamped, v_half));
            let as_i32 = _mm256_cvttps_epi32(rounded);
            let saturated = _mm256_min_epi32(as_i32, v_u8_max);

            let mut tmp = [0i32; SIMD_LANES];
            _mm256_storeu_si256(tmp.as_mut_ptr().cast(), saturated);
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
/// Caller must ensure AVX2 is available on the running CPU and
/// `dst.len() == src.len()`. Loop bound `i + SIMD_LANES <= simd_len <=
/// len` keeps every load/write inside both allocations.
///
/// Computed as `offset + (src[i] as f32) * scale` — mul then add, NOT
/// fma — to match the scalar reference's two separate rounding steps
/// bit-for-bit. Widening `u8 -> i32 -> f32` is exact for the full
/// `0..=255` range (well within f32's 24-bit mantissa).
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn dequantize_u8_to_f32_avx2(src: &[u8], dst: &mut [f32], scale: f32, offset: f32) {
    let len = src.len();
    let simd_len = len - (len % SIMD_LANES);

    // SAFETY: AVX2 gated by `#[target_feature]`. `_mm_loadl_epi64`
    // reads exactly 8 bytes (the low half of a 128-bit register,
    // upper half zeroed), and `i + SIMD_LANES <= simd_len <= len`
    // keeps that 8-byte read inside `src`'s allocation.
    unsafe {
        let v_scale = _mm256_set1_ps(scale);
        let v_offset = _mm256_set1_ps(offset);

        let mut i = 0;
        while i < simd_len {
            let bytes = _mm_loadl_epi64(src.as_ptr().add(i).cast());
            let widened = _mm256_cvtepu8_epi32(bytes);
            let as_f32 = _mm256_cvtepi32_ps(widened);
            let result = _mm256_add_ps(v_offset, _mm256_mul_ps(as_f32, v_scale));
            _mm256_storeu_ps(dst.as_mut_ptr().add(i), result);
            i += SIMD_LANES;
        }

        for idx in simd_len..len {
            dst[idx] = offset + (src[idx] as f32) * scale;
        }
    }
}

/// # Safety
///
/// Caller must ensure AVX2 is available on the running CPU; `a` and
/// `b` must have equal length. Loop bound `i + LANES <= simd_len <=
/// len` keeps every load inside both allocations.
///
/// Both operands are true SIGNED `i8` (unlike VNNI's `vpdpbusd`, which
/// wants one unsigned operand), so this sign-extends both sides to
/// `i16` via `_mm256_cvtepi8_epi16` — exact, no bias trick needed —
/// then uses `_mm256_madd_epi16` to multiply i16 pairs into i32 and
/// sum adjacent pairs. Both steps are exact: the widened values stay
/// in `[-128, 127]`, so each product is at most `128 * 128 = 16384`
/// and each pairwise sum at most `32768` — nowhere near `i32`
/// overflow, and `madd_epi16`'s pairwise add does not saturate (unlike
/// `maddubs_epi16`, which would saturate the intermediate `i16` sum
/// for large-magnitude `u8 * i8` products — the reason this kernel
/// avoids `maddubs` entirely).
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn int8_dot_product_avx2(a: &[i8], b: &[i8]) -> i32 {
    const LANES: usize = 16; // `_mm256_cvtepi8_epi16` widens 16 x i8 -> 16 x i16 per call.

    let len = a.len();
    let simd_len = len - (len % LANES);

    // SAFETY: AVX2 gated by `#[target_feature]`. `_mm_loadu_si128`
    // reads exactly 16 bytes per call; `i + LANES <= simd_len <= len`
    // keeps every read inside both slices' allocations.
    unsafe {
        let mut acc = _mm256_setzero_si256();

        let mut i = 0;
        while i < simd_len {
            let va = _mm256_cvtepi8_epi16(_mm_loadu_si128(a.as_ptr().add(i).cast()));
            let vb = _mm256_cvtepi8_epi16(_mm_loadu_si128(b.as_ptr().add(i).cast()));
            let prod = _mm256_madd_epi16(va, vb);
            acc = _mm256_add_epi32(acc, prod);
            i += LANES;
        }

        let mut result = horizontal_sum_epi32_avx2(acc);
        for idx in simd_len..len {
            result += (a[idx] as i32) * (b[idx] as i32);
        }
        result
    }
}

/// # Safety
///
/// AVX2 must be available. Pure shuffle/add intrinsics on `i32` lanes
/// — only called from [`int8_dot_product_avx2`], which already
/// enforces the precondition. Integer addition is exact (no rounding),
/// so this reduction is bit-exact regardless of the tree shape.
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn horizontal_sum_epi32_avx2(v: __m256i) -> i32 {
    // SAFETY: AVX2 gated by `#[target_feature]`. All operations are
    // pure register shuffles + adds with no memory access.
    unsafe {
        let hi = _mm256_extracti128_si256(v, 1);
        let lo = _mm256_castsi256_si128(v);
        let sum128 = _mm_add_epi32(hi, lo);
        let sum64 = _mm_hadd_epi32(sum128, sum128);
        let sum32 = _mm_hadd_epi32(sum64, sum64);
        _mm_cvtsi128_si32(sum32)
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
        let got = Avx2Backend::auto_detect().dot_product(&a, &b);
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
        let got = Avx2Backend::auto_detect().dot_product(&a, &b);
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
        let got = Avx2Backend::auto_detect().euclidean_distance_squared(&a, &b);
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
        let got = Avx2Backend::auto_detect().cosine_similarity(&a, &b);
        assert!((got - 1.0).abs() < 1e-6);
    }

    #[test]
    fn l2_norm_matches_scalar_sqrt() {
        if skip_unless_avx2() {
            return;
        }
        let a = vec![3.0, 4.0, 0.0, 0.0]; // sqrt(9+16) = 5
        let got = Avx2Backend::auto_detect().l2_norm(&a);
        assert!((got - 5.0).abs() < 1e-5);
    }

    #[test]
    fn large_vector_relative_error_under_1e_minus_4() {
        if skip_unless_avx2() {
            return;
        }
        let a: Vec<f32> = (0..1000).map(|i| i as f32 * 0.1).collect();
        let b: Vec<f32> = (0..1000).map(|i| i as f32 * 0.2).collect();
        let got = Avx2Backend::auto_detect().dot_product(&a, &b);
        let want: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let rel = (got - want).abs() / want.abs().max(1e-6);
        assert!(rel < 1e-4, "rel={rel} got={got} want={want}");
    }

    #[test]
    fn name_includes_fma_when_present() {
        let backend = Avx2Backend::auto_detect();
        let want = if backend.has_fma() {
            "avx2+fma"
        } else {
            "avx2"
        };
        assert_eq!(backend.name(), want);
    }

    #[test]
    fn name_without_fma() {
        // Force the non-FMA path even on FMA-capable CPUs to verify
        // the name string flips.
        assert_eq!(Avx2Backend::without_fma().name(), "avx2");
    }

    #[test]
    fn fma_path_matches_non_fma_path_within_rounding() {
        if skip_unless_avx2() {
            return;
        }
        // Random-but-deterministic vectors at a length that exercises
        // both the SIMD chunks and a tail.
        let len = 137;
        let a: Vec<f32> = (0..len).map(|i| (i as f32 * 0.137).sin()).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32 * 0.241).cos()).collect();
        let with_fma = Avx2Backend { with_fma: true }.dot_product(&a, &b);
        let no_fma = Avx2Backend::without_fma().dot_product(&a, &b);
        // FMA's single rounding step shifts the result by at most a
        // few ulp on a 137-element reduction.
        assert!(
            (with_fma - no_fma).abs() < 1e-4,
            "fma={with_fma} no_fma={no_fma} diff={}",
            (with_fma - no_fma).abs()
        );
    }

    // ── Oracle tests: AVX2 quantize/dequantize/int8_dot_product vs
    // `ScalarBackend`. Elementwise kernels (quantize, dequantize) are
    // asserted bit-exact (no cross-lane reduction, so no rounding-order
    // divergence is possible); `int8_dot_product` reduces via integer
    // addition, which is exact regardless of summation order, so it
    // is also asserted bit-exact. ──────────────────────────────────

    use crate::simd::scalar::ScalarBackend;

    /// Linear congruential generator — deterministic, no `rand` dep.
    /// Mirrors the generator in `tests/simd/scalar_oracle.rs`.
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
    /// edge, which is the value most likely to expose a signed/
    /// unsigned confusion in the SIMD kernel).
    fn random_i8_vector(seed: u64, len: usize) -> Vec<i8> {
        let mut state = seed;
        (0..len)
            .map(|_| (lcg(&mut state) % 256) as u8 as i8)
            .collect()
    }

    const ORACLE_LENGTHS: [usize; 11] = [0, 1, 7, 8, 9, 31, 32, 33, 255, 256, 1000];

    #[test]
    fn quantize_f32_to_u8_matches_scalar_oracle() {
        if skip_unless_avx2() {
            return;
        }
        let scale = 2.0 / 255.0;
        let offset = -1.0;
        let levels = 256u32;
        for &len in &ORACLE_LENGTHS {
            let src = random_f32_vector(0xA5A5_0000 ^ len as u64, len);
            let mut got = vec![0u8; len];
            let mut want = vec![0u8; len];
            Avx2Backend::auto_detect().quantize_f32_to_u8(&src, &mut got, scale, offset, levels);
            ScalarBackend.quantize_f32_to_u8(&src, &mut want, scale, offset, levels);
            assert_eq!(got, want, "len={len}");
        }
    }

    #[test]
    fn quantize_f32_to_u8_zero_scale_matches_scalar() {
        if skip_unless_avx2() {
            return;
        }
        for &len in &ORACLE_LENGTHS {
            let src = random_f32_vector(0xB6B6_0000 ^ len as u64, len);
            let mut got = vec![0xFFu8; len];
            let mut want = vec![0xFFu8; len];
            Avx2Backend::auto_detect().quantize_f32_to_u8(&src, &mut got, 0.0, 0.0, 256);
            ScalarBackend.quantize_f32_to_u8(&src, &mut want, 0.0, 0.0, 256);
            assert_eq!(got, want, "len={len}");
        }
    }

    #[test]
    fn dequantize_u8_to_f32_matches_scalar_oracle() {
        if skip_unless_avx2() {
            return;
        }
        let scale = 2.0 / 255.0;
        let offset = -1.0;
        for &len in &ORACLE_LENGTHS {
            let src = random_u8_vector(0xC7C7_0000 ^ len as u64, len);
            let mut got = vec![0.0f32; len];
            let mut want = vec![0.0f32; len];
            Avx2Backend::auto_detect().dequantize_u8_to_f32(&src, &mut got, scale, offset);
            ScalarBackend.dequantize_u8_to_f32(&src, &mut want, scale, offset);
            assert_eq!(got, want, "len={len}");
        }
    }

    #[test]
    fn int8_dot_product_matches_scalar_oracle() {
        if skip_unless_avx2() {
            return;
        }
        for &len in &ORACLE_LENGTHS {
            let a = random_i8_vector(0xD8D8_0000 ^ len as u64, len);
            let b = random_i8_vector(0xE9E9_0000 ^ len as u64, len);
            let got = Avx2Backend::auto_detect().int8_dot_product(&a, &b);
            let want = ScalarBackend.int8_dot_product(&a, &b);
            assert_eq!(got, want, "len={len}");
        }
    }

    #[test]
    fn int8_dot_product_extremes_do_not_overflow_i16_intermediate() {
        if skip_unless_avx2() {
            return;
        }
        // All-(-128) against all-(-128): the classic maddubs-style
        // saturation trap (255 * 128 summed pairwise would overflow a
        // signed i16 intermediate). The sign-extend + madd_epi16
        // design in `int8_dot_product_avx2` must not saturate here.
        let a = vec![-128i8; 64];
        let b = vec![-128i8; 64];
        let got = Avx2Backend::auto_detect().int8_dot_product(&a, &b);
        let want = ScalarBackend.int8_dot_product(&a, &b);
        assert_eq!(got, want);
        assert_eq!(want, 128 * 128 * 64);
    }
}
