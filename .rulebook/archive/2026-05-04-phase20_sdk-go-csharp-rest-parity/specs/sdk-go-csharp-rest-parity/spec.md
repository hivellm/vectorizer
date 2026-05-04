# Spec: sdk-go-csharp-rest-parity

## ADDED Requirements

### Requirement: Go REST admin/observability surface

The Go SDK MUST expose REST helpers for every admin and observability
endpoint already shipped in the Rust SDK at version 3.4 (phase12 §4).

#### Scenario: Stats round-trip
Given a vectorizer-server running on the default port
When a Go client calls `GetStats(ctx)`
Then it receives a `Stats` value populated from `GET /stats` with no
JSON unmarshalling errors

#### Scenario: Admin-gated config update is rejected for non-admin
Given a Go client authenticated with a non-admin API key
When it calls `UpdateConfig(ctx, patch)`
Then the server returns HTTP 403 and the SDK surfaces a typed
`*VectorizerError` with code `forbidden`

### Requirement: Go REST auth/RBAC surface

The Go SDK MUST expose REST helpers for the auth/RBAC routes shipped
in phase12 §5 (`/auth/me`, `/auth/logout`, `/auth/refresh`,
`/auth/validate-password`, `/auth/keys`, `/auth/users`).

#### Scenario: Create + list + revoke API key
Given an authenticated admin client
When it calls `CreateApiKey`, then `ListApiKeys`, then `RevokeApiKey`
Then the created key appears in the list and is absent after revoke

### Requirement: Go REST replication surface

The Go SDK MUST expose REST helpers for `/replication/status`,
`/replication/configure`, `/replication/stats`, `/replication/replicas`.

#### Scenario: Status decode
Given a master node with one connected replica
When the Go client calls `GetReplicationStatus(ctx)`
Then the response decodes into `ReplicationStatus` with a non-empty
`Replicas` slice

### Requirement: Go REST hub backup + usage surface

The Go SDK MUST expose REST helpers for the hub backup CRUD routes
(`/hub/users/{u}/backups[/{id}][/download]`) and the hub usage routes
(`/hub/users/{u}/usage`, `/hub/users/{u}/quota`,
`/hub/api-keys/{k}/validate`).

### Requirement: Go REST discovery pipeline surface

The Go SDK MUST expose REST helpers for every stage of the discovery
pipeline shipped in phase12 §3.4-3.9: broad discovery, semantic focus,
README promotion, evidence compression, answer plan, LLM prompt
rendering.

### Requirement: Go REST vectors single + batch + search variants

The Go SDK MUST expose REST helpers for the vector ops added in
phase12 §2-3 that the existing Go SDK does not yet cover, including
`update_vector`, `insert_text` (single), `list_vectors`,
`get_vector_by_path`, `batch_insert_texts`, `insert_vectors` (raw),
`batch_search`, `batch_update_vectors`, `search_by_text`,
`search_by_file`.

### Requirement: Go REST tier-control surface

The Go SDK MUST expose REST helpers for the six tier-control
primitives shipped in phase13: `delete_by_filter`,
`bulk_update_metadata`, `copy_vectors`, `reencode_collection`,
`set_collection_ttl`, `set_vector_expiry`.

#### Scenario: Empty filter is rejected client-side
Given a Go client
When it calls `DeleteByFilter(ctx, "c1", Filter{})` with an empty
filter
Then the SDK returns a validation error before issuing any HTTP
request, mirroring the server-side 400 protection

### Requirement: Go REST schema evolution + explain + slow queries

The Go SDK MUST expose REST helpers for `rename_collection`,
`reindex_collection`, `snapshot_native` (create/list/restore),
`explain_search`, and `slow_queries` (list + config), shipped in
phase14.

### Requirement: Go REST cluster + auth admin surface

The Go SDK MUST expose REST helpers for `cluster_failover`,
`cluster_resync_replica`, `cluster_add_peer`, `cluster_rebalance`,
`cluster_rebalance_status`, `rotate_api_key`, `create_scoped_api_key`,
`introspect_token`, `list_audit_log`, shipped in phase15.

### Requirement: C# REST control surface mirrors Go

The C# SDK MUST expose every method enumerated for the Go SDK above
on `HttpVectorizerClient` and the `IVectorizerClient` interface,
named in idiomatic .NET PascalCase, returning `Task<T>` for response
data and `Task` for void results.

#### Scenario: IVectorizerClient is the single REST entrypoint
Given consumer code holding only an `IVectorizerClient` reference
When it invokes any phase12-15 helper
Then the method is reachable through the interface (not only the
concrete `HttpVectorizerClient`)

### Requirement: SDK versions bumped to 3.9.0

Both `sdks/go/version.go` and the two C# csproj files
(`Vectorizer.csproj`, `Vectorizer.Rpc.csproj`) MUST advertise
version `3.9.0` once the parity surface lands.

### Requirement: Wire-shape tests per surface

Every method added in this phase MUST have at least one wire-shape
unit test that asserts the request URL, method, body shape, and
response decoding against a captured server response.

#### Scenario: HTTP fixture round-trip
Given a captured 200-OK response body for `GET /stats`
When the Go test calls `GetStats(ctx)` against an `httptest.Server`
returning that body
Then the parsed `Stats` matches a hand-written expected value
field-for-field

### Requirement: Live-server integration gated behind opt-in tags

Live-server integration tests for both SDKs MUST be gated so the
default `go test ./...` and `dotnet test` runs do not require a
running vectorizer-server.

#### Scenario: Default Go test run is hermetic
Given a clean checkout
When CI runs `go test ./...`
Then no test attempts to dial port 15002 or 15503

### Requirement: Documentation reflects parity

`sdks/COVERAGE_REPORT.md` MUST be updated to drop the
`🚧 partial` markers from the Go and C# rows, and
`docs/users/api/API_REFERENCE.md` MUST gain Go and C# columns in the
SDK 3.4-3.7 control surface tables.
