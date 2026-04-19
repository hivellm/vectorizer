## 1. Layout

- [ ] 1.1 Create `sdks/javascript/src/_base.js` with transport helpers.
- [ ] 1.2 Extract per-surface files (collections, vectors, search, graph, admin, auth).
- [ ] 1.3 Rewrite `client.js` as a facade.
- [ ] 1.4 `_base.js::Transport` is a duck-typed interface (object with `request(method, path, body)`); `RestTransport` is the concrete impl shipped here; `RpcTransport` from `phase6_sdk-javascript-rpc` (Node-only) implements the same shape. Per-surface modules call `transport.request(...)`, never `fetch` directly.

## 2. Public API

- [ ] 2.1 Default export `VectorizerClient` unchanged.
- [ ] 2.2 Add named exports for the sub-clients.

## 3. Verification

- [ ] 3.1 `npm test` in `sdks/javascript/` passes unchanged.
- [ ] 3.2 No sub-file exceeds 500 lines.
- [ ] 3.3 Add a unit test that injects a `MockTransport` (a plain object exposing `request`) and asserts every per-surface module routes through it — proves the surface modules are not coupled to `RestTransport` and acts as the RPC-readiness regression guard.

## 4. Tail (mandatory)

- [ ] 4.1 Update `sdks/javascript/README.md` — note that the layout hosts the RPC client (`phase6_sdk-javascript-rpc`) using the canonical `vectorizer://host:15503` URL scheme as the default transport in Node builds (browsers stay REST-only).
- [ ] 4.2 Add the `MockTransport` test from item 3.3 — doubles as the RPC-readiness regression guard.
- [ ] 4.3 Run SDK tests and confirm pass.
