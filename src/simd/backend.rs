//! The `SimdBackend` trait — the contract every per-ISA implementation
//! satisfies.
//!
//! v1 of the trait (phase 7a) covered the four `f32` primitives the
//! hot paths exercise today: `dot_product`,
//! `euclidean_distance_squared`, `cosine_similarity`, and `l2_norm`.
//! Phase 7b adds `int8_dot_product` for the upcoming quantization
//! work (phase 7f's INT8 asymmetric distance) — backends that have a
//! single-instruction implementation (AVX-512 VNNI, NEON DOTPROD)
//! override it; everything else inherits the scalar fallback.
//!
//! Adding a method here without a matching scalar fallback breaks
//! every backend at once, which is the desired tripwire — but
//! providing the fallback keeps the per-ISA backends simple.
//!
//! Invariants every implementation MUST uphold:
//!
//! - The two slices have equal length. Caller checks; backends may
//!   `debug_assert` but should NOT panic in release. Mismatched
//!   lengths are a correctness bug at the call site.
//! - `cosine_similarity` assumes both inputs are pre-normalised; it
//!   is implemented as a clamped dot product, NOT as
//!   `dot / (|a| * |b|)`. Callers that need full cosine should
//!   normalise first or use `models::DistanceCalculator::cosine_similarity`
//!   from `src/models/mod.rs`.
//! - `euclidean_distance_squared` returns the SQUARED distance to
//!   save the `sqrt` for callers that only compare distances. The
//!   convenience function `crate::simd::euclidean_distance` does the
//!   sqrt for callers that need it.
//! - Every method MUST return the same value (within f32 rounding) as
//!   the [`crate::simd::scalar::ScalarBackend`] oracle. The
//!   `tests/simd/scalar_oracle.rs` integration test pins this on
//!   random vectors.

/// Implemented by every per-ISA backend. `Send + Sync + 'static`
/// because the dispatcher caches a `&'static dyn SimdBackend` and
/// hands it across threads.
pub trait SimdBackend: Send + Sync + 'static {
    /// Sum of pairwise products: `∑ a[i] * b[i]`.
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32;

    /// `∑ (a[i] - b[i])²`. Caller takes `sqrt` if Euclidean distance
    /// (rather than its square) is needed.
    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32;

    /// Cosine similarity ASSUMING pre-normalised inputs — implemented
    /// as a clamped dot product. See trait-level docs.
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32;

    /// L2 norm: `sqrt(∑ a[i]²)`.
    fn l2_norm(&self, a: &[f32]) -> f32;

    /// INT8 dot product: `∑ a[i] * b[i]` returning an `i32`. Used by
    /// the phase 7f quantized-distance code path. Default impl is a
    /// straight scalar loop; backends with a hardware primitive
    /// (AVX-512 VNNI, NEON DOTPROD) override this. The `i32`
    /// accumulator absorbs the worst-case `127 * 127 * len` without
    /// overflow for `len < 130_000`.
    fn int8_dot_product(&self, a: &[i8], b: &[i8]) -> i32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as i32) * (*y as i32))
            .sum()
    }

    /// Manhattan (L1) distance: `∑ |a[i] - b[i]|`. Default impl is a
    /// straight scalar loop; SIMD backends override with `vabsq_f32`
    /// (NEON), `_mm_andnot_ps` + sign mask (SSE2/AVX2), or
    /// `_mm512_abs_ps` (AVX-512). Used by the new
    /// `DistanceMetric::Manhattan` collection setting.
    fn manhattan_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
    }

    /// Normalise `a` in-place to unit L2 norm. Returns silently
    /// without modifying `a` when the L2 norm is zero (a zero vector
    /// has no meaningful direction). Default impl runs a scalar
    /// `l2_norm` then a scalar divide; SIMD backends benefit because
    /// both passes vectorise.
    fn normalize_in_place(&self, a: &mut [f32]) {
        let norm = self.l2_norm(a);
        if norm == 0.0 || !norm.is_finite() {
            return;
        }
        let inv = 1.0 / norm;
        for x in a.iter_mut() {
            *x *= inv;
        }
    }

    /// Element-wise `a[i] += b[i]`. Default impl is a scalar loop;
    /// SIMD backends override using `_mm256_add_ps` / `vaddq_f32` /
    /// `f32x4_add`.
    fn add_assign(&self, a: &mut [f32], b: &[f32]) {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        for (x, y) in a.iter_mut().zip(b.iter()) {
            *x += *y;
        }
    }

    /// Element-wise `a[i] -= b[i]`. Default impl is a scalar loop.
    fn sub_assign(&self, a: &mut [f32], b: &[f32]) {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        for (x, y) in a.iter_mut().zip(b.iter()) {
            *x -= *y;
        }
    }

    /// Element-wise `a[i] *= s`. Default impl is a scalar loop.
    fn scale(&self, a: &mut [f32], s: f32) {
        for x in a.iter_mut() {
            *x *= s;
        }
    }

    /// Returns `Some((argmin, min))` over a non-empty slice, `None`
    /// for an empty slice. NaN values follow `f32::partial_cmp`
    /// semantics — they propagate as the larger element so a NaN
    /// never wins the argmin race.
    fn horizontal_min_index(&self, a: &[f32]) -> Option<(usize, f32)> {
        if a.is_empty() {
            return None;
        }
        let mut min_idx = 0usize;
        let mut min_val = a[0];
        for (i, &v) in a.iter().enumerate().skip(1) {
            if v < min_val {
                min_val = v;
                min_idx = i;
            }
        }
        Some((min_idx, min_val))
    }

    /// Quantize `src` into `dst` as 8-bit unsigned codes:
    /// `dst[i] = clamp(round((src[i] - offset) / scale), 0, levels - 1)`.
    ///
    /// Default implementation is a scalar loop; SIMD backends benefit
    /// because every step (subtract, multiply by `1/scale`, clamp,
    /// round, narrow-convert) vectorises cleanly.
    ///
    /// Caller invariants: `dst.len() == src.len()`, `levels > 0`.
    /// Out-of-range inputs (NaN, infinite, magnitudes past the clamp
    /// boundary) are silently clamped.
    ///
    /// Edge case: `scale == 0.0` (constant-valued dataset where
    /// `max - min == 0`) writes all-zero codes. Mathematically every
    /// input maps to the same value, so any constant code is
    /// correct — 0 is the natural choice and it preserves the
    /// dst-buffer's initial state. This matches the semantics of
    /// the pre-7f scalar loop, which divided by zero (producing NaN)
    /// then clamped to 0 — without the panic risk that
    /// `1.0 / 0.0 → ∞ * (s - offset)` gives in debug builds.
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
            // Constant-input short circuit; see method docs.
            for d in dst.iter_mut() {
                *d = 0;
            }
            return;
        }
        let inv_scale = 1.0 / scale;
        let max_level = (levels - 1) as f32;
        for (s, d) in src.iter().zip(dst.iter_mut()) {
            let normalised = (s - offset) * inv_scale;
            let clamped = normalised.clamp(0.0, max_level);
            *d = clamped.round() as u8;
        }
    }

    /// Dequantize `src` back to f32 as `dst[i] = offset + src[i] * scale`.
    ///
    /// Default implementation is a scalar loop; SIMD backends benefit
    /// from the load-widen-FMA pattern (one cycle per lane on every
    /// CPU since AVX2).
    ///
    /// Caller invariants: `dst.len() == src.len()`.
    fn dequantize_u8_to_f32(&self, src: &[u8], dst: &mut [f32], scale: f32, offset: f32) {
        debug_assert_eq!(dst.len(), src.len(), "Buffers must have same length");
        for (s, d) in src.iter().zip(dst.iter_mut()) {
            *d = offset + (*s as f32) * scale;
        }
    }

    /// Diagnostic name. Must be a constant `&'static str`; surfaced
    /// by `dispatch::selected_backend_name()` and the startup log.
    fn name(&self) -> &'static str;
}
