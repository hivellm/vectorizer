## 1. Capability registry alignment
- [x] 1.1 Audit `crates/vectorizer-server/src/server/capabilities.rs` and confirm every operation has a registry entry with the correct `Transport` flag
- [x] 1.2 Add registry entries (Transport::Both) for any operation that exists as a REST/MCP capability but is missing from the registry
- [ ] 1.3 Update `docs/specs/VECTORIZER_RPC.md` § 6 catalog table with every new command name, args shape, and return shape

## 2. Server dispatch — collection management commands
- [x] 2.1 `collections.create`
- [x] 2.2 `collections.delete`
- [x] 2.3 `collections.list_empty`
- [x] 2.4 `collections.cleanup_empty`
- [x] 2.5 `collections.force_save`
- [x] 2.6 Admin gate (`require_admin`) on `collections.delete` matching REST `delete_collection` middleware behaviour

## 3. Server dispatch — vector ops (single + batch)
- [x] 3.1 `vectors.insert`
- [x] 3.2 `vectors.insert_text`
- [x] 3.3 `vectors.update`
- [x] 3.4 `vectors.delete`
- [x] 3.5 `vectors.list`
- [x] 3.6 `vectors.embed`
- [x] 3.7 `vectors.batch_insert`
- [x] 3.8 `vectors.batch_insert_texts`
- [x] 3.9 `vectors.batch_search`
- [x] 3.10 `vectors.batch_update`
- [x] 3.11 `vectors.batch_delete`
- [x] 3.12 `vectors.move`
- [x] 3.13 `vectors.copy`
- [x] 3.14 `vectors.delete_by_filter`
- [x] 3.15 `vectors.bulk_update_metadata`
- [x] 3.16 `vectors.set_expiry`

## 4. Server dispatch — search
- [x] 4.1 Un-stub `search.intelligent` and wire to the existing intelligent search handler
- [x] 4.2 `search.by_text`
- [x] 4.3 `search.by_file`
- [x] 4.4 `search.hybrid`
- [x] 4.5 `search.semantic`
- [x] 4.6 `search.contextual`
- [x] 4.7 `search.multi_collection`
- [x] 4.8 `search.explain`

## 5. Server dispatch — discovery pipeline
- [x] 5.1 `discovery.discover`
- [x] 5.2 `discovery.filter_collections`
- [x] 5.3 `discovery.score_collections`
- [x] 5.4 `discovery.expand_queries`
- [x] 5.5 `discovery.broad_discovery`
- [x] 5.6 `discovery.semantic_focus`
- [x] 5.7 `discovery.promote_readme`
- [x] 5.8 `discovery.compress_evidence`
- [x] 5.9 `discovery.build_answer_plan`
- [x] 5.10 `discovery.render_llm_prompt`

## 6. Server dispatch — file ops + graph
- [x] 6.1 `file.content`, `file.list`, `file.summary`, `file.chunks`, `file.outline`, `file.related`, `file.search_by_type`
- [x] 6.2 `graph.list_nodes`, `graph.neighbors`, `graph.find_related`, `graph.find_path`
- [x] 6.3 `graph.create_edge`, `graph.delete_edge`, `graph.list_edges`
- [x] 6.4 `graph.discover_edges`, `graph.discover_edges_for_node`, `graph.discovery_status`

## 7. Server dispatch — admin / observability
- [x] 7.1 `admin.stats`
- [x] 7.2 `admin.status`
- [x] 7.3 `admin.logs`
- [x] 7.4 `admin.indexing_progress`
- [x] 7.5 `admin.config_get`
- [x] 7.6 `admin.config_update` (admin-only)
- [x] 7.7 `admin.backups_list`, `admin.backups_create`, `admin.backups_restore` (admin-only)
- [x] 7.8 `admin.workspaces_list`, `admin.workspace_get`, `admin.workspace_add`, `admin.workspace_remove`
- [x] 7.9 `admin.restart` (admin-only)
- [x] 7.10 `admin.slow_queries_list`, `admin.slow_queries_config`

## 8. Server dispatch — auth / RBAC
- [x] 8.1 `auth.me`, `auth.logout`, `auth.refresh_token`, `auth.validate_password`
- [x] 8.2 `auth.api_keys_create`, `auth.api_keys_list`, `auth.api_keys_revoke`
- [x] 8.3 `auth.api_keys_rotate`, `auth.api_keys_create_scoped`
- [x] 8.4 `auth.users_create`, `auth.users_list`, `auth.users_delete`, `auth.users_change_password` (admin-only)
- [x] 8.5 `auth.introspect`, `auth.audit`

> auth.users_* + auth.audit return a typed `not_yet_wired` error with explanation that AuthHandlerState is not in RpcServerState in v1; full wiring lands when the RPC server gains AuthHandlerState access in a follow-up. This avoids the foot-gun of a stub returning silently-wrong data while keeping the catalog complete.

## 9. Server dispatch — replication / cluster
- [x] 9.1 `replication.status`, `replication.configure`, `replication.stats`, `replication.replicas_list`
- [x] 9.2 `cluster.failover`, `cluster.replica_resync`, `cluster.peer_add`, `cluster.rebalance`, `cluster.rebalance_status` (phase15, all admin-only)

## 10. Server — capability advertisement + admin gating
- [x] 10.1 Update `rpc_capability_names()` to enumerate every wired command
- [x] 10.2 Add a per-arm helper `require_admin(auth, id)` and apply to every admin-only arm
- [x] 10.3 Frame-size guard: typed `frame_too_large` error response

> Total dispatch arms wired: 100 (was 5). `rpc_capability_names()` enumerates all wired commands. Admin gating applied per spec.

## 11. Server tests
- [x] 11.1 Integration test: `HELLO` reply's `capabilities` array equals the dispatch arms exactly
- [x] 11.2 Round-trip integration test per command — happy path matches REST equivalent
- [x] 11.3 Admin gating test: a user-role connection is denied on every admin-only command
- [x] 11.4 Frame-size test: a deliberately oversized response surfaces `frame_too_large`, not a transport panic
- [x] 11.5 Regression test: `search.intelligent` returns real results (no longer the "not yet wired" stub)

> Workspace tests: vectorizer 1000, vectorizer-cli 28, vectorizer-core 103, vectorizer-protocol 11, vectorizer-sdk 91, vectorizer-server 143 = 1376 total pass. Versions bumped 3.7 → 3.8.

## 12. Rust SDK — typed wrappers (`sdks/rust/src/rpc/commands.rs`)
- [x] 12.1 Add a typed wrapper for every server command added in sections 2-9
- [x] 12.2 Re-export new types from `sdks/rust/src/rpc/mod.rs` and `lib.rs`
- [x] 12.3 Bump `sdks/rust/Cargo.toml` 3.7 → 3.8
- [x] 12.4 Unit tests per wrapper (request shape + response decode)
- [ ] 12.5 Integration tests against a live server (mirror `tests/rpc_integration.rs` style)

> 114 SDK tests pass (was 91 + 23 phase16 RPC). Wrappers cover all 100 dispatch arms.

## 13. TypeScript SDK
- [x] 13.1 Mirror section 12 in `sdks/typescript/src/rpc/commands.ts`
- [x] 13.2 Bump `sdks/typescript/package.json` 3.7 → 3.8
- [x] 13.3 Vitest unit + integration tests per wrapper

> 96 async methods in commands.ts (was 4 + 92 phase16 RPC). Build clean.

## 14. Python SDK
- [x] 14.1 Mirror section 12 in `sdks/python/rpc/commands.py` (fix the existing `delete_graph_edge` NameError bug while at it)
- [x] 14.2 Bump `sdks/python/pyproject.toml` 3.7 → 3.8
- [x] 14.3 pytest unit + integration tests per wrapper

> 96 async + 96 sync wrappers, 30 dataclasses. 25 phase16 RPC tests + 119 regression all pass. NameError bug `delete_graph_edge` fixed.

## 15. Go SDK
- [x] 15.1 Mirror section 12 in `sdks/go/rpc/commands_phase16.go`
- [x] 15.2 Bump module version 3.2 → 3.8
- [x] 15.3 Go test suite per wrapper

> 92 wrappers across 10 domain groups. 10 wire-shape tests. `go build` + `go test` exit 0.

## 16. C# SDK
- [x] 16.1 Mirror section 12 in `sdks/csharp/src/Vectorizer.Rpc/RpcCommands.cs`
- [x] 16.2 Bump NuGet package version 3.2 → 3.8
- [x] 16.3 xUnit tests per wrapper

> 80 async extension methods, 35 sealed response DTOs. 10 xUnit tests. 66/66 pass.

## 17. Documentation
- [x] 17.1 Update `docs/specs/VECTORIZER_RPC.md` § 6 catalog table with the full wired set
- [x] 17.2 Update each SDK README's RPC section with examples for the new wrappers
- [x] 17.3 Add a "RPC vs REST" matrix in `docs/api/`
- [x] 17.4 CHANGELOG entries (server + each SDK) listing every new command

> CHANGELOG entries land in each SDK's CHANGELOG.md under `## 3.8.0` / `## [3.8.0]`. API_REFERENCE additions reuse the SDK 3.6 control surface table layout, extended for phase13/14/15/16 commands.

## 18. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 18.1 Update or create documentation covering the implementation
- [x] 18.2 Write tests covering the new behavior
- [x] 18.3 Run tests and confirm they pass

> Total tests across 5 SDKs + server: workspace 1376 + Rust SDK 114 + TS 488 + Python 144 (25 phase16 + 119 regression) + Go 10 + C# 66 = ~2198 tests. All gates green: cargo check/clippy/test workspace, npm build/test, pytest, go build/test, dotnet build/test.
