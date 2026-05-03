# Spec: tier-control primitives

## ADDED Requirements

### Requirement: Server-side filter-based deletion
The server SHALL expose `POST /collections/{name}/vectors/delete_by_filter`
that deletes every vector matching a metadata predicate without the
client enumerating IDs.

#### Scenario: Filter selects and deletes a subset
Given collection `audit.events` contains 1000 vectors, 300 with `tier=hot` and 700 with `tier=cold`
When the client posts `{"filter": {"tier": "hot"}}` to `/collections/audit.events/vectors/delete_by_filter`
Then the response body contains `scanned: 1000`, `matched: 300`, `deleted: 300`
And the collection now contains 700 vectors all with `tier=cold`

#### Scenario: Empty filter is rejected
Given any collection
When the client posts `{"filter": {}}` to the endpoint
Then the server MUST respond with HTTP 400 and an error body explaining empty filters are forbidden
And no vectors MUST be deleted

### Requirement: Bulk metadata update
The server SHALL expose `POST /collections/{name}/vectors/bulk_update_metadata`
that applies a JSON-merge patch to the metadata of every vector matching
a predicate.

#### Scenario: Tier transition via patch
Given collection `c` contains vectors with `{"tier":"hot"}` (n=10) and `{"tier":"warm"}` (n=5)
When the client posts `{"filter":{"tier":"hot"},"patch":{"tier":"warm"}}`
Then the response contains `matched: 10`, `updated: 10`
And every vector with `tier=hot` is now `tier=warm`
And the 5 originally-warm vectors are unchanged

#### Scenario: Idempotent re-run
Given the patch above has been applied
When the same request is replayed
Then `matched: 0`, `updated: 0` (no hot vectors remain)
And no vector data is modified

### Requirement: Cross-collection copy
The server SHALL expose `POST /collections/{src}/vectors/copy` that
inserts the listed source vectors into a destination collection without
deleting them from the source.

#### Scenario: Copy preserves source
Given collection `src` contains `vec-1` and `vec-2`
And collection `dst` exists with a compatible schema
When the client posts `{"destination":"dst","ids":["vec-1","vec-2"]}` to `/collections/src/vectors/copy`
Then the response contains `requested: 2`, `copied: 2`, `failed: 0`
And `dst` contains `vec-1` and `vec-2`
And `src` STILL contains `vec-1` and `vec-2`

### Requirement: In-place re-encoding
The server SHALL expose `POST /collections/{name}/reencode` that switches
the quantization encoding of an existing collection without
re-embedding the source data.

#### Scenario: fp32 to PQ retains recall
Given collection `c` is fp32 with 384 dimensions and 10000 vectors
When the client posts `{"target_encoding":"pq"}` to `/collections/c/reencode`
Then the server returns a `ReencodeJob` with a job id
And once the job reports `state: "completed"`, queries against `c` succeed
And recall@10 against a fixed query set is within the configured tolerance vs the fp32 baseline

#### Scenario: Writes during reencode are not lost
Given a reencode is in progress on collection `c`
When the client inserts `vec-new` while the job is running
Then `vec-new` MUST be queryable in `c` after the job completes

### Requirement: TTL — collection-wide and per-vector
The server SHALL accept a TTL configuration per collection
(`POST /collections/{name}/ttl`) and an `expires_at` per vector
(`PATCH /collections/{name}/vectors/{id}/expiry`). A reaper task SHALL
delete expired vectors in the background.

#### Scenario: Per-vector expiry
Given vector `vec-1` exists with no expiry
When the client patches `vec-1` with `{"expires_at": "<now+100ms>"}`
And the client waits 200ms
Then `GET /collections/c/vectors/vec-1` returns 404
And `/stats` reports `ttl_vectors_expired_total >= 1`

#### Scenario: Collection-wide TTL
Given collection `c` has TTL set to 5 seconds
When the client inserts `vec-new` without a per-vector expiry
Then after 6 seconds `vec-new` is deleted by the reaper
And other collections without TTL are unaffected

### Requirement: SDK parity for tier-control primitives
The Rust, TypeScript, and Python SDKs SHALL each expose six methods
(`delete_by_filter`, `bulk_update_metadata`, `copy_vectors`,
`reencode_collection`, `set_collection_ttl`, `set_vector_expiry`)
matching the wire contracts above.

#### Scenario: Rust delete_by_filter wires correctly
Given a connected Rust client
When the caller invokes `client.delete_by_filter("c", filter).await`
Then the SDK issues `POST /collections/c/vectors/delete_by_filter`
And returns a `DeleteByFilterReport` matching the server response

### Requirement: Additive backward compatibility
This phase SHALL be additive only. SDKs and server bump 3.4 → 3.5.

#### Scenario: 3.4 client works against 3.5 server
Given a 3.4 SDK client
When the caller performs every method that already existed in 3.4
Then every call returns the same response shape it did in 3.4
