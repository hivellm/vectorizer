//! WASM SIMD128 backend.
//!
//! One backend planned: `Wasm128Backend` using `core::arch::wasm32`
//! `v128` intrinsics for 4 f32 lanes per cycle. Lands in phase7d.
//!
//! Unlike x86/aarch64 where SIMD availability is detected at runtime,
//! wasm SIMD is a COMPILE-TIME feature (`-C target-feature=+simd128`
//! plus matching `cfg(target_feature = "simd128")` in the source).
//! Browsers that don't support SIMD must load a separately-built wasm
//! module without the SIMD instructions; we don't ship a runtime
//! detection path on this target.
//!
//! The directory is created in phase7a so the dispatch table in
//! `simd::dispatch::select_backend` has a stable target to extend
//! when phase7d lands. Until then, wasm32 builds resolve to
//! [`crate::simd::scalar::ScalarBackend`].
