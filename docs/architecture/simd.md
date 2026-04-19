# SIMD architecture

`src/simd/` is the central dispatch layer for all SIMD-accelerated
vector primitives. Before phase 7, SIMD lived only in
`src/models/vector_utils_simd.rs` as an AVX2 fast path for three
primitives — every other ISA, every other primitive, and every other
call site went scalar. This document covers the layout that replaced
it, the selection rules, and the invariants the test suite enforces.

## Layout

```
src/simd/
├── mod.rs          — public re-exports + 4 convenience functions
├── backend.rs      — `SimdBackend` trait (the contract)
├── dispatch.rs     — runtime CPU detection + `OnceLock` cache
├── scalar.rs       — `ScalarBackend` (fallback + correctness oracle)
├── x86/
│   ├── mod.rs      — gated by cfg(target_arch = "x86_64")
│   └── avx2.rs     — `Avx2Backend` (8 f32 lanes per cycle)
├── aarch64/
│   └── mod.rs      — NEON / SVE backends scheduled for phase7c
└── wasm/
    └── mod.rs      — SIMD128 backend scheduled for phase7d
```

## Dispatch flow

```
┌─────────────────┐
│   call site     │ e.g. crate::simd::cosine_similarity(a, b)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ simd::mod::*    │ thin wrapper: backend().<method>(a, b)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ dispatch::      │ OnceLock cache → resolves once per process
│   backend()     │
└────────┬────────┘
         │ first call only
         ▼
┌─────────────────┐
│ select_backend()│ per-arch priority list (see below)
└────────┬────────┘
         │
   ┌─────┼─────────────┬─────────────┐
   ▼     ▼             ▼             ▼
Avx2Backend  NeonBackend   ScalarBackend  ...
```

After the first call, every subsequent dispatch is a single relaxed
atomic load + indirect call through the trait object.

## Selection rules

Per-arch priority lists in `dispatch::select_backend`. Every branch
is gated by both `cfg(target_arch = ...)` AND a Cargo feature
(`simd-avx2`, `simd-neon`, etc.) so disabling a feature shrinks the
binary.

| Target          | Priority                                                  |
|-----------------|-----------------------------------------------------------|
| `x86_64`        | AVX-512 VNNI > AVX-512F > AVX2+FMA > AVX2 > SSE2 > scalar |
| `aarch64`       | SVE2 > SVE > NEON > scalar                                |
| `wasm32`        | SIMD128 if `cfg(target_feature = "simd128")` else scalar (7d) |
| anything else   | scalar                                                    |

Phase 7a scaffolded the dispatch + the AVX2 path. Phase 7b filled
in the rest of the x86 ladder (SSE2, FMA fusion, AVX-512F, VNNI).
Phase 7c filled in the aarch64 family (NEON, SVE, SVE2). Selection
is cached in a `OnceLock` and never re-evaluated for the life of
the process.

### Env override

`VECTORIZER_SIMD_BACKEND` accepts: `scalar | sse2 | avx2 | avx512 |
avx512vnni | neon | sve | sve2`. When set, the dispatcher forces
that backend regardless of CPU capability. If the requested backend's
runtime check fails (e.g. asking for SVE on Apple Silicon), the
dispatcher logs a warning and falls back to scalar — never silently
picking a faster path. An unknown value (typo) is also warned and
ignored, falling through to auto-detection.

Two operator stories the knob serves:

- **AVX-512 downclock** on Skylake-X server CPUs trips a ~10%
  sustained frequency drop. For latency-bound (not throughput-bound)
  workloads, `VECTORIZER_SIMD_BACKEND=avx2` can be a net win.
- **Numerical-drift debugging**: `VECTORIZER_SIMD_BACKEND=scalar`
  collapses every code path to the oracle so divergences in test
  output have one suspect.

## Cargo features

| Feature        | Purpose                                          |
|----------------|--------------------------------------------------|
| `simd`         | Master flag. Off → every dispatch goes scalar.   |
| `simd-avx2`    | Compile the AVX2 backend on `x86_64` (FMA fusion auto-detected at construction). |
| `simd-avx512`  | Compile the AVX-512F + AVX-512 VNNI backends on `x86_64`. |
| `simd-neon`    | Compile NEON on `aarch64` (always-on baseline once enabled). |
| `simd-sve`     | Compile SVE + SVE2 backends on `aarch64`. Runtime detection picks SVE2 over SVE; both fall through to NEON when the CPU lacks them. |
| `simd-wasm`    | Compile SIMD128 on `wasm32` (7d).                |

Default set: `["simd", "simd-avx2", "simd-neon", "simd-wasm"]` — the
widely-available baselines. `simd-avx512` and `simd-sve` are opt-in
because their backends use intrinsics the project's MSRV may not
expose by default; turning them on a build whose target doesn't
support those features is a no-op (no extra binary cost).

## Apple Silicon caveat

M1/M2/M3/M4 do NOT implement SVE — they are NEON-only. The
dispatcher correctly falls back to NEON on Apple Silicon: the SVE
runtime detector returns false, the SVE2 detector returns false,
and selection lands on `NeonBackend`. This is why `simd-neon` is in
the default feature set — without it, every M-series Mac would
silently run on the scalar oracle.

For SVE you need Graviton3+, Neoverse V1+, Fujitsu A64FX, or Apple's
A17-class designs (which are NOT shipped in the Mac SoCs at the
time of writing). SVE2 needs Graviton4 / Neoverse V2 / ARM v9
silicon.

## The `SimdBackend` trait

```rust
pub trait SimdBackend: Send + Sync + 'static {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32;
    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32;
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32;
    fn l2_norm(&self, a: &[f32]) -> f32;

    // Phase 7b — INT8 path for the upcoming quantization work in 7f.
    // Default impl is a scalar loop; AVX-512 VNNI overrides with one
    // `vpdpbusd` instruction per 64 lanes.
    fn int8_dot_product(&self, a: &[i8], b: &[i8]) -> i32 { /* default */ }

    fn name(&self) -> &'static str;
}
```

### Invariants every backend MUST uphold

1. **Equal-length slices.** Caller's responsibility; backends may
   `debug_assert` but must not panic in release. A length mismatch is
   a bug at the call site, not a runtime condition to handle.
2. **Cosine assumes pre-normalised inputs.** Implemented as
   `dot.clamp(-1.0, 1.0)`. Callers that need full cosine should
   normalise first or use `models::DistanceCalculator::cosine_similarity`
   from `src/models/mod.rs`.
3. **`euclidean_distance_squared` returns the SQUARED distance.** The
   `sqrt` lives in the convenience function `simd::euclidean_distance`
   so callers that only compare distances can skip it.
4. **Numerical agreement with `ScalarBackend`** within f32 rounding.
   Pinned by `tests/simd/scalar_oracle.rs` on random vectors at
   lengths 5, 8, 13, 128, 256, 999, 1024 (exercises both the SIMD
   chunk loop and the tail).

## `ScalarBackend` — the oracle

`src/simd/scalar.rs` is always available, on every target. It
doubles as the **correctness oracle**: integration tests compare the
dispatched backend's output against straight-loop `ScalarBackend`
implementations within a tolerance of `eps * sqrt(len) * 8` (the
standard worst-case bound for an `n`-element f32 reduction).

If you change a primitive in `ScalarBackend`, mirror the change
across every per-ISA backend in lock-step. If you can't (because the
ISA can't express it precisely), re-derive the tolerance and
document the divergence in the backend's module doc.

## Migration from `models/vector_utils_simd.rs`

`src/models/vector_utils_simd.rs` is now a compatibility shim — its
three public functions (`dot_product_simd`, `euclidean_distance_simd`,
`cosine_similarity_simd`) forward to `crate::simd::*`. External
crates and older tests that imported from that path keep working
without changes; new code should call `crate::simd::*` directly.

## Bug fix in this phase

`src/quantization/hnsw_integration.rs::search_brute_force` and the
quantized fallback at `:173` previously called a file-local scalar
`cosine_similarity` instead of the SIMD path. Both call sites now
route through `crate::simd::cosine_similarity`; the dead local
helper at `:385–399` is removed.

The two functions returned the same number when both vectors were
finite and non-zero, so the bug only surfaced as a missing 3-8×
speedup — no test ever caught it. The regression test at
`tests/quantization/brute_force_uses_simd.rs` now pins the dispatch
contract so a re-introduction (e.g. a refactor that copies the local
helper back) breaks loudly.

## Observability

`crate::simd::selected_backend_name()` returns the chosen backend
(`"avx2" | "scalar" | ...`). Surfaced in two places:

- **Startup log** (`src/server/core/bootstrap.rs:42`): one
  `tracing::info!` line at server boot so operators can confirm the
  binary is using the expected vector instructions.
- **Prometheus** — phase7g will expose this as a `simd_backend`
  gauge label so dashboards can group hosts by ISA.

## Cross-references

- `src/simd/` — implementation
- `src/models/vector_utils_simd.rs` — compatibility shim
- `src/quantization/hnsw_integration.rs` — call site of the
  pre-phase7 brute-force bug
- `tests/simd/scalar_oracle.rs` — numerical-parity oracle (5 tests)
- `tests/quantization/brute_force_uses_simd.rs` — bug-fix
  regression (3 tests)
- `.rulebook/tasks/phase7a_simd-fix-and-infrastructure/` — this task
- `.rulebook/tasks/phase7b–7g/` — follow-up tasks (AVX-512, NEON/SVE,
  WASM128, new vector ops, quantization SIMD, benchmarks)
