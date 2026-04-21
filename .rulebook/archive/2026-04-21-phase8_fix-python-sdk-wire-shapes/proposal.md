# Proposal: phase8_fix-python-sdk-wire-shapes

## Why

Probe 4.2 of `phase8_release-v3-runtime-verification` ran
`cd sdks/python && python -m pytest` against the live `release/v3.0.0`
binary. Result: **259 passed, 114 failed, 9 skipped**. Expected per
the probe acceptance: 46 previously-failing server-integration tests
now pass. Got the opposite — test expectations and SDK wire shapes
have drifted materially from the server's current REST + UMICP
responses.

Failures cluster in:

- `tests/test_sdk_comprehensive.py::TestVectorizerClientAsync::*`
  (9 failures noted at tail of the test run): `test_health_check_*`,
  `test_list_collections_success`, `test_search_vectors_*`,
  `test_insert_texts_success`, `test_get_vector_success`,
  `test_network_error_handling`, `test_full_workflow_mock`.
- `tests/test_umicp.py::TestVectorizerClientUMICP::*` (5 failures):
  `test_umicp_from_connection_string`, `test_umicp_explicit_*`,
  `test_umicp_default_host_port`, `test_umicp_protocol_support`,
  `test_umicp_configuration_options`.
- Additional ~100 failures not dumped at tail of the run (log
  truncated) — likely share root cause: payload field renames
  between SDK expectations and current REST handlers (e.g. F1
  rewrote `/batch_insert` response shape to `{inserted, failed,
  count, results}` from the prior `{message, count}`; F5 rewrote
  `/embed` to return a real embedding instead of `[0.1; 512]`;
  F2 added `query_type` field to raw-vector search).

Source: `docs/releases/v3.0.0-verification.md` section 4.

## What Changes

Audit + update every SDK response-parser and expectation fixture in
`sdks/python/tests/` + `sdks/python/{client,models}.py` so every
assertion matches the real REST + UMICP shapes the server emits on
`release/v3.0.0`.

1. Capture a golden response fixture for each of the 46 previously-
   failing endpoints by running the SDK against the live binary and
   dumping the response to `sdks/python/tests/fixtures/<endpoint>.json`.
2. Rewrite the SDK parsers to accept the current shapes (`inserted /
   failed / count / results` for `batch_insert`; `query_type` +
   `total_results` for raw-vector `search`; UMICP envelope with
   `{v, msg_id, ts, from, to, op, capabilities}`).
3. Keep backwards-compat where feasible (e.g. coerce the old
   `{message, count}` shape to the new one at parse time with a
   `DeprecationWarning`).
4. Re-run `python -m pytest --ignore=tests/test_file_upload.py
   --ignore=tests/test_routing.py` against the live server —
   acceptance is 0 failures (or a documented subset of
   non-server-dependent unit-test failures that existed pre-v3).

## Impact

- Affected specs: `docs/users/sdks/PYTHON.md` (update response-shape
  documentation to match the v3 REST emitters); `CHANGELOG.md` under
  `3.0.0 > Fixed` inside the Python SDK changelog
  (`sdks/python/CHANGELOG.md` if present).
- Affected code:
  - `sdks/python/client.py` (response parsers)
  - `sdks/python/models.py` (response dataclasses)
  - `sdks/python/tests/test_sdk_comprehensive.py`
  - `sdks/python/tests/test_umicp.py`
  - new `sdks/python/tests/fixtures/` golden responses
- Breaking change: MAYBE (SDK consumer code that reads
  `response["message"]` on a batch-insert now reads
  `response["results"][i]["status"]`; document under `3.0.0 >
  BREAKING CHANGES` inside `sdks/python/CHANGELOG.md`).
- User benefit: Python SDK 3.0.0 works against the v3 server.
  Unblocks probe 4.2.
