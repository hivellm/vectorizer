# Spec: SDK tier-demotion API

## ADDED Requirements

### Requirement: Move vectors endpoint
The server SHALL expose `POST /collections/{src}/vectors/move` that
relocates one or more vectors from a source collection to a destination
collection without re-embedding.

#### Scenario: Happy path moves vectors
Given collection `cortex.consolidation.fp32` contains vectors `vec-1` and `vec-2`
And collection `cortex.consolidation.pq` exists with a compatible schema
When the client posts `{"destination": "cortex.consolidation.pq", "ids": ["vec-1", "vec-2"]}` to `/collections/cortex.consolidation.fp32/vectors/move`
Then the response body contains `requested: 2`, `moved: 2`, `failed: 0`
And every entry in `results` has `status: "ok"`
And `cortex.consolidation.pq` now holds `vec-1` and `vec-2`
And `cortex.consolidation.fp32` no longer holds `vec-1` or `vec-2`

#### Scenario: Missing id reported per-row without aborting
Given collection `cortex.consolidation.fp32` contains `vec-1` but not `vec-missing`
When the client posts `{"destination": "cortex.consolidation.pq", "ids": ["vec-1", "vec-missing"]}`
Then the response body contains `requested: 2`, `moved: 1`, `failed: 1`
And `results` includes `{"id": "vec-1", "status": "ok"}`
And `results` includes `{"id": "vec-missing", "status": "missing_in_src"}`

#### Scenario: Insert-before-delete on dim mismatch
Given collection `cortex.consolidation.fp32` has 384-dim vectors
And collection `cortex.dst.different-dim` has 768-dim vectors
And `vec-1` exists in `cortex.consolidation.fp32`
When the client posts `{"destination": "cortex.dst.different-dim", "ids": ["vec-1"]}`
Then the response status for `vec-1` MUST be `dst_insert_failed`
And `vec-1` MUST still exist in `cortex.consolidation.fp32`

### Requirement: SDKs expose delete_vector / delete_vectors / move_to_collection
The Rust, TypeScript, and Python SDKs SHALL each expose three methods —
`delete_vector`, `delete_vectors`, `move_to_collection` (snake_case for
Rust + Python, camelCase `deleteVector` / `deleteVectors` /
`moveToCollection` for TypeScript) — with the wire contract above.

#### Scenario: Rust SDK delete_vector calls the server
Given a connected Rust `VectorizerClient`
When the caller invokes `client.delete_vector("cortex.consolidation.fp32", "vec-1").await`
Then the SDK issues `DELETE /collections/cortex.consolidation.fp32/vectors/vec-1`
And returns `Ok(())` on a 2xx response

#### Scenario: Rust SDK delete_vectors calls batch_delete
Given a connected Rust `VectorizerClient`
When the caller invokes `client.delete_vectors("cortex.consolidation.fp32", &["vec-1", "vec-2"]).await`
Then the SDK issues `POST /batch_delete` with `{"collection": "cortex.consolidation.fp32", "ids": ["vec-1", "vec-2"]}`
And returns a `DeleteReport` whose `results` mirror per-id status

#### Scenario: Rust SDK move_to_collection calls the new endpoint
Given a connected Rust `VectorizerClient`
When the caller invokes `client.move_to_collection("cortex.consolidation.fp32", "cortex.consolidation.pq", &["vec-1"]).await`
Then the SDK issues `POST /collections/cortex.consolidation.fp32/vectors/move` with `{"destination": "cortex.consolidation.pq", "ids": ["vec-1"]}`
And returns a `MoveReport` matching the server response

### Requirement: Additive backward compatibility
The endpoint, handlers, and SDK methods introduced by this task SHALL
NOT break any existing 3.2 client. Existing routes and methods MUST
remain unchanged.

#### Scenario: 3.2 SDK still works against 3.3 server
Given a running 3.3 vectorizer-server
When a 3.2 SDK client performs `get_vector` / `insert_texts` / `search`
Then every operation returns the same response shape it did against a 3.2 server
