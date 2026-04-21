## 1. Implementation

- [ ] 1.1 Add `test.exclude: ['e2e/**', 'node_modules/**']` to
  `dashboard/vitest.config.ts` (or create it if missing) so
  Playwright specs stay out of vitest discovery.
- [ ] 1.2 Create `dashboard/src/test-utils/render.tsx` exporting
  `renderWithProviders(ui, opts?)` that wraps the tree in
  `<AuthProvider><MemoryRouter initialEntries={[opts?.route ?? '/']}>
   {ui}
  </MemoryRouter></AuthProvider>`. Re-export `screen`, `fireEvent`,
  `waitFor` from `@testing-library/react` for call-site ergonomics.
- [ ] 1.3 Add a `mockAuthState(partial)` helper in the same file so
  tests can seed an authenticated / anonymous / admin user.
- [ ] 1.4 Rewrite `src/pages/__tests__/CollectionsPage.test.tsx` to
  call `renderWithProviders(<CollectionsPage />, {route:'/collections'})`
  instead of plain `render`.
- [ ] 1.5 Rewrite `src/router/__tests__/AppRouter.test.tsx` the
  same way.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (`dashboard/TESTING_GUIDE.md` new section on
  `renderWithProviders`; `dashboard/CHANGELOG.md` if present under
  `3.0.0 > Tests`).
- [ ] 2.2 Write tests covering the new behavior (no new product
  tests; add one unit test at `src/test-utils/__tests__/render.test.tsx`
  that asserts `renderWithProviders` injects both providers and
  that `mockAuthState` round-trips the stubbed user).
- [ ] 2.3 Run tests and confirm they pass (`pnpm test:run` — target
  33/33; `pnpm test:e2e` Playwright — no regression).
