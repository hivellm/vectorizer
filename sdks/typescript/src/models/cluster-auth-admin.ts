/**
 * Cluster + auth admin models (phase15).
 *
 * Wire shapes are byte-for-byte matches of the server handlers in
 * `replication_handlers.rs` and `auth_handlers/phase15.rs`.
 */

// ── Cluster failover ──────────────────────────────────────────────────────────

/**
 * Report returned by `POST /cluster/failover`.
 *
 * Server contract: `{promoted_replica_id, master_offset_at_promotion,
 * replica_offset_at_promotion, residual_lag_operations}`.
 */
export interface FailoverReport {
  /** ID of the replica that was promoted to primary. */
  promoted_replica_id: string;
  /** Master WAL offset at the time of promotion. */
  master_offset_at_promotion: number;
  /** Replica's confirmed offset at the time of promotion. */
  replica_offset_at_promotion: number;
  /** Remaining lag in WAL operations. */
  residual_lag_operations: number;
}

// ── Cluster resync ────────────────────────────────────────────────────────────

/**
 * Report returned by `POST /cluster/replicas/{id}/resync`.
 *
 * Server contract: `{replica_id, snapshot_offset, full_snapshot}`.
 */
export interface ResyncJob {
  /** ID of the replica being resynced. */
  replica_id: string;
  /** Master WAL offset used as the snapshot baseline. */
  snapshot_offset: number;
  /** Whether a full snapshot transfer was initiated. */
  full_snapshot: boolean;
}

// ── Cluster peers ─────────────────────────────────────────────────────────────

/**
 * Information about a newly added cluster peer.
 *
 * Returned by `POST /cluster/peers`.
 * Server contract: `{node_id, address, role}`.
 */
export interface PeerInfo {
  /** Opaque node id assigned by the server. */
  node_id: string;
  /** Address of the peer (host:port). */
  address: string;
  /** Role string: `"member"` or `"observer"`. */
  role: string;
}

/**
 * Request body for `POST /cluster/peers`.
 */
export interface AddPeerRequest {
  /** Address of the new peer (host:port). */
  address: string;
  /** Role: `"member"` (default) or `"observer"`. */
  role?: string;
}

// ── Cluster rebalance ─────────────────────────────────────────────────────────

/**
 * Job descriptor returned by `POST /cluster/rebalance` and
 * `GET /cluster/rebalance/status`.
 *
 * Server contract: `{job_id, status, shards_to_move, shards_moved,
 * last_checkpoint_node?, message}`.
 */
export interface RebalanceJob {
  /** Opaque job id. */
  job_id: string;
  /** Lifecycle state: `"running"` | `"paused"` | `"completed"` | `"failed"`. */
  status: string;
  /** Total shards that need to move. */
  shards_to_move: number;
  /** Shards moved so far. */
  shards_moved: number;
  /** Node-id of the last checkpoint (last completed shard move target). */
  last_checkpoint_node?: string;
  /** Human-readable status message. */
  message: string;
}

// ── Auth key rotation ─────────────────────────────────────────────────────────

/**
 * Response from `POST /auth/keys/{id}/rotate`.
 *
 * Server contract: `{old_key_id, new_key_id, new_token, grace_until}`.
 */
export interface RotatedKey {
  /** The old key id (still valid until `grace_until`). */
  old_key_id: string;
  /** The new key id. */
  new_key_id: string;
  /** The new key token value — store it securely. */
  new_token: string;
  /** Unix timestamp until which the OLD key is still accepted. */
  grace_until: number;
}

// ── Auth scoped key creation ──────────────────────────────────────────────────

/**
 * Per-collection permission scope in `CreateScopedApiKeyRequest`.
 */
export interface TokenScope {
  /** Collection name this scope applies to. */
  collection: string;
  /** Permissions granted on that collection (e.g. `["read", "write"]`). */
  permissions?: string[];
}

/**
 * Request body for `POST /auth/keys` — extended with optional per-collection scopes.
 */
export interface CreateScopedApiKeyRequest {
  /** Key name / description. */
  name: string;
  /** Global permissions (defaults to `["Read"]`). */
  permissions?: string[];
  /** TTL in seconds from now (undefined = never expires). */
  expires_in?: number;
  /** Per-collection scopes. Empty = default-deny on scope-enforced routes. */
  scopes?: TokenScope[];
}

// ── Auth introspection ────────────────────────────────────────────────────────

/**
 * RFC 7662 token introspection response from `POST /auth/introspect`.
 *
 * Server contract: `{active, scope?, sub?, exp?, username?}`.
 */
export interface TokenIntrospection {
  /** Whether the token is currently active. */
  active: boolean;
  /** Space-separated scope string. */
  scope?: string;
  /** Subject (user_id or key_id). */
  sub?: string;
  /** Expiry (Unix timestamp). */
  exp?: number;
  /** Username (non-standard extension; omitted for inactive tokens). */
  username?: string;
}

// ── Auth audit log ────────────────────────────────────────────────────────────

/**
 * One entry in the admin audit log returned by `GET /auth/audit`.
 *
 * Server contract: `{actor, action, target, at, correlation_id?}`.
 */
export interface AuditEntry {
  /** Username or key-id of the actor. */
  actor: string;
  /** Canonical action name, e.g. `"create_api_key"`. */
  action: string;
  /** Target resource (key-id, collection name, username). */
  target: string;
  /** UTC timestamp (RFC-3339). */
  at: string;
  /** Correlation-ID propagated from the request middleware. */
  correlation_id?: string;
}

/**
 * Query parameters for `GET /auth/audit`.
 */
export interface AuditQuery {
  /** Filter by actor. */
  actor?: string;
  /** Filter by action name. */
  action?: string;
  /** Entries at or after this RFC-3339 timestamp. */
  since?: string;
  /** Entries at or before this RFC-3339 timestamp. */
  until?: string;
  /** Maximum entries to return (server default 200). */
  limit?: number;
}
