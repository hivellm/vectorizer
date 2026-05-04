## 1. Diagnose the no-op path

- [ ] 1.1 Grep the rename branch in `crates/vectorizer-server/src/server/rest_handlers/collections.rs` and identify which storage layer is (or is not) being mutated
- [ ] 1.2 Trace through `VectorStore` / `Collection` / `dynamic` storage to find where the canonical name is stored
- [ ] 1.3 Document the no-op cause in a comment in the test file (so the regression cannot return)

## 2. Implementation

- [ ] 2.1 Update the rename handler to call a real `VectorStore::rename_collection(old, new)` primitive that:
  - Mutates the in-memory map atomically
  - Renames the on-disk `.vecdb` (or equivalent) file
  - Updates the persisted catalog so a server restart preserves the rename
  - Emits a `Rename` op to the replication WAL
- [ ] 2.2 Reject the call with HTTP 409 `collection_already_exists` when `new_name` is already taken
- [ ] 2.3 Reject the call with HTTP 400 `validation_error` when `old == new` or `new_name` is empty / contains invalid chars (mirror the validation `create_collection` already uses)
- [ ] 2.4 Surface a WAL-level write failure as HTTP 500 with a stable `error_type` (`persistence_error`) — do not silently roll forward

## 3. Regression test

- [ ] 3.1 Add `crates/vectorizer-server/tests/rename_collection_persists.rs` that:
  - Creates a collection, inserts vectors
  - Calls `POST /collections/{old}/rename` with `{"new_name": "..."}`
  - Asserts the rename returns 200
  - Asserts `GET /collections` lists ONLY the new name (old gone)
  - Asserts `GET /collections/{new}/vectors` returns the original vectors
  - Asserts `GET /collections/{old}/vectors` returns 404
- [ ] 3.2 Add a test for the 409 collision case
- [ ] 3.3 Add a test for the 400 self-rename case
- [ ] 3.4 Add a server-restart test that verifies the rename survives a fresh boot (load the persisted catalog)

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `docs/users/api/API_REFERENCE.md` rename row with the new error responses (409, 400)
- [ ] 4.2 Run `cargo test -p vectorizer-server --test rename_collection_persists` and confirm pass
- [ ] 4.3 Run `cargo clippy -p vectorizer-server -- -D warnings` and confirm zero warnings
