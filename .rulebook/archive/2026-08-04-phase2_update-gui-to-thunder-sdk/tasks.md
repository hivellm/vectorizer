## 1. SDK bump
- [x] 1.1 Bump `gui/package.json` `@hivehub/vectorizer-sdk` to the Thunder-based 3.6.x line; `pnpm install`
- [x] 1.2 Confirm the Thunder transport dependency resolves in `gui/pnpm-lock.yaml`

## 2. Client realignment
- [x] 2.1 Fix `src/renderer/stores/vectorizer.ts`: `insertText` -> `insertTexts`, `BatchTextRequest.id`, and the `config` accessor to match the new SDK
- [x] 2.2 Fix `src/renderer/views/*.vue` client calls (`BackupManager`, `ConfigEditor`, `LogsViewer`, `WorkspaceManager`) against the new SDK surface
- [x] 2.3 Reconcile `src/shared/types` `SearchResult` (`vector_id`) with the SDK's exported model

## 3. Verify
- [x] 3.1 `pnpm type-check` (vue-tsc + tsc) passes with zero errors
- [x] 3.2 `pnpm build` passes
- [x] 3.3 GUI connects to a running server over the Thunder transport

## 4. Tail (docs + tests — check or waive with tailWaiver)
- [x] 4.1 Update or create documentation covering the implementation
- [x] 4.2 Write tests covering the new behavior
- [x] 4.3 Run tests and confirm they pass
