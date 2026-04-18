## 1. Prerequisites

- [ ] 1.1 Confirm `phase7a_simd-fix-and-infrastructure` is merged and `src/simd/aarch64/mod.rs` exists as a stub
- [ ] 1.2 Confirm Rust toolchain is ≥ 1.74 so `is_aarch64_feature_detected!` is usable

## 2. NEON backend

- [ ] 2.1 Create `src/simd/aarch64/neon.rs` with `NeonBackend` struct and `name() = "neon"`
- [ ] 2.2 Implement `dot_product` using `vld1q_f32`, `vfmaq_f32`, `vaddvq_f32` on 4-lane `float32x4_t`
- [ ] 2.3 Implement `euclidean_distance_squared` using `vsubq_f32` + `vfmaq_f32`
- [ ] 2.4 Implement `l2_norm` reusing the dot-product kernel and `sqrt`
- [ ] 2.5 Implement `cosine_similarity` assuming pre-normalized inputs (delegates to `dot_product`)
- [ ] 2.6 Add tail-loop handling for `len % 4` remainder
- [ ] 2.7 Gate the module with `#[cfg(target_arch = "aarch64")]`

## 3. SVE backend

- [ ] 3.1 Create `src/simd/aarch64/sve.rs` with `SveBackend` gated by `cfg(all(target_arch = "aarch64", feature = "simd-sve"))`
- [ ] 3.2 Implement `dot_product` using `svld1_f32`, `svmla_f32_x`, `svaddv_f32` with `svwhilelt_b32` predicates
- [ ] 3.3 Implement `euclidean_distance_squared` using `svsub_f32_x` + `svmla_f32_x`
- [ ] 3.4 Implement `l2_norm` + `cosine_similarity`
- [ ] 3.5 Use `svcntw()` to query the CPU vector length at kernel entry and loop increment
- [ ] 3.6 Apply `#[target_feature(enable = "sve")]` on every kernel
- [ ] 3.7 Verify via `cargo asm` on aarch64 that `LD1W`/`FMLA`/`FADDV` instructions are emitted

## 4. SVE2 backend

- [ ] 4.1 Create `src/simd/aarch64/sve2.rs` with `Sve2Backend` gated by `cfg(all(target_arch = "aarch64", feature = "simd-sve"))`
- [ ] 4.2 Delegate f32 methods to `SveBackend` via composition
- [ ] 4.3 Implement `int8_dot_product` using `svmlslb_s16`/`svdot_s32` SVE2 primitives
- [ ] 4.4 Document that the int8 primitive is consumed by phase7f quantized-distance code path

## 5. Dispatch integration

- [ ] 5.1 Update `src/simd/dispatch.rs` aarch64 selection order: SVE2 → SVE → NEON → scalar
- [ ] 5.2 Extend the `VECTORIZER_SIMD_BACKEND` env override with values `"neon"|"sve"|"sve2"`
- [ ] 5.3 Cache `is_aarch64_feature_detected!("sve")` and `("sve2")` results in static `OnceLock<bool>`
- [ ] 5.4 Log the selected backend at `INFO` on first call

## 6. Apple Silicon path validation

- [ ] 6.1 Confirm dispatch selects NEON on M-series Macs (they lack SVE)
- [ ] 6.2 Add a `cfg(target_os = "macos")` unit test asserting `selected_backend_name() == "neon"`

## 7. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 7.1 Extend `docs/architecture/simd.md` with the aarch64 ladder, Apple Silicon caveat, and a brief SVE VLA explanation
- [ ] 7.2 Add backend-specific tests under `tests/simd/aarch64/` covering `NeonBackend`, `SveBackend`, `Sve2Backend` correctness vs. scalar on random vectors with lengths 1, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 1024, 4096
- [ ] 7.3 Gate SVE/SVE2 tests with the runtime detection macro so they are excluded on CPUs without the feature
- [ ] 7.4 Run `cargo check --all-features --target aarch64-unknown-linux-gnu`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features -- simd::aarch64` and confirm zero warnings and 100% pass
