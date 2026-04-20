//! x86_64 SIMD backends.
//!
//! Four backends in this family:
//!
//! - [`sse2::Sse2Backend`] — 128-bit registers, 4 f32 lanes.
//!   Always-on baseline (SSE2 is in the x86_64 psABI).
//! - [`avx2::Avx2Backend`] — 256-bit registers, 8 f32 lanes; FMA
//!   fusion in the inner loops when the CPU advertises it.
//! - [`avx512::Avx512Backend`] — 512-bit registers, 16 f32 lanes,
//!   FMA always (AVX-512F includes the FMA primitive).
//! - [`avx512_vnni::Avx512VnniBackend`] — adds the
//!   `_mm512_dpbusd_epi32` INT8 dot-product primitive on top of the
//!   AVX-512F f32 path. Consumed by phase 7f's quantization work.
//!
//! Backends are gated by both `cfg(target_arch = "x86_64")` (this
//! module's parent gate in `simd::mod`) and the per-feature flag
//! (`simd-avx2`, `simd-avx512`) in `Cargo.toml`. Disabling a feature
//! shrinks the binary; runtime detection still happens but returns
//! the next-best backend.

pub mod sse2;

#[cfg(feature = "simd-avx2")]
pub mod avx2;

#[cfg(feature = "simd-avx512")]
pub mod avx512;

#[cfg(feature = "simd-avx512")]
pub mod avx512_vnni;
