# Proposal: phase7d_simd-wasm128

## Why

Vectorizer is distributed as a Rust crate that end-users may compile to WebAssembly for browser-side or edge-runtime deployments (Cloudflare Workers, Fastly, Deno Deploy, browser extensions doing local semantic search). Without WASM SIMD the WASM build runs every distance calculation scalar, which on a 768-dim embedding is ~4× slower than native WASM SIMD128 and ~16× slower than native AVX2 — untenable for any interactive search UX.

WebAssembly SIMD128 (the "fixed-width SIMD" proposal) is now in Phase 5 (standardised), shipping in every modern browser engine (Chrome 91+, Firefox 89+, Safari 16.4+, Node 16.4+) and every major serverless WASM runtime (Wasmtime, Wasmer, V8 isolates, WasmEdge). Rust exposes it via `std::arch::wasm32::*` and the `v128` type.

This task adds the WASM backend so the existing `src/simd/` dispatch picks SIMD128 on `target_arch = "wasm32"` when `target_feature = "simd128"` is enabled (the standard Cargo way to opt into WASM SIMD).

## What Changes

New file:

- `src/simd/wasm/simd128.rs` — `Wasm128Backend`. Uses the `v128` type and intrinsics `f32x4_splat`, `v128_load`, `v128_store`, `f32x4_mul`, `f32x4_add`, `f32x4_sub`, `f32x4_extract_lane`. The horizontal sum uses `f32x4_extract_lane` × 4 + scalar add (WASM has no single-instruction horizontal reduction; this is the idiomatic pattern and the engine vectorizes it well in practice).
- Update `src/simd/wasm/mod.rs` from phase7a's stub to expose `Wasm128Backend` when `cfg(target_feature = "simd128")`.

Dispatch update in `src/simd/dispatch.rs`:

- Selection order on wasm32: `Wasm128Backend` (compile-time gated on `target_feature = "simd128"`) → scalar.
- No runtime detection is needed or possible in WASM — SIMD128 availability is a compile-time contract with the host engine, negotiated during module instantiation. If the module was compiled with SIMD128 but the engine does not support it, instantiation fails, which is the correct behaviour.

Build ergonomics:

- Document the required flag in `docs/architecture/simd.md`: `RUSTFLAGS="-C target-feature=+simd128"` for `cargo build --target wasm32-unknown-unknown`, or via `.cargo/config.toml` `[target.wasm32-unknown-unknown]` section.
- Add an example `.cargo/config.toml` snippet (commented, opt-in) showing the recommended setup for WASM consumers.

Non-goals:

- No wasm64 support — the proposal is still experimental and Rust nightly-only.
- No relaxed-SIMD (`f32x4_relaxed_fma` etc.) — relaxed-SIMD is a separate proposal at Phase 4 and gives at most 10-15% extra; revisit once it reaches Phase 5.
- No JS bindings / `wasm-bindgen` surface changes — this is purely a compile target for the existing crate.

## Impact

- Affected specs: `.rulebook/tasks/phase7d_simd-wasm128/specs/simd-wasm/spec.md` (new).
- Affected code: new `src/simd/wasm/simd128.rs`; edits to `src/simd/wasm/mod.rs`, `src/simd/dispatch.rs`, `docs/architecture/simd.md`, optional snippet in `.cargo/config.toml`.
- Breaking change: NO. Gated by `target_arch = "wasm32"`; native builds unaffected.
- User benefit: 4× on the WASM target for every f32 SIMD operation, making browser-side semantic search viable on mid-range devices.
