# Proposal: phase9_fix-insert-texts-id-and-chunk-metadata

## Why

External user (Sonda Legislative ingestion pipeline, v3.0.13 Docker, mpnet-base 768d, ~143k docs / 116k vectors across 6 collections) reported that `POST /insert_texts` is structurally inconsistent and silently breaks idempotency:

1. **Client-provided `id` is silently discarded.** `crates/vectorizer-server/src/server/rest_handlers/vectors.rs:337` parses the `id` field from each text entry but only echoes it back as `client_id` in the response. The actual `Vector.id` is always `Uuid::new_v4()` (`insert.rs:398` for chunked, `insert.rs:436` for non-chunked). This breaks:
   - Re-ingest idempotency — re-running the same `/insert_texts` creates duplicates instead of upserting.
   - Delete-by-doc — there is no path from `client_id` to the (multiple) UUIDs spawned by chunking.
   - RAG citation — every hit must be JOINed back to Postgres via `ILIKE %trecho%` to recover author/date.

2. **Metadata is preserved on chunks but at a *different* JSON path** than non-chunked vectors:
   - Non-chunked (`insert.rs:420-425`): metadata flat at payload root → `{_id, casa, tipo, ...}`.
   - Chunked (`insert.rs:374-385`): metadata nested under `metadata` sub-object → `{content, metadata: {file_path, chunk_index, _id, casa, ...}}`.

   This breaks Qdrant payload filters (`payload.parlamentar = "X"` no longer hits chunked rows) and confuses MCP `search_semantic` consumers (`mcp_tools.rs:516-525` returns `payload.data` verbatim, so chunked hits surface metadata under `result.metadata.metadata.parlamentar` instead of `result.metadata.parlamentar`). The user perceives this as "metadata was discarded" because their filter and field reads silently return empty.

3. **No way to insert pre-vectorized data with a deterministic ID.** The Qdrant-compat `PUT /qdrant/collections/{name}/points` works but is a poor DX for the common Vectorizer use case (client has its own embedder, just wants to upsert).

## What Changes

### 1. Honor client-provided `id` in `/insert` and `/insert_texts`

- Extend `insert_one_text(...)` signature in `crates/vectorizer-server/src/server/rest_handlers/insert.rs:293-303` with `client_id: Option<&str>`.
- `vectors.rs:360-371` — pass `client_id.as_deref()` from the parsed `entry.id` field.
- Vector ID resolution:
  - Non-chunked: use `client_id` verbatim if present, else `Uuid::new_v4()`.
  - Chunked: use `format!("{}#{}", client_id, chunk_index)` if present, else `Uuid::new_v4()`.
- Add `parent_id` field to chunk payloads — even when `client_id` is absent, store the source request's logical id so chunks can be grouped post-hoc.

### 2. Flatten metadata on chunked payloads (consistency with non-chunked)

- Rewrite `insert.rs:374-385` so user metadata sits at the payload root, alongside `content`, `chunk_index`, `parent_id`, `file_path`. Drop the `metadata: {...}` nesting.
- Audit and update any reader that currently expects `payload.data.metadata.file_path`. Known site: `FileOperations::list_files_in_collection` (per inline comment at `insert.rs:369-373`). Migrate it to read the new flat path.
- Add a one-shot migration helper or document the breaking change for users with chunked collections written by ≤ v3.0.13.

### 3. Add `POST /insert_vectors` endpoint

- Accept `{collection, vectors: [{id, embedding, payload?, metadata?}, ...]}` for clients that already have embeddings.
- Behavior: upsert by `id`, validate `embedding.len() == collection.dimension`, return same response shape as `/insert_texts`.
- Implementation: thin wrapper over `VectorStore::insert()` — no embedding manager call.

### 4. Deprecate (don't break) the old chunked payload shape

- Keep both code paths readable for one minor version; emit a structured warning log when reading `payload.data.metadata.*` from on-disk vectors.
- Document in CHANGELOG and migration notes.

## Impact

- **Affected specs:** `/.rulebook/specs/RULEBOOK.md` (none directly), but a new spec under `.rulebook/tasks/phase9_.../specs/insert_texts/spec.md` will encode the SHALL contracts.
- **Affected code:**
  - `crates/vectorizer-server/src/server/rest_handlers/insert.rs` (chunked + non-chunked branches, `insert_one_text` signature)
  - `crates/vectorizer-server/src/server/rest_handlers/vectors.rs` (`do_batch_insert_texts`, response shape)
  - `crates/vectorizer-server/src/server/core/routing.rs` (new `/insert_vectors` route)
  - `crates/vectorizer/src/file_operations/*` (read-side migration for `metadata.file_path`)
  - `crates/vectorizer/src/intelligent_search/mcp_tools.rs` (verify MCP search returns flat payload)
  - `crates/vectorizer/tests/api/rest/batch_insert_real.rs` and new test files
  - `docs/users/api/BATCH.md`, `CHANGELOG.md`
- **Breaking change:** YES — payload shape of chunked vectors changes. Mitigated by:
  - Bump minor version (v3.1.0).
  - Migration note in CHANGELOG.
  - One release window where readers tolerate both shapes with a deprecation warning.
- **User benefit:**
  - Idempotent re-ingest via deterministic IDs (delete-by-doc, upsert work natively).
  - Qdrant payload filters work uniformly across short and long texts.
  - MCP `search_semantic` returns metadata at the same path regardless of chunking.
  - `/insert_vectors` removes the only reason to drop down to the Qdrant compat layer for users with their own embedder.
