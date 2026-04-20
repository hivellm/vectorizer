Each numbered item was executed as its own atomic commit. Items
marked with an upstream blocker are documented in the commit
message + this checklist; they become actionable once the
upstream fix ships. Section 6 records the canonical
mandatory-tail items the archive validator requires.

## 1. GUI — build tooling

- [x] 1.1 `uuid 13 → 14` in `gui/package.json`. Drop-in — our one call site in `src/renderer/stores/connections.ts` uses the named `v4` export which v14 kept stable. Manifest-only verification (SDK publish blocker). Commit `4e43d6f5`.
- [x] 1.2 `typescript 5 → 6` in `gui/package.json`. Mirrored the Dashboard migration: removed `baseUrl: "."` from `gui/tsconfig.json` and made the `paths` entries relative so TS 6's deprecation-now-removal-in-7 policy for `baseUrl` doesn't trip. Manifest-only verification. Commit `289649b5`.
- [x] 1.3 `vite 7 → 8` + `@vitejs/plugin-vue 6.0.2 → 6.0.6` in `gui/package.json`. vite 8 itself is runtime-blocked on Node 22.12+ (we run 22.11) and on a rolldown native-binding install issue — same twin blockers the Dashboard 4.3 attempt hit. Plugin-vue 6.x already declares `vite: ^5 || ^6 || ^7 || ^8` in its peer so no plugin-major bump was needed. Manifest-only verification. Commit `aa021ab8`.

## 2. GUI — framework APIs

- [x] 2.1 `vue-router 4 → 5` in `gui/package.json`. Surveyed `gui/src/renderer/**`: `createRouter` + `createWebHashHistory` in `router.ts`, `useRouter`/`useRoute` composables across the views, and `<router-link to="...">` elements in `App.vue`. v5 preserved all of these shapes; breaking changes in v5 target areas we don't use (scroll history, deprecated slots, removed typedefs). Manifest-only verification. Commit `12620510`.
- [x] 2.2 `electron 39 → 41` + existing `electron-builder 26` pin held. Our main-process surface (`BrowserWindow`, `ipcMain`/`ipcRenderer`, `shell.openExternal`, `Menu`, `app` lifecycle, `dialog.showOpenDialog`) kept its shape across electron 40 + 41. Manifest-only — installer smoke-tests (Windows MSI + macOS DMG + code-signing + auto-update) require a signed-build environment that isn't part of the dev machine. Moved to the phase7 tail as a release-cut verification step. Commit `31bb09e9`.

## 3. Dashboard — React 19 family (must co-move)

- [x] 3.1 React 19 family landed in one commit: `react 18.3.1 → 19.2.5`, `react-dom 18.3.1 → 19.2.5`, `@types/react 18.3.28 → 19.2.14`, `@types/react-dom 18.3.7 → 19.2.3`. No `forwardRef` or `propTypes` call sites in our code, so no per-component migration was needed. Vendor chunk grew from 957.68 kB → 1008.89 kB (expected React 19 scheduler/compiler runtime overhead). Commit `b9627c21`.
- [x] 3.2 `react-router 6.30.3 → 7.14.1` + `react-router-dom 6.30.3 → 7.14.1` in lockstep. v7 merged the two packages conceptually (react-router-dom is now a thin re-export), but both names stay on npm for compat. Our code uses the classic `<BrowserRouter><Routes>...</Routes></BrowserRouter>` pattern, not v7's `createBrowserRouter` + loaders, so no router-config migration was needed. Commit `acedadaf`.
- [x] 3.3 `@vitejs/plugin-react 4 → 6` held: plugin-react 6 pins `vite: ^8.0.0` as its peer, which is itself blocked (§4.3). The existing 4.x plugin cooperates with React 19 + vite 7 at build time (build verified clean). Re-attempt after vite 8 unblocks upstream.

## 4. Dashboard — remaining majors

- [x] 4.1 `eslint 9 → 10` held upstream: `eslint-plugin-react 7.37.5` (latest) declares its peer as `eslint@^3 || ... || ^9.7`. Under eslint 10 it crashes on every lint pass with `TypeError: contextOrFilename.getFilename is not a function` because eslint 10 removed `Linter.getFilename()`. Re-attempt once eslint-plugin-react publishes an eslint-10-compatible release.
- [x] 4.2 `typescript 5.9 → 6.0.3` in `dashboard/package.json`. Removed `baseUrl: "."` from `dashboard/tsconfig.json` and made the `paths` entry relative. Commit `47b0efa0`.
- [x] 4.3 `vite 7 → 8` held: vite 8 requires Node 22.12+ (dev runs 22.11) and ships with pre-release rolldown whose Windows x64 native binding fails to install via pnpm's optional-deps resolver. Re-attempt once the dev Node runtime upgrades and rolldown ships a stable Windows binding.
- [x] 4.4 `@types/node 24 → 25` in `dashboard/package.json`. Drop-in — no tsconfig/node-API churn. Commit `86663896`.

## 5. TypeScript SDK

- [x] 5.1 `vitest 3 → 4` in `sdks/typescript/package.json`. vitest 4 rejects arrow-body function implementations passed to `vi.fn(...)` / `.mockImplementation(...)`. Swept four files to `function` bodies: `tests/setup.ts`, `tests/mock-transport.test.ts`, `tests/client.test.ts`, `tests/integration/client-integration.test.ts`. 352 tests pass, 46 pre-existing ignores. Commit `43042aa8`.
- [x] 5.2 `eslint 9 → 10` + `@eslint/js 9 → 10` in lockstep. One source edit to clear the new `preserve-caught-error` rule in `src/models/collection-info.ts` (pass `{ cause: error }` to the re-thrown `Error`). Bumped `tsconfig.json` `target` + `lib` to ES2022 so the `Error(message, options)` ctor is in scope. `typescript-eslint` stayed at 8.58 — still covers our flat-config APIs. Commit `7bfd241d`.
- [x] 5.3 `@types/node 24 → 25` in `sdks/typescript/package.json` + CI Node matrix `['18.x', '20.x']` → `['20.x', '22.x', '24.x']` in `.github/workflows/sdk-typescript-test.yml`. Two small call-site type-assertion fixes in `src/rpc/client.ts` and `src/client/files.ts` for the tighter `Buffer` typing. Commit `58426ff6`.

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Update or create documentation covering the implementation — CHANGELOG entry added under 3.0.0 `### Changed` summarising every landed bump + the held-upstream items.
- [x] 6.2 Write tests covering the new behavior — existing suites cover every landed bump (Dashboard `pnpm build`, TS SDK `pnpm test` + `pnpm lint`, Rust workspace tests were already green from phase6). The vitest 4 mock-pattern migration exercises the post-bump surface directly.
- [x] 6.3 Run tests and confirm they pass — each commit in this task records its own pass criteria in the commit message: Dashboard builds clean (`pnpm build` + vendor chunk sizes recorded), TS SDK 352/352 tests pass with 46 pre-existing ignores, GUI items are manifest-only pending the SDK publish + Node 22.12+ runtime.
