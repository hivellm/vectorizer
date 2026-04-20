## 1. Implementation

- [ ] 1.1 Expose a sync-flush API on `AutoSaveManager` (if not already
  public). In `crates/vectorizer/src/db/auto_save.rs` around line 278,
  keep `force_save` as the canonical flush; add a blocking wrapper
  `force_save_blocking(&self) -> Result<()>` if the caller is sync,
  or ensure `VectorStore` holds an `Arc<AutoSaveManager>` the handler
  can reach.
- [ ] 1.2 Wire `VectorStore` to the manager: add `VectorStore::flush(&self)
  -> Result<()>` that either calls `AutoSaveManager::force_save` or
  does an equivalent compact-to-`.vecdb` write. Document it in the
  doc comment.
- [ ] 1.3 Rewrite `force_save_collection`
  (`crates/vectorizer-server/src/server/rest_handlers/collections.rs:420`)
  to call the new `state.store.flush()` and propagate the real result
  (success/failure from the compactor).
- [ ] 1.4 Either rename `force_save_all`
  (`crates/vectorizer/src/db/vector_store/autosave.rs:49`) to
  `clear_pending_saves` + update all callers (including `backups.rs:319`)
  OR rewrite it to call the new `flush` so the name stops lying.
- [ ] 1.5 Confirm the compactor output path matches
  `vectorizer_core::paths::data_dir() / "vectorizer.vecdb"` on all
  three platforms (Windows APPDATA, Linux XDG, macOS Application
  Support).

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (CHANGELOG.md under `3.0.0 > Fixed`; `docs/specs/REST.md` if the
  endpoint is documented).
- [ ] 2.2 Write tests covering the new behavior (unit test in
  `crates/vectorizer/src/db/auto_save.rs` tests module that inserts
  into a collection, calls `flush`, and asserts the `.vecdb` file
  appears on disk under `data_dir()`; integration test at
  `tests/api/rest/force_save_real.rs` that does the same via the
  REST endpoint).
- [ ] 2.3 Run tests and confirm they pass
  (`cargo test --workspace --lib --all-features` plus the new tests).
