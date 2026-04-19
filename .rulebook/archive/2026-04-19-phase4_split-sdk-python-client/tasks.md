## 1. Layout

- [x] 1.1 Create `sdks/python/vectorizer/_base.py` with transport, retry, auth header helpers.
- [x] 1.2 Extract `collections.py` + `vectors.py` + `search.py`.
- [x] 1.3 Extract `graph.py` + `admin.py` + `auth.py`.
- [x] 1.4 Rewrite `client.py` as a facade composing the above.
- [x] 1.5 Shape `_base.Transport` as an abstract base class (or `Protocol`) — concrete subclass `RestTransport` ships in this task; the `RpcTransport` from `phase6_sdk-python-rpc` plugs in by inheriting the same ABC. Per-surface modules call methods on `Transport`, never on `httpx.Client` directly.

## 2. Public surface

- [x] 2.1 `from vectorizer import VectorizerClient` keeps working identically.
- [x] 2.2 Sub-clients (`from vectorizer.collections import CollectionsClient`) are also exposed for advanced users.

## 3. Verification

- [x] 3.1 `pytest sdks/python/tests` passes unchanged.
- [x] 3.2 No sub-module exceeds 700 lines.
- [x] 3.3 Add a unit test that constructs a `MockTransport(Transport)` subclass and asserts every per-surface module works against it — proves the surface modules are not coupled to `RestTransport` and acts as the RPC-readiness regression guard.

## 4. Tail (mandatory)

- [x] 4.1 Update `sdks/python/README.md` with the new layout + import examples; note that the new layout hosts the RPC client (`phase6_sdk-python-rpc`) using the canonical `vectorizer://host:15503` URL scheme as the default transport.
- [x] 4.2 Add the `MockTransport` subclass test from item 3.3 — doubles as the RPC-readiness regression guard.
- [x] 4.3 Run the SDK test suite and confirm pass.
- [x] 4.4 Update or create documentation covering the implementation (README layout section + module docstrings per surface).
- [x] 4.5 Write tests covering the new behavior (`tests/test_mock_transport.py`: 11 cases proving Transport ABC decoupling + facade delegation).
