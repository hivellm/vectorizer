# Proposal: phase6_sdk-python-rpc

## Why

Python is the most-used SDK consumer (ML/data-science workflows). Add RPC transport so Python users get the fast path by default. Reference: `../Synap/sdks/python/`.

## What Changes

Inside `sdks/python/`:

1. New module `vectorizer.rpc` exposing `RpcClient` (sync) and `AsyncRpcClient` (asyncio).
2. Uses `msgpack` (PyPI) for MessagePack encoding; raw `socket` or `asyncio.open_connection` for TCP.
3. Connection pool for sync client; per-loop pool for async client.
4. Typed method wrappers for collections/vectors/search/admin.
5. Make `vectorizer.Client` (top-level) default to RPC; keep `vectorizer.HttpClient` available.
6. Python >=3.9 minimum (matches existing SDK baseline); type hints on every public method.
7. Update `sdks/python/README.md` and examples under `sdks/python/examples/` to RPC-first.
8. Publish to PyPI as a minor bump (current version + minor) with CHANGELOG entry.

## Impact

- Affected specs: SDK spec
- Affected code: `sdks/python/vectorizer/` (new `rpc.py`, `async_rpc.py`), `pyproject.toml`, README, examples, tests
- Breaking change: YES (default transport changes) — document migration
- User benefit: Python users get RPC without opt-in; async support aligns with modern stacks.

## Default URL scheme

`vectorizer.Client(url)` parses `url` as follows:

- `vectorizer://host:15503` → RPC (binary MessagePack via the
  `msgpack` PyPI package, see `docs/specs/VECTORIZER_RPC.md`).
- `vectorizer://host` → RPC on default port 15503.
- `host:15503` (no scheme) → RPC.
- `http://host:15002` / `https://host` → REST (legacy fallback).

`vectorizer://` is the canonical default per
`phase6_make-rpc-default-transport`; the README quickstart uses it
first, with REST documented under a "Switching transports" header.
