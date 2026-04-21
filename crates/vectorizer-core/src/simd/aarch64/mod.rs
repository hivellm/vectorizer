//! aarch64 SIMD backends.
//!
//! Three backends in this family:
//!
//! - [`neon::NeonBackend`] — 128-bit registers, 4 f32 lanes. Always
//!   available (NEON is in the aarch64 psABI). FMA is the
//!   multiply-add primitive (no separate gate; every aarch64 CPU
//!   has it). Single-instruction horizontal reduction via
//!   `vaddvq_f32`.
//! - [`sve::SveBackend`] — Vector-length-agnostic. CPU decides the
//!   lane count; the kernel queries it via `svcntw()`. Predication
//!   eliminates the scalar tail loop entirely.
//! - [`sve2::Sve2Backend`] — Adds the single-instruction INT8 dot
//!   product (`svdot_s32`) on top of the SVE f32 path. Consumed by
//!   phase 7f's quantization work.
//!
//! Backends are gated by both `cfg(target_arch = "aarch64")` (this
//! module's parent gate in `simd::mod`) and the per-feature flag
//! (`simd-neon`, `simd-sve`) in `Cargo.toml`. Disabling a feature
//! shrinks the binary; runtime detection still happens but returns
//! the next-best backend.
//!
//! ## Apple Silicon caveat
//!
//! M1/M2/M3/M4 do NOT implement SVE — they are NEON-only. The
//! dispatcher correctly falls back to NEON on Apple Silicon.
//! Documented in `docs/architecture/simd.md` § "Apple Silicon".

#[cfg(feature = "simd-neon")]
pub mod neon;

#[cfg(feature = "simd-sve")]
pub mod sve;

#[cfg(feature = "simd-sve")]
pub mod sve2;
