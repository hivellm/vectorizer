# Proposal: phase6_sdk-typescript-rpc

## Why

`sdks/typescript/` is the primary client for Node and browser-side TS consumers. Add an RPC transport so Node.js consumers get the fast path by default. Browser builds fall back to HTTP since browsers cannot open raw TCP sockets.

Reference: Synap's `../Synap/sdks/typescript/`.

## What Changes

Inside `sdks/typescript/`:

1. New module `src/rpc/` with `RpcClient` (Node) and `HttpClient` (browser + Node).
2. Dependencies: `@msgpack/msgpack` for MessagePack; plain `net` for TCP in Node (gated behind platform check / conditional exports).
3. Typed API with generated methods from the capability registry.
4. Dual-package: `main` resolves to CJS, `module` to ESM, with `exports` map so bundlers pick the right entry.
5. `Client` class auto-selects RPC in Node, HTTP in browser; consumer can force via `new Client(addr, { transport: 'http' | 'rpc' })`.
6. Update README quickstart to RPC-first for Node.
7. TypeScript strict mode; 100% type coverage on the public API.

## Impact

- Affected specs: SDK spec
- Affected code: `sdks/typescript/src/rpc/` (new), `src/index.ts`, `package.json`, README, tests
- Breaking change: YES (default transport changes in Node) — semver major bump
- User benefit: fastest path out-of-the-box for Node users; browser compat preserved; types never drift.

## Default URL scheme

`new Client(url, opts?)` parses the URL as follows:

- `vectorizer://host:15503` → RPC (Node only; binary MessagePack via
  `@msgpack/msgpack`, see `docs/specs/VECTORIZER_RPC.md`).
- `vectorizer://host` → RPC on default port 15503.
- `host:15503` (no scheme) → RPC.
- `http://host:15002` / `https://host` → REST (default in browser
  builds; explicit fallback in Node).

`vectorizer://` is the canonical default per
`phase6_make-rpc-default-transport`. Browser builds reject
`vectorizer://` URLs at construction time with a clear error pointing
at the REST form (browsers don't speak raw TCP).
