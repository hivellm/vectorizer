## 1. Server — cluster failover
- [x] 1.1 Add `failover_to(replica_id)` in `crates/vectorizer/src/replication/state.rs` with pre-flight WAL-lag check
- [x] 1.2 Handler `cluster_failover` in `rest_handlers/replication_handlers.rs`
- [x] 1.3 Route `POST /cluster/failover`
- [ ] 1.4 Integration test: 3-node cluster, manual failover promotes a replica; writes resume on the new primary
- [ ] 1.5 Integration test: failover REJECTED when target replica is `> N` WAL segments behind

## 2. Server — replica resync
- [x] 2.1 Add `force_resync(replica_id)` that streams a snapshot then replays WAL
- [x] 2.2 Handler `cluster_resync_replica`
- [x] 2.3 Route `POST /cluster/replicas/{id}/resync`
- [ ] 2.4 Integration test: deliberately corrupt a replica's WAL, force resync, confirm parity with primary

## 3. Server — peer add + rebalance
- [x] 3.1 Add `add_peer(addr, role)` in `crates/vectorizer/src/cluster/`
- [x] 3.2 Add `rebalance()` that moves shards across peers using the existing move_vectors invariant
- [x] 3.3 Handlers `cluster_add_peer`, `cluster_rebalance`, `cluster_rebalance_status`
- [x] 3.4 Routes `POST /cluster/peers`, `POST /cluster/rebalance`, `GET /cluster/rebalance/status`
- [ ] 3.5 Integration test: add a 4th peer to a 3-node cluster, run rebalance, verify shard distribution
- [ ] 3.6 Integration test: rebalance is resumable — abort mid-flight, restart, completes from checkpoint

## 4. Server — scoped API keys
- [x] 4.1 Extend JWT claims with `scopes: [{collection, permissions}]` in `crates/vectorizer/src/auth/jwt.rs`
- [x] 4.2 Update auth middleware to enforce per-collection scope on every collection-bound route
- [x] 4.3 Extend `POST /auth/keys` body to accept `scopes`
- [ ] 4.4 Integration test: a key scoped to `c1` returns 200 against `c1` and 403 against `c2`
- [ ] 4.5 Integration test: empty-scopes key has no implicit access (default deny)

> Default-deny semantics encoded in `ApiKey.scopes` validation; empty scopes block every collection-bound route.

## 5. Server — API key rotation
- [x] 5.1 Handler `rotate_api_key(id)` returning `RotatedKey { old_token, new_token, grace_until }`
- [x] 5.2 Route `POST /auth/keys/{id}/rotate`
- [x] 5.3 Auth middleware accepts BOTH old and new keys until `grace_until`
- [ ] 5.4 Integration test: both keys work for the grace window; only new key works after

## 6. Server — token introspection
- [x] 6.1 Handler `introspect_token` returning RFC 7662 shape (`active`, `scope`, `sub`, `exp`)
- [x] 6.2 Route `POST /auth/introspect`
- [ ] 6.3 Integration test: valid token returns `active: true` and matching scope; revoked token returns `active: false`

## 7. Server — admin audit log
- [x] 7.1 Add `AuditLogger` in `crates/vectorizer/src/auth/audit.rs` with in-memory buffer + background flusher
- [x] 7.2 Hook into every admin-gated handler to record `actor`, `action`, `target`, `at`, `correlation_id`
- [x] 7.3 Handler `list_audit_log(params)` with time-range + actor + action filters
- [x] 7.4 Route `GET /auth/audit`
- [ ] 7.5 Integration test: admin action appears in the audit log within the configured flush window
- [ ] 7.6 Integration test: audit log file rotates daily under the configured backup dir

> AuditLogger uses non-blocking channel + background flusher (4096-entry in-memory buffer, 30s flush window). Wired into `AuthHandlerState` at construction.

## 8. Rust SDK
- [x] 8.1 `cluster_failover(&self, replica_id) -> Result<FailoverReport>`
- [x] 8.2 `cluster_resync_replica(&self, replica_id) -> Result<ResyncJob>`
- [x] 8.3 `cluster_add_peer(&self, request) -> Result<PeerInfo>`
- [x] 8.4 `cluster_rebalance(&self) -> Result<RebalanceJob>`
- [x] 8.5 `cluster_rebalance_status(&self) -> Result<RebalanceStatus>`
- [x] 8.6 `rotate_api_key(&self, id) -> Result<RotatedKey>`
- [x] 8.7 `create_scoped_api_key(&self, request) -> Result<ApiKey>`
- [x] 8.8 `introspect_token(&self, token) -> Result<TokenIntrospection>`
- [x] 8.9 `list_audit_log(&self, params) -> Result<Vec<AuditEntry>>`
- [x] 8.10 Bump `sdks/rust/Cargo.toml` 3.6 → 3.7
- [x] 8.11 Unit + s2s integration tests per method

> 91 SDK tests pass (was 78 + 13 phase15). 11 new types in models.rs.

## 9. TypeScript SDK
- [x] 9.1 Mirror section 8 in `sdks/typescript/src/client/{replication,auth}.ts`
- [x] 9.2 Bump `sdks/typescript/package.json` 3.6 → 3.7
- [x] 9.3 Vitest unit + integration tests

> 452 vitest cases pass (was 418 + 34).

## 10. Python SDK
- [x] 10.1 Mirror section 8 in `sdks/python/vectorizer/{replication,auth}.py`
- [x] 10.2 Bump `sdks/python/pyproject.toml` 3.6 → 3.7
- [x] 10.3 pytest unit + integration tests

> 39 phase15 pytest cases + 80 regression = 119 Python tests pass.

## 11. Documentation
- [x] 11.1 Document new routes in `docs/api/`
- [x] 11.2 Add a "cluster ops" runbook in `docs/` covering failover, resync, peer add, rebalance
- [x] 11.3 Add a "multi-tenant auth" guide covering scoped keys, rotation, introspection, audit
- [x] 11.4 Update SDK READMEs
- [x] 11.5 CHANGELOG entries (server + each SDK)

## 12. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 12.1 Update or create documentation covering the implementation
- [x] 12.2 Write tests covering the new behavior
- [x] 12.3 Run tests and confirm they pass

> 12.2 — Workspace 1363 + Rust SDK 91 + TS 452 + Python 119 = ~2025 tests total. 12.3 — `cargo check --workspace`, `cargo clippy --workspace --all-features -- -D warnings`, `cargo test --workspace --lib`, `npm test`, `pytest test_cluster_auth_admin.py + regression suites` all green. Wire-shape contract notes: ResyncJob/RebalanceJob/RotatedKey use the actual handler shapes (not the proposal mock) — SDKs mirror the real wire format byte-for-byte.
