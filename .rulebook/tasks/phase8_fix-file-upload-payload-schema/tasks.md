## 1. Implementation

- [ ] 1.1 In `crates/vectorizer-server/src/server/files/upload.rs`,
  relocate per-chunk metadata under a `metadata:` sub-object:
  `{content, metadata: {file_path, chunk_index, original_filename,
  language, source, file_extension}}` — keep `content` at the root.
- [ ] 1.2 In the shared `insert_one_text` helper at
  `crates/vectorizer-server/src/server/rest_handlers/insert.rs`,
  detect when the caller is the upload path (filename/file_path in
  metadata) vs a plain `POST /insert` text, and nest the metadata
  the same way for uploads while keeping the flat shape for plain
  inserts (or nest both — prefer uniform shape).
- [ ] 1.3 Add a dual-shape fallback in
  `crates/vectorizer/src/file_operations/operations.rs`
  (`list_files_in_collection`, `get_file_chunks_ordered`,
  `get_file_summary`, `get_project_outline`, `get_related_files`,
  `search_by_file_type`): read `payload.data.metadata.file_path`
  first, fall back to `payload.data.file_path` for v3.0.x.
- [ ] 1.4 Log a DEBUG line when the flat-shape fallback fires so
  operators can quantify how many rows still carry the old shape
  before the v3.1.0 removal.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (CHANGELOG entry under `3.0.0 > Fixed` + `docs/users/api/FILE_OPERATIONS.md`
  canonical-shape block + deprecation note for the flat shape).
- [ ] 2.2 Write tests covering the new behavior (integration test at
  `crates/vectorizer/tests/api/rest/file_upload_real.rs` that:
  (a) uploads a `.md` file via `POST /files/upload`,
  (b) asserts `POST /file/list` returns it,
  (c) asserts `POST /file/chunks` returns the chunks in order,
  (d) asserts a forged flat-shape vector inserted directly into the
      store is still discoverable via the fallback reader).
- [ ] 2.3 Run tests and confirm they pass
  (`cargo test --workspace --lib --all-features` plus the new
  integration test against the live release binary).
