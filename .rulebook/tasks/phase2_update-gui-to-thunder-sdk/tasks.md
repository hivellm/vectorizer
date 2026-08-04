## 1. SDK bump
- [ ] 1.1 Bump `gui/package.json` `@hivehub/vectorizer-sdk` to the Thunder-based 3.6.x line; `pnpm install`
- [ ] 1.2 Confirm the Thunder transport dependency resolves in `gui/pnpm-lock.yaml`

## 2. Client realignment
- [ ] 2.1 Fix `src/renderer/stores/vectorizer.ts`: `insertText` -> `insertTexts`, `BatchTextRequest.id`, and the `config` accessor to match the new SDK
- [ ] 2.2 Fix `src/renderer/views/*.vue` client calls (`BackupManager`, `ConfigEditor`, `LogsViewer`, `WorkspaceManager`) against the new SDK surface
- [ ] 2.3 Reconcile `src/shared/types` `SearchResult` (`vector_id`) with the SDK's exported model

## 3. Verify
- [ ] 3.1 `pnpm type-check` (vue-tsc + tsc) passes with zero errors
- [ ] 3.2 `pnpm build` passes
- [ ] 3.3 GUI connects to a running server over the Thunder transport

## 4. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 4.1 Update or create documentation covering the implementation
- [ ] 4.2 Write tests covering the new behavior
- [ ] 4.3 Run tests and confirm they pass
