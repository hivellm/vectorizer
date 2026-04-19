//! SIMD-accelerated vector primitives with runtime CPU dispatch.
//!
//! ## Why this module exists
//!
//! Before phase 7, SIMD lived only in `src/models/vector_utils_simd.rs`
//! as an AVX2 fast path for three primitives — every other ISA, every
//! other primitive, and every other call site went scalar. This
//! module is the central dispatch layer that future phases (7b–7g)
//! extend: one [`backend::SimdBackend`] trait, one [`dispatch::backend`]
//! cache, one set of scalar oracles, and one set of per-ISA backends.
//!
//! ## Architecture (one-liner)
//!
//! ```text
//! call site ──► simd::cosine_similarity (this module)
//!                       │
//!                       ▼
//!              dispatch::backend()  ◄── OnceLock
//!                       │
//!         ┌─────────────┼─────────────┐
//!         ▼             ▼             ▼
//!    Avx2Backend   ScalarBackend   ... (other ISAs in 7b–7d)
//! ```
//!
//! Read [`dispatch`] for the selection rules and
//! [`scalar::ScalarBackend`] for the correctness oracle. The
//! per-ISA backends live in `simd::x86`, `simd::aarch64`, `simd::wasm`.
//!
//! ## Public API
//!
//! Most callers want the four convenience functions exported from
//! this module — they hide the backend lookup behind a normal
//! function call. Use the trait directly only if you want to bind to
//! a specific backend (testing, benchmarking).
//!
//! See [`docs/architecture/simd.md`](../../docs/architecture/simd.md)
//! for the full design.

pub mod backend;
pub mod dispatch;
pub mod scalar;

#[cfg(target_arch = "x86_64")]
pub mod x86;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use backend::SimdBackend;
pub use dispatch::{backend, selected_backend_name};

// ── Convenience functions ────────────────────────────────────────────

/// Sum of pairwise products: `∑ a[i] * b[i]`.
///
/// Routes through the cached [`dispatch::backend`] — first call
/// resolves the per-CPU backend, subsequent calls are a single
/// indirect call. Mismatched-length slices are a debug-asserted
/// caller bug.
#[inline]
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    backend().dot_product(a, b)
}

/// `sqrt(∑ (a[i] - b[i])²)` — Euclidean distance between two equal-
/// length vectors. If you need the squared distance (cheaper, no
/// `sqrt`), call [`euclidean_distance_squared`] directly.
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    backend().euclidean_distance_squared(a, b).sqrt()
}

/// `∑ (a[i] - b[i])²` — Euclidean SQUARED distance. Use this when
/// comparing distances; the `sqrt` is monotonic so the ranking is
/// preserved and you save the call.
#[inline]
pub fn euclidean_distance_squared(a: &[f32], b: &[f32]) -> f32 {
    backend().euclidean_distance_squared(a, b)
}

/// Cosine similarity ASSUMING pre-normalised inputs — implemented as
/// a clamped dot product (`dot.clamp(-1.0, 1.0)`). If your vectors
/// are not unit-length, normalise first or call
/// `models::DistanceCalculator::cosine_similarity`.
#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    backend().cosine_similarity(a, b)
}

/// `sqrt(∑ a[i]²)` — L2 norm of a single vector.
#[inline]
pub fn l2_norm(a: &[f32]) -> f32 {
    backend().l2_norm(a)
}

/// Normalise `a` in-place to unit L2 norm. No-op on a zero vector
/// (a zero vector has no meaningful direction; the alternative is
/// returning NaN, which propagates badly through downstream math).
#[inline]
pub fn normalize_in_place(a: &mut [f32]) {
    backend().normalize_in_place(a);
}

/// Manhattan (L1) distance: `∑ |a[i] - b[i]|`.
#[inline]
pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    backend().manhattan_distance(a, b)
}

/// Element-wise in-place `a[i] += b[i]`.
#[inline]
pub fn add_assign(a: &mut [f32], b: &[f32]) {
    backend().add_assign(a, b);
}

/// Element-wise in-place `a[i] -= b[i]`.
#[inline]
pub fn sub_assign(a: &mut [f32], b: &[f32]) {
    backend().sub_assign(a, b);
}

/// Element-wise in-place `a[i] *= s`.
#[inline]
pub fn scale(a: &mut [f32], s: f32) {
    backend().scale(a, s);
}

/// Returns `Some((argmin, min))` over `a`, or `None` when `a` is
/// empty. NaN propagates as the larger element via
/// `f32::partial_cmp` semantics.
#[inline]
pub fn horizontal_min_index(a: &[f32]) -> Option<(usize, f32)> {
    backend().horizontal_min_index(a)
}
