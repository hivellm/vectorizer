## 1. In-process harness

- [x] 1.1 Shared harness in `crates/vectorizer-server/tests/common/`: `TestApp` over the REAL production router (`build_router` extracted as a reusable fn in routing.rs) + real in-memory state (VectorStore cpu-only, fitted bm25 provider, per-instance tempdir data dir), dispatched via `tower::ServiceExt::oneshot`; `TestApp::with_auth` variant with real AuthManager + JWT login for auth tests
- [x] 1.2 All 12 live-server REST suites migrated (98 tests, zero `#[ignore]`, run on every PR): vector_search (5), batch_insert (13), batch_ops (5), move_vectors (4), embedding_provider (5), qdrant_compat (3), force_save (2), auth_enforcement (4), file_upload (9) + handler-coverage suites from §2; originals kept untouched; behavioral deltas documented in-file
- [x] 1.3 Encryption suites (local ECC payload-encryption, 8 ignored tests) migrated to `rest_encryption.rs`; `hub_integration_live` / GraphQL-hub / hub failover suites REQUIRE an external HiveHub instance — they stay reason-annotated Category A (environment-dependent) per the testing-doc taxonomy, same class as the TCP replication tests

## 2. Handler coverage

- [x] 2.1 `rest_lifecycle_handlers.rs` (23) + `rest_discovery_admin_handlers.rs` (25): router-level happy-path coverage for all ~30 previously-untested handlers (explain_search, copy_vectors, set_vector_expiry, delete_by_filter, bulk_update_metadata, TTL, rename, reindex, reencode, native snapshots round-trip, list_empty_collections, project outline, related files, search_by_file_type, file summary, slow queries, get_config, workspace config, discovery pipeline, backup dir; restart_server excluded with doc-comment — process-level side effects). **The very first run caught a production deadlock**: bulk_update_metadata held the DashMap collection Ref across store.update's RefMut on the same shard (analysis §1.5 trap) — every production call hung forever; fixed (drop before mutation, commit 4256567b)
- [x] 2.2 Error-branch tests per handler: missing collection, invalid/missing fields, empty IDs, unknown snapshot id — asserting status + standard error shape

## 3. Orphaned + mislabeled tests

- [x] 3.1 Finding REFUTED: the five files are compiled via `#[path]` includes in `integration.rs`/`qdrant.rs` — the analysis's mod.rs-only scan missed it. Verified all pass (13 + 26, 5 TCP tests stay reason-annotated); `integration_basic`'s "known bug" snapshot-sync tests pass (bug long fixed). Analysis §3.6 + testing.md corrected; NOTE left in mod.rs
- [x] 3.2 gRPC "Update operation fails in CI" — bug no longer reproduces: all 4 tests pass in-process; ignores removed so CI gets a fresh signal (grpc_integration 9/9, grpc_comprehensive 11/11)
- [x] 3.3 Every bare `#[ignore]` annotated with a reason (failover, comprehensive, gpu/hive_gpu, product, cluster_performance, graph)
- [x] 3.4 C# QdrantAdvancedTests: server-unavailable detection now type-based (SocketException chain walk) — 21/21 green on pt-BR Windows without a server (was 21 false failures)

## 4. CI + SDK integration

- [x] 4.1 `tests/ignore_count_gate.rs`: fails when the repo-wide `#[ignore]` count exceeds the baseline (154 → 150 after the gRPC un-ignores), when a bare ignore appears, or when the baseline goes stale vs a shrinking count
- [x] 4.2 `.github/workflows/sdk-integration.yml`: weekly + manual job that builds the slim server, boots it on the runner, and runs TS/Python/Go/C# integration suites against it (first server-backed SDK coverage in CI)
- [x] 4.3 `docs/development/testing.md` updated: real counts, harness pattern, corrected orphan status, gate documentation

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 5.1 Update or create documentation covering the implementation — testing.md + CHANGELOG [3.5.0] (deadlock fix) + harness doc-comments + analysis §3.6 correction
- [x] 5.2 Write tests covering the new behavior — the task IS tests: ~150 new/unlocked in-process tests (98 migrated/coverage + un-ignored gRPC + wired replication) + ignore_count_gate
- [x] 5.3 Run tests and confirm they pass — all new suites green (98 in-process + gate 1/1 + grpc 20 + replication 39); clippy -D warnings 0
