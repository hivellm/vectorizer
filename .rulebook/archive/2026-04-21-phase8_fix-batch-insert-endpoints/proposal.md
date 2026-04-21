# Proposal: phase8_fix-batch-insert-endpoints

## Why

`POST /batch_insert` (`batch_insert_texts`) and `POST /insert_texts`
(`insert_texts`) at
`crates/vectorizer-server/src/server/rest_handlers/vectors.rs:249-282`
are documented v3 endpoints that currently accept a `texts` array,
log the count, and return `{message, count}` success — but do NOT
write a single vector into the target collection. The handlers ignore
`payload["collection"]`, never call `state.store`, never embed, never
persist.

This is a data-loss bug: clients that POST to either endpoint get a
200 with a "inserted N texts" message while the server silently
discards the payload. Found during v3 runtime verification (probe 2.1
of `phase8_release-v3-runtime-verification`), which needs a working
batch-insert to seed 100 vectors before the snapshot round-trip.

Source: `docs/releases/v3.0.0-verification.md` finding F1.

## What Changes

Implement the real batch-insert write path in both handlers, mirroring
`insert_text` (`insert.rs`) so each text goes through the same
chunk → embed → store → replicate → metrics pipeline. Specifically:

- Parse `collection`, `texts[]` (array of `{id?, text, metadata?}`),
  `auto_chunk`, `chunk_size`, `chunk_overlap`, `public_key` from the
  body.
- For each entry, call the same internal helper `insert_text` uses
  (extract it into a `vectors::insert_one(state, tenant, collection,
  text, metadata, ...)` shared helper if not already shared).
- Aggregate per-text results into `[{id, status:"ok"|"error", error?}]`.
- Return `{inserted: N, failed: M, results: [...]}` with an HTTP 200
  on partial success and 4xx when the collection is missing or all
  entries fail validation.
- Add an integration test under `tests/api/rest/` that creates a
  collection, POSTs 10 texts via `/batch_insert`, and verifies
  `list_vectors` returns 10 entries with matching payload metadata.

If the endpoints were intentionally removed in v3 (unlikely — both
are still routed in `server/core/routing.rs:292-293`), delete the
routes and document under `CHANGELOG.md#3.0.0 > BREAKING CHANGES`.

## Impact

- Affected specs: REST API spec under `docs/specs/REST.md` (if the
  response shape changes) and `CHANGELOG.md` under `3.0.0 > Fixed`.
- Affected code:
  - `crates/vectorizer-server/src/server/rest_handlers/vectors.rs`
    (`batch_insert_texts`, `insert_texts`)
  - possibly `crates/vectorizer-server/src/server/rest_handlers/insert.rs`
    (extract shared helper)
  - `tests/api/rest/` (new integration test)
- Breaking change: NO — endpoint shape is compatible; we just add
  real behavior and a per-text results array.
- User benefit: batch insert actually works. Unblocks v3 release
  verification probes 2.1, 3.2, 5.1.
