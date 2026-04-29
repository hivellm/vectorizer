## 1. Honor client-provided `id` in `/insert` and `/insert_texts`
- [x] 1.1 Add `client_id: Option<&str>` parameter to `insert_one_text` (`crates/vectorizer-server/src/server/rest_handlers/insert.rs:293-303`)
- [x] 1.2 Wire `entry.get("id")` through `do_batch_insert_texts` (`vectors.rs:337,360-371`) into `insert_one_text`
- [x] 1.3 Wire `client_id` from `/insert` (singular) handler payload into `insert_one_text`
- [x] 1.4 Use `client_id` verbatim for non-chunked Vector.id (replace `Uuid::new_v4()` at `insert.rs:436`)
- [x] 1.5 Use `format!("{}#{}", client_id, chunk_index)` for chunked Vector.id (replace `Uuid::new_v4()` at `insert.rs:398`)
- [x] 1.6 Validate client_id format (no whitespace, max length, no `#` collision with chunk separator); reject with 400 on invalid
- [x] 1.7 Add `parent_id` field to chunk payloads even when client_id is absent (use generated UUID as parent for groupability)

## 2. Flatten metadata in chunked payloads
- [x] 2.1 Rewrite payload construction at `insert.rs:374-385` to merge user metadata at payload root
- [x] 2.2 Keep `content`, `chunk_index`, `parent_id`, `file_path` at root; remove the `metadata: {...}` nesting
- [x] 2.3 Audit `FileOperations::list_files_in_collection` and any reader of `payload.data.metadata.*`; migrate to flat path (existing `metadata_view` already tolerated both shapes; updated comments + flipped legacy-tracking counters)
- [x] 2.4 Grep the workspace for `payload.data.metadata` and `data.get("metadata")` to find all readers (`file_watcher/mod.rs:433,502` + `autosave.rs:361,471` + `file_operations/operations.rs` 3 sites + `file_loader/indexer.rs:167` writer kept as-is for bulk path)
- [x] 2.5 Add tolerant-reader shim (read both shapes) with `tracing::warn!` on legacy nested layout (file_watcher: dual-shape lookup added; operations.rs: flipped `flat_shape_hits` → `nested_shape_hits` with deprecation log)
- [ ] 2.6 Confirm Qdrant compat scroll/filter endpoints surface the new flat shape correctly (deferred to integration test in §6)

## 3. Add `POST /insert_vectors` endpoint
- [x] 3.1 Define request schema: `{collection, vectors: [{id, embedding, payload?, metadata?}], public_key?}`
- [x] 3.2 Implement handler in `crates/vectorizer-server/src/server/rest_handlers/insert.rs` (`insert_vectors` + `insert_one_vector` + `build_vector_payload`)
- [x] 3.3 Validate `embedding.len() == collection.dimension`; reject with structured 400 (per-entry, also catches non-numeric values and non-array embedding)
- [x] 3.4 Honor `id` as `Vector.id` (deterministic upsert path) — same `validate_client_id` as `/insert_texts`
- [x] 3.5 Register route in `crates/vectorizer-server/src/server/core/routing.rs`
- [x] 3.6 Wire HiveHub quota check + Raft replication + cache invalidation (`check_insert_quota` + `record_insert_usage` + `mark_collection_dirty` reused from `insert_one_text`)

## 4. MCP search consistency check
- [x] 4.1 Verify `mcp_tools.rs:516-525` returns flat metadata after Phase 2 changes (no double-nesting) — confirmed; replaced 4 inline metadata-extraction sites with shared `flatten_payload_metadata` helper that lifts legacy nested keys to root for backwards compat with pre-phase9 collections
- [x] 4.2 Add MCP unit tests asserting `result.metadata.<custom_field>` is reachable on chunked vectors (5 tests in `flatten_payload_tests` covering flat round-trip, legacy nested lift, root-wins-collisions, non-object payload, non-object metadata value)

## 5. Migration & compatibility
- [x] 5.1 Decide version bump — v3.1.0 (minor). Bumped 6 Cargo.toml files: `vectorizer`, `vectorizer-server`, `vectorizer-core`, `vectorizer-cli`, `vectorizer-protocol`, `sdks/rust`
- [x] 5.2 Document breaking change in CHANGELOG under `## [3.1.0]` with migration note (Added: `/insert_vectors`, client-id honoring, `parent_id`; Changed: chunk payload shape with full migration guidance)
- [x] 5.3 Document `/insert_vectors` in `docs/users/api/BATCH.md` (added to endpoints table + new request-shape section with full field reference)
- [x] 5.4 Document client-id contract in BATCH.md `texts[].id` row + CHANGELOG (format: non-empty, ≤ 256 chars, no `#`, no edge whitespace; semantics: `<id>#<chunk_index>` for chunked, in-place upsert on re-ingest)
- [ ] 5.5 Provide a re-ingest script for users with chunked collections written by <= v3.0.13 (optional, deferred — tolerant readers + CHANGELOG migration steps cover the deprecation window)

## 6. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 6.1 Update or create documentation covering the implementation (CHANGELOG.md `[3.1.0]` section + migration guide; `docs/users/api/BATCH.md` updated with `/insert_vectors` and client-id semantics; tasks.md + proposal.md + spec.md complete)
- [x] 6.2 Write tests covering the new behavior — **17 unit tests** (`validate_client_id` ×5, `build_chunk_payload` ×3, `build_vector_payload` ×4, `flatten_payload_metadata` ×5) + **8 integration tests** (`insert_texts_honors_client_id_for_short_text`, `insert_texts_chunked_id_uses_parent_hash_index_pattern`, `insert_texts_chunked_payload_is_flat_with_user_metadata_at_root`, `insert_texts_idempotent_re_ingest_replaces_in_place`, `insert_texts_rejects_invalid_client_id`, `insert_vectors_round_trips_with_explicit_id`, `insert_vectors_rejects_dimension_mismatch`, `insert_vectors_falls_back_to_metadata_when_payload_absent`)
- [x] 6.3 Run tests and confirm they pass — `cargo check --workspace` clean; `cargo clippy --workspace --lib` zero warnings; `cargo test --workspace` 885 passed / 124 ignored (1 unrelated flake `cache::query_cache_behaviour::prometheus_counter_increments_on_every_cache_get` from cumulative Prometheus counter state — passes in isolation, no relation to phase9)
