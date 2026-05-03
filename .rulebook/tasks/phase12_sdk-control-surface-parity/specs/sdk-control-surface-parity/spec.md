# Spec: SDK control-surface parity

## ADDED Requirements

### Requirement: Single-vector and batch ops in all SDKs
The Rust, TypeScript, and Python SDKs SHALL each expose typed wrappers
for `update_vector`, `insert_text` (single), `list_vectors`,
`get_vector_by_path`, `batch_insert_texts`, `insert_vectors` (raw
vectors), `batch_search`, and `batch_update_vectors` matching the
existing server routes registered in
`crates/vectorizer-server/src/server/core/routing.rs:285-314`.

#### Scenario: Rust update_vector hits POST /update
Given a connected Rust `VectorizerClient`
When the caller invokes `client.update_vector("c", "vec-1", req).await`
Then the SDK issues `POST /update` with `{"collection":"c","id":"vec-1",...}`
And returns the updated `Vector` on a 2xx response

#### Scenario: Rust batch_search hits POST /batch_search
Given a connected Rust `VectorizerClient`
When the caller invokes `client.batch_search(requests).await` with three queries
Then the SDK issues `POST /batch_search` with the array of queries
And returns three `SearchResult` items in the same order

#### Scenario: list_vectors paginates
Given a collection `c` with 250 vectors
When the caller invokes `client.list_vectors("c", page=2, limit=100).await`
Then the response contains vectors 100..199 inclusive
And the response includes `total: 250`

### Requirement: Search variants and discovery pipeline coverage
The SDKs SHALL expose `search_vectors_by_text`, `search_by_file`,
`hybrid_search`, `broad_discovery`, `semantic_focus`,
`promote_readme`, `compress_evidence`, `build_answer_plan`, and
`render_llm_prompt` mapping to the routes registered at
`routing.rs:273-364`.

#### Scenario: broad_discovery wires to the matching route
Given a connected client in any SDK
When the caller invokes the `broad_discovery` method with a request body
Then the SDK issues `POST /discovery/broad_discovery` with that body
And returns the typed response shape used by the server handler

### Requirement: Admin and observability surface in all SDKs
The SDKs SHALL expose typed wrappers for `get_stats`, `get_status`,
`get_logs`, `get_indexing_progress`, `force_save_collection`,
`list_empty_collections`, `cleanup_empty_collections`, `get_config`,
`update_config`, `list_backups`, `create_backup`, `restore_backup`,
`restart_server`, and the workspace surface
(`list_workspaces`, `get_workspace_config`, `add_workspace`,
`remove_workspace`).

#### Scenario: force_save_collection completes a flush
Given a collection with unsaved buffered writes
When the caller invokes `force_save_collection("c")`
Then the SDK issues `POST /collections/c/force-save`
And returns `Ok(())` once the server confirms persistence

#### Scenario: Admin-only routes surface 403 as a typed error
Given a non-admin authenticated client
When the caller invokes `restart_server()`
Then the SDK MUST return a typed `Forbidden` error
And MUST NOT panic

### Requirement: Auth surface in all SDKs
Beyond the existing `login`, the SDKs SHALL expose `me`, `logout`,
`refresh_token`, `validate_password`, `create_api_key`,
`list_api_keys`, `revoke_api_key`, `create_user`, `list_users`,
`delete_user`, and `change_password` matching
`routing.rs:651-678`.

#### Scenario: refresh_token returns a fresh JWT
Given a Rust client with a valid (unexpired) JWT
When the caller invokes `client.refresh_token().await`
Then the SDK issues `POST /auth/refresh`
And returns a new `JwtToken` with a later `exp`

### Requirement: Replication and Hub surface in all SDKs
The SDKs SHALL expose `get_replication_status`,
`configure_replication`, `get_replication_stats`, `list_replicas`,
plus the `/hub/backups/*` and `/hub/usage/*` routes registered at
`routing.rs:208-249`.

#### Scenario: get_replication_status reports primary state
Given a server configured as a primary
When the caller invokes `get_replication_status()`
Then the SDK issues `GET /replication/status`
And the response `role` field is `"primary"`

### Requirement: Additive backward compatibility
This phase SHALL be additive only. No existing SDK 3.3 method
signature MAY change, and no server route MAY change. SDKs bump
3.3 → 3.4.

#### Scenario: 3.3 SDK keeps working against 3.4 SDK server-side contracts
Given a 3.3 SDK client
When the caller performs every method that already existed in 3.3
Then every call returns the same response shape it did in 3.3
