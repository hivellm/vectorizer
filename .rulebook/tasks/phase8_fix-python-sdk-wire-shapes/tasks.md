## 1. Implementation

- [ ] 1.1 Capture golden response fixtures from the live release
  binary for the 46 previously-failing endpoints; save under
  `sdks/python/tests/fixtures/<endpoint>.json`.
- [ ] 1.2 Rewrite `sdks/python/client.py` response parsers to match
  the F1 / F2 / F5 shapes:
    - `batch_insert` / `insert_texts`: `{collection, inserted,
      failed, count, results: [{index, client_id, status,
      vector_ids?, vectors_created?, chunked?, error?,
      error_type?}]}`.
    - `search` (raw vector): `{collection, limit, query_type,
      total_results, results: [{id, score, vector, payload}]}`.
    - `force_save`: `{success, message, flushed}`.
    - `embed`: `{embedding, text, dimension}` (real embedding, not
      the old 0.1-placeholder).
    - `batch_search` / `batch_update` / `batch_delete`: per-item
      results shape.
- [ ] 1.3 Rewrite `sdks/python/tests/test_sdk_comprehensive.py` +
  `tests/test_umicp.py` to expect the current shapes (delete the
  stale fixtures embedded in the tests).
- [ ] 1.4 Add `DeprecationWarning` coercion for the legacy
  `{message, count}` batch-insert shape so operator scripts that
  still hit a v2 server during migration don't silently break.
- [ ] 1.5 Update `sdks/python/models.py` dataclasses if field names
  changed.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (`docs/users/sdks/PYTHON.md` response-shape block +
  `sdks/python/CHANGELOG.md` `3.0.0 > Breaking Changes` entry).
- [ ] 2.2 Write tests covering the new behavior (the rewritten
  `test_sdk_comprehensive.py` + `test_umicp.py` ARE the tests;
  add one fresh test that asserts the `DeprecationWarning` fires
  on the legacy shape).
- [ ] 2.3 Run tests and confirm they pass
  (`cd sdks/python && python -m pytest --ignore=tests/test_file_upload.py
   --ignore=tests/test_routing.py` against the live server — target
  0 failures).
