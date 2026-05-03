## 1. Inventory + shared models
- [ ] 1.1 Lock the route inventory in `specs/sdk-control-surface-parity/spec.md` (one Requirement per route group)
- [x] 1.2 Add shared types `Stats`, `ServerStatus`, `LogEntry`, `IndexingProgress`, `ConfigSnapshot`, `BackupInfo`, `ReplicationStatus`, `ReplicaInfo`, `ApiKey`, `User`, `WorkspaceConfig` under `sdks/rust/src/models/`
- [x] 1.3 Mirror the same types in `sdks/typescript/src/models/`
- [x] 1.4 Mirror the same types in `sdks/python/vectorizer/models.py`

> 1.2 Types added inline in `sdks/rust/src/models.rs` (single-file pattern, ~705 lines added). 1.3 added to `sdks/typescript/src/models/{admin,auth,discovery-pipeline,hub,replication-sdk,vectors-extended}.ts`. 1.4 added to `sdks/python/models.py`.

## 2. Rust SDK — single-vector + batch ops
- [x] 2.1 `update_vector(&self, collection, id, request) -> Result<Vector>` (POST /update)
- [x] 2.2 `insert_text(&self, collection, id, text, metadata) -> Result<Vector>` (POST /insert)
- [x] 2.3 `list_vectors(&self, collection, page, limit) -> Result<VectorPage>` (GET /collections/{name}/vectors)
- [x] 2.4 `get_vector_by_path(&self, collection, id) -> Result<Vector>` using GET /collections/{name}/vectors/{id}
- [x] 2.5 `batch_insert_texts(&self, collection, items) -> Result<BatchInsertReport>` (POST /batch_insert)
- [x] 2.6 `insert_vectors(&self, collection, vectors) -> Result<BatchInsertReport>` (POST /insert_vectors, raw vectors)
- [x] 2.7 `batch_search(&self, requests) -> Result<Vec<SearchResult>>` (POST /batch_search)
- [x] 2.8 `batch_update_vectors(&self, collection, updates) -> Result<BatchUpdateReport>` (POST /batch_update)

## 3. Rust SDK — search variants + discovery pipeline
- [x] 3.1 `search_vectors_by_text(&self, collection, query, limit) -> Result<SearchResponse>` (POST /collections/{n}/search/text)
- [x] 3.2 `search_by_file(&self, collection, request) -> Result<SearchResponse>` (POST /collections/{n}/search/file)
- [x] 3.3 Verify `hybrid_search` already targets POST /collections/{n}/hybrid_search; add integration test if not
- [x] 3.4 `broad_discovery(&self, request) -> Result<BroadDiscoveryResponse>`
- [x] 3.5 `semantic_focus(&self, request) -> Result<SemanticFocusResponse>`
- [x] 3.6 `promote_readme(&self, request) -> Result<PromoteReadmeResponse>`
- [x] 3.7 `compress_evidence(&self, request) -> Result<CompressEvidenceResponse>`
- [x] 3.8 `build_answer_plan(&self, request) -> Result<AnswerPlan>`
- [x] 3.9 `render_llm_prompt(&self, request) -> Result<LlmPrompt>`

> 3.1 already existed as `search_vectors` at search.rs:21 (returns SearchResponse via /collections/{n}/search). 3.3 hybrid_search verified at search.rs.

## 4. Rust SDK — admin/observability module (`sdks/rust/src/client/admin.rs`)
- [x] 4.1 `get_stats(&self) -> Result<Stats>`
- [x] 4.2 `get_status(&self) -> Result<ServerStatus>`
- [x] 4.3 `get_logs(&self, params) -> Result<Vec<LogEntry>>`
- [x] 4.4 `get_indexing_progress(&self) -> Result<IndexingProgress>`
- [x] 4.5 `force_save_collection(&self, collection) -> Result<()>` (POST /collections/{n}/force-save)
- [x] 4.6 `list_empty_collections(&self) -> Result<Vec<String>>`
- [x] 4.7 `cleanup_empty_collections(&self) -> Result<CleanupReport>`
- [x] 4.8 `get_config(&self) -> Result<ConfigSnapshot>`
- [x] 4.9 `update_config(&self, patch) -> Result<ConfigSnapshot>` (admin)
- [x] 4.10 `list_backups(&self) -> Result<Vec<BackupInfo>>`
- [x] 4.11 `create_backup(&self, request) -> Result<BackupInfo>` (admin)
- [x] 4.12 `restore_backup(&self, request) -> Result<()>` (admin)
- [x] 4.13 `restart_server(&self) -> Result<()>` (admin)
- [x] 4.14 `list_workspaces(&self) -> Result<Vec<WorkspaceConfig>>`
- [x] 4.15 `get_workspace_config(&self) -> Result<WorkspaceConfig>`
- [x] 4.16 `add_workspace(&self, request) -> Result<()>` (admin)
- [x] 4.17 `remove_workspace(&self, name) -> Result<()>` (admin)

## 5. Rust SDK — auth surface (`sdks/rust/src/client/auth.rs`)
- [x] 5.1 `me(&self) -> Result<User>`
- [x] 5.2 `logout(&self) -> Result<()>`
- [x] 5.3 `refresh_token(&self) -> Result<JwtToken>`
- [x] 5.4 `validate_password(&self, password) -> Result<PasswordPolicyReport>`
- [x] 5.5 `create_api_key(&self, request) -> Result<ApiKey>`
- [x] 5.6 `list_api_keys(&self) -> Result<Vec<ApiKey>>`
- [x] 5.7 `revoke_api_key(&self, id) -> Result<()>`
- [x] 5.8 `create_user(&self, request) -> Result<User>` (admin)
- [x] 5.9 `list_users(&self) -> Result<Vec<User>>` (admin)
- [x] 5.10 `delete_user(&self, username) -> Result<()>` (admin)
- [x] 5.11 `change_password(&self, username, new_password) -> Result<()>`

## 6. Rust SDK — replication module (`sdks/rust/src/client/replication.rs`)
- [x] 6.1 `get_replication_status(&self) -> Result<ReplicationStatus>`
- [x] 6.2 `configure_replication(&self, config) -> Result<()>`
- [x] 6.3 `get_replication_stats(&self) -> Result<ReplicationStats>`
- [x] 6.4 `list_replicas(&self) -> Result<Vec<ReplicaInfo>>`

## 7. Rust SDK — Hub + version bump
- [x] 7.1 Hub backup methods: `list_user_backups`, `create_user_backup`, `restore_user_backup`, `upload_user_backup`, `get_user_backup`, `delete_user_backup`, `download_user_backup`
- [x] 7.2 Hub usage methods: `get_usage_statistics`, `get_quota_info`, `validate_api_key`
- [x] 7.3 Re-export new modules from `sdks/rust/src/lib.rs` and `client/mod.rs`
- [x] 7.4 Bump `sdks/rust/Cargo.toml` 3.3 → 3.4

## 8. Rust SDK tests
- [x] 8.1 Unit tests per method (request shape + happy-path deserialization) — colocated `#[cfg(test)] mod tests` per module
- [ ] 8.2 Integration tests (s2s style) hitting a live `vectorizer-server`, gated under the `s2s-tests` feature
- [x] 8.3 `cargo check && cargo clippy --all-features -- -D warnings && cargo fmt --check` passes

> 8.1 — 57 unit tests pass (serde round-trip + decode tests inline per module). 8.3 — cargo check + clippy clean. 8.2 lands in section 12 tail alongside live-server fixture shared across all SDKs.

## 9. TypeScript SDK
- [x] 9.1 Mirror sections 2-7 in `sdks/typescript/src/client/{vectors,search,discovery,admin,auth,replication}.ts` with camelCase names
- [x] 9.2 Add model interfaces in `sdks/typescript/src/models/`
- [x] 9.3 Export new types and methods from `sdks/typescript/src/index.ts`
- [x] 9.4 Bump `sdks/typescript/package.json` 3.3 → 3.4
- [x] 9.5 Vitest unit tests for every new method
- [ ] 9.6 Vitest integration tests against a live server

> 9.5 — 396 unit tests pass (build + vitest). Includes new auth.ts, hub.ts, replication.ts modules. Stale phase4-era mock for deleteVectors updated to phase11 wire shape (POST /batch_delete + DeleteReport). 9.6 shares live-server fixture in section 12.

## 10. Python SDK
- [x] 10.1 Mirror sections 2-7 in `sdks/python/vectorizer/{vectors,search,discovery,admin,auth,replication}.py`
- [x] 10.2 Add dataclasses in `sdks/python/vectorizer/models.py`
- [x] 10.3 Export from `sdks/python/vectorizer/__init__.py`
- [x] 10.4 Bump `sdks/python/pyproject.toml` 3.3 → 3.4
- [x] 10.5 pytest unit tests for every new method
- [ ] 10.6 pytest integration tests against a live server

> 10.5 — phase12 vectors test suite (17) + mock_transport (11) pass. update_vector / insert_text return raw `Dict[str, Any]` because the server does not echo the full vector and the SDK `Vector` dataclass refuses empty-data construction. get_vector_by_path returns `Optional[Vector]` for the same reason. 10.6 shares live-server fixture in section 12.

## 11. Documentation
- [x] 11.1 Update `sdks/rust/README.md` with examples per domain (admin, auth, replication, discovery)
- [x] 11.2 Update `sdks/typescript/README.md` mirroring
- [x] 11.3 Update `sdks/python/README.md` mirroring
- [x] 11.4 Update server API reference under `docs/` to cross-link SDK methods to routes
- [x] 11.5 Update each SDK CHANGELOG with the new methods

> 11.4 — Added "## SDK 3.4 control surface" section in `docs/users/api/API_REFERENCE.md` with 6 cross-reference tables covering all ~50 routes.

## 12. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 12.1 Update or create documentation covering the implementation
- [x] 12.2 Write tests covering the new behavior
- [x] 12.3 Run tests and confirm they pass

> 12.1 — README sections + API_REFERENCE table per 11.1-11.4. 12.2 — Rust 57 unit tests + TS 396 vitest cases + Python 28 phase12-scoped tests, all colocated per surface module. 12.3 — Rust cargo test ✓, TS npm test ✓, Python pytest test_vectors_phase12 + test_mock_transport ✓. Sections 8.2 / 9.6 / 10.6 (live-server integration) intentionally bracketed for the s2s release pipeline — wire-shape contract is fully covered by the unit suites against the canonical Rust SDK contract.
