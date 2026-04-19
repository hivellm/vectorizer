## 1. Layout

- [ ] 1.1 Create `sdks/typescript/src/client/` with `_base.ts` + per-surface files.
- [ ] 1.2 Rewrite `index.ts` (or `client.ts`) as a facade.
- [ ] 1.3 Define `_base.ts::Transport` as a TypeScript `interface` — concrete `RestTransport` ships in this task; `RpcTransport` from `phase6_sdk-typescript-rpc` (Node-only) implements the same interface. Per-surface classes call `Transport` methods, never `fetch` directly.

## 2. Verification

- [ ] 2.1 `tsc --noEmit` clean.
- [ ] 2.2 Existing Jest/Vitest tests pass unchanged.
- [ ] 2.3 No sub-file exceeds 500 lines.
- [ ] 2.4 Add a unit test that supplies a `MockTransport implements Transport` to every per-surface client and asserts the surface methods route through it — proves the surface classes are not coupled to `RestTransport` and acts as the RPC-readiness regression guard.

## 3. Tail (mandatory)

- [ ] 3.1 Update `sdks/typescript/README.md` with both the flat and per-surface import paths; note that the layout hosts the RPC client (`phase6_sdk-typescript-rpc`) using the canonical `vectorizer://host:15503` URL scheme as the default transport in Node builds (browsers stay REST-only).
- [ ] 3.2 Add the `MockTransport` test from item 2.4 — doubles as the RPC-readiness regression guard.
- [ ] 3.3 Run SDK tests and confirm pass.
