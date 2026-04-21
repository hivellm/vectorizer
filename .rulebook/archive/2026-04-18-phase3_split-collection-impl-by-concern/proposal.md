# Proposal: phase3_split-collection-impl-by-concern

## Why

`phase3_split-collection-monolith` pulled the 819-line test module
out into its own file via `#[path]`, bringing `src/db/collection.rs`
from 2,636 → 1,821 lines. The remaining 1,821 lines are all packed
into a **single** `impl Collection { ... }` block running from L78
to L1817 — ~1,740 lines of inherent methods tangling five concerns:
data (insert/update/delete/get), index (HNSW construction + tuning),
persistence (save/load from `.vecdb`), graph (edge/node helpers),
quantization (dequantize/requantize).

Mechanical `sed`-extraction can't split a single impl block — that
breaks Rust's rules. The cleaner path is to first restructure the
one `impl Collection { ... }` into multiple `impl Collection { ... }`
blocks, each carrying the methods for one concern, then move each
block into its own sibling file under `src/db/collection/`.

## What Changes

1. Inside `collection.rs`, reorganize the single `impl Collection`
   into five explicit blocks grouped by concern: `impl Collection
   { /* data */ }`, `impl Collection { /* index */ }`, etc. No
   method body changes.
2. Create `src/db/collection/` directory; move the existing file
   to `src/db/collection/mod.rs`.
3. Move each per-concern block to its own sibling file:
   - `data.rs` — insert / insert_batch / update / delete / get_vector.
   - `index.rs` — HNSW ops, rebuild, fast_load.
   - `persistence.rs` — load_from_cache, cache+hnsw dump/load,
     memory-usage accounting.
   - `graph.rs` — enable_graph, populate_graph_if_empty, graph
     accessors.
   - `quantization.rs` — quantize_vector, dequantize_vector,
     requantize_existing_vectors.
4. Constructors (`new`, `new_with_owner`, …) + accessors (`name`,
   `config`, `metadata`) stay in `mod.rs` as part of the struct
   definition's own impl.
5. `collection_tests.rs` keeps its `#[path]` wiring (already done
   in the prior task).

## Impact

- Affected specs: none (internal refactor).
- Affected code: `src/db/collection.rs` → `src/db/collection/*.rs`.
- Breaking change: NO — public surface preserved by virtue of the
  methods living on the same type; nothing visible to external
  callers changes.
- User benefit: each concern reviewable in isolation; target
  per-file size under 600 lines so the whole module fits reviewer
  attention; future fixes scoped to one file.
