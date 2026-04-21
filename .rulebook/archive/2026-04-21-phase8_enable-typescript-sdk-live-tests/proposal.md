# Proposal: phase8_enable-typescript-sdk-live-tests

## Why

Probe 4.1 of `phase8_release-v3-runtime-verification` ran
`cd sdks/typescript && pnpm test` against the live `release/v3.0.0`
binary. Result: **352 tests passed, 46 skipped**. The 46 skipped are
NOT env-guarded as the probe acceptance expected ("remove the
env-guard marker that currently excludes them"). They are
unconditional `.skip()` / `.describe.skip()` calls in three test
files:

- `sdks/typescript/tests/discovery.test.ts:130` —
  `it.skip('should include citations in evidence', ...)`
- `sdks/typescript/tests/file-operations.test.ts:17` —
  `describe.skip('File Operations', ...)` (entire suite)
- `sdks/typescript/tests/routing.test.ts:14` —
  `describe.skip('Master/Replica Routing', ...)` (entire suite)

Running them against the v3 server would exercise wire paths that
currently have zero SDK-side coverage. Without the gate flip, any
TS-SDK consumer hitting discovery / file-ops / routing gets
production traffic without a single regression test catching a
shape drift.

Source: `docs/releases/v3.0.0-verification.md` probe 4.1.

## What Changes

Replace each unconditional `.skip()` / `.describe.skip()` with an
env-guard that defaults to "run against live server on
`127.0.0.1:15002`" and skips cleanly when the health check fails
(matches the `checkServerAvailability` pattern already in
`tests/test-helpers.ts`).

1. In each of the three test files, replace `it.skip(...)` /
   `describe.skip(...)` with a live-server probe:
   ```ts
   const live = await checkServerAvailability(VECTORIZER_BASE_URL);
   (live ? describe : describe.skip)('File Operations', () => { ... });
   ```
2. Update `tests/test-helpers.ts` to expose the probe + a
   `VECTORIZER_BASE_URL` constant (env var with
   `"http://127.0.0.1:15002"` default).
3. Rewrite any stale assertion fixtures the gated tests carry to
   match the current REST shapes (same F1-F5 drift the Python SDK
   hit).
4. Run `pnpm test` against the live server. Acceptance: 46 tests
   move from `skipped` to `passed` (or a subset fail with a
   documented follow-up task for each — e.g. a shape still
   unrewritten).

## Impact

- Affected specs: `sdks/typescript/TESTING_GUIDE.md` — document the
  env-guard + how to run the live-server subset.
- Affected code:
  - `sdks/typescript/tests/discovery.test.ts`
  - `sdks/typescript/tests/file-operations.test.ts`
  - `sdks/typescript/tests/routing.test.ts`
  - `sdks/typescript/tests/test-helpers.ts` (env-guard helper)
- Breaking change: NO (test gate only; no production code change).
- User benefit: discovery / file-operations / routing paths of the
  TS SDK have real regression coverage against the v3 server.
  Unblocks probe 4.1.
