# Proposal: phase8_fix-file-upload-payload-schema

## Why

`POST /files/upload` writes each chunk's metadata FLAT into the vector
payload (`payload.data.file_path`, `payload.data.chunk_index`,
`payload.data.original_filename`, ...). But
`FileOperations::list_files_in_collection` at
`crates/vectorizer/src/file_operations/operations.rs:242-249` reads
`payload.data.metadata.file_path` (nested under a `metadata` key).

Result: uploaded files exist as vectors in the collection but the
whole file-navigation REST surface is broken for uploads done through
the documented upload endpoint:

- `POST /file/list` returns empty `files: []`.
- `POST /file/chunks` returns "File not found in collection".
- `POST /file/summary`, `/file/outline`, `/file/related` all share
  the same reader.

Empirical repro on 2026-04-20 during probe 3.3 of
`phase8_release-v3-runtime-verification`:

```
$ curl -X POST http://127.0.0.1:15002/files/upload \
    -F "collection_name=p33" -F "file=@/tmp/probe33.md"
{"success":true,"chunks_created":1,"vectors_created":1,...}

$ curl -X POST http://127.0.0.1:15002/file/list \
    -H 'Content-Type: application/json' -d '{"collection":"p33"}'
{"collection":"p33","files":[],"total_chunks":0,"total_files":0}
```

The vector IS in the collection — `GET /collections/p33/vectors`
returns it — but its payload shape doesn't match what the reader
expects.

Source: `docs/releases/v3.0.0-verification.md` finding F8.

## What Changes

Preferred direction: make the upload path write `file_path` nested
under a `metadata:` key so it matches the existing reader shape AND
stays compatible with every operator script / SDK client that already
depends on that shape.

1. Update the chunk-writer inside
   `crates/vectorizer-server/src/server/files/upload.rs` (and the
   shared `insert_one_text` helper at
   `crates/vectorizer-server/src/server/rest_handlers/insert.rs` that
   `/files/upload` now shares with `/insert` + `/batch_insert` — we
   ALSO inherited the flat shape from the F1 fix).
2. Write each chunk's per-chunk data as:
   ```json
   {
     "content": "...",
     "metadata": {
       "file_path": "probe33.md",
       "chunk_index": 0,
       "original_filename": "probe33.md",
       "language": "markdown",
       "source": "upload"
     }
   }
   ```
3. Keep `content` at the payload-object root (search.rs handlers
   index on it).
4. Migration for existing `.vecdb` files: DO NOT rewrite on load —
   the `list_files_in_collection` reader should accept BOTH shapes
   (nested + flat) for one release, deprecate the flat shape in
   `CHANGELOG.md > 3.0.0 > Fixed`, drop the flat-shape fallback in
   v3.1.0.

## Impact

- Affected specs: `docs/users/api/FILE_OPERATIONS.md` (document the
  canonical shape); `CHANGELOG.md > 3.0.0 > Fixed`.
- Affected code:
  - `crates/vectorizer-server/src/server/files/upload.rs` (writer)
  - `crates/vectorizer-server/src/server/rest_handlers/insert.rs`
    (shared `insert_one_text` helper when called from upload)
  - `crates/vectorizer/src/file_operations/operations.rs`
    (`list_files_in_collection`, `get_file_chunks_ordered`, etc. —
    add the dual-shape fallback reader)
- Breaking change: NO (dual-read during v3.0.x; writes converge on
  the nested shape).
- User benefit: every file-navigation REST endpoint works on uploaded
  files. Unblocks probe 3.3 + the dashboard's "browse uploaded files"
  view.
