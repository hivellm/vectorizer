## 1. Server — cluster failover
- [ ] 1.1 Add `failover_to(replica_id)` in `crates/vectorizer/src/replication/state.rs` with pre-flight WAL-lag check
- [ ] 1.2 Handler `cluster_failover` in `rest_handlers/replication_handlers.rs`
- [ ] 1.3 Route `POST /cluster/failover`
- [ ] 1.4 Integration test: 3-node cluster, manual failover promotes a replica; writes resume on the new primary
- [ ] 1.5 Integration test: failover REJECTED when target replica is `> N` WAL segments behind

## 2. Server — replica resync
- [ ] 2.1 Add `force_resync(replica_id)` that streams a snapshot then replays WAL
- [ ] 2.2 Handler `cluster_resync_replica`
- [ ] 2.3 Route `POST /cluster/replicas/{id}/resync`
- [ ] 2.4 Integration test: deliberately corrupt a replica's WAL, force resync, confirm parity with primary

## 3. Server — peer add + rebalance
- [ ] 3.1 Add `add_peer(addr, role)` in `crates/vectorizer/src/cluster/`
- [ ] 3.2 Add `rebalance()` that moves shards across peers using the existing move_vectors invariant
- [ ] 3.3 Handlers `cluster_add_peer`, `cluster_rebalance`, `cluster_rebalance_status`
- [ ] 3.4 Routes `POST /cluster/peers`, `POST /cluster/rebalance`, `GET /cluster/rebalance/status`
- [ ] 3.5 Integration test: add a 4th peer to a 3-node cluster, run rebalance, verify shard distribution
- [ ] 3.6 Integration test: rebalance is resumable — abort mid-flight, restart, completes from checkpoint

## 4. Server — scoped API keys
- [ ] 4.1 Extend JWT claims with `scopes: [{collection, permissions}]` in `crates/vectorizer/src/auth/jwt.rs`
- [ ] 4.2 Update auth middleware to enforce per-collection scope on every collection-bound route
- [ ] 4.3 Extend `POST /auth/keys` body to accept `scopes`
- [ ] 4.4 Integration test: a key scoped to `c1` returns 200 against `c1` and 403 against `c2`
- [ ] 4.5 Integration test: empty-scopes key has no implicit access (default deny)

## 5. Server — API key rotation
- [ ] 5.1 Handler `rotate_api_key(id)` returning `RotatedKey { old_token, new_token, grace_until }`
- [ ] 5.2 Route `POST /auth/keys/{id}/rotate`
- [ ] 5.3 Auth middleware accepts BOTH old and new keys until `grace_until`
- [ ] 5.4 Integration test: both keys work for the grace window; only new key works after

## 6. Server — token introspection
- [ ] 6.1 Handler `introspect_token` returning RFC 7662 shape (`active`, `scope`, `sub`, `exp`)
- [ ] 6.2 Route `POST /auth/introspect`
- [ ] 6.3 Integration test: valid token returns `active: true` and matching scope; revoked token returns `active: false`

## 7. Server — admin audit log
- [ ] 7.1 Add `AuditLogger` in `crates/vectorizer/src/auth/audit.rs` with in-memory buffer + background flusher
- [ ] 7.2 Hook into every admin-gated handler to record `actor`, `action`, `target`, `at`, `correlation_id`
- [ ] 7.3 Handler `list_audit_log(params)` with time-range + actor + action filters
- [ ] 7.4 Route `GET /auth/audit`
- [ ] 7.5 Integration test: admin action appears in the audit log within the configured flush window
- [ ] 7.6 Integration test: audit log file rotates daily under the configured backup dir

## 8. Rust SDK
- [ ] 8.1 `cluster_failover(&self, replica_id) -> Result<FailoverReport>`
- [ ] 8.2 `cluster_resync_replica(&self, replica_id) -> Result<ResyncJob>`
- [ ] 8.3 `cluster_add_peer(&self, request) -> Result<PeerInfo>`
- [ ] 8.4 `cluster_rebalance(&self) -> Result<RebalanceJob>`
- [ ] 8.5 `cluster_rebalance_status(&self) -> Result<RebalanceStatus>`
- [ ] 8.6 `rotate_api_key(&self, id) -> Result<RotatedKey>`
- [ ] 8.7 `create_scoped_api_key(&self, request) -> Result<ApiKey>`
- [ ] 8.8 `introspect_token(&self, token) -> Result<TokenIntrospection>`
- [ ] 8.9 `list_audit_log(&self, params) -> Result<Vec<AuditEntry>>`
- [ ] 8.10 Bump `sdks/rust/Cargo.toml` 3.6 → 3.7
- [ ] 8.11 Unit + s2s integration tests per method

## 9. TypeScript SDK
- [ ] 9.1 Mirror section 8 in `sdks/typescript/src/client/{replication,auth}.ts`
- [ ] 9.2 Bump `sdks/typescript/package.json` 3.6 → 3.7
- [ ] 9.3 Vitest unit + integration tests

## 10. Python SDK
- [ ] 10.1 Mirror section 8 in `sdks/python/vectorizer/{replication,auth}.py`
- [ ] 10.2 Bump `sdks/python/pyproject.toml` 3.6 → 3.7
- [ ] 10.3 pytest unit + integration tests

## 11. Documentation
- [ ] 11.1 Document new routes in `docs/api/`
- [ ] 11.2 Add a "cluster ops" runbook in `docs/` covering failover, resync, peer add, rebalance
- [ ] 11.3 Add a "multi-tenant auth" guide covering scoped keys, rotation, introspection, audit
- [ ] 11.4 Update SDK READMEs
- [ ] 11.5 CHANGELOG entries (server + each SDK)

## 12. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 12.1 Update or create documentation covering the implementation
- [ ] 12.2 Write tests covering the new behavior
- [ ] 12.3 Run tests and confirm they pass
