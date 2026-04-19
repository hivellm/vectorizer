# Proposal: phase6_sdk-javascript-rpc

## Why

Node.js is a common host for Vectorizer clients (backend APIs, ETL pipelines). The JavaScript SDK under `sdks/javascript/` needs an RPC transport so Node consumers get the fast path by default. Browser builds fall back to HTTP (TCP sockets aren't available in browsers).

Reference: Synap's `../Synap/sdks/typescript/` (they ship only TS; JS consumers use compiled output). We'll do the same: primary implementation lives in `sdks/typescript/` (`phase6_sdk-typescript-rpc`), and `sdks/javascript/` either re-exports the compiled artifact or is kept as a vanilla JS thin shim.

## What Changes

Decision path — record via `rulebook_decision_create`:

**Option A — Deprecate `sdks/javascript/`.** If it was always "pre-TypeScript legacy", remove it and point users at `sdks/typescript/` which publishes to npm as CJS + ESM.

**Option B — Keep `sdks/javascript/` as a CJS wrapper.** Re-export from the TS build; add an `rpc` export. Node 18+ minimum.

In either case:

1. The TS SDK is the source of truth (see `phase6_sdk-typescript-rpc`).
2. `sdks/javascript/README.md` is updated to either (A) point at TS or (B) document the CJS wrapper.
3. Examples published with `require()` syntax for Node-CJS consumers.

## Impact

- Affected specs: SDK spec
- Affected code: `sdks/javascript/` (shrunk or repurposed), maybe `package.json`, README
- Breaking change: YES under Option A
- User benefit: removes duplicated implementation; single source of truth reduces drift.

## Default URL scheme

Inherits from `phase6_sdk-typescript-rpc` since the JS package
re-exports the TS artifact. Constructor parses the URL as follows:

- `vectorizer://host:15503` → RPC (Node only; binary MessagePack via
  `@msgpack/msgpack`, see `docs/specs/VECTORIZER_RPC.md`).
- `vectorizer://host` → RPC on default port 15503.
- `host:15503` (no scheme) → RPC.
- `http://host:15002` / `https://host` → REST (legacy fallback).

`vectorizer://` is the canonical default per
`phase6_make-rpc-default-transport`.
