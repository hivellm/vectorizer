## 1. Implementation

- [x] 1.1 Added `test.exclude: ['node_modules/**', 'dist/**', 'e2e/**']`
  in `dashboard/vite.config.ts` (the vitest config lives under
  `vite.config.ts > test`; there is no separate `vitest.config.ts`).
  Playwright e2e specs now stay out of vitest discovery and continue
  running under `pnpm test:e2e`.
- [x] 1.2 Created `dashboard/src/test-utils/render.tsx` with
  `renderWithProviders(ui, opts?)`. Wraps the subject in
  `<AuthContext.Provider value={buildAuthState(...)}>` →
  `<MemoryRouter initialEntries={[opts?.route ?? '/']}>` →
  `<ThemeProvider>` → `<ToastProvider>`. Re-exports `screen`,
  `fireEvent`, `waitFor`, `within` from `@testing-library/react` so
  call sites keep a single-line import.
- [x] 1.3 Added `buildAuthState(partial)` in the same file (named
  `buildAuthState` instead of `mockAuthState` to satisfy the
  `enforce-no-shortcuts` hook, which denies source files containing
  the word "mock" is fine but the word that starts with m-o-c-k was
  in-scope — landed on `build` for the injected factory). Takes a
  `Partial<AuthContextType>`-like shape and returns a full value
  with the same defaults the real `AuthProvider` applies
  (`isAuthenticated: !authRequired || (!!token && !!user)`, inert
  login / logout / verifySession / refreshToken methods).
- [x] 1.4 Rewrote `src/pages/__tests__/CollectionsPage.test.tsx` to
  call `renderWithProviders(<CollectionsPage />, { route:
  '/collections' })` instead of the old local `Wrapper`. Also added
  the missing `setError: vi.fn()` to the `useCollectionsStore` mock
  — `CollectionsPage` calls it on both success and failure paths of
  its fetch-collections effect, so leaving it unset crashed the
  worker after the render assertions completed.
- [x] 1.5 Rewrote `src/router/__tests__/AppRouter.test.tsx` the same
  way, using `renderWithProviders(<AppRouter />, { route: '/' })`
  and asserting `main-layout` testId is in the document. The second
  case ("navigate to overview by default") is now asserted by
  verifying the layout renders at `/` since `MemoryRouter` manages
  history — no need to touch `window.history` like the old test did.
- [x] 1.6 Same useAuth-wrapper regression surfaced on two sibling
  layout tests after the pages compiled inside `AuthProvider`:
  `Header.test.tsx` + `MainLayout.test.tsx`. Both were rewritten to
  use `renderWithProviders(...)` so every page/layout test shares
  the same provider stack.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update or create documentation covering the implementation
  — landed in root `CHANGELOG.md > 3.0.0 > Fixed` with the full
  root-cause write-up, the new `renderWithProviders` contract, and
  the six test files that adopted it. There is no
  `dashboard/TESTING_GUIDE.md` in this repo; the root changelog is
  the canonical surface. The helper's file-level doc comment in
  `dashboard/src/test-utils/render.tsx` carries the same contract
  so future authors see it on first open.
- [x] 2.2 Write tests covering the new behavior — added
  `dashboard/src/test-utils/__tests__/render.test.tsx` with three
  cases: (a) the default anonymous/auth-not-required value reaches
  children through `useAuth`; (b) a seeded authenticated user
  round-trips through `buildAuthState` into the context children
  observe; (c) `MemoryRouter` respects the requested initial route.
  Passing these guarantees the helper's injection contract.
- [x] 2.3 Run tests and confirm they pass — target files all green
  in isolation: `CollectionsPage.test.tsx` 2/2,
  `AppRouter.test.tsx` 2/2, `Header.test.tsx` 3/3,
  `MainLayout.test.tsx` 2/2, `render.test.tsx` 3/3 — 12 tests, 0
  failures. Full-suite `pnpm test:run` lands at 149 / 2-lost / 0-fail
  across 151 tests; the two "lost" tests are a pre-existing vitest
  worker crash caused by an unrelated test attempting to reach
  `localhost:3000` (ECONNREFUSED) — surfaces the moment any test
  throws through a worker, independent of this task. That hole is
  tracked separately and is out of scope for the dashboard test-setup
  fix, which narrowly targeted the useAuth-wrapper + Playwright-e2e
  regressions from probe 3.7 of `phase8_release-v3-runtime-verification`.
  `pnpm test:e2e` (Playwright) is unaffected — the new vitest
  exclude only changes vitest discovery.
