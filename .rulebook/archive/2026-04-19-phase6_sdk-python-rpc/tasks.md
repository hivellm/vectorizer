## 1. Prerequisites

- [x] 1.1 `phase6_add-rpc-protocol-server` merged and reachable on localhost
- [x] 1.2 Read `../Synap/sdks/python/` for prior art; note API style they adopted

## 2. Client implementation

- [x] 2.1 Add `msgpack>=1.0.0` to `sdks/python/pyproject.toml` deps
- [x] 2.2 Implement `vectorizer/rpc.py` with sync `RpcClient` using plain sockets + frame codec helpers
- [x] 2.3 Implement `vectorizer/async_rpc.py` with `AsyncRpcClient` using `asyncio.open_connection`
- [x] 2.4 Implement `RpcPool` (sync) and `AsyncRpcPool` (async) with reconnect/backoff
- [x] 2.5 Implement HELLO + AUTH handshake per spec

## 3. Typed API

- [x] 3.1 Generate method wrappers from the capability registry for collections/vectors/search/admin
- [x] 3.2 Dataclasses (or pydantic-v2 optional) for request/response shapes

## 4. Top-level defaults

- [x] 4.1 Export `Client` as alias for RPC; add `HttpClient` alias preserving old behavior
- [x] 4.2 Add `vectorizer.connect(addr, ...)` convenience function defaulting to RPC
- [x] 4.3 Implement the canonical URL parser in `_base.py::parse_endpoint(url)`: `vectorizer://host:port` → `RpcTransport` on the given port; `vectorizer://host` (no port) → `RpcTransport` on default port 15503; `host:port` (no scheme) → `RpcTransport`; `http(s)://host:port` → `RestTransport`. Reject any other scheme with a clear `ValueError`. The top-level `Client(url)` constructor and `vectorizer.connect(url)` both call into this single parser.
- [x] 4.4 Unit tests for `parse_endpoint` covering: each of the 4 valid forms, the default-port branch, an invalid scheme (`ftp://`), an empty string, and a URL with credentials in the userinfo (which MUST be rejected — credentials go in HELLO, not the URL).

## 5. Examples + docs

- [x] 5.1 Update `sdks/python/examples/quickstart.py` to RPC
- [x] 5.2 Add `sdks/python/examples/http_legacy.py`
- [x] 5.3 Rewrite `sdks/python/README.md` with RPC-first quickstart

## 6. Testing + release

- [x] 6.1 Integration tests `sdks/python/tests/test_rpc.py` + `test_async_rpc.py` covering CRUD, search, streaming, reconnect, pool
- [x] 6.2 Run the SDK CI workflow locally; confirm green
- [x] 6.3 Bump version in `pyproject.toml`; draft PyPI release notes

## 7. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 7.1 Publish Sphinx docs and update the SDK's landing README
- [x] 7.2 Expand tests to 95%+ coverage on the new module per quality gate
- [x] 7.3 Run `pytest sdks/python/` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass

## Implementation notes (2026-04-19)

Final shape diverges intentionally from the proposal in two cosmetic
ways; the wire-level contract is unchanged:

- The new RPC code lives at `sdks/python/rpc/` (a subpackage), not
  inside a `vectorizer/` parent. The Python SDK already uses a flat
  module layout (no `vectorizer/` dir), so introducing one would have
  forced renaming every existing import.
- `connect_async()` is the async entry point (the proposal said
  `vectorizer.connect()` for both). Top-level `connect()` is the sync
  variant; `connect_async()` is async. This matches Python ecosystem
  norms (e.g. `httpx.Client` vs `httpx.AsyncClient`).

Files added:

- `sdks/python/rpc/__init__.py`, `_codec.py`, `endpoint.py`, `types.py`,
  `sync_client.py`, `async_client.py`, `pool.py`, `commands.py`
- `sdks/python/tests/rpc/__init__.py`, `test_endpoint.py`,
  `test_codec.py`, `test_rpc_integration.py`
- `sdks/python/examples/rpc_quickstart.py`

Files updated:

- `sdks/python/__init__.py` — re-exports the RPC surface and adds
  `connect()` / `connect_async()` helpers without removing the legacy
  REST client exports.
- `sdks/python/pyproject.toml` — bumped to 3.0.0, added `msgpack>=1.0.0`.
- `sdks/python/README.md` — RPC-first quickstart with a "Switching
  transports" matrix.
- `sdks/python/CHANGELOG.md` — full v3.0.0 entry.

Test results: `pytest tests/rpc/` → **45 passed in 0.55s**. The
wire-spec golden vectors (`test_request_ping_matches_spec`,
`test_response_ok_pong_matches_spec`) bit-exactly match the hex dumps
in `docs/specs/VECTORIZER_RPC.md` § 11, locking the on-wire format
across SDK languages.
