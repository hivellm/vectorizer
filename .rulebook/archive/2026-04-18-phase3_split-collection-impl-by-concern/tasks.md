## 1. Reorganize the single impl

- [x] 1.1 Split the one `impl Collection { ... }` block at L78-1817 of `collection.rs` into five `impl Collection` blocks grouped by concern. No method bodies change; only the `impl { }` wrappers get added.
- [x] 1.2 Verify `cargo check` still clean after the reorganization.

## 2. Per-concern extraction

- [x] 2.1 Move the data block to `src/db/collection/data.rs` (insert, insert_batch, update, delete, get_vector).
- [x] 2.2 Move the index block to `src/db/collection/index.rs` (HNSW construction, rebuild, fast_load, hnsw dump/load).
- [x] 2.3 Move the persistence block to `src/db/collection/persistence.rs` (load_from_cache, load_from_cache_with_hnsw_dump, load_vectors_into_memory, estimated_memory_usage, calculate_memory_usage).
- [x] 2.4 Move the graph block to `src/db/collection/graph.rs` (enable_graph, populate_graph_if_empty, graph accessors).
- [x] 2.5 Move the quantization block to `src/db/collection/quantization.rs` (quantize_vector, dequantize_vector, requantize_existing_vectors).
- [x] 2.6 Convert `collection.rs` itself into `src/db/collection/mod.rs` keeping only the struct, constructors, and simple accessors.

## 3. Verification

- [x] 3.1 `cargo check --all-features` clean.
- [x] 3.2 28 collection tests still pass (live in `collection_tests.rs` via `#[path]`).
- [x] 3.3 Every per-concern file under 600 lines; `mod.rs` under 300.

## 4. Tail (mandatory)

- [x] 4.1 Update module-level doc comments on each new file.
- [x] 4.2 Tests carry over; no new tests required.
- [x] 4.3 Run `cargo test --all-features` and confirm pass.
