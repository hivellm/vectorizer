# Spec: insert_texts — client IDs and chunk metadata consistency

## ADDED Requirements

### Requirement: Honor client-provided id in /insert and /insert_texts
The system SHALL use the `id` field provided in each text entry of `/insert_texts` (and the equivalent field in `/insert`) as the `Vector.id` for the resulting point(s), instead of always generating a server-side UUID.

#### Scenario: Non-chunked text with client id
Given a `POST /insert_texts` request with `texts: [{"id": "doc:42", "text": "<150 chars>", "metadata": {...}}]`
And the text length is below the chunking threshold
When the server inserts the vector
Then the resulting `Vector.id` MUST equal `"doc:42"` exactly
And `points/scroll` MUST return that vector with `id == "doc:42"`

#### Scenario: Chunked text with client id
Given a `POST /insert_texts` request with `texts: [{"id": "doc:42", "text": "<3500 chars>", "metadata": {...}}]`
And the text triggers auto-chunking and produces N chunks
When the server inserts the chunks
Then each resulting `Vector.id` MUST equal `format!("doc:42#{}", chunk_index)` (e.g., `"doc:42#0"`, `"doc:42#1"`, ...)
And the response body MUST list those ids in `vector_ids`

#### Scenario: Re-ingest is idempotent (upsert by client id)
Given a vector with `id == "doc:42"` already exists in the collection
When the same `POST /insert_texts` request is repeated with the same `id`
Then the existing vector MUST be replaced (not duplicated)
And the count of vectors with `id == "doc:42"` MUST remain 1

#### Scenario: Missing client id falls back to UUID
Given a `POST /insert_texts` request with `texts: [{"text": "...", "metadata": {...}}]` (no `id` field)
When the server inserts the vector
Then `Vector.id` MUST be a freshly generated UUID v4

#### Scenario: Invalid client id is rejected
Given a `POST /insert_texts` request with `texts: [{"id": "  bad id  ", "text": "..."}]`
When the server validates the request
Then it MUST return HTTP 400 with `error_type == "validation_error"` and a message naming the offending field

### Requirement: Chunk payloads MUST use a flat shape consistent with non-chunked payloads
The system SHALL store user-provided metadata at the payload root for chunked vectors, identical to the layout used for non-chunked vectors. The `metadata: {...}` nesting introduced before this spec MUST NOT be produced by new writes.

#### Scenario: Chunked payload places user metadata at root
Given a `POST /insert_texts` with `texts: [{"id": "x", "text": "<3500 chars>", "metadata": {"author": "alice", "year": "2026"}}]`
When the server inserts the chunks
Then `points/scroll` for any chunk MUST return a payload object containing `author == "alice"` and `year == "2026"` at the root
And the payload MUST NOT contain a `metadata` sub-object holding those fields

#### Scenario: Chunk-specific fields live at root
Given a chunked insert
When the server constructs each chunk's payload
Then `content`, `chunk_index`, `parent_id`, and `file_path` MUST be at the payload root
And the payload shape MUST be a flat `Object<String, Value>`

#### Scenario: Qdrant payload filter matches chunked vectors
Given chunked vectors inserted with `metadata.parlamentar = "Jack Rocha"`
When a client issues `POST /qdrant/collections/{name}/points/scroll` with `filter.must = [{"key": "parlamentar", "match": {"value": "Jack Rocha"}}]`
Then the result set MUST include all chunks of that document
And the filter MUST NOT need to use the path `metadata.parlamentar`

#### Scenario: Legacy chunked payloads still readable (one release window)
Given a vector written by Vectorizer ≤ v3.0.13 with payload `{content, metadata: {parlamentar: "X", ...}}`
When a reader (search, scroll, FileOperations) accesses it
Then the reader MUST surface `parlamentar` as if it were at the root
And the server MUST emit a `tracing::warn!` once per collection-load identifying the legacy layout

### Requirement: parent_id field links chunks back to their source
The system SHALL include a `parent_id` field in the payload of every chunked vector, equal to the source request's `client_id` (or a generated UUID if no `client_id` was provided), to enable delete-by-doc and citation without server-side state.

#### Scenario: parent_id is the client_id when provided
Given a `POST /insert_texts` with `texts: [{"id": "doc:42", "text": "<long>"}]`
When the server inserts the chunks
Then every resulting chunk's `payload.parent_id` MUST equal `"doc:42"`

#### Scenario: parent_id is a stable UUID when client_id is absent
Given a `POST /insert_texts` with `texts: [{"text": "<long>"}]` (no id)
When the server inserts N chunks
Then all N chunks MUST share the same `parent_id`
And that `parent_id` MUST be a UUID v4

### Requirement: POST /insert_vectors accepts pre-computed embeddings with client IDs
The system SHALL expose a new endpoint `POST /insert_vectors` that accepts pre-computed embeddings for clients that have their own embedder.

#### Scenario: Insert pre-vectorized data with deterministic id
Given a request `POST /insert_vectors` with `{collection: "c", vectors: [{id: "doc:1", embedding: [0.1, 0.2, ...], payload: {author: "alice"}}]}`
And the embedding length matches the collection dimension
When the server processes the request
Then a vector with `id == "doc:1"` MUST be inserted with `payload.author == "alice"`
And the response MUST report `inserted == 1, failed == 0`

#### Scenario: Embedding dimension mismatch is rejected
Given `POST /insert_vectors` with an embedding of length 384 against a collection with `dimension = 768`
When the server validates
Then it MUST return HTTP 400 with `error_type == "validation_error"` naming the dimension mismatch
And no vector MUST be inserted

#### Scenario: Missing id falls back to UUID
Given `POST /insert_vectors` with `{vectors: [{embedding: [...], payload: {...}}]}` (no id)
When the server inserts
Then `Vector.id` MUST be a freshly generated UUID v4

## MODIFIED Requirements

### Requirement: MCP search_semantic returns flat metadata
The `mcp__vectorizer__search_semantic` MCP tool's response MUST surface user-provided metadata fields at `result.metadata.<field>` (one level of nesting), not at `result.metadata.metadata.<field>` (two levels), regardless of whether the underlying vector was chunked.

#### Scenario: Metadata reachable at one level for chunked hits
Given a collection containing chunked vectors written under the new flat-payload contract
When a client calls `mcp__vectorizer__search_semantic` with `{collection, query}`
Then each hit's `metadata` MUST contain user fields directly (e.g., `result.metadata.parlamentar == "X"`)
And `result.metadata.metadata` MUST NOT exist
