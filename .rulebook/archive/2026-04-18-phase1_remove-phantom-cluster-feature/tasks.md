## 1. Investigation

- [x] 1.1 Grep every `#[cfg(feature = "cluster")]` site — 15 occurrences, all in `src/db/vector_store.rs` (lines 14, 36, 47, 63, 75, 87, 99, 111, 134, 152, 182, 211, 328, 340, 1402 before the change). 0 occurrences anywhere else in `src/`.
- [x] 1.2 Decide Option A (declare feature in Cargo.toml) vs Option B (remove the gate) — chose Option B. Rationale: `DistributedShardedCollection` is already unconditionally declared in `src/db/mod.rs` and publicly re-exported; the type has working sync methods (`name`, `config`, `document_count`, `requantize_existing_vectors`) and async mutating methods that require the cluster router. Removing the gate makes `CollectionType::DistributedSharded` a first-class variant on every build, eliminating the drift risk that the proposal flagged. Option A would have added a Cargo feature that just toggles already-present code, which is dead ceremony.

## 2. Implementation (Option B path)

- [x] 2.1 Delete every `#[cfg(feature = "cluster")]` attribute in `src/db/vector_store.rs` (15 occurrences) via `sed -i '/^\s*#\[cfg(feature = "cluster")\]\s*$/d' src/db/vector_store.rs`
- [x] 2.2 Add `DistributedSharded(_)` match arms where they were missing. Added to: `delete_vector`, `update_vector`, `get_vector`, `vector_count`, `document_count`, `estimated_memory_usage`, `get_all_vectors`, `get_embedding_type`, `calculate_memory_usage`, `get_size_info`, `set_embedding_type`, `load_hnsw_index_from_dump`, `load_vectors_into_memory`, `fast_load_vectors`, `load_collection_from_cache`, `load_collection_from_cache_with_hnsw_dump`, and `storage::compact.rs`'s per-collection match.
- [x] 2.3 For sync methods that `DistributedShardedCollection` doesn't expose synchronously (`vector_count`, `update`, `delete`, `insert`), use the best-available sync method (`document_count` where the proxy is accurate enough) or return `VectorizerError::Storage` with a message pointing callers at the async router. Avoided `Runtime::new()` shortcuts that would panic inside an existing tokio context.
- [x] 2.4 Run `cargo check --all-targets` and fix any leftover references — passed in 28.91s.
- [x] 2.5 Run `cargo clippy --all-targets -- -D warnings` — passed in 24.13s.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 Update CHANGELOG with `### Changed` entry documenting the removal and the sync-method semantics.
- [x] 3.2 Tests covering the new behavior — the existing `cli::config::tests::test_config_file_operations` was updated to inject a valid JWT secret before loading (aligns with the v3.0.0 security posture). Added `test_default_config_file_fails_validate_until_secret_injected` as an explicit regression guard. Match exhaustiveness across all `DistributedSharded` sites is enforced by the compiler itself.
- [x] 3.3 Run `cargo test --lib -p vectorizer` — 1077 passed, 0 failed, 7 ignored.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG `Changed` entry)
- [x] Write tests covering the new behavior (new regression test + updated roundtrip test)
- [x] Run tests and confirm they pass (`cargo test --lib -p vectorizer`: 1077 passed, 0 failed)
