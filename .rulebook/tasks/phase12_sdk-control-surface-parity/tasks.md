## 1. Inventory + shared models
- [ ] 1.1 Lock the route inventory in `specs/sdk-control-surface-parity/spec.md` (one Requirement per route group)
- [ ] 1.2 Add shared types `Stats`, `ServerStatus`, `LogEntry`, `IndexingProgress`, `ConfigSnapshot`, `BackupInfo`, `ReplicationStatus`, `ReplicaInfo`, `ApiKey`, `User`, `WorkspaceConfig` under `sdks/rust/src/models/`
- [ ] 1.3 Mirror the same types in `sdks/typescript/src/models/`
- [ ] 1.4 Mirror the same types in `sdks/python/vectorizer/models.py`

## 2. Rust SDK — single-vector + batch ops
- [ ] 2.1 `update_vector(&self, collection, id, request) -> Result<Vector>` (POST /update)
- [ ] 2.2 `insert_text(&self, collection, id, text, metadata) -> Result<Vector>` (POST /insert)
- [ ] 2.3 `list_vectors(&self, collection, page, limit) -> Result<VectorPage>` (GET /collections/{name}/vectors)
- [ ] 2.4 `get_vector_by_path(&self, collection, id) -> Result<Vector>` using GET /collections/{name}/vectors/{id}
- [ ] 2.5 `batch_insert_texts(&self, collection, items) -> Result<BatchInsertReport>` (POST /batch_insert)
- [ ] 2.6 `insert_vectors(&self, collection, vectors) -> Result<BatchInsertReport>` (POST /insert_vectors, raw vectors)
- [ ] 2.7 `batch_search(&self, requests) -> Result<Vec<SearchResult>>` (POST /batch_search)
- [ ] 2.8 `batch_update_vectors(&self, collection, updates) -> Result<BatchUpdateReport>` (POST /batch_update)

## 3. Rust SDK — search variants + discovery pipeline
- [ ] 3.1 `search_vectors_by_text(&self, collection, query, limit) -> Result<SearchResponse>` (POST /collections/{n}/search/text)
- [ ] 3.2 `search_by_file(&self, collection, request) -> Result<SearchResponse>` (POST /collections/{n}/search/file)
- [ ] 3.3 Verify `hybrid_search` already targets POST /collections/{n}/hybrid_search; add integration test if not
- [ ] 3.4 `broad_discovery(&self, request) -> Result<BroadDiscoveryResponse>`
- [ ] 3.5 `semantic_focus(&self, request) -> Result<SemanticFocusResponse>`
- [ ] 3.6 `promote_readme(&self, request) -> Result<PromoteReadmeResponse>`
- [ ] 3.7 `compress_evidence(&self, request) -> Result<CompressEvidenceResponse>`
- [ ] 3.8 `build_answer_plan(&self, request) -> Result<AnswerPlan>`
- [ ] 3.9 `render_llm_prompt(&self, request) -> Result<LlmPrompt>`

## 4. Rust SDK — admin/observability module (`sdks/rust/src/client/admin.rs`)
- [ ] 4.1 `get_stats(&self) -> Result<Stats>`
- [ ] 4.2 `get_status(&self) -> Result<ServerStatus>`
- [ ] 4.3 `get_logs(&self, params) -> Result<Vec<LogEntry>>`
- [ ] 4.4 `get_indexing_progress(&self) -> Result<IndexingProgress>`
- [ ] 4.5 `force_save_collection(&self, collection) -> Result<()>` (POST /collections/{n}/force-save)
- [ ] 4.6 `list_empty_collections(&self) -> Result<Vec<String>>`
- [ ] 4.7 `cleanup_empty_collections(&self) -> Result<CleanupReport>`
- [ ] 4.8 `get_config(&self) -> Result<ConfigSnapshot>`
- [ ] 4.9 `update_config(&self, patch) -> Result<ConfigSnapshot>` (admin)
- [ ] 4.10 `list_backups(&self) -> Result<Vec<BackupInfo>>`
- [ ] 4.11 `create_backup(&self, request) -> Result<BackupInfo>` (admin)
- [ ] 4.12 `restore_backup(&self, request) -> Result<()>` (admin)
- [ ] 4.13 `restart_server(&self) -> Result<()>` (admin)
- [ ] 4.14 `list_workspaces(&self) -> Result<Vec<WorkspaceConfig>>`
- [ ] 4.15 `get_workspace_config(&self) -> Result<WorkspaceConfig>`
- [ ] 4.16 `add_workspace(&self, request) -> Result<()>` (admin)
- [ ] 4.17 `remove_workspace(&self, name) -> Result<()>` (admin)

## 5. Rust SDK — auth surface (`sdks/rust/src/client/auth.rs`)
- [ ] 5.1 `me(&self) -> Result<User>`
- [ ] 5.2 `logout(&self) -> Result<()>`
- [ ] 5.3 `refresh_token(&self) -> Result<JwtToken>`
- [ ] 5.4 `validate_password(&self, password) -> Result<PasswordPolicyReport>`
- [ ] 5.5 `create_api_key(&self, request) -> Result<ApiKey>`
- [ ] 5.6 `list_api_keys(&self) -> Result<Vec<ApiKey>>`
- [ ] 5.7 `revoke_api_key(&self, id) -> Result<()>`
- [ ] 5.8 `create_user(&self, request) -> Result<User>` (admin)
- [ ] 5.9 `list_users(&self) -> Result<Vec<User>>` (admin)
- [ ] 5.10 `delete_user(&self, username) -> Result<()>` (admin)
- [ ] 5.11 `change_password(&self, username, new_password) -> Result<()>`

## 6. Rust SDK — replication module (`sdks/rust/src/client/replication.rs`)
- [ ] 6.1 `get_replication_status(&self) -> Result<ReplicationStatus>`
- [ ] 6.2 `configure_replication(&self, config) -> Result<()>`
- [ ] 6.3 `get_replication_stats(&self) -> Result<ReplicationStats>`
- [ ] 6.4 `list_replicas(&self) -> Result<Vec<ReplicaInfo>>`

## 7. Rust SDK — Hub + version bump
- [ ] 7.1 Hub backup methods: `list_user_backups`, `create_user_backup`, `restore_user_backup`, `upload_user_backup`, `get_user_backup`, `delete_user_backup`, `download_user_backup`
- [ ] 7.2 Hub usage methods: `get_usage_statistics`, `get_quota_info`, `validate_api_key`
- [ ] 7.3 Re-export new modules from `sdks/rust/src/lib.rs` and `client/mod.rs`
- [ ] 7.4 Bump `sdks/rust/Cargo.toml` 3.3 → 3.4

## 8. Rust SDK tests
- [ ] 8.1 Unit tests per method (request shape + happy-path deserialization) — colocated `#[cfg(test)] mod tests` per module
- [ ] 8.2 Integration tests (s2s style) hitting a live `vectorizer-server`, gated under the `s2s-tests` feature
- [ ] 8.3 `cargo check && cargo clippy --all-features -- -D warnings && cargo fmt --check` passes

## 9. TypeScript SDK
- [ ] 9.1 Mirror sections 2-7 in `sdks/typescript/src/client/{vectors,search,discovery,admin,auth,replication}.ts` with camelCase names
- [ ] 9.2 Add model interfaces in `sdks/typescript/src/models/`
- [ ] 9.3 Export new types and methods from `sdks/typescript/src/index.ts`
- [ ] 9.4 Bump `sdks/typescript/package.json` 3.3 → 3.4
- [ ] 9.5 Vitest unit tests for every new method
- [ ] 9.6 Vitest integration tests against a live server

## 10. Python SDK
- [ ] 10.1 Mirror sections 2-7 in `sdks/python/vectorizer/{vectors,search,discovery,admin,auth,replication}.py`
- [ ] 10.2 Add dataclasses in `sdks/python/vectorizer/models.py`
- [ ] 10.3 Export from `sdks/python/vectorizer/__init__.py`
- [ ] 10.4 Bump `sdks/python/pyproject.toml` 3.3 → 3.4
- [ ] 10.5 pytest unit tests for every new method
- [ ] 10.6 pytest integration tests against a live server

## 11. Documentation
- [ ] 11.1 Update `sdks/rust/README.md` with examples per domain (admin, auth, replication, discovery)
- [ ] 11.2 Update `sdks/typescript/README.md` mirroring
- [ ] 11.3 Update `sdks/python/README.md` mirroring
- [ ] 11.4 Update server API reference under `docs/` to cross-link SDK methods to routes
- [ ] 11.5 Update each SDK CHANGELOG with the new methods

## 12. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 12.1 Update or create documentation covering the implementation
- [ ] 12.2 Write tests covering the new behavior
- [ ] 12.3 Run tests and confirm they pass
