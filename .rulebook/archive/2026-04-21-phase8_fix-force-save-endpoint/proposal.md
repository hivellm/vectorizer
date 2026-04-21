# Proposal: phase8_fix-force-save-endpoint

## Why

`POST /collections/{name}/force-save` at
`crates/vectorizer-server/src/server/rest_handlers/collections.rs:420-450`
returns `{success:true, message:"Collection '{name}' saved
successfully"}` but does NOT write anything to disk. The handler
calls `state.store.force_save_all()`
(`crates/vectorizer/src/db/vector_store/autosave.rs:49-75`), which
merely clears the `pending_saves` set and logs "✅ Force save
completed" before returning Ok. The real `.vecdb` flush is owned by
`AutoSaveManager` (`crates/vectorizer/src/db/auto_save.rs:278`),
which runs on a 5-minute timer.

Empirical repro (on 2026-04-20 during probe 2.1 of
`phase8_release-v3-runtime-verification`): after a 200 OK from
`/force-save`, `%APPDATA%\vectorizer\data\vectorizer.vecdb` was
still absent.

Source: `docs/releases/v3.0.0-verification.md` finding F3.

## What Changes

Route the endpoint to the real flush path. Two options:

1. **Preferred** — expose `AutoSaveManager::force_save` via
   `VectorStore` (e.g. `VectorStore::force_flush(&self) -> Result<()>`)
   that awaits the manager's flush future, then call it from
   `force_save_collection`. Returns the real success/failure status.
2. Alternatively — delete the endpoint, rename the client-facing
   behavior to be 5-min interval only, document in `CHANGELOG.md >
   3.0.0 > BREAKING CHANGES`. Not preferred because `backups.rs:319`
   and the GUI already call `force_save_all()` expecting a sync
   flush.

Whichever path is taken, `force_save_all` in
`vector_store/autosave.rs` must stop lying: it should either forward
to the real flush or be renamed to something honest like
`clear_pending_saves`.

## Impact

- Affected specs: `docs/specs/REST.md` for the endpoint semantics.
- Affected code:
  - `crates/vectorizer/src/db/auto_save.rs` (expose a sync-flush API
    the server can call)
  - `crates/vectorizer/src/db/vector_store/autosave.rs`
    (`force_save_all` — either wire to real flush or rename)
  - `crates/vectorizer-server/src/server/rest_handlers/collections.rs`
    (`force_save_collection`)
  - `crates/vectorizer-server/src/server/rest_handlers/backups.rs:319`
    (also calls `force_save_all()`)
- Breaking change: NO if option 1; YES if option 2.
- User benefit: `/force-save` actually persists. Unblocks v3 release
  verification probe 2.1 and backup/restore round-trip.
