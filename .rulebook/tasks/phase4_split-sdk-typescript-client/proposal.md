# Proposal: phase4_split-sdk-typescript-client

## Why

`sdks/typescript/src/client.ts` is **1,879 lines** — same shape as the JavaScript SDK but with types. Splitting it mirrors the JS split and restores parity. See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Split `sdks/typescript/src/client/`:

- `_base.ts` — transport + shared types.
- Per-surface files: `collections.ts`, `vectors.ts`, `search.ts`, `graph.ts`, `admin.ts`, `auth.ts` — each exports its own typed client.
- `index.ts` — `VectorizerClient` facade that composes the sub-clients.

TypeScript declaration files (`.d.ts`) regenerate automatically from the split source.

## Impact

- Affected specs: none.
- Affected code: `sdks/typescript/src/`.
- Breaking change: NO — `import { VectorizerClient } from '@hivellm/vectorizer'` keeps working.
- User benefit: per-surface typed clients directly importable (`import { SearchClient } from '@hivellm/vectorizer/search'`); smaller bundles when tree-shaking.
