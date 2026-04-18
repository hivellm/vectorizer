## 1. Prerequisites

- [ ] 1.1 Confirm phases 7a, 7b, 7c, 7d, 7e, 7f are merged and every backend is in place
- [ ] 1.2 Confirm `criterion` is available as a dev-dependency in `Cargo.toml` (add if absent)

## 2. Benchmark harness

- [ ] 2.1 Create `benches/simd/util.rs` with seeded random-vector generators and backend enumeration helper
- [ ] 2.2 Add `[[bench]]` entries in `Cargo.toml` for each bench file
- [ ] 2.3 Configure Criterion global settings (warm-up 3s, measurement 5s, sample size 100) via `benches/simd/util.rs`

## 3. Per-primitive benchmarks

- [ ] 3.1 `benches/simd/dot_product.rs` — every backend × dims {64, 128, 256, 384, 512, 768, 1024, 1536, 3072}
- [ ] 3.2 `benches/simd/euclidean.rs` — same sweep
- [ ] 3.3 `benches/simd/cosine.rs` — same sweep
- [ ] 3.4 `benches/simd/l2_norm.rs` — same sweep
- [ ] 3.5 `benches/simd/normalize.rs` — in-place form
- [ ] 3.6 `benches/simd/manhattan.rs` — same sweep
- [ ] 3.7 `benches/simd/quantize.rs` — 8-bit and 4-bit quantize + dequantize across same dims
- [ ] 3.8 `benches/simd/pq_distance.rs` — PQ ADC scan with n_subquantizers ∈ {4, 8, 16, 32}, n_codes ∈ {1k, 10k, 100k, 1M}

## 4. End-to-end bench

- [ ] 4.1 `benches/simd/search_end_to_end.rs` — HNSW search on a 100k × 768 fixture; report P50/P95/P99 with and without SIMD backends forced via `VECTORIZER_SIMD_BACKEND=scalar`
- [ ] 4.2 Commit the fixture dataset generator (not the dataset itself) under `scripts/simd/gen_fixture.rs`

## 5. Baseline + regression guard

- [ ] 5.1 Create `benches/simd/baselines/` directory with initial JSON baselines per backend × primitive × dim
- [ ] 5.2 Write `scripts/simd/check-regression.sh` that parses Criterion `estimates.json` and compares against the committed baselines
- [ ] 5.3 Make tolerance configurable via `VECTORIZER_SIMD_REGRESSION_TOLERANCE` (default 10%)
- [ ] 5.4 Produce a Markdown diff report showing deltas per row; fail the job on any row exceeding tolerance

## 6. CI matrix

- [ ] 6.1 Create `.github/workflows/simd-matrix.yml` with the six jobs listed in the proposal
- [ ] 6.2 x86-avx2 job: `cargo test --all-features -- simd::x86`, then bench + regression check
- [ ] 6.3 x86-avx512 job: pin to an `ubuntu-latest-xlarge` AVX-512-capable host (or self-hosted); run full ladder
- [ ] 6.4 macos-arm job: `macos-14` runner, `cargo test --all-features -- simd::aarch64::neon`, bench
- [ ] 6.5 linux-arm-neon job: `ubuntu-22.04-arm` runner; full NEON test + bench
- [ ] 6.6 linux-arm-sve job: QEMU user-static with `-cpu max,sve=on` to emulate SVE-capable CPU; run `cargo test --all-features -- simd::aarch64::sve`
- [ ] 6.7 wasm-simd job: compile with `RUSTFLAGS="-C target-feature=+simd128"` and run `wasm-pack test --node`
- [ ] 6.8 Aggregate job: download artefacts from every backend job and publish a single `simd-report.md` comment on the PR

## 7. Docs

- [ ] 7.1 Add a "Measured speedups" table to `docs/architecture/simd.md` populated from the committed baselines
- [ ] 7.2 Add a short SIMD support section to `README.md` linking to the doc
- [ ] 7.3 Add a `CHANGELOG.md` entry under "Added" announcing phase 7 completion with the ISA matrix

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Document the benchmark harness and regression-guard workflow in `docs/architecture/simd-benchmarks.md`
- [ ] 8.2 Add smoke-level self-tests for `scripts/simd/check-regression.sh` in `tests/scripts/simd_regression.rs` (invoke the script with synthetic Criterion output)
- [ ] 8.3 Run the full matrix locally on at least one x86_64 and one aarch64 host; commit the resulting baselines
- [ ] 8.4 Run `cargo check --all-features`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, `cargo bench --bench simd_dot_product -- --save-baseline phase7-final` and confirm zero warnings, 100% pass, and regression guard green
