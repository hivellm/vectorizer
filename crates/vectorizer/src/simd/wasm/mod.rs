//! WASM SIMD128 backend.
//!
//! One backend in this family: [`simd128::Wasm128Backend`] using the
//! `v128` type and `core::arch::wasm32` intrinsics for 4 f32 lanes
//! per cycle.
//!
//! Unlike x86/aarch64 where SIMD availability is detected at runtime,
//! wasm SIMD is a COMPILE-TIME feature
//! (`-C target-feature=+simd128` plus matching
//! `cfg(target_feature = "simd128")` in the source). Browsers /
//! engines that don't support SIMD128 fail the module instantiation,
//! which is the desired behaviour for this target — a WASM build
//! either has SIMD128 throughout or it doesn't, no graceful runtime
//! fallback like x86 has.
//!
//! Build setup for downstream consumers:
//!
//! ```toml
//! # .cargo/config.toml
//! [target.wasm32-unknown-unknown]
//! rustflags = ["-C", "target-feature=+simd128"]
//! ```
//!
//! Or one-shot:
//!
//! ```sh
//! RUSTFLAGS="-C target-feature=+simd128" \
//!     cargo build --target wasm32-unknown-unknown --features simd-wasm
//! ```

#[cfg(all(feature = "simd-wasm", target_feature = "simd128"))]
pub mod simd128;
