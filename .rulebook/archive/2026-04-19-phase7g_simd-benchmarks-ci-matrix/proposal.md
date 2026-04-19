# Proposal: phase7g_simd-benchmarks-ci-matrix

## Why

Phases 7a-7f add a large amount of unsafe SIMD code across eight backends and two dozen primitives. Without a dedicated benchmark suite and a CI matrix, regressions are invisible: a change that accidentally forces `Avx2Backend::with_fma = false` or routes a hot path through scalar would not fail any test but would silently cost 20-50% in production. Equally, we cannot claim speedup numbers in docs without reproducible evidence per ISA.

This task closes out phase 7 by building:

1. A criterion-based benchmark suite with one bench per primitive × backend, producing comparable numbers per ISA.
2. A CI matrix that builds and tests every backend on appropriate hardware (x86_64 AVX2, x86_64 AVX-512, aarch64 NEON, aarch64 SVE via QEMU, wasm32 SIMD128).
3. A regression guard: per-primitive throughput thresholds that fail CI if performance drops more than a configurable tolerance vs. the committed baseline.

## What Changes

Benchmark suite under `benches/simd/`:

- `benches/simd/dot_product.rs` — Criterion bench running dot product across backends. Dimensions: 64, 128, 256, 384, 512, 768, 1024, 1536, 3072 (covering the common embedding dimensions and HNSW codebook sizes). For each dim it runs `ScalarBackend`, then every detected SIMD backend, producing a comparable ratio.
- `benches/simd/euclidean.rs`, `benches/simd/cosine.rs`, `benches/simd/l2_norm.rs`, `benches/simd/normalize.rs`, `benches/simd/manhattan.rs` — one file per primitive following the same template.
- `benches/simd/quantize.rs` — quantize/dequantize 8-bit and 4-bit across the same dimensions.
- `benches/simd/pq_distance.rs` — PQ asymmetric-distance scan with n_subquantizers ∈ {4, 8, 16, 32} and n_codes ∈ {1k, 10k, 100k, 1M}.
- `benches/simd/search_end_to_end.rs` — HNSW search on a 100k × 768 fixture dataset with and without phase 7 enabled, reporting P50/P95/P99 query latency.
- A helper `benches/simd/util.rs` generates random vectors with a fixed seed so all backends see identical inputs.

Baseline tracking:

- `benches/simd/baselines/` contains committed JSON baselines per backend × primitive × dimension, generated once at merge time of this task.
- `scripts/simd/check-regression.sh` reads Criterion's `target/criterion/**/estimates.json`, compares against the baseline, and fails the job if any throughput number drops more than 10% (default; configurable via `VECTORIZER_SIMD_REGRESSION_TOLERANCE`).

CI matrix (`.github/workflows/simd-matrix.yml`):

| Job | Runner | Target | Backends exercised |
|-----|--------|--------|--------------------|
| x86-avx2 | ubuntu-latest | x86_64-unknown-linux-gnu | scalar, sse2, avx2, avx2+fma |
| x86-avx512 | ubuntu-latest-xlarge (AVX-512 host) or self-hosted | x86_64-unknown-linux-gnu | scalar, sse2, avx2+fma, avx512, avx512_vnni |
| macos-arm | macos-14 (M-series) | aarch64-apple-darwin | scalar, neon |
| linux-arm-neon | ubuntu-22.04-arm | aarch64-unknown-linux-gnu | scalar, neon |
| linux-arm-sve | ubuntu-latest + qemu-user-static | aarch64-unknown-linux-gnu with `-cpu max,sve=on` | scalar, neon, sve, sve2 |
| wasm-simd | ubuntu-latest | wasm32-unknown-unknown | scalar (compile only) + `wasm-pack test --node --features simd-wasm` |

Each job runs `cargo test --all-features -- simd`, `cargo bench --bench simd_dot_product -- --save-baseline ci`, then `scripts/simd/check-regression.sh`.

Docs:

- `docs/architecture/simd.md` gains a "Measured speedups" table auto-updated from the baseline JSON.
- `README.md` gets a short section summarising which ISAs are supported and where to find the benchmark results.
- `CHANGELOG.md` entry announcing phase 7 completion.

Non-goals:

- No perf dashboard website; the baseline JSON + README table is enough.
- No continuous-benchmarking service (Codspeed, Bencher). If the team wants one later, the criterion harness from this task is what it will consume.

## Impact

- Affected specs: `.rulebook/tasks/phase7g_simd-benchmarks-ci-matrix/specs/simd-bench/spec.md` (new).
- Affected code: new `benches/simd/` tree, new `scripts/simd/check-regression.sh`, new `.github/workflows/simd-matrix.yml`, edits to `docs/architecture/simd.md`, `README.md`, `CHANGELOG.md`, `Cargo.toml` (add `criterion` as a dev-dependency if absent, and `[[bench]]` entries).
- Breaking change: NO.
- User benefit: verifiable speedup claims, CI-level protection against SIMD regressions, and a clear public signal of which ISAs are first-class in Vectorizer.
