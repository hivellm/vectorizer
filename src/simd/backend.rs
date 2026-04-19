//! The `SimdBackend` trait — the contract every per-ISA implementation
//! satisfies.
//!
//! v1 of the trait covers the four `f32` primitives the hot paths
//! exercise today: `dot_product`, `euclidean_distance_squared`,
//! `cosine_similarity`, and `l2_norm`. Quantization-specific
//! primitives (u8 PQ codes, sparse vectors) land in phase7f and will
//! extend this trait with new methods carrying their own scalar
//! fallbacks — adding a method here without a matching scalar
//! implementation breaks every backend at once, which is the desired
//! tripwire.
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

    /// Diagnostic name. Must be a constant `&'static str`; surfaced
    /// by `dispatch::selected_backend_name()` and the startup log.
    fn name(&self) -> &'static str;
}
