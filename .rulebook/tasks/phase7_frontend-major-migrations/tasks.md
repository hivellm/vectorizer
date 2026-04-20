## 1. GUI

- [ ] 1.1 `typescript 5 → 6` in `gui/package.json`. Check `tsconfig*.json`; rerun `pnpm type-check && pnpm build`.
- [ ] 1.2 `vite 7 → 8` + `@vitejs/plugin-vue` matching major in `gui/package.json`. Audit `vite.config.ts`; rerun `pnpm build`.
- [ ] 1.3 `vue-router 4 → 5` in `gui/package.json`. Call-site sweep across every `useRouter()` / `useRoute()` / `<router-link>` prop.
- [ ] 1.4 `uuid 13 → 14` in `gui/package.json`. Grep `from 'uuid'`; rerun `pnpm build`.
- [ ] 1.5 `electron 39 → 41` + verify `electron-builder` + code-signing + auto-update on Windows and macOS.

## 2. Dashboard — React 19 family (must co-move)

- [ ] 2.1 `react 18 → 19` + `react-dom 18 → 19` + `@types/react 18 → 19` + `@types/react-dom 18 → 19` in one commit. Audit `forwardRef` patterns + `propTypes` surface.
- [ ] 2.2 `react-router 6 → 7` + `react-router-dom 6 → 7` in lockstep. Sweep `dashboard/src/router/**`.
- [ ] 2.3 `@vitejs/plugin-react 4 → 6` (gates on §2.1 landing first).

## 3. Dashboard — remaining majors

- [ ] 3.1 `eslint 9 → 10` + `@eslint/js 9 → 10`. Resolve any removed rules in `dashboard/eslint.config.*`.
- [ ] 3.2 `typescript 5 → 6` in `dashboard/package.json`.
- [ ] 3.3 `vite 7 → 8` in `dashboard/package.json`.
- [ ] 3.4 `@types/node 24 → 25` in `dashboard/package.json` (gate on CI Node matrix bump).

## 4. TypeScript SDK

- [ ] 4.1 `vitest 3 → 4` in `sdks/typescript/package.json`. Rewrite every `vi.fn()` mock in 19 test files to satisfy the new "mock must use function or class" rule.
- [ ] 4.2 `eslint 9 → 10` + typescript-eslint 9 line (gates on §4.1 landing first; shared eslint config).
- [ ] 4.3 `@types/node 24 → 25` in `sdks/typescript/package.json` (gate on CI Node matrix bump).

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Update or create documentation covering the implementation (CHANGELOG entry per sub-task under `### Changed`; migration-guide notes under `docs/migration/` where a public API changed).
- [ ] 5.2 Write tests covering the new behavior (each API-breaking migration must land with a test that exercises the post-bump call site).
- [ ] 5.3 Run tests and confirm they pass.
