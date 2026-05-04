## 1. Unescaped entities (19 sites)

- [ ] 1.1 Replace `"` / `'` literals inside JSX text across the affected pages with `&quot;` / `&apos;` or expression literals
- [ ] 1.2 Confirm `react/no-unescaped-entities` reports zero errors

## 2. `preserve-caught-error` (13 sites)

- [ ] 2.1 Audit every `throw new Error(...)` reachable from a `catch (e) { ... }` and grow `{ cause: e }` so the original error is preserved
- [ ] 2.2 Confirm the rule reports zero errors

## 3. Hooks + refs (4 sites)

- [ ] 3.1 `WorkspacePage.tsx` — convert `loadData` to `useCallback` and reorder so the `useEffect` consumer follows its declaration
- [ ] 3.2 Move the `Cannot access refs during render` reads into the appropriate `useEffect` / event handler
- [ ] 3.3 Fix the dependency-list shape that triggers `react-hooks/exhaustive-deps`

## 4. Misc (5 sites)

- [ ] 4.1 `cn.test.ts` — fix the two `no-constant-binary-expression` predicates so the `&&` has a meaningful left operand
- [ ] 4.2 Resolve `no-useless-assignment` (drop the unused `body` write or use it)
- [ ] 4.3 Add a `displayName` to the component flagged by `react/display-name`

## 5. Verify

- [ ] 5.1 `pnpm lint` reports `0 errors` under `--max-warnings 50`
- [ ] 5.2 `pnpm vitest --run` is green (or the same baseline of pre-existing failures phase26 inherited — no new regressions)
- [ ] 5.3 `pnpm build` is green

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
