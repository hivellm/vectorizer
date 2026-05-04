# Proposal: phase22_fix-rename-collection-no-op

## Why

`POST /collections/{name}/rename` is documented (phase14, all five SDKs)
as atomically renaming a collection. The endpoint returns HTTP 200 OK,
but the rename does not persist: a subsequent `GET /collections` lists
the original name, and the new name is reachable as a 404.

Verified end-to-end against `vectorizer:3.3.0` on 2026-05-04:

```
$ curl -X POST .../collections/test_p20/rename -d '{"new_name":"test_p20_renamed"}'
(empty 200 OK body)

$ curl .../collections | jq '.collections[].name'
"test_p20"           # rename did NOT take effect
"test_p20_warm"

$ curl -X POST .../collections/test_p20_renamed/reindex ...
{"error_type":"collection_not_found","message":"Collection not found: test_p20_renamed"}
```

The handler in `crates/vectorizer-server/src/server/rest_handlers/collections.rs`
(rename branch) returns success without persisting the new name to the
in-memory registry, the WAL/journal, and/or the on-disk `.vecdb` index
filename. SDK callers that rely on `RenameCollection(old, new)` (Go +
C#), `rename_collection` (Rust + TS + Python) silently lose data
referential integrity.

## What Changes

Audit the rename handler end-to-end and make it actually persist the
rename:

1. Update the in-memory `VectorStore` collection map keyed by name.
2. Update the on-disk `.vecdb` index file path.
3. Update any persisted catalog (collection registry / `dynamic` storage).
4. Emit a WAL entry so replicas observe the rename.
5. Reject the call with 409 if the destination name already exists.
6. Reject with 400 if the source equals the destination.

Per CHANGELOG entry "## [3.3.0] 2026-05-03 — Schema evolution + observability"
the rename was supposed to be live — this task closes the gap.

## Impact

- Affected code:
  - `crates/vectorizer-server/src/server/rest_handlers/collections.rs` —
    rename handler must invoke real persistence path, not just respond
  - `crates/vectorizer/src/db/vector_store.rs` (or wherever the rename
    primitive lives) — ensure rename mutates the canonical map
  - `crates/vectorizer/src/persistence/dynamic.rs` — persist the rename
  - `crates/vectorizer/src/replication/log.rs` — append a Rename op to
    the replication log so replicas track it
  - `crates/vectorizer-server/tests/rename_collection_persists.rs` (new)
- Affected specs:
  - phase14 `rename_collection` requirement — already documented; this
    task brings the implementation up to spec
- Breaking change: NO (the documented behavior was already "rename
  persists"; this fixes a regression / never-shipped path)
- User benefit: Rename actually works in all five SDKs and via REST.
  Restores the contract advertised in `docs/users/api/API_REFERENCE.md`.
