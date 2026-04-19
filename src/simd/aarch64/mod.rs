//! aarch64 SIMD backends.
//!
//! Two backends are planned for this family:
//!
//! - `neon::NeonBackend` — NEON 128-bit registers, 4 f32 lanes.
//!   Lands in phase7c.
//! - `sve::SveBackend` — SVE variable-length registers (most often
//!   256 or 512 bits on production silicon). Lands in phase7c
//!   alongside NEON; runtime detection prefers SVE when available.
//!
//! The directory is created in phase7a so the dispatch table in
//! `simd::dispatch::select_backend` has a stable target to extend
//! when phase7c lands the actual implementations. Until then,
//! aarch64 builds resolve to [`crate::simd::scalar::ScalarBackend`].
//!
//! Backends are gated by both `cfg(target_arch = "aarch64")` (this
//! module's parent gate in `simd::mod`) and the per-feature flag
//! (`simd-neon`, `simd-sve`) in `Cargo.toml`. Disabling a feature
//! shrinks the binary; runtime detection still happens but returns
//! the next-best backend.
