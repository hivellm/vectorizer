//! x86_64 SIMD backends.
//!
//! Two backends planned for this family:
//!
//! - [`avx2::Avx2Backend`] — AVX2 + 256-bit registers, 8 f32 lanes.
//!   Lands in phase7a (this task).
//! - `avx512::Avx512Backend` — AVX-512F + 512-bit registers, 16 f32
//!   lanes. Lands in phase7b alongside FMA fusion.
//!
//! Backends are gated by both `cfg(target_arch = "x86_64")` (this
//! module's parent gate in `simd::mod`) and the per-feature flag
//! (`simd-avx2`, `simd-avx512`) in `Cargo.toml`. Disabling a feature
//! shrinks the binary; runtime detection still happens but returns
//! the next-best backend.

#[cfg(feature = "simd-avx2")]
pub mod avx2;
