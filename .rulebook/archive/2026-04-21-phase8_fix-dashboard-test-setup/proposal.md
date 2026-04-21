# Proposal: phase8_fix-dashboard-test-setup

## Why

`cd dashboard && pnpm test:run` produces **6 failed / 27 passed**
(baseline state verified via `git stash` + re-test in commit
`9592d0e3`). The failures predate this phase and are test-setup
issues, not product regressions:

```
FAIL  e2e/login.spec.ts [ e2e/login.spec.ts ]
  Error: Playwright Test did not expect test.describe() to be called here.
FAIL  e2e/setup-wizard.spec.ts [ e2e/setup-wizard.spec.ts ]
  Error: Playwright Test did not expect test.describe() to be called here.
FAIL  src/pages/__tests__/CollectionsPage.test.tsx
  > CollectionsPage > should render collections page
  Error: useAuth must be used within an AuthProvider
FAIL  src/pages/__tests__/CollectionsPage.test.tsx
  > CollectionsPage > should render create collection button
FAIL  src/router/__tests__/AppRouter.test.tsx
  > AppRouter > should render router with MainLayout
FAIL  src/router/__tests__/AppRouter.test.tsx
  > AppRouter > should navigate to overview by default
```

Two distinct bugs:

1. **Playwright e2e specs are executed by vitest.** `vitest run`
   discovers `e2e/*.spec.ts` because the default `include` glob
   catches them, but Playwright's `test.describe` throws when it
   is not being driven by the Playwright runner. Need a vitest
   `exclude` for `e2e/**` so Playwright runs only under
   `playwright test`.
2. **Component tests miss the AuthProvider wrapper.** After F7 (auth
   gating), every page under `src/pages/` is rendered inside an
   `AuthProvider`; the unit tests render the page bare and crash on
   `useAuth` at mount. Fix: shared `renderWithProviders` helper
   under `src/test-utils/` that wraps the tree in `AuthProvider`
   (plus React Router + any other context the production shell
   provides).

Source: `docs/releases/v3.0.0-verification.md` (recorded during
probe 3.7 / section 4 sweeps).

## What Changes

1. `dashboard/vite.config.ts` (or `vitest.config.ts`): add
   `test.exclude: ['e2e/**', 'node_modules/**']` so Playwright
   specs stay out of vitest discovery.
2. New `dashboard/src/test-utils/render.tsx` that exports
   `renderWithProviders(ui, {route?})` wrapping the subject in
   `<AuthProvider><MemoryRouter>...</MemoryRouter></AuthProvider>`.
   Also export a `mockAuthState(state)` helper for tests that want
   to stub an authenticated / anonymous user.
3. Update `src/pages/__tests__/CollectionsPage.test.tsx` +
   `src/router/__tests__/AppRouter.test.tsx` to call
   `renderWithProviders` instead of `render`.
4. Run `pnpm test:run`. Acceptance: 33/33 pass (6 that were failing
   now pass; 27 previously passing still pass).
5. Run `pnpm test:e2e` (Playwright) — no regression in the e2e
   suite.

## Impact

- Affected specs: `dashboard/TESTING_GUIDE.md` (if present) — document
  `renderWithProviders` as the canonical pattern for page-level
  tests.
- Affected code:
  - `dashboard/vitest.config.ts` (new exclude)
  - `dashboard/src/test-utils/render.tsx` (new helper)
  - `dashboard/src/pages/__tests__/CollectionsPage.test.tsx` (rewrite)
  - `dashboard/src/router/__tests__/AppRouter.test.tsx` (rewrite)
- Breaking change: NO (tests only).
- User benefit: dashboard test suite is green end-to-end, so
  regressions in `CollectionsPage` / `AppRouter` / the Playwright
  e2e flows surface on every PR instead of silently drifting.
