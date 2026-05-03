## 1. Capability registry alignment
- [ ] 1.1 Audit `crates/vectorizer-server/src/server/capabilities.rs` and confirm every operation has a registry entry with the correct `Transport` flag
- [ ] 1.2 Add registry entries (Transport::Both) for any operation that exists as a REST/MCP capability but is missing from the registry
- [ ] 1.3 Update `docs/specs/VECTORIZER_RPC.md` § 6 catalog table with every new command name, args shape, and return shape

## 2. Server dispatch — collection management commands
- [ ] 2.1 `collections.create` — args `[Str(name), Map(config)]` → `Map { name, ... }`
- [ ] 2.2 `collections.delete` — args `[Str(name)]` → `Map { success }`
- [ ] 2.3 `collections.list_empty` — args `[]` → `Array<Str>`
- [ ] 2.4 `collections.cleanup_empty` — args `[]` → `Map { removed: Int, names: Array<Str> }`
- [ ] 2.5 `collections.force_save` — args `[Str(name)]` → `Map { success }`
- [ ] 2.6 Admin gate (`require_admin`) on `collections.delete` matching REST `delete_collection` middleware behaviour

## 3. Server dispatch — vector ops (single + batch)
- [ ] 3.1 `vectors.insert` — args `[Str(coll), Str(id), Array(data), Map(payload)]`
- [ ] 3.2 `vectors.insert_text` — args `[Str(coll), Str(id), Str(text), Map(payload)]`
- [ ] 3.3 `vectors.update` — args `[Str(coll), Str(id), Array(data), Map(payload)]`
- [ ] 3.4 `vectors.delete` — args `[Str(coll), Str(id)]`
- [ ] 3.5 `vectors.list` — args `[Str(coll), Int(page), Int(limit)]` → `Map { items, total }`
- [ ] 3.6 `vectors.embed` — args `[Str(text), Str(model)?]` → `Map { embedding, model, dimension }`
- [ ] 3.7 `vectors.batch_insert` — args `[Str(coll), Array<Map>(items)]`
- [ ] 3.8 `vectors.batch_insert_texts` — args `[Str(coll), Array<Map>(items)]`
- [ ] 3.9 `vectors.batch_search` — args `[Array<Map>(requests)]`
- [ ] 3.10 `vectors.batch_update` — args `[Str(coll), Array<Map>(updates)]`
- [ ] 3.11 `vectors.batch_delete` — args `[Str(coll), Array<Str>(ids)]`
- [ ] 3.12 `vectors.move` (phase11) — args `[Str(src), Str(dst), Array<Str>(ids)]`
- [ ] 3.13 `vectors.copy` (phase13) — args `[Str(src), Str(dst), Array<Str>(ids)]`
- [ ] 3.14 `vectors.delete_by_filter` (phase13) — args `[Str(coll), Map(filter)]`
- [ ] 3.15 `vectors.bulk_update_metadata` (phase13) — args `[Str(coll), Map(filter), Map(patch)]`
- [ ] 3.16 `vectors.set_expiry` (phase13) — args `[Str(coll), Str(id), Str(expires_at)]`

## 4. Server dispatch — search
- [ ] 4.1 Un-stub `search.intelligent` in `dispatch.rs:69-71` and wire to the existing intelligent search handler
- [ ] 4.2 `search.by_text` — args `[Str(coll), Str(query), Int(limit)?]`
- [ ] 4.3 `search.by_file` — args `[Str(coll), Map(request)]`
- [ ] 4.4 `search.hybrid` — args `[Str(coll), Map(request)]`
- [ ] 4.5 `search.semantic` — args `[Map(request)]`
- [ ] 4.6 `search.contextual` — args `[Map(request)]`
- [ ] 4.7 `search.multi_collection` — args `[Map(request)]`
- [ ] 4.8 `search.explain` (phase14) — args `[Str(coll), Map(request)]` → `Map { hits, trace }`

## 5. Server dispatch — discovery pipeline
- [ ] 5.1 `discovery.discover` — args `[Map(request)]`
- [ ] 5.2 `discovery.filter_collections` — args `[Map(request)]`
- [ ] 5.3 `discovery.score_collections` — args `[Map(request)]`
- [ ] 5.4 `discovery.expand_queries` — args `[Map(request)]`
- [ ] 5.5 `discovery.broad_discovery` — args `[Map(request)]`
- [ ] 5.6 `discovery.semantic_focus` — args `[Map(request)]`
- [ ] 5.7 `discovery.promote_readme` — args `[Map(request)]`
- [ ] 5.8 `discovery.compress_evidence` — args `[Map(request)]`
- [ ] 5.9 `discovery.build_answer_plan` — args `[Map(request)]`
- [ ] 5.10 `discovery.render_llm_prompt` — args `[Map(request)]`

## 6. Server dispatch — file ops + graph
- [ ] 6.1 `file.content`, `file.list`, `file.summary`, `file.chunks`, `file.outline`, `file.related`, `file.search_by_type` (one arm per existing REST file route)
- [ ] 6.2 `graph.list_nodes`, `graph.neighbors`, `graph.find_related`, `graph.find_path`
- [ ] 6.3 `graph.create_edge`, `graph.delete_edge`, `graph.list_edges`
- [ ] 6.4 `graph.discover_edges`, `graph.discover_edges_for_node`, `graph.discovery_status`

## 7. Server dispatch — admin / observability
- [ ] 7.1 `admin.stats` — args `[]` → `Map { collections_count, total_vectors, ... }`
- [ ] 7.2 `admin.status` — args `[]` → `Map { ready, ... }`
- [ ] 7.3 `admin.logs` — args `[Map(params)]` → `Array<Map>`
- [ ] 7.4 `admin.indexing_progress` — args `[]`
- [ ] 7.5 `admin.config_get` — args `[]`
- [ ] 7.6 `admin.config_update` — args `[Map(patch)]` (admin-only)
- [ ] 7.7 `admin.backups_list`, `admin.backups_create`, `admin.backups_restore` (admin-only)
- [ ] 7.8 `admin.workspaces_list`, `admin.workspace_get`, `admin.workspace_add`, `admin.workspace_remove` (last two admin-only)
- [ ] 7.9 `admin.restart` (admin-only)
- [ ] 7.10 `admin.slow_queries_list`, `admin.slow_queries_config` (phase14)

## 8. Server dispatch — auth / RBAC
- [ ] 8.1 `auth.me`, `auth.logout`, `auth.refresh_token`, `auth.validate_password`
- [ ] 8.2 `auth.api_keys_create`, `auth.api_keys_list`, `auth.api_keys_revoke`
- [ ] 8.3 `auth.api_keys_rotate`, `auth.api_keys_create_scoped` (phase15)
- [ ] 8.4 `auth.users_create`, `auth.users_list`, `auth.users_delete`, `auth.users_change_password` (admin-only)
- [ ] 8.5 `auth.introspect`, `auth.audit` (phase15)

## 9. Server dispatch — replication / cluster
- [ ] 9.1 `replication.status`, `replication.configure`, `replication.stats`, `replication.replicas_list`
- [ ] 9.2 `cluster.failover`, `cluster.replica_resync`, `cluster.peer_add`, `cluster.rebalance`, `cluster.rebalance_status` (phase15, all admin-only)

## 10. Server — capability advertisement + admin gating
- [ ] 10.1 Update `rpc_capability_names()` in `dispatch.rs:392-403` to enumerate every wired command (one source of truth)
- [ ] 10.2 Add a per-arm helper `require_admin(auth, id)` that returns `Response::err` early if `!auth.admin`; apply to every admin-only arm
- [ ] 10.3 Frame-size guard: any response that would exceed 64 MiB MUST return a typed `frame_too_large` error response (no panic, no truncation)

## 11. Server tests
- [ ] 11.1 Integration test: `HELLO` reply's `capabilities` array equals the dispatch arms exactly (no drift)
- [ ] 11.2 Round-trip integration test per command — happy path matches REST equivalent
- [ ] 11.3 Admin gating test: a user-role connection is denied on every admin-only command
- [ ] 11.4 Frame-size test: a deliberately oversized response surfaces `frame_too_large`, not a transport panic
- [ ] 11.5 Regression test: `search.intelligent` returns real results (no longer the "not yet wired" stub)

## 12. Rust SDK — typed wrappers (`sdks/rust/src/rpc/commands.rs`)
- [ ] 12.1 Add a typed wrapper for every server command added in sections 2-9, mirroring the existing `list_collections` / `get_collection_info` / `search_basic` style
- [ ] 12.2 Re-export new types from `sdks/rust/src/rpc/mod.rs` and `lib.rs`
- [ ] 12.3 Bump `sdks/rust/Cargo.toml` (version aligned with the latest preceding phase, minimum 3.6 → 3.7)
- [ ] 12.4 Unit tests per wrapper (request shape + response decode)
- [ ] 12.5 Integration tests against a live server (mirror `tests/rpc_integration.rs` style)

## 13. TypeScript SDK
- [ ] 13.1 Mirror section 12 in `sdks/typescript/src/rpc/commands.ts`
- [ ] 13.2 Bump `sdks/typescript/package.json`
- [ ] 13.3 Vitest unit + integration tests per wrapper

## 14. Python SDK
- [ ] 14.1 Mirror section 12 in `sdks/python/rpc/commands.py` (fix the existing `delete_graph_edge` NameError bug while at it — see `scripts/gap-analysis/sdk_python_findings.md`)
- [ ] 14.2 Bump `sdks/python/pyproject.toml`
- [ ] 14.3 pytest unit + integration tests per wrapper

## 15. Go SDK
- [ ] 15.1 Mirror section 12 in `sdks/go/rpc/commands.go`
- [ ] 15.2 Bump module version
- [ ] 15.3 Go test suite per wrapper

## 16. C# SDK
- [ ] 16.1 Mirror section 12 in `sdks/csharp/.../Rpc/Commands.cs`
- [ ] 16.2 Bump NuGet package version
- [ ] 16.3 xUnit/NUnit tests per wrapper

## 17. Documentation
- [ ] 17.1 Update `docs/specs/VECTORIZER_RPC.md` § 6 catalog table with the full wired set (no "not yet wired" entries)
- [ ] 17.2 Update each SDK README's RPC section with examples for the new wrappers
- [ ] 17.3 Add a "RPC vs REST" matrix in `docs/api/` so consumers can pick a transport without source-diving
- [ ] 17.4 CHANGELOG entries (server + each SDK) listing every new command

## 18. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 18.1 Update or create documentation covering the implementation
- [ ] 18.2 Write tests covering the new behavior
- [ ] 18.3 Run tests and confirm they pass
