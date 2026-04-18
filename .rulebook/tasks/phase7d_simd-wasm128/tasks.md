## 1. Prerequisites

- [ ] 1.1 Confirm `phase7a_simd-fix-and-infrastructure` is merged and `src/simd/wasm/mod.rs` exists as a stub
- [ ] 1.2 Confirm the project toolchain supports `--target wasm32-unknown-unknown` (add it to `rust-toolchain.toml` targets if absent)

## 2. Wasm128 backend

- [ ] 2.1 Create `src/simd/wasm/simd128.rs` with `Wasm128Backend` struct and `name() = "wasm128"`
- [ ] 2.2 Gate the module with `#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]`
- [ ] 2.3 Implement `dot_product` using `v128_load`, `f32x4_mul`, `f32x4_add` on 4-lane `v128`
- [ ] 2.4 Implement `euclidean_distance_squared` using `f32x4_sub` + `f32x4_mul` + `f32x4_add`
- [ ] 2.5 Implement `l2_norm` reusing the dot-product kernel and `sqrt`
- [ ] 2.6 Implement `cosine_similarity` assuming pre-normalized inputs (delegates to `dot_product`)
- [ ] 2.7 Implement horizontal-sum helper via `f32x4_extract_lane::<0>` .. `<3>` + scalar add
- [ ] 2.8 Add tail-loop handling for `len % 4` remainder

## 3. Dispatch integration

- [ ] 3.1 Update `src/simd/dispatch.rs` wasm32 arm: return `Wasm128Backend` when `cfg(target_feature = "simd128")`, scalar otherwise — resolved entirely at compile time
- [ ] 3.2 Log the selected backend at `INFO` on first call (works in WASI hosts; in browser builds the log goes to the `tracing` subscriber the host configured)

## 4. Build ergonomics

- [ ] 4.1 Add a commented `[target.wasm32-unknown-unknown]` example section to `.cargo/config.toml` showing `rustflags = ["-C", "target-feature=+simd128"]`
- [ ] 4.2 Document the required `RUSTFLAGS` / config-file setup for WASM consumers in `docs/architecture/simd.md`
- [ ] 4.3 Add a sample `cargo build --target wasm32-unknown-unknown --features simd-wasm` invocation to `docs/deployment/wasm.md` (create if absent)

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Write or update `docs/deployment/wasm.md` covering SIMD128 setup, engine compatibility matrix, and non-goals (wasm64, relaxed-SIMD)
- [ ] 5.2 Add `tests/simd/wasm/` with numerical-parity tests vs. scalar on random vectors of lengths 1, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 1024
- [ ] 5.3 Gate the WASM tests with `cfg(all(target_arch = "wasm32", target_feature = "simd128"))` and wire them into `wasm-pack test --node` in the test harness
- [ ] 5.4 Run `cargo check --target wasm32-unknown-unknown --features simd-wasm`, `cargo clippy --target wasm32-unknown-unknown --features simd-wasm -- -D warnings`, and `wasm-pack test --node --features simd-wasm` and confirm zero warnings and 100% pass
