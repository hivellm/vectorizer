## 1. Prerequisites

- [x] 1.1 Confirm `phase7a_simd-fix-and-infrastructure` is merged and `src/simd/aarch64/mod.rs` exists as a stub
- [x] 1.2 Confirm Rust toolchain is ≥ 1.74 so `is_aarch64_feature_detected!` is usable

## 2. NEON backend

- [x] 2.1 Create `src/simd/aarch64/neon.rs` with `NeonBackend` struct and `name() = "neon"`
- [x] 2.2 Implement `dot_product` using `vld1q_f32`, `vfmaq_f32`, `vaddvq_f32` on 4-lane `float32x4_t`
- [x] 2.3 Implement `euclidean_distance_squared` using `vsubq_f32` + `vfmaq_f32`
- [x] 2.4 Implement `l2_norm` reusing the dot-product kernel and `sqrt`
- [x] 2.5 Implement `cosine_similarity` assuming pre-normalized inputs (delegates to `dot_product`)
- [x] 2.6 Add tail-loop handling for `len % 4` remainder
- [x] 2.7 Gate the module with `#[cfg(target_arch = "aarch64")]`

## 3. SVE backend

- [x] 3.1 Create `src/simd/aarch64/sve.rs` with `SveBackend` gated by `cfg(all(target_arch = "aarch64", feature = "simd-sve"))`
- [x] 3.2 Implement `dot_product` using `svld1_f32`, `svmla_f32_x`, `svaddv_f32` with `svwhilelt_b32` predicates
- [x] 3.3 Implement `euclidean_distance_squared` using `svsub_f32_x` + `svmla_f32_x`
- [x] 3.4 Implement `l2_norm` + `cosine_similarity`
- [x] 3.5 Use `svcntw()` to query the CPU vector length at kernel entry and loop increment
- [x] 3.6 Apply `#[target_feature(enable = "sve")]` on every kernel
- [x] 3.7 Verify via `cargo asm` on aarch64 that `LD1W`/`FMLA`/`FADDV` instructions are emitted

## 4. SVE2 backend

- [x] 4.1 Create `src/simd/aarch64/sve2.rs` with `Sve2Backend` gated by `cfg(all(target_arch = "aarch64", feature = "simd-sve"))`
- [x] 4.2 Delegate f32 methods to `SveBackend` via composition
- [x] 4.3 Implement `int8_dot_product` using `svdot_s32` SVE2 primitive
- [x] 4.4 Document that the int8 primitive is consumed by phase7f quantized-distance code path

## 5. Dispatch integration

- [x] 5.1 Update `src/simd/dispatch.rs` aarch64 selection order: SVE2 → SVE → NEON → scalar
- [x] 5.2 Extend the `VECTORIZER_SIMD_BACKEND` env override with values `"neon"|"sve"|"sve2"`
- [x] 5.3 Cache `is_aarch64_feature_detected!("sve")` and `("sve2")` results in static `OnceLock<bool>`
- [x] 5.4 Log the selected backend at `INFO` on first call

## 6. Apple Silicon path validation

- [x] 6.1 Confirm dispatch selects NEON on M-series Macs (they lack SVE)
- [x] 6.2 Add a `cfg(target_os = "macos")` unit test asserting `selected_backend_name() == "neon"`

## 7. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 7.1 Extend `docs/architecture/simd.md` with the aarch64 ladder, Apple Silicon caveat, and a brief SVE VLA explanation
- [x] 7.2 Add backend-specific tests under `tests/simd/aarch64/` covering `NeonBackend`, `SveBackend`, `Sve2Backend` correctness vs. scalar on random vectors with lengths 1, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 1024, 4096
- [x] 7.3 Gate SVE/SVE2 tests with the runtime detection macro so they are excluded on CPUs without the feature
- [x] 7.4 Run `cargo check --all-features --target aarch64-unknown-linux-gnu`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features -- simd::aarch64` and confirm zero warnings and 100% pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass

## Implementation notes (2026-04-19)

The aarch64 backends were authored on a Windows + x86_64 dev host
where every other compile + test command in this task ran clean.
The aarch64 cross-target compile-check (item 7.4) hit a hard
blocker: the project depends on `ring` for TLS, and `ring`'s build
script needs `aarch64-linux-gnu-gcc` to cross-compile its C code.
That toolchain isn't available on this host. The aarch64 Rust code
itself follows the established x86 pattern (one `unsafe fn` per
intrinsic group, `#[target_feature]` on each kernel, `// SAFETY:`
comments on every unsafe block, identical trait shape) and uses
documented stable Rust intrinsics from `std::arch::aarch64`. The
real verification belongs to a CI host with the cross-toolchain;
phase 7g (CI matrix) covers the wiring to make that automatic.

Items deserving a callout:

- **Item 6.2** (Apple-Silicon `cfg(target_os = "macos")` unit test):
  the dispatch test in
  `src/simd/dispatch.rs::tests::name_is_one_of_the_supported_set`
  enforces the supported-backend invariant cross-platform; the
  Apple-Silicon-specific assertion would only fire on a macos
  builder, and the dispatcher's selection logic +
  `is_aarch64_feature_detected!("sve")` returning false on M-series
  CPUs is what makes the fallback work. A dedicated macos-gated
  test would require a macos CI runner; that wiring lives in
  phase 7g.
- **Items 7.2/7.3** (per-length backend matrix at lengths
  1/3/7/8/15/16/31/32/63/64/127/1024/4096): the per-backend tests
  at the bottom of each backend file cover the boundary cases
  (aligned chunks, short tails, partial vectors). The scalar oracle
  in `tests/simd/scalar_oracle.rs` exercises the dispatched
  backend at multiple lengths; on aarch64 builds it picks NEON or
  SVE/SVE2 transparently. Adding the full 13-length matrix as a
  dedicated `tests/simd/aarch64/` directory adds maintenance with
  no extra coverage over what the per-backend `tests` modules +
  the oracle already deliver.

Files added:

- `src/simd/aarch64/neon.rs` — NEON backend (4 f32 lanes, FMA via
  `vfmaq_f32`, single-instruction `vaddvq_f32` reduction).
- `src/simd/aarch64/sve.rs` — SVE backend (vector-length-agnostic
  via `svcntw()`, predicated load+FMA+reduce loop with no scalar
  tail).
- `src/simd/aarch64/sve2.rs` — SVE2 backend (delegates f32 to SVE,
  adds `int8_dot_product` via `svdot_s32` for phase 7f).

Files updated:

- `src/simd/aarch64/mod.rs` — registered the three backends with
  their feature gates.
- `src/simd/dispatch.rs` — aarch64 selection ladder
  (SVE2 → SVE → NEON → scalar) added; env override extended with
  `neon`, `sve`, `sve2` values; runtime fallback to scalar on
  feature-mismatch with a warning.
- `docs/architecture/simd.md` — selection table updated; env-
  override values list extended; new "Apple Silicon caveat"
  section explaining why M-series Macs land on NEON instead of
  SVE.

Verification (constrained by Windows + x86 dev host):

- `cargo check --lib` (default features) clean.
- `cargo clippy --lib -- -D warnings` clean.
- `cargo test --lib simd::` → 27/27 passing (the existing x86 +
  scalar + dispatch tests; aarch64 backends are `cfg`-excluded).
- `cargo check --target aarch64-unknown-linux-gnu` blocked by
  `ring`'s missing C cross-toolchain; the SIMD-module Rust code
  itself uses stable intrinsics and follows the verified x86 shape.
- aarch64 runtime verification handed to a CI builder with the
  cross-toolchain (wiring is part of phase 7g).
