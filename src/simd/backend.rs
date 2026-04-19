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

    /// Diagnostic name. Must be a constant `&'static str`; surfaced
    /// by `dispatch::selected_backend_name()` and the startup log.
    fn name(&self) -> &'static str;
}
