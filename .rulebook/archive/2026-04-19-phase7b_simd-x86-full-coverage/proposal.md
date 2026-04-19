# Proposal: phase7b_simd-x86-full-coverage

## Why

After phase7a lands, the x86_64 dispatch slot still only has an AVX2 backend ported in-place from `src/models/vector_utils_simd.rs`. To deliver on the goal of "all SIMD types, not just AVX2", x86_64 needs a full ladder:

- **SSE2** — guaranteed baseline on every x86_64 CPU; currently absent, so pre-AVX2 CPUs (still common in cloud base tiers) fall all the way back to scalar.
- **AVX2 + FMA** — the existing AVX2 code uses separate multiply and add; FMA (`_mm256_fmadd_ps`) merges them, yielding ~20% on dot/euclidean plus better numerical behaviour. FMA is available on every Haswell+ CPU (2013+), so pairing it with AVX2 is essentially free.
- **AVX-512F/DQ/BW** — 16 f32 lanes (2× AVX2); significant on Ice Lake server, Tiger Lake/Sapphire Rapids, Zen4+, Intel 12th gen+. Worth shipping despite the on-some-CPUs downclock, because batch search and bulk indexing are throughput-bound not latency-bound.
- **AVX-512 VNNI** — INT8 dot product in a single instruction. Critical for the quantization path in phase7f: 8-bit asymmetric distance can become 4× faster than the f32 AVX2 path.

Without this task, phase7a gives us the plumbing but the x86_64 story is still a single backend.

## What Changes

New files under `src/simd/x86/`:

- `src/simd/x86/sse2.rs` — `Sse2Backend`. Uses 128-bit registers (`__m128`), 4 f32 lanes. No feature gate beyond `target_arch = "x86_64"` because SSE2 is in the psABI baseline.
- `src/simd/x86/avx2.rs` — `Avx2Backend`. Ports the current inline-in-`vector_utils_simd.rs` code; adds an FMA variant selected when `is_x86_feature_detected!("fma")` is true (most CPUs that have AVX2 also have FMA, so this is almost always picked).
- `src/simd/x86/avx512.rs` — `Avx512Backend`. 512-bit registers, 16 f32 lanes. Uses AVX-512F for basic ops, AVX-512DQ where needed for 64-bit lanes, AVX-512BW for byte/word masks used later in quantization.
- `src/simd/x86/avx512_vnni.rs` — `Avx512VnniBackend`. Only adds the INT8 dot-product primitive (`_mm512_dpbusd_epi32`); for f32 ops it delegates to `Avx512Backend`. Exposed for phase7f to call directly.

Dispatch update in `src/simd/dispatch.rs`:

- Selection order on x86_64: `AVX-512F + (VNNI?)` → `AVX2 + FMA` → `AVX2` → `SSE2` → scalar.
- Detection via `is_x86_feature_detected!("avx512f")`, `"avx512dq"`, `"avx512bw"`, `"avx512vnni"`, `"avx2"`, `"fma"`, `"sse2"`.
- Each detection result cached in `OnceLock`.

FMA integration:

- `Avx2Backend` holds a `with_fma: bool` flag chosen at construction. Hot loops in `dot_product` and `euclidean_distance_squared` use `_mm256_fmadd_ps` when true and split multiply+add when false. No runtime branching inside the loop — we materialize two monomorphized methods and pick one at dispatch time.

Clean-up:

- After this task, `src/models/vector_utils_simd.rs` contains no intrinsics; it is a 5-line forwarding shim. All `#[target_feature(enable = "avx2")]` attributes live under `src/simd/x86/`.

Tail-latency note (non-goal for this task but documented):

- AVX-512 can cause frequency downclock on older Intel client CPUs (Skylake-X); document this in `docs/architecture/simd.md` and add an environment-override knob `VECTORIZER_SIMD_BACKEND=avx2` that forces `dispatch::backend()` to skip AVX-512 at startup. The knob is read once at `OnceLock` init time.

## Impact

- Affected specs: `.rulebook/tasks/phase7b_simd-x86-full-coverage/specs/simd-x86/spec.md` (new).
- Affected code: new `src/simd/x86/{sse2,avx2,avx512,avx512_vnni}.rs`; edits to `src/simd/x86/mod.rs`, `src/simd/dispatch.rs`, `src/models/vector_utils_simd.rs` (trim to shim), `docs/architecture/simd.md`, `Cargo.toml` (no new deps; just feature-gate refinement).
- Breaking change: NO. All changes are behind dispatch; public API unchanged.
- User benefit: ~20% on AVX2 CPUs (FMA), up to 2× on AVX-512 CPUs, non-zero speedup on pre-AVX2 CPUs (SSE2), 4× INT8 dot product once phase7f wires VNNI.
