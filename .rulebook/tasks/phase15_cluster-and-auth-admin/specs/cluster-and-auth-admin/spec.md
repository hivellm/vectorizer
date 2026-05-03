# Spec: cluster and auth admin

## ADDED Requirements

### Requirement: Manual cluster failover
The server SHALL expose `POST /cluster/failover` that promotes a named
replica to primary, demoting the prior primary to replica, gated by a
WAL-lag pre-flight check.

#### Scenario: Failover succeeds when caught up
Given a 3-node cluster where every replica's WAL lag is ≤ N segments
When an admin client posts `{"replica_id":"r2"}` to `/cluster/failover`
Then the server promotes `r2` to primary
And subsequent writes succeed against `r2`
And the prior primary becomes a replica

#### Scenario: Failover refused on lagging replica
Given a 3-node cluster where `r2` is `> N` WAL segments behind primary
When an admin client posts `{"replica_id":"r2"}` to `/cluster/failover`
Then the server MUST return HTTP 409 Conflict with an error body explaining the lag
And the cluster topology MUST NOT change

### Requirement: Replica resync
The server SHALL expose `POST /cluster/replicas/{id}/resync` that
streams a fresh snapshot from primary to the named replica then
catches up via WAL.

#### Scenario: Resync recovers a corrupted replica
Given replica `r2` has a corrupted WAL
When an admin client posts to `/cluster/replicas/r2/resync`
Then the server returns a `ResyncJob` with a job id
And once the job completes, `r2`'s state is byte-identical to primary's

### Requirement: Cluster peer add and rebalance
The server SHALL expose `POST /cluster/peers` to add a peer, and
`POST /cluster/rebalance` (with `GET /cluster/rebalance/status`) to
redistribute shards.

#### Scenario: Rebalance after adding a peer
Given a 3-node cluster
When the admin adds a 4th peer and triggers rebalance
Then shards MUST be redistributed so that no peer holds more than `ceil(total / 4)` shards
And the rebalance status endpoint reports `state: "completed"` with the moved-shard count

#### Scenario: Rebalance is resumable
Given a rebalance is in progress
When the rebalance is aborted mid-flight and restarted
Then the second run resumes from a persisted checkpoint
And no shard data is duplicated or lost across the two runs

### Requirement: Scoped API keys
The server SHALL accept `scopes: [{collection, permissions}]` in
`POST /auth/keys` and SHALL enforce scopes on every collection-bound
route.

#### Scenario: Scoped key allowed on its collection
Given an API key scoped to collection `c1` with read+write permissions
When the holder calls any data route against `c1`
Then the response is 200 (success)

#### Scenario: Scoped key denied on other collections
Given the same API key
When the holder calls any data route against `c2`
Then the response MUST be 403 Forbidden

#### Scenario: Empty-scopes key has no implicit access
Given an API key created with empty scopes
When the holder calls any collection-bound route
Then the response MUST be 403 Forbidden

### Requirement: API key rotation with grace window
The server SHALL expose `POST /auth/keys/{id}/rotate` that issues a new
key while keeping the prior key valid for a configured grace window.

#### Scenario: Both keys valid during grace window
Given an admin rotates key `k1`
When a client uses either the old or the new key inside the grace window
Then both calls succeed (200)

#### Scenario: Old key invalidated after grace
Given the grace window has elapsed
When a client uses the old key
Then the response MUST be 401 Unauthorized
And the new key continues to succeed

### Requirement: Token introspection
The server SHALL expose `POST /auth/introspect` returning the RFC 7662
introspection response shape.

#### Scenario: Active token introspection
Given a valid unrevoked token with scope `c1:read`
When a service posts the token to `/auth/introspect`
Then the response contains `active: true`, `sub`, `scope: "c1:read"`, and `exp`

#### Scenario: Revoked token introspection
Given a token whose key was revoked
When a service posts the token to `/auth/introspect`
Then the response contains `active: false`

### Requirement: Admin audit log
The server SHALL append every admin-gated action to an audit log
exposed via `GET /auth/audit`.

#### Scenario: Admin action recorded
Given an admin creates an API key
When the admin queries `/auth/audit` after the configured flush window
Then the audit log contains an entry with `actor`, `action: "auth.key.create"`, `target`, `at`, and `correlation_id`

#### Scenario: Audit log filtering by actor and action
Given an audit log with entries for multiple actors and actions
When the admin queries `/auth/audit?actor=admin&action=auth.key.rotate`
Then the response contains only entries matching both filters

### Requirement: SDK parity for cluster + auth admin
The Rust, TypeScript, and Python SDKs SHALL each expose nine methods
matching the wire contracts above:
`cluster_failover`, `cluster_resync_replica`, `cluster_add_peer`,
`cluster_rebalance`, `cluster_rebalance_status`, `rotate_api_key`,
`create_scoped_api_key`, `introspect_token`, `list_audit_log`.

#### Scenario: Rust cluster_failover wires correctly
Given a connected Rust admin client
When the caller invokes `client.cluster_failover("r2").await`
Then the SDK issues `POST /cluster/failover` with `{"replica_id":"r2"}`
And returns a `FailoverReport` matching the server response

### Requirement: Additive backward compatibility
This phase SHALL be additive only. Existing API keys remain global by
default; scopes are opt-in. SDKs and server bump 3.6 → 3.7.

#### Scenario: 3.6 client works against 3.7 server
Given a 3.6 SDK client
When the caller performs every method that already existed in 3.6
Then every call returns the same response shape it did in 3.6
