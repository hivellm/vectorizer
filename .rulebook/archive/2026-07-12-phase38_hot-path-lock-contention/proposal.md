# Proposal: phase38_hot-path-lock-contention

Source: docs/analysis/2026-07-11-improvement-analysis/ (§4, §1.6)

## Why

The 2026-07-11 improvement analysis identified the insert hot path as
the largest performance liability in the engine:

1. `insert_batch` (`db/collection/data.rs:61-226`) holds the HNSW
   `index.write()` for the **entire** batch loop — payload indexing,
   sparse indexing, quantization, and graph discovery all execute
   under the write lock. Concurrent searches (which take
   `index.read()` at `data.rs:449`) stall for the whole batch,
   collapsing search p99 under mixed read/write load.
2. Per-vector clone churn (`data.rs:75,87,141,153`): 3-4 full copies
   of each 768-1024-dim f32 array per inserted vector.
3. PQ and Binary quantization are fully implemented (900+ lines in
   `quantization/{product,binary}.rs`) but never wired into HNSW —
   `hnsw_integration.rs:74-83` accepts only `Scalar` and silently
   falls back to hardcoded 8-bit on other types (`:122-125`).
4. SIMD gaps: `quantize_f32_to_u8`/`dequantize_u8_to_f32` are
   scalar-only on every backend (`vectorizer-core/src/simd/backend.rs:156-194`
   — the doc comment falsely claims SIMD coverage); `int8_dot_product`
   has no AVX2 path; SIMD correctness is no longer CI-verified since
   the simd-matrix removal (`cf298f7d`).
5. Hot-path allocs: `upsert_queue.rs:134-138` heap-allocates the
   collection name on every admission; `query_cache.rs:67-85` runs
   SHA-256 over the full query vector per cache lookup.

## What Changes

- Narrow the HNSW write-lock scope in `insert_batch` to the actual
  `index.add` calls; move payload/sparse/graph work outside the lock.
- Eliminate redundant clones on the insert path (move `data` into the
  index; single ownership transfer of `vector`).
- Wire PQ and Binary quantization into `hnsw_integration.rs`; turn the
  silent 8-bit fallback into an explicit error.
- Add AVX2/NEON quantize/dequantize kernels and an AVX2
  `int8_dot_product` (via `_mm256_maddubs_epi16`); fix the false doc
  comment; restore a minimal SIMD correctness matrix in CI (forced
  `VECTORIZER_SIMD_BACKEND` oracle runs).
- Fast-path `try_admit` with `get()` before `entry(to_string())`;
  switch query-cache keys from SHA-256 to xxh3.
- Re-register the commented-out benches (`core_operations`,
  `search_bench`, etc. in `Cargo.toml:221-265`) and add insert-path +
  BM25 benches so regressions are detectable.

## Impact

- Affected specs: `specs/hot-path/spec.md` (new, in this task)
- Affected code: `crates/vectorizer/src/db/collection/data.rs`,
  `crates/vectorizer/src/db/upsert_queue.rs`,
  `crates/vectorizer/src/cache/query_cache.rs`,
  `crates/vectorizer/src/quantization/hnsw_integration.rs`,
  `crates/vectorizer-core/src/simd/`, `crates/vectorizer/Cargo.toml`
  (benches), `.github/workflows/` (SIMD matrix)
- Breaking change: NO (non-Scalar quantization configs that silently
  degraded to 8-bit will now work as configured)
- User benefit: stable search latency under concurrent inserts;
  higher bulk-insert throughput; PQ/Binary compression usable
