# Proposal: phase8_fix-search-endpoint-placeholders

## Why

`POST /search` and `POST /collections/{name}/search`, both routed to
`search_vectors`
(`crates/vectorizer-server/src/server/rest_handlers/search.rs:351-367`),
accept a `vector` array and always return
`{results: [], query_vector, limit}`. Clients that POST a query
vector get an empty hit list — not a 404, not a 501, just a silent
zero-result response.

The real text-search path (`search_vectors_by_text`, same file line
37) is reachable only at `POST /collections/{name}/search/text`, and
the hybrid path is at `/collections/{name}/hybrid_search`. There is
no working raw-vector search endpoint today.

Found during v3 runtime verification (probe 2.1 of
`phase8_release-v3-runtime-verification`), which needs a working
search call before and after a server restart to confirm the snapshot
round-trip. Source: `docs/releases/v3.0.0-verification.md` finding F2.

## What Changes

Implement raw-vector search at both routes. Behavior:

- Parse `vector: [f32]` (required), `limit: usize` (default 10),
  `threshold: f64?` (optional), `filter: object?` (optional).
- For `/collections/{name}/search`: take the collection from the
  path; for `/search`: take it from `payload["collection"]` (returns
  400 if absent).
- Validate vector length matches collection `dimension` (400 mismatch
  error otherwise — same error shape as `insert_text`).
- Call the same `Collection::search` the text handler uses, skipping
  the embedding step.
- Return `{results: [{id, score, payload, vector?}], collection,
  limit, query_type: "vector"}` — same shape as the text handler
  minus the `query` echo.
- Apply the query cache (`QueryKey::from_vector(...)`) as
  `search_vectors_by_text` does.

Alternative: delete both routes and document as BREAKING in
`CHANGELOG.md > 3.0.0`. Not preferred — raw-vector search is a core
vector-DB primitive and the routes are documented.

## Impact

- Affected specs: `docs/specs/REST.md` if documented; `CHANGELOG.md`
  `3.0.0 > Fixed`.
- Affected code:
  - `crates/vectorizer-server/src/server/rest_handlers/search.rs`
    (`search_vectors`)
  - possibly refactor `search_vectors_by_text` to extract a shared
    core `search_collection(collection, query_embedding, limit,
    threshold, filter, tenant_id, query_cache)`.
- Breaking change: NO — `/search` currently returns empty; fixing it
  only adds hits.
- User benefit: raw-vector search works. Unblocks v3 release
  verification probes 2.1, 2.7 (query cache), 3.2 (batch search).
