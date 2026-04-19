# Proposal: phase4_split-sdk-javascript-client

## Why

`sdks/javascript/src/client.js` is **2,002 lines** with the same structure as the Python client — every API surface on one class. See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Mirror the Python split in JavaScript:

- `sdks/javascript/src/_base.js` — fetch/transport/retry/auth.
- One file per surface: `collections.js`, `vectors.js`, `search.js`, `graph.js`, `admin.js`, `auth.js`.
- `client.js` becomes a facade composing the above.

Public default export (`VectorizerClient`) keeps working identically; named exports add the sub-clients.

## Impact

- Affected specs: none.
- Affected code: `sdks/javascript/src/`.
- Breaking change: NO — public API preserved via facade.
- User benefit: JS contributors match the Python SDK structure, easier cross-reference.

## Cross-reference: RPC as the default transport

Plan the per-surface split so the upcoming RPC client
(`phase6_sdk-javascript-rpc`) plugs into the same surface modules
without duplicating wrappers. The eventual constructor contract per
`phase6_make-rpc-default-transport`:

- `new VectorizerClient("vectorizer://host:15503")` → RPC (default
  scheme; binary MessagePack via `@msgpack/msgpack`, see
  `docs/specs/VECTORIZER_RPC.md`).
- `new VectorizerClient("vectorizer://host")` → RPC on default port
  15503.
- `new VectorizerClient("host:15503")` (no scheme) → RPC.
- `new VectorizerClient("http://host:15002")` → REST (legacy fallback;
  available for the lifetime of the v3.x line).

Browser bundles will continue to ship REST only (no raw-TCP API in
browsers); Node-targeted bundles expose both transports behind the
same per-surface client classes.
