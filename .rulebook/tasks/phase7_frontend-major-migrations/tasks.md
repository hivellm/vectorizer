Each numbered item is its own atomic migration: one manifest edit
(or small family of co-moving edits), one call-site sweep, one
verification run, one commit. Within a section items can land in
any order unless a sub-task explicitly gates on an earlier one.
Section 5 records the canonical mandatory-tail items the archive
validator requires.

## 1. GUI — build tooling

- [ ] 1.1 `uuid 13 → 14` in `gui/package.json`. The smallest surface in the GUI bucket — do this first to clear a trivial win. v14 made the default export ESM-only and dropped the CommonJS fallback; grep `from 'uuid'` and `require('uuid')` across `gui/src/**`. Most imports should already be `import { v4 } from 'uuid'` which is unchanged. Rerun `pnpm install && pnpm build`.
- [ ] 1.2 `typescript 5 → 6` in `gui/package.json`. Audit `gui/tsconfig.json` + `gui/tsconfig.main.json` for the tightened 6.x defaults: `moduleResolution` may need to move to `"bundler"`, `lib` additions for newer DOM types, strict-null-checks on narrowing that 6.x caught. Rerun `pnpm type-check && pnpm build`.
- [ ] 1.3 `vite 7 → 8` + `@vitejs/plugin-vue` to the matching vite-8-compatible major in `gui/package.json`. Audit `gui/vite.config.ts` — v8 moved several plugin options and dropped the legacy-CJS transform from the default pipeline. Rerun `pnpm build`.

## 2. GUI — framework APIs

- [ ] 2.1 `vue-router 4 → 5` in `gui/package.json`. Call-site sweep across `gui/src/**/*.{ts,vue}` for:
  - every `useRouter()` + `useRoute()` composable (some field names moved — `route.query`/`route.params` signatures changed).
  - every `<router-link>` prop (the `active-class` / `exact-active-class` defaults + `custom` slot surface changed).
  - every `router.push(...)` / `router.replace(...)` with a location object (some shorthand fields renamed).
  Rerun `pnpm build` + a manual smoke of every route in the running GUI.
- [ ] 2.2 `electron 39 → 41` (two majors) + bump `electron-builder` to the v27+ line that supports electron 41. Rebuild the installers locally on Windows (`pnpm electron:build:win` produces the MSI) and on macOS (`pnpm electron:build:mac` produces the DMG). Smoke-test: install the built package, launch, confirm the app connects to a running server on port 15002, confirm code-signing verifies on both OSes, confirm auto-update still targets the HiveLLM release feed.

## 3. Dashboard — React 19 family (must co-move)

- [ ] 3.1 `react 18 → 19` + `react-dom 18 → 19` + `@types/react 18 → 19` + `@types/react-dom 18 → 19` in one commit. These four packages are version-locked by the React team — bumping them individually will fail type checking. Audit targets in `dashboard/src/**`:
  - `React.forwardRef` usages — React 19 deprecated the pattern in favour of passing `ref` as a regular prop. Legacy forwardRef still compiles but emits a dev-only warning; decide per-call whether to migrate or ignore.
  - `propTypes` — removed in 19; replace with TypeScript prop interfaces (most of `dashboard/src/components/**` is already typed, so this should be a no-op).
  - `useOptimistic` / `useFormStatus` / `use` — new hooks available; not required but worth flagging in `docs/migration/react-19.md` if any form submits get reworked to use them.
  - `ReactDOM.render` → `ReactDOM.createRoot` — already done during the 17→18 jump.
  Rerun `pnpm build && pnpm test:run`.
- [ ] 3.2 `react-router 6 → 7` + `react-router-dom 6 → 7` in lockstep. The v7 stable line collapsed the split between `react-router` and `react-router-dom` (and merged `@remix-run/router`) — `dashboard/package.json` should land on `react-router` only after v7. Sweep `dashboard/src/router/**`:
  - every `createBrowserRouter(...)` call — the data-API loader/action signature stabilised.
  - every `<Route>` config — the `element` prop stays, but `errorElement` → `ErrorBoundary` component mount, and `loader` / `action` now auto-infer types from the generated `Route.tsx` file.
  - every `useNavigate()` / `useLoaderData()` / `useActionData()` — v7 types narrow by route, so untyped calls may fail under `@types/react` 19.
  Rerun the Dashboard gates.
- [ ] 3.3 `@vitejs/plugin-react 4 → 6` in `dashboard/package.json`. Gates on 3.1 landing first — the v6 plugin pins React 19 as its peer. Rerun `pnpm build && pnpm test:run`.

## 4. Dashboard — remaining majors

- [ ] 4.1 `eslint 9 → 10` + `@eslint/js 9 → 10` in lockstep. The Dashboard's flat-config file already exists at `dashboard/eslint.config.*`. v10 dropped some rule exports that moved to typescript-eslint (e.g. `@typescript-eslint/no-var-requires`). Bump, resolve any removed-rule errors, rerun `pnpm lint`.
- [ ] 4.2 `typescript 5 → 6` in `dashboard/package.json`. Same playbook as GUI 1.2 — audit `dashboard/tsconfig.json`, rerun `pnpm build`.
- [ ] 4.3 `vite 7 → 8` in `dashboard/package.json`. Same playbook as GUI 1.3 — audit `dashboard/vite.config.*`, rerun `pnpm build`.
- [ ] 4.4 `@types/node 24 → 25` in `dashboard/package.json`. Gate on the Node matrix pin in `.github/workflows/dashboard-*.yml` — bump CI to Node 25 first, then the types.

## 5. TypeScript SDK

- [ ] 5.1 `vitest 3 → 4` in `sdks/typescript/package.json`. vitest 4 tightened the mock API: every `vi.fn()` mock implementation must use `function` or `class` syntax (arrow bodies are rejected with "mock did not use 'function' or 'class' in its implementation"). Sweep `sdks/typescript/tests/**/*.test.ts`:
  - `vi.fn(() => foo)` → `vi.fn(function () { return foo; })` at every site.
  - `vi.fn().mockResolvedValue(...)` — still legal; the issue is only with implementations passed to `vi.fn(...)` directly.
  - Co-bump `@vitest/coverage-v8` to the matching major.
  Rerun `pnpm install && pnpm build && pnpm test`.
- [ ] 5.2 `eslint 9 → 10` + `typescript-eslint 8 → 9` in lockstep in `sdks/typescript/package.json`. Gates on 5.1 so the shared flat-config layout doesn't drift across Dashboard and SDK at the same time. Rerun `pnpm lint` — resolve any removed-rule errors.
- [ ] 5.3 `@types/node 24 → 25` in `sdks/typescript/package.json`. Gate on the Node matrix pin in `.github/workflows/sdk-typescript-*.yml` — bump CI to Node 25 first, then the types.

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation (CHANGELOG entry per sub-task under `### Changed`; for each framework-level migration — React 19, react-router 7, vue-router 5, electron 41 — add a migration-guide note under `docs/migration/` pointing downstream consumers at the upstream changelog).
- [ ] 6.2 Write tests covering the new behavior (each framework migration should land with a regression test on at least one representative route / component / IPC handler that exercises the post-bump surface).
- [ ] 6.3 Run tests and confirm they pass (per-ecosystem gates: Rust workspace unaffected; `pnpm build && pnpm test:run` for Dashboard; `pnpm build && pnpm test` for TS SDK; `pnpm type-check && pnpm build` + installer smoke for GUI).
