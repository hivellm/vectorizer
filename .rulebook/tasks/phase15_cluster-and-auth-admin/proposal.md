# Proposal: phase15_cluster-and-auth-admin

Source: gap audit follow-up to phase11_sdk-tier-demotion-api. Companion
to phase12 (parity), phase13 (tier control), and phase14 (Day-2 ops).

## Why

Two operational gaps remain after phases 12-14:

1. **Cluster admin** — `crates/vectorizer-server/src/server/core/routing.rs:382-397`
   exposes only `/replication/status`, `/replication/configure`,
   `/replication/stats`, `/replication/replicas`. There is no:
   - **Failover trigger** — promote a replica to primary on demand.
   - **Replica resync** — force a full state replay when WAL drifts.
   - **Shard rebalance** — redistribute shards after adding a peer.
   - **Peer add** beyond the Qdrant cluster API which only handles
     remove (`qdrant::cluster_handlers::remove_peer`).

2. **Auth/RBAC admin** — phase12 ships the existing surface
   (login/me/logout/refresh/keys/users/change_password). What is still
   missing for production multi-tenant deployments:
   - **API key rotation** — atomic rotate without revoking the old key
     until the client confirms it has the new one.
   - **Scoped tokens** — issue tokens that can read/write only specific
     collections (today every key is global).
   - **Audit log** of admin actions (who created which key, who
     promoted whom).
   - **Token introspection** — `POST /auth/introspect` so a downstream
     service can validate a token without re-implementing JWT logic.

Without these, every operator wires their own ad-hoc cluster scripts
and every multi-tenant deployment ends up sharing a single root-power
key — an antipattern the security-reviewer flags every time.

## What Changes

### 1. Server — cluster admin endpoints

| Method | Route | Purpose |
|---|---|---|
| `POST` | `/cluster/failover` | Promote a named replica to primary; old primary becomes replica |
| `POST` | `/cluster/replicas/{id}/resync` | Force a full state replay from primary to the named replica |
| `POST` | `/cluster/peers` | Add a peer (the Qdrant API only supports remove) |
| `POST` | `/cluster/rebalance` | Trigger shard rebalance across peers |
| `GET`  | `/cluster/rebalance/status` | Report progress of an active rebalance |

### 2. Server — auth/RBAC admin endpoints

| Method | Route | Purpose |
|---|---|---|
| `POST` | `/auth/keys/{id}/rotate` | Atomically rotate an API key; old + new both valid for grace window |
| `POST` | `/auth/keys` (extended body) | Accept `scopes: [{collection, permissions}]` for collection-scoped keys |
| `POST` | `/auth/introspect` | Return token validity, subject, scopes, expiry |
| `GET`  | `/auth/audit` | Read the admin-action audit log |

### 3. Server — implementation notes

- **Failover**: leverage existing replication state machine; gate
  behind admin role; expose pre-flight check that all WAL segments are
  caught up before the promote.
- **Resync**: stream a fresh snapshot to the target replica then catch
  up via WAL.
- **Rebalance**: shard-by-shard move with the existing move_vectors
  invariant (insert-before-delete) reused.
- **Scoped tokens**: extend the JWT claim set with `scopes: [{c,p}]`;
  middleware checks the scope on every collection-bound route.
- **Audit log**: append-only file under the configured backup dir;
  rotated daily; includes `actor`, `action`, `target`, `at`,
  `correlation_id`.
- **Token introspection**: RFC 7662 shape (`active`, `scope`, `sub`,
  `exp`).

### 4. SDKs (Rust + TS + Python)

```rust
client.cluster_failover(replica_id)                    -> Result<FailoverReport>
client.cluster_resync_replica(replica_id)              -> Result<ResyncJob>
client.cluster_add_peer(request)                       -> Result<PeerInfo>
client.cluster_rebalance()                             -> Result<RebalanceJob>
client.cluster_rebalance_status()                      -> Result<RebalanceStatus>
client.rotate_api_key(id)                              -> Result<RotatedKey>
client.create_scoped_api_key(request)                  -> Result<ApiKey>
client.introspect_token(token)                         -> Result<TokenIntrospection>
client.list_audit_log(params)                          -> Result<Vec<AuditEntry>>
```

Workspace bump: server + SDKs 3.6 → 3.7.

## Impact

- Affected specs: `.rulebook/tasks/phase15_cluster-and-auth-admin/specs/cluster-and-auth-admin/spec.md`
- Affected code:
  - `crates/vectorizer-server/src/server/core/routing.rs`
  - `crates/vectorizer-server/src/server/rest_handlers/replication_handlers.rs`
  - `crates/vectorizer-server/src/server/auth_handlers.rs`
  - `crates/vectorizer/src/replication/` (failover, resync, peer add)
  - `crates/vectorizer/src/cluster/` (rebalance)
  - `crates/vectorizer/src/auth/` (scoped JWT, introspection, audit log)
  - `sdks/{rust,typescript,python}/...`
- Breaking change: NO — additive. Existing keys remain global by
  default; scopes are optional.
- User benefit:
  - Operators can drive cluster topology changes from the SDK.
  - Multi-tenant deployments can issue per-collection scoped keys.
  - Audit trail exists for compliance reviews.
  - Downstream services can validate tokens via a standard endpoint.

## Constraints

- **Failover** MUST refuse if the target replica is more than `N` WAL
  segments behind primary (configurable, default 1).
- **Rotate** MUST keep the old key valid for the configured grace
  window (default 5 min) so deployed clients can roll over without
  downtime.
- **Scoped tokens** MUST default to "no implicit access" — a key with
  empty scopes can do nothing.
- **Audit log** MUST NOT block writes; uses an in-memory buffer +
  background flusher.

## Acceptance

- Nine new server routes with handlers + integration tests.
- Failover test: 3-node cluster, primary fails, manual failover
  promotes a replica, all writes resume on the new primary.
- Rotate test: rotated key works concurrently with the prior key for
  the grace window, then the prior key starts returning 401.
- Scoped-key test: a key scoped to `c1` returns 403 against `c2`.
- Audit-log test: admin action appears in the audit log within
  configured flush window.
- All three SDKs expose the nine methods with typed reports.
- Server + SDKs bumped 3.6 → 3.7.
