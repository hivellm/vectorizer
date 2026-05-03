# Spec: schema evolution and explain

## ADDED Requirements

### Requirement: Atomic collection rename with alias grace window
The server SHALL expose `POST /collections/{name}/rename` that
atomically renames a collection and keeps the old name reachable as an
alias for one minor version.

#### Scenario: Reads through old name continue to work
Given collection `c1` exists and contains `vec-1`
When the client posts `{"new_name":"c2"}` to `/collections/c1/rename`
Then subsequent reads of `c1/vec-1` succeed and return the same vector as `c2/vec-1`
And subsequent writes to `c1` MUST land in `c2`

#### Scenario: Alias expires after grace window
Given the rename above has occurred
When the configured alias-grace window has elapsed
Then reads of `c1` return HTTP 404
And reads of `c2` continue to work

### Requirement: Reindex without re-embedding
The server SHALL expose `POST /collections/{name}/reindex` that rebuilds
the HNSW index with new parameters using the existing stored vectors.
No re-embedding occurs.

#### Scenario: M=16 to M=32
Given collection `c` is built with `M=16`, `ef_construction=200`
When the client posts `{"M":32,"ef_construction":400}` to `/collections/c/reindex`
Then the server returns a `ReindexJob` with a job id
And once the job completes, queries against `c` succeed
And recall@10 against a fixed query set is greater than or equal to the M=16 baseline

#### Scenario: Concurrent writes during reindex
Given a reindex on collection `c` is in progress
When the client inserts `vec-new` while the job is running
Then `vec-new` MUST be queryable in `c` after the job completes

### Requirement: Native snapshot endpoints
The server SHALL expose native snapshot create/list/restore endpoints
parallel to the Qdrant-compatible snapshot routes.

#### Scenario: Round-trip a collection through snapshot
Given collection `c` contains 1000 vectors with metadata
When the client creates a snapshot, drops `c`, and restores from the snapshot
Then `c` again contains exactly 1000 vectors
And every vector's id, dimensions, and metadata match the pre-drop state

### Requirement: Explain search
The server SHALL expose `POST /collections/{name}/explain` that runs the
same search code path as production search and returns an execution
trace alongside hits.

#### Scenario: Trace fields populated
Given a search query against collection `c` configured with `ef_search=64`
When the client posts the query to `/collections/c/explain`
Then the response contains `hits` (same shape as a regular search response)
And the response contains `trace` with fields `visited_nodes`, `layer_path`, `payload_filter_evals`, `quantization_score_ms`
And `visited_nodes <= ef_search × layers`

#### Scenario: Explain hits match production search hits
Given the same query
When the client runs `search` and `explain` against `c`
Then the ordered list of hit ids MUST be identical between the two responses

### Requirement: Slow-query log (in-memory ring buffer)
The server SHALL maintain an in-memory ring buffer of search queries
whose duration meets or exceeds a configured threshold, and expose it
via `GET /slow_queries` and `POST /slow_queries/config`.

#### Scenario: Above-threshold query is recorded
Given the slow-query threshold is configured to 100 ms
When a search completes in 200 ms
Then `GET /slow_queries` includes that search with its duration, query payload, and collection name

#### Scenario: Below-threshold query is NOT recorded
Given the slow-query threshold is 100 ms
When a search completes in 20 ms
Then `GET /slow_queries` MUST NOT include that search

#### Scenario: Ring buffer evicts at capacity
Given the slow-query buffer capacity is 10
When 11 above-threshold queries occur in sequence
Then `GET /slow_queries` returns 10 entries
And the oldest entry MUST be the second query, not the first

### Requirement: SDK parity for schema-evolution and explain
The Rust, TypeScript, and Python SDKs SHALL each expose eight methods
matching the wire contracts above:
`rename_collection`, `reindex_collection`,
`snapshot_collection_native`, `list_collection_snapshots_native`,
`restore_collection_snapshot_native`, `explain_search`,
`list_slow_queries`, `set_slow_query_config`.

#### Scenario: Rust explain_search wires correctly
Given a connected Rust client
When the caller invokes `client.explain_search("c", request).await`
Then the SDK issues `POST /collections/c/explain`
And returns an `ExplainResponse` matching the server response

### Requirement: Additive backward compatibility
This phase SHALL be additive only. SDKs and server bump 3.5 → 3.6.

#### Scenario: 3.5 client works against 3.6 server
Given a 3.5 SDK client
When the caller performs every method that already existed in 3.5
Then every call returns the same response shape it did in 3.5
