## 1. Implementation

- [ ] 1.1 In `tests/test-helpers.ts`, export a `runIfServer` helper
  that returns `describe` or a no-op describe based on a synchronous
  health-check probe of `VECTORIZER_BASE_URL`
  (`http://127.0.0.1:15002` by default; env var override).
- [ ] 1.2 Rewrite the `describe` call at `tests/file-operations.test.ts:17`
  to use `runIfServer('File Operations', () => { ... })` instead of
  the current unconditional gate.
- [ ] 1.3 Rewrite the `describe` call at `tests/routing.test.ts:14`
  to use `runIfServer('Master/Replica Routing', () => { ... })`.
- [ ] 1.4 Rewrite the `it` call at `tests/discovery.test.ts:130`
  ('should include citations in evidence') with the same
  `runIfServer`-style per-test gating.
- [ ] 1.5 Rewrite assertion fixtures in all three files to match the
  current F1 / F2 / F5 response shapes. If any test cannot be
  fixed against the current server, open a focused follow-up task
  for that endpoint and reference it from this task.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (`sdks/typescript/TESTING_GUIDE.md` — new section "Live-server
  gate"; `sdks/typescript/CHANGELOG.md` under `3.0.0 > Tests`).
- [ ] 2.2 Write tests covering the new behavior (the 46 formerly-
  gated tests ARE the coverage; ensure they fail-fast with a
  clear error when the live server is absent so developers know to
  boot the server before re-running).
- [ ] 2.3 Run tests and confirm they pass (`pnpm test` against the
  live server — target: all 46 gated tests execute and pass).
