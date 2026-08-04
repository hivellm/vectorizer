# Proposal: phase2_update-gui-to-thunder-sdk

## Why
The Electron/Vue GUI (`gui/`) consumes the published `@hivehub/vectorizer-sdk`
(pinned at `^3.0.3`). Once `phase1_replace-vectorizer-protocol-with-thunder`
lands, the TypeScript SDK moves its RPC transport from the bespoke
`vectorizer-protocol` to Thunder, and its public API surface changes. The GUI
already drifts from the current SDK: `vue-tsc` fails with API-mismatch errors
(`config`, `insertText` vs `insertTexts`, `SearchResult.vector_id`,
`BatchTextRequest.id`). Shipping the Thunder SDK without realigning the GUI
would leave the desktop client uncompilable and unable to talk to the server.

## What Changes
Bump `gui/package.json` to the Thunder-based `@hivehub/vectorizer-sdk` (3.6.x)
and realign the GUI's client usage with the new SDK contract:
- Replace stale method/property names (`insertText` -> `insertTexts`,
  `config` accessor, `SearchResult.vector_id`) across
  `src/renderer/stores/vectorizer.ts` and the `src/renderer/views/*` that use
  the client (`BackupManager`, `ConfigEditor`, `LogsViewer`,
  `WorkspaceManager`).
- Reconcile `src/shared/types` with the SDK's exported models so `vue-tsc`
  passes with zero errors.
- Confirm the GUI connects over the Thunder transport (default in 3.x) against
  a running server.

## Impact
- Affected specs: none (client integration only)
- Affected code: `gui/package.json`, `gui/pnpm-lock.yaml`,
  `gui/src/renderer/stores/vectorizer.ts`, `gui/src/renderer/views/*.vue`,
  `gui/src/shared/types`
- Breaking change: NO (internal desktop client; no external contract)
- User benefit: the desktop GUI compiles cleanly and talks to the server over
  the Thunder RPC transport, matching the shipped SDK
- Depends on: `phase1_replace-vectorizer-protocol-with-thunder`
