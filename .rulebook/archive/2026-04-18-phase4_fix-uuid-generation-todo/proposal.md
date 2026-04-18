# Proposal: phase4_fix-uuid-generation-todo

## Why

`src/server/rest_handlers.rs:1340` and `:1394` carry the comment **"TODO: Use actual collection UUID"** — and currently generate a **fresh UUID via `Uuid::new_v4()` on every request**. This is a correctness bug, not a style issue:

- Consumers expecting stable collection IDs across requests see a new ID each call → breaks caching, cross-reference, external integrations.
- If any downstream logic persists the generated ID (logs, metrics, audit trail), it accumulates garbage.
- Possible data-loss path: if a write operation uses the fresh UUID as a key, the write lands on a phantom collection.

This is one of the clearest symptoms of Cursor-generated scaffolding never revisited.

## What Changes

1. Replace the `Uuid::new_v4()` calls at lines 1340 and 1394 with lookup of the collection's persistent UUID from the `VectorStore` / `Collection` metadata.
2. Add the UUID field to `CollectionConfig` / `Collection` if it does not yet exist (it probably does; confirm during design).
3. On collection creation, generate a UUID once and persist it in the collection's on-disk metadata.
4. Add a regression test asserting that two reads of the same collection return identical UUIDs.

## Impact

- Affected specs: data model spec
- Affected code: `src/server/rest_handlers.rs` (or its successor after split), `src/db/collection.rs`, `src/models/mod.rs`
- Breaking change: NO for consumers, YES for any external system that stored the previously-random UUIDs (they were never real references)
- User benefit: stable collection identity; correct audit/metrics/logs; closes a subtle data-integrity bug.
