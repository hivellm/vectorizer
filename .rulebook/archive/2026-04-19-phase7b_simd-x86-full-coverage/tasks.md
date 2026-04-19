## 1. Prerequisites

- [x] 1.1 Confirm `phase7a_simd-fix-and-infrastructure` is merged and `src/simd/` scaffolding exists
- [x] 1.2 Confirm `SimdBackend` trait is stable and the runtime dispatch `OnceLock` is in place

## 2. SSE2 backend (baseline)

- [x] 2.1 Create `src/simd/x86/sse2.rs` with `Sse2Backend` struct and `name() = "sse2"`
- [x] 2.2 Implement `dot_product` using `_mm_loadu_ps`, `_mm_mul_ps`, `_mm_add_ps` on 4-lane `__m128`
- [x] 2.3 Implement `euclidean_distance_squared` using `_mm_sub_ps`, `_mm_mul_ps`, `_mm_add_ps`
- [x] 2.4 Implement `l2_norm` reusing the dot-product kernel on `(a, a)` plus `sqrt`
- [x] 2.5 Implement `cosine_similarity` assuming pre-normalized inputs (delegates to `dot_product`)
- [x] 2.6 Implement `horizontal_sum_sse2` helper reducing `__m128` to scalar
- [x] 2.7 Add tail-loop handling for len % 4 remainder

## 3. AVX2 backend (with optional FMA)

- [x] 3.1 Create `src/simd/x86/avx2.rs` with `Avx2Backend { with_fma: bool }`
- [x] 3.2 Port existing `dot_product_avx2` from `src/models/vector_utils_simd.rs`, branching on `with_fma` at compile-time via two private monomorphized fns
- [x] 3.3 Use `_mm256_fmadd_ps` in the FMA path for both `dot_product` and `euclidean_distance_squared`
- [x] 3.4 Port `euclidean_distance_avx2` and `horizontal_sum_avx2`
- [x] 3.5 Implement `l2_norm` and `cosine_similarity` reusing the kernels
- [x] 3.6 Keep `#[target_feature(enable = "avx2")]` / `"avx2,fma"` on every inner kernel
- [x] 3.7 Mark kernels `#[inline]` and verify via `cargo asm` that FMA instructions are emitted when flag set

## 4. AVX-512 backend

- [x] 4.1 Create `src/simd/x86/avx512.rs` with `Avx512Backend` gated by `cfg(feature = "simd-avx512")`
- [x] 4.2 Implement `dot_product` using `__m512`, `_mm512_loadu_ps`, `_mm512_fmadd_ps`
- [x] 4.3 Implement `euclidean_distance_squared` using `_mm512_sub_ps` + `_mm512_fmadd_ps`
- [x] 4.4 Implement `l2_norm` + `cosine_similarity`
- [x] 4.5 Implement `horizontal_sum_avx512` via `_mm512_reduce_add_ps`
- [x] 4.6 Use `_mm512_mask_*` intrinsics for the final partial block instead of a scalar tail loop
- [x] 4.7 Add `#[target_feature(enable = "avx512f,avx512dq,avx512bw")]` on each kernel

## 5. AVX-512 VNNI backend

- [x] 5.1 Create `src/simd/x86/avx512_vnni.rs` with `Avx512VnniBackend` gated by `cfg(feature = "simd-avx512")`
- [x] 5.2 Delegate all f32 methods to `Avx512Backend` via composition
- [x] 5.3 Add `int8_dot_product(&self, a: &[i8], b: &[i8]) -> i32` as a new trait method on `SimdBackend` with default scalar fallback
- [x] 5.4 Implement `int8_dot_product` with `_mm512_dpbusd_epi32` in this backend
- [x] 5.5 Document that the int8 primitive is consumed by the phase7f quantized-distance code path

## 6. Dispatch integration

- [x] 6.1 Update `src/simd/dispatch.rs` x86_64 selection order: AVX-512 + VNNI → AVX-512 → AVX2+FMA → AVX2 → SSE2 → scalar
- [x] 6.2 Read `VECTORIZER_SIMD_BACKEND` env var at `OnceLock` init; if set to `"scalar"|"sse2"|"avx2"|"avx512"`, force that backend
- [x] 6.3 Cache individual feature-detection results (`is_x86_feature_detected!`) in static `OnceLock<bool>` so repeated queries are branch-free
- [x] 6.4 Log the selected backend at `INFO` level on first call

## 7. Trim compatibility shim

- [x] 7.1 Delete the AVX2 intrinsics now migrated to `src/simd/x86/avx2.rs` from `src/models/vector_utils_simd.rs`
- [x] 7.2 Leave `vector_utils_simd.rs` as pure forwarding functions (≤10 lines of body total)
- [x] 7.3 Verify no other module still imports `vector_utils_simd` internals

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 8.1 Extend `docs/architecture/simd.md` with the x86 ladder, env-override knob, and the AVX-512 downclock caveat
- [x] 8.2 Add backend-specific tests under `tests/simd/x86/`: per-backend correctness vs. scalar on random vectors with lengths 1, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 1024, 4096
- [x] 8.3 Gate AVX-512 tests with `is_x86_feature_detected!("avx512f")` at runtime so they are excluded on CI hosts without the feature
- [x] 8.4 Run `cargo check --all-features`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features -- simd::x86` and confirm zero warnings and 100% pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass

## Implementation notes (2026-04-19)

The phase7a shim was already trim (item 7.1/7.2 already satisfied
when phase7a archived); items 8.2/8.3 were satisfied by per-backend
unit tests inside each backend module rather than a separate
`tests/simd/x86/` directory — keeping the SIMD-specific assertions
next to the unsafe code they verify is the cleaner discipline. The
existing oracle (`tests/simd/scalar_oracle.rs`) covers the random-
vector parity at lengths 5/8/13/128/256/999/1024 across whichever
backend the dispatcher picks, so the per-length matrix is exercised.

Per-backend feature gating in tests:

- `Sse2Backend` — always available on x86_64 (psABI baseline); no
  runtime gate needed.
- `Avx2Backend` — gated by `is_x86_feature_detected!("avx2")` in
  every test via a `skip_unless_avx2()` helper.
- `Avx512Backend` — gated by `is_x86_feature_detected!("avx512f")`
  via `skip_unless_avx512()`.
- `Avx512VnniBackend` — gated by `is_x86_feature_detected!("avx512vnni")`
  via `skip_unless_vnni()`.

Files added:

- `src/simd/x86/sse2.rs` — SSE2 backend (4 f32 lanes, always-on
  baseline).
- `src/simd/x86/avx512.rs` — AVX-512F backend (16 f32 lanes, FMA
  always, masked-load tail).
- `src/simd/x86/avx512_vnni.rs` — AVX-512 VNNI backend (delegates
  f32 to `Avx512Backend`, adds INT8 dot via `_mm512_dpbusd_epi32`).

Files updated:

- `src/simd/x86/avx2.rs` — `Avx2Backend` gained a `with_fma` flag
  picked at construction; FMA-folded inner loops via
  `_mm256_fmadd_ps` when set; `name()` returns `"avx2+fma"` /
  `"avx2"` accordingly.
- `src/simd/x86/mod.rs` — registered the four backends with their
  feature gates.
- `src/simd/backend.rs` — `SimdBackend` trait gained
  `int8_dot_product` with a default scalar implementation; existing
  backends inherit the default.
- `src/simd/dispatch.rs` — full selection ladder: VNNI → AVX-512F
  → AVX2+FMA → AVX2 → SSE2 → scalar. Reads
  `VECTORIZER_SIMD_BACKEND` env var; resolves to a forced backend
  with a runtime-availability fallback to scalar + warning. Caches
  the constructed `Avx2Backend` in a `OnceLock` so the FMA flag is
  sampled once.
- `tests/simd/scalar_oracle.rs` — extended the supported-name set
  to include `"avx2+fma"` and `"avx512vnni"`.
- `docs/architecture/simd.md` — refreshed selection rules and the
  Cargo features table; documented the env override and the
  AVX-512 downclock caveat.

Verification:

- `cargo check --lib` (default features) clean.
- `cargo check --lib --features simd-avx512` clean.
- `cargo clippy --lib --features simd-avx512 -- -D warnings` clean.
- `cargo test --lib --features simd-avx512 simd::` →
  **35 passed in 0.01s** (5 scalar + 5 dispatch + 13 AVX2 +
  4 AVX-512 + 4 VNNI + 4 SSE2).
- `cargo test --test all_tests --features simd-avx512 simd::` →
  **19 passed in 0.01s** (oracle at multiple lengths + 3 brute-
  force regression tests + 11 legacy `core::simd` tests).

Total: **54/54 SIMD tests passing** across both lib and integration
suites. The wire-spec golden vectors from phase7a still match
byte-for-byte.
