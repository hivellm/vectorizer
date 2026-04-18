## 1. Prerequisites

- [ ] 1.1 `phase6_add-rpc-protocol-server` merged and reachable on localhost
- [ ] 1.2 Read `../Synap/sdks/python/` for prior art; note API style they adopted

## 2. Client implementation

- [ ] 2.1 Add `msgpack>=1.0.0` to `sdks/python/pyproject.toml` deps
- [ ] 2.2 Implement `vectorizer/rpc.py` with sync `RpcClient` using plain sockets + frame codec helpers
- [ ] 2.3 Implement `vectorizer/async_rpc.py` with `AsyncRpcClient` using `asyncio.open_connection`
- [ ] 2.4 Implement `RpcPool` (sync) and `AsyncRpcPool` (async) with reconnect/backoff
- [ ] 2.5 Implement HELLO + AUTH handshake per spec

## 3. Typed API

- [ ] 3.1 Generate method wrappers from the capability registry for collections/vectors/search/admin
- [ ] 3.2 Dataclasses (or pydantic-v2 optional) for request/response shapes

## 4. Top-level defaults

- [ ] 4.1 Export `Client` as alias for RPC; add `HttpClient` alias preserving old behavior
- [ ] 4.2 Add `vectorizer.connect(addr, ...)` convenience function defaulting to RPC

## 5. Examples + docs

- [ ] 5.1 Update `sdks/python/examples/quickstart.py` to RPC
- [ ] 5.2 Add `sdks/python/examples/http_legacy.py`
- [ ] 5.3 Rewrite `sdks/python/README.md` with RPC-first quickstart

## 6. Testing + release

- [ ] 6.1 Integration tests `sdks/python/tests/test_rpc.py` + `test_async_rpc.py` covering CRUD, search, streaming, reconnect, pool
- [ ] 6.2 Run the SDK CI workflow locally; confirm green
- [ ] 6.3 Bump version in `pyproject.toml`; draft PyPI release notes

## 7. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 7.1 Publish Sphinx docs and update the SDK's landing README
- [ ] 7.2 Expand tests to 95%+ coverage on the new module per quality gate
- [ ] 7.3 Run `pytest sdks/python/` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
