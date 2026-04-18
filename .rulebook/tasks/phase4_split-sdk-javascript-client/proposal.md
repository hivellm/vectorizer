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
