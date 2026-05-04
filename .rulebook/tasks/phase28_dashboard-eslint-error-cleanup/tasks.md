## 1. Unescaped entities (19 sites)

- [x] 1.1 `CollectionsPage.tsx:134` (×2 in one line), `OverviewPage.tsx:223`, `SearchPage.tsx:337-343` (×16 in the curl example block) — all literal `"` / `'` inside JSX text replaced with `&quot;` / `&apos;`
- [x] 1.2 `react/no-unescaped-entities` reports zero errors

## 2. `preserve-caught-error` (13 sites)

- [x] 2.1 `useApiKeys.ts:45` plus 12 `throw new Error(...)` sites in `useGraph.ts` (lines 126, 162, 182, 207, 236, 272, 289, 313, 338, 358, 379, 399) all grew `, { cause: error }` so the original `ApiClientError` is preserved on the rethrow
- [x] 2.2 `preserve-caught-error` reports zero errors

## 3. Hooks + refs (4 sites)

- [x] 3.1 `WorkspacePage.tsx:57` — `loadData` rewritten as a `useCallback([getConfig])` and the `useEffect` now declares `[loadData]` as its dependency. The eslint-disable directive was removed
- [x] 3.1.b `PasswordStrengthIndicator.tsx:72` — `validateClientSide` hoisted to module scope (it has no closure over component state) so the inner `useCallback` reads it from the lexical scope without tripping the "Cannot access variable before it is declared" rule
- [x] 3.2 `GraphPage.tsx:792` — added a `hasNetwork` state mirror of `networkInstanceRef.current != null`, set to `true` when `Network` is created and `false` in both teardown paths. The render path now reads `hasNetwork` instead of the ref, so `react-hooks/immutability` no longer flags the ref read during render
- [x] 3.3 `useSetupRedirect.ts:46` — replaced the inline `JSON.stringify(options?.excludePaths)` dep with a precomputed `excludePathsKey` identifier so `react-hooks/exhaustive-deps` accepts the simple-expression form under eslint@10

## 4. Misc (5 sites)

- [x] 4.1 `cn.test.ts:14` — the `true` / `false` literals are now widened with `as boolean`, so the `&&` short-circuits at runtime and `no-constant-binary-expression` stays quiet without changing test intent
- [x] 4.2 `configuration-save.spec.ts:53` — `let body: unknown = null;` → `let body: unknown;`. Both branches of the try/catch assign `body` before it is read, so dropping the initialiser silences `no-useless-assignment` without changing behaviour
- [x] 4.3 `Icons.tsx:33` — the `make` factory now returns a named `IconWrapper` with `displayName = 'Icon'`. The `IconComponent` alias was widened to `... & { displayName?: string }` so the assignment type-checks under `tsc --noEmit`

## 5. Verify

- [x] 5.1 `pnpm lint` reports `0 errors, 24 warnings` — well under `--max-warnings 50`
- [x] 5.2 `pnpm vitest --run` keeps the same `219/224` baseline phase26 inherited (5 pre-existing ApiKeysPage / MonitoringPage failures, no new regressions)
- [x] 5.3 `pnpm build` (`tsc --noEmit && vite build`) clean — 738 ms vite step, no new TS errors

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Update or create documentation covering the implementation — every fix carries an inline comment explaining *why* the change is needed (the eslint v10 rule trigger, the original error preservation contract, the ref-during-render gate, etc.) so future readers don't undo them by mistake
- [x] 6.2 Write tests covering the new behavior — no new behaviour was introduced; the existing 219 vitest tests continue to validate the affected components, and `cn.test.ts` was preserved with intent (the `as boolean` cast keeps the conditional class branches under test)
- [x] 6.3 Run tests and confirm they pass — `pnpm vitest --run` 219/224 baseline unchanged; `pnpm lint` 0 errors; `pnpm build` clean
