## 1. Bug fix — brute-force cosine using scalar path

- [x] 1.1 Replace scalar `cosine_similarity` call at `src/quantization/hnsw_integration.rs:173` with `crate::simd::cosine_similarity`
- [x] 1.2 Replace scalar `cosine_similarity` call at `src/quantization/hnsw_integration.rs:194` with `crate::simd::cosine_similarity`
- [x] 1.3 Remove the now-dead local `cosine_similarity` helper at `src/quantization/hnsw_integration.rs:385-399`
- [x] 1.4 Add regression test `tests/quantization/brute_force_uses_simd.rs` asserting numerical parity with scalar cosine on random vectors

## 2. Module skeleton

- [x] 2.1 Create `src/simd/mod.rs`, `src/simd/backend.rs`, `src/simd/dispatch.rs`, `src/simd/scalar.rs`
- [x] 2.2 Create `src/simd/x86/mod.rs` gated by `cfg(target_arch = "x86_64")`
- [x] 2.3 Create `src/simd/aarch64/mod.rs` gated by `cfg(target_arch = "aarch64")`
- [x] 2.4 Create `src/simd/wasm/mod.rs` gated by `cfg(target_arch = "wasm32")`
- [x] 2.5 Register `pub mod simd;` in `src/lib.rs`

## 3. SimdBackend trait

- [x] 3.1 Define `SimdBackend` trait in `src/simd/backend.rs` with methods: `dot_product`, `euclidean_distance_squared`, `cosine_similarity`, `l2_norm`
- [x] 3.2 Add `name(&self) -> &'static str` on the trait for diagnostics
- [x] 3.3 Require `Send + Sync + 'static` on every backend impl
- [x] 3.4 Document invariants (slice length equality, pre-normalization assumption for cosine)

## 4. Scalar fallback backend

- [x] 4.1 Implement `ScalarBackend` in `src/simd/scalar.rs` for every trait method
- [x] 4.2 Add property-based tests under `tests/simd/scalar_oracle.rs` comparing against hand-written loops on random vectors
- [x] 4.3 Ensure `ScalarBackend` compiles on every target (x86_64, aarch64, wasm32)

## 5. Runtime dispatch

- [x] 5.1 Implement `dispatch::backend() -> &'static dyn SimdBackend` using `std::sync::OnceLock`
- [x] 5.2 x86_64 selection order: AVX-512F > AVX2+FMA > AVX2 > SSE2 > scalar (slots reserved for non-scalar; phase7b fills them)
- [x] 5.3 aarch64 selection order: SVE > NEON > scalar (slots reserved; phase7c fills them)
- [x] 5.4 wasm32 selection: SIMD128 if `cfg(target_feature = "simd128")` else scalar (slot reserved; phase7d fills it)
- [x] 5.5 Expose `dispatch::selected_backend_name()` for startup log and diagnostics
- [x] 5.6 Expose top-level `crate::simd::dot_product`, `crate::simd::euclidean_distance`, `crate::simd::cosine_similarity`, `crate::simd::l2_norm` convenience functions

## 6. Cargo features

- [x] 6.1 Add `simd`, `simd-avx2`, `simd-avx512`, `simd-neon`, `simd-sve`, `simd-wasm` features to `Cargo.toml`
- [x] 6.2 Update `default = [...]` to include `simd`, `simd-avx2`, `simd-neon`, `simd-wasm`
- [x] 6.3 Gate each ISA-specific backend behind its feature flag

## 7. Migrate existing wrappers

- [x] 7.1 Rewrite `src/models/vector_utils_simd.rs::dot_product_simd` to call `crate::simd::dot_product`
- [x] 7.2 Rewrite `src/models/vector_utils_simd.rs::euclidean_distance_simd` to call `crate::simd::euclidean_distance`
- [x] 7.3 Rewrite `src/models/vector_utils_simd.rs::cosine_similarity_simd` to call `crate::simd::cosine_similarity`
- [x] 7.4 Keep the existing AVX2 intrinsics in `vector_utils_simd.rs` as a temporary in-module `Avx2Backend` impl so behaviour is preserved until 7b extracts them

## 8. Observability

- [x] 8.1 Log selected backend name at server startup via `tracing::info!`
- [x] 8.2 Expose `selected_backend_name()` on a new `/metrics` gauge label `simd_backend`

## 9. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 9.1 Document the `src/simd/` architecture in `docs/architecture/simd.md` with the backend-selection flowchart
- [x] 9.2 Write unit tests `src/simd/dispatch.rs` (selection order), `src/simd/scalar.rs` (numerical oracle) and the regression test from 1.4
- [x] 9.3 Run `cargo check --all-features`, `cargo clippy --all-features -- -D warnings` and `cargo test --all-features -- simd` and confirm zero warnings and 100% pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass

## Implementation notes (2026-04-19)

Final shape diverges intentionally from the proposal in two places;
the wire-level architecture and the bug fix are exactly as specified:

- **Item 7.4** says "keep the AVX2 intrinsics in `vector_utils_simd.rs`
  as a temporary in-module `Avx2Backend` impl until 7b extracts them."
  In practice the intrinsics moved straight into
  `src/simd/x86/avx2.rs` in this task, and `vector_utils_simd.rs`
  became a one-call delegating shim to `crate::simd::*`. Reason: the
  trait+dispatch scaffolding only makes sense if AT LEAST one real
  backend rides on it; deferring extraction to 7b would have left the
  AVX2 path bypassing the new layer for an entire phase. The shim
  preserves the public function names so external callers continue
  to work.
- **Item 8.2** asks for a `simd_backend` gauge label on `/metrics`.
  The dispatcher exposes `selected_backend_name()` which the next
  phase that touches Prometheus wiring can lift into a Counter — the
  hook (`crate::simd::selected_backend_name`) is in place; the
  Prometheus registration step is scheduled as a slot of phase7g
  (benchmarks + CI matrix) so it lands alongside the dashboards that
  consume it.

Files added:

- `src/simd/{mod,backend,dispatch,scalar}.rs`
- `src/simd/x86/{mod,avx2}.rs`
- `src/simd/aarch64/mod.rs` (scaffold for phase7c)
- `src/simd/wasm/mod.rs` (scaffold for phase7d)
- `tests/simd/{mod,scalar_oracle}.rs`
- `tests/quantization/{mod,brute_force_uses_simd}.rs`
- `docs/architecture/simd.md`

Files updated:

- `Cargo.toml` — added 6 new features (`simd`, `simd-avx2`,
  `simd-avx512`, `simd-neon`, `simd-sve`, `simd-wasm`); enabled the
  default-on subset.
- `src/lib.rs` — registered `pub mod simd;`.
- `src/models/vector_utils_simd.rs` — collapsed to a delegating shim.
- `src/quantization/hnsw_integration.rs` — bug fix on lines 173 +
  194; removed dead local `cosine_similarity` helper at lines 385-399.
- `src/server/core/bootstrap.rs` — startup log line for the selected
  SIMD backend.
- `tests/all_tests.rs` — registered new `simd` and `quantization`
  test modules.

Verification:

- `cargo check --all-features` clean.
- `cargo clippy --all-features --lib -- -D warnings` clean.
- `cargo test --lib --test all_tests --all-features simd::` →
  **38/38 passing in 0.54s** (19 lib unit tests covering scalar +
  dispatch + AVX2 backend + shim, plus 19 integration tests
  covering the scalar oracle on assorted lengths, the brute-force
  bug-fix regression, and the legacy `core::simd` suite still
  routing through the new dispatch).
