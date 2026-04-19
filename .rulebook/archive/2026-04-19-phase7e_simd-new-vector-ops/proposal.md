# Proposal: phase7e_simd-new-vector-ops

## Why

Phases 7a-7d give us a complete multi-ISA backend layer but still cover only the three pre-existing primitives (`dot_product`, `euclidean_distance`, `cosine_similarity`). Several hot paths in Vectorizer remain scalar even though they are trivially vectorizable:

- `normalize_vector` at `src/models/mod.rs:661` — called on every vector insertion and update. Two scalar passes: sum-of-squares then divide. 3-5× achievable with one SIMD fused pass.
- Sparse-vector L2 norm at `src/models/sparse_vector.rs:119` — scalar reduction over the `values: Vec<f32>` field. 6-8× achievable.
- No Manhattan (L1) distance available at all. L1 is a requested distance metric (per `CollectionConfig::distance_metric` design in `src/models/`) and a natural SIMD target with `abs` + sum.
- No element-wise add / sub / scale primitives. These are needed by phase7f (PQ residual computation) and by the upcoming batch-distance optimisation.
- Top-k selection in search results uses a scalar binary heap. A 4-lane horizontal min/max is not a full replacement for the heap but accelerates the partial-sort inner loop by 2-3×.

This task adds these operations to the `SimdBackend` trait, implements them across every backend from phases 7a-7d, and wires the call sites in `src/models/` and the search path to use them.

## What Changes

New methods on `SimdBackend` (`src/simd/backend.rs`):

- `fn l2_norm(&self, a: &[f32]) -> f32` — sum of squares then sqrt (already introduced in 7a stub; this task wires call sites).
- `fn normalize_in_place(&self, a: &mut [f32])` — compute norm and divide, one fused pass. Returns early on zero-norm.
- `fn manhattan_distance(&self, a: &[f32], b: &[f32]) -> f32` — sum of `|a[i] - b[i]|`. Uses `_mm256_andnot_ps` with sign-bit mask on x86, `vabsq_f32` on NEON, `f32x4_abs` on WASM.
- `fn add_assign(&self, a: &mut [f32], b: &[f32])`, `sub_assign`, `scale(a: &mut [f32], s: f32)` — element-wise primitives.
- `fn horizontal_min_index(&self, a: &[f32]) -> (usize, f32)` — returns `(argmin, min)` over the slice. Used by partial-sort in top-k.

Implementation ladder per backend:

- `ScalarBackend` — plain loops (always correct, oracle for tests).
- `Sse2Backend` — 4-lane.
- `Avx2Backend` — 8-lane, FMA-aware where applicable.
- `Avx512Backend` — 16-lane, uses `_mm512_abs_ps`.
- `NeonBackend` — 4-lane, `vabsq_f32` for Manhattan.
- `SveBackend` / `Sve2Backend` — VLA with predicates.
- `Wasm128Backend` — 4-lane, `f32x4_abs`.

Call-site rewiring:

- `src/models/mod.rs::normalize_vector` — delegate to `crate::simd::normalize_in_place` after cloning into a `Vec<f32>` (to preserve the return-by-value public API) or introduce `normalize_vector_in_place` as a new public helper and migrate internal callers. Keep the old function as a one-line shim that allocates + calls `_in_place`.
- `src/models/sparse_vector.rs::norm` — delegate to `crate::simd::l2_norm(&self.values)`.
- `src/db/collection.rs` lines 594, 788 — swap the scalar `vector.iter()...sqrt()` pattern for the SIMD helper when the vector enters the insertion / update path. (Expected single-line edits per site.)
- Introduce `CollectionConfig::distance_metric::Manhattan` handling by wiring `crate::simd::manhattan_distance` into the metric dispatch in `src/db/`.

Top-k hook:

- `src/db/optimized_hnsw.rs` search path: replace the inner partial-sort scan over candidate results (`for (id, dist) in candidates { if dist < heap.peek() ...}`) with `horizontal_min_index` on batched slices of 8 candidates at a time. Heap is still used across batches.

Non-goals:

- Keep `SparseVector::dot_product` scalar — its two-pointer merge structure is not a good SIMD target and is covered by phase7f's batch variants.
- Not adding Hamming / Jaccard distances in this task (no current consumer).

## Impact

- Affected specs: `.rulebook/tasks/phase7e_simd-new-vector-ops/specs/simd-ops/spec.md` (new).
- Affected code: `src/simd/backend.rs` (trait extension), every `src/simd/**/*.rs` backend file (implementations), `src/models/mod.rs`, `src/models/sparse_vector.rs`, `src/db/collection.rs`, `src/db/optimized_hnsw.rs`, config layer for `Manhattan` distance metric.
- Breaking change: NO (public API preserved; `Manhattan` is an additive enum variant).
- User benefit: 3-5× on every insertion/update (normalize), 6-8× on sparse-norm, 2-3× on top-k inner loop, new L1/Manhattan metric available to users.
