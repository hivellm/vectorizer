# Proposal: phase4_split-sdk-typescript-client

## Why

`sdks/typescript/src/client.ts` is **1,879 lines** with every API surface on one `VectorizerClient`. The standalone JavaScript SDK was retired in v3.0.0 (the TypeScript SDK ships compiled JS), so this is now the only Node-side client and the split is more important — there's no separate JS-only file to mirror. See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Split `sdks/typescript/src/client/`:

- `_base.ts` — transport + shared types.
- Per-surface files: `collections.ts`, `vectors.ts`, `search.ts`, `graph.ts`, `admin.ts`, `auth.ts` — each exports its own typed client.
- `index.ts` — `VectorizerClient` facade that composes the sub-clients.

TypeScript declaration files (`.d.ts`) regenerate automatically from the split source.

## Impact

- Affected specs: none.
- Affected code: `sdks/typescript/src/`.
- Breaking change: NO — `import { VectorizerClient } from '@hivehub/vectorizer-sdk'` keeps working.
- User benefit: per-surface typed clients directly importable (`import { SearchClient } from '@hivehub/vectorizer-sdk/search'`); smaller bundles when tree-shaking.

## Cross-reference: RPC as the default transport

Plan the per-surface split so the upcoming RPC client
(`phase6_sdk-typescript-rpc`) plugs into the same surface modules
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

Keep the `_base.Transport` interface generic over `RestTransport |
RpcTransport` so the same `SearchClient` impl works against either.
Browser builds keep `RestTransport` only (browsers don't speak raw
TCP); Node/Deno/Bun builds expose both.
