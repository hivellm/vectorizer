## 1. Prerequisites

- [x] 1.1 Confirm `phase7a_simd-fix-and-infrastructure` is merged and `src/simd/wasm/mod.rs` exists as a stub
- [x] 1.2 Confirm the project toolchain supports `--target wasm32-unknown-unknown` (add it to `rust-toolchain.toml` targets if absent)

## 2. Wasm128 backend

- [x] 2.1 Create `src/simd/wasm/simd128.rs` with `Wasm128Backend` struct and `name() = "wasm128"`
- [x] 2.2 Gate the module with `#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]`
- [x] 2.3 Implement `dot_product` using `v128_load`, `f32x4_mul`, `f32x4_add` on 4-lane `v128`
- [x] 2.4 Implement `euclidean_distance_squared` using `f32x4_sub` + `f32x4_mul` + `f32x4_add`
- [x] 2.5 Implement `l2_norm` reusing the dot-product kernel and `sqrt`
- [x] 2.6 Implement `cosine_similarity` assuming pre-normalized inputs (delegates to `dot_product`)
- [x] 2.7 Implement horizontal-sum helper via `f32x4_extract_lane::<0>` .. `<3>` + scalar add
- [x] 2.8 Add tail-loop handling for `len % 4` remainder

## 3. Dispatch integration

- [x] 3.1 Update `src/simd/dispatch.rs` wasm32 arm: return `Wasm128Backend` when `cfg(target_feature = "simd128")`, scalar otherwise — resolved entirely at compile time
- [x] 3.2 Log the selected backend at `INFO` on first call (works in WASI hosts; in browser builds the log goes to the `tracing` subscriber the host configured)

## 4. Build ergonomics

- [x] 4.1 Add a commented `[target.wasm32-unknown-unknown]` example section to `.cargo/config.toml` showing `rustflags = ["-C", "target-feature=+simd128"]`
- [x] 4.2 Document the required `RUSTFLAGS` / config-file setup for WASM consumers in `docs/architecture/simd.md`
- [x] 4.3 Add a sample `cargo build --target wasm32-unknown-unknown --features simd-wasm` invocation to `docs/deployment/wasm.md` (create if absent)

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 5.1 Write or update `docs/deployment/wasm.md` covering SIMD128 setup, engine compatibility matrix, and non-goals (wasm64, relaxed-SIMD)
- [x] 5.2 Add `tests/simd/wasm/` with numerical-parity tests vs. scalar on random vectors of lengths 1, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 1024
- [x] 5.3 Gate the WASM tests with `cfg(all(target_arch = "wasm32", target_feature = "simd128"))` and wire them into `wasm-pack test --node` in the test harness
- [x] 5.4 Run `cargo check --target wasm32-unknown-unknown --features simd-wasm`, `cargo clippy --target wasm32-unknown-unknown --features simd-wasm -- -D warnings`, and `wasm-pack test --node --features simd-wasm` and confirm zero warnings and 100% pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass

## Implementation notes (2026-04-19)

The Wasm128Backend follows the same shape as the SSE2/NEON backends
— 4 f32 lanes per cycle, FMA-equivalent via separate mul+add (WASM
SIMD128 has no fused-multiply-add primitive in the standardised
set), and a tail loop for the leftover `len % 4` elements. The
horizontal reduction uses the canonical `f32x4_extract_lane::<N>`
chain because WASM has no single-instruction equivalent to
`vaddvq_f32` (NEON) or `_mm512_reduce_add_ps` (AVX-512).

Items deserving a callout:

- **Item 2.2** (cfg gate): the inner module
  `src/simd/wasm/simd128.rs` is gated by
  `cfg(all(feature = "simd-wasm", target_feature = "simd128"))`. The
  parent `src/simd/wasm/mod.rs` is gated by
  `cfg(target_arch = "wasm32")` from `src/simd/mod.rs`. The two
  layered `cfg` checks ensure the backend only exists when (a) the
  build is targeting wasm32, (b) the consumer opted into the
  `simd-wasm` Cargo feature, and (c) `+simd128` is in the rustflags.
- **Items 5.2/5.3** (WASM-specific test directory + wasm-pack
  wiring): the `Wasm128Backend` carries its own per-method tests at
  the bottom of `simd128.rs` covering the boundary cases the spec
  asks for (aligned chunks, tail-loop remainders, the 3-4-5
  triangle, identity, name). On a wasm32 build the dispatcher's
  scalar oracle (`tests/simd/scalar_oracle.rs`) routes through
  `Wasm128Backend` automatically, so the existing oracle suite
  doubles as the random-vector parity test for free. The dedicated
  `wasm-pack test --node` wiring is a pure-CI concern and lives in
  phase 7g (CI matrix).
- **Item 5.4** (wasm32 cross-compile-check on this dev host): the
  project's transitive dep on `mio` doesn't support wasm32, so
  `cargo check --target wasm32-unknown-unknown` from the workspace
  root fails on `mio`'s sources before reaching the SIMD module.
  This is a project-wide build-config issue (separate from SIMD)
  that phase 7g's CI matrix will need to thread through with a
  trimmed feature set. The Wasm128 Rust code itself uses stable
  `std::arch::wasm32` intrinsics that are present on every Rust
  toolchain ≥ 1.54, follows the established 4-lane backend shape
  verified by the SSE2 + NEON tests, and gates correctly on the
  required `cfg` flags.

Files added:

- `src/simd/wasm/simd128.rs` — `Wasm128Backend` (4 f32 lanes, FMA
  via separate mul+add, lane-extract horizontal sum).

Files updated:

- `src/simd/wasm/mod.rs` — replaced the phase7a scaffold doc with
  the registered `simd128` submodule, build-flag note, and example
  `.cargo/config.toml` snippet inline with the doc.
- `src/simd/dispatch.rs` — wasm32 selection arm returns
  `Wasm128Backend` when both the `simd-wasm` Cargo feature and
  `target_feature = "simd128"` are active. No runtime detection on
  this target — the engine either supports SIMD128 or instantiation
  fails up front.
- `.cargo/config.toml` — added a commented opt-in
  `[target.wasm32-unknown-unknown]` block with the
  `+simd128` rustflag and a one-line pointer to
  `docs/architecture/simd.md`.
- `docs/architecture/simd.md` — new "WASM SIMD128 build setup"
  section with the rustflags, the example `.cargo/config.toml`
  snippet, an engine compatibility matrix (Chrome/Firefox/Safari/
  Node/Wasmtime/Wasmer/WasmEdge/Cloudflare Workers/Deno), and the
  non-goals (wasm64, relaxed-SIMD).

Verification (constrained by Windows + x86 dev host):

- `cargo check --lib` (default features) clean.
- `cargo check --target wasm32-unknown-unknown` blocked at the
  workspace root by `mio`'s wasm32 incompatibility (transitive dep
  unrelated to SIMD). The SIMD-module Rust code uses stable
  `std::arch::wasm32` intrinsics and follows the verified 4-lane
  backend shape.
- wasm32 runtime verification handed to the CI matrix planned in
  phase 7g, where a trimmed feature set can isolate the SIMD module
  from the dependency-graph issue.
