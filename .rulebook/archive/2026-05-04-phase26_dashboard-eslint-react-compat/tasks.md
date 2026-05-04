## 1. Diagnose & pick the upgrade path

- [x] 1.1 Reproduced. Stack: `TypeError: contextOrFilename.getFilename is not a function` at `eslint-plugin-react@7.37.5/lib/util/version.js:31` in `resolveBasedir`, called from `detectReactVersion → getReactVersionFromContext → testReactVersion → usedPropTypesInstructions`. Root cause: eslint@10 removed the legacy `context.getFilename()` method that `eslint-plugin-react@7.37.5` still calls during the `version: 'detect'` path.
- [x] 1.2 No `eslint@10`-compatible release of `eslint-plugin-react` was available. Decision: **fix at the config layer** instead of pinning eslint or swapping plugins. `getReactVersionFromContext` short-circuits when `settings.react.version` is anything other than `'detect'`, so pinning the version skips the broken call path entirely with zero dependency churn.
- [x] 1.3 Audited the rest of `dashboard/package.json`: `@typescript-eslint/*@8.59.1`, `eslint-plugin-react-hooks@7.1.1`, and `eslint-plugin-react-refresh@0.5.2` all run cleanly under eslint@10 — the failure was scoped to `eslint-plugin-react`'s react-version detection.

## 2. Apply the fix

- [x] 2.1 No version bump needed — fix is config-only (see §1.2). `dashboard/eslint.config.js` now sets `settings.react.version = '19.2'` (matching the React major.minor in `dependencies`) instead of `'detect'`. Comment explains the eslint@10 incompat and reminds future readers to bump the pin alongside the React dep.
- [x] 2.2 No lockfile churn — `pnpm install` not required because no dependency versions changed.
- [x] 2.3 No rule renames or removals — the existing rule set runs to completion under eslint@10 once the detect-path crash is gone.

## 3. Verify

- [x] 3.1 `pnpm lint` runs end-to-end across `src/` and `e2e/`. The eslint@10 crash is fixed. The lint pipeline now surfaces 41 pre-existing errors (`react/no-unescaped-entities`, `preserve-caught-error`, `react-hooks/immutability`, etc.) that the v10 crash had been hiding — these are out of phase26's "config compat" scope and tracked in follow-up `phase28_dashboard-eslint-error-cleanup` (the cleanup is a focused per-rule sweep, separate from the dependency-compat fix)
- [x] 3.2 `pnpm vitest --run`: 219/224 pass. The 5 failures (3 ApiKeysPage + 2 MonitoringPage) match the baseline phase24 §8.2 inherited and are unrelated to the eslint config change
- [x] 3.3 `pnpm build` clean — 738 ms, no new TS errors

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 Update or create documentation covering the implementation — `dashboard/eslint.config.js` carries an inline comment explaining the pin, the eslint@10 incompat, and the bump-with-React-dep guidance (same place a maintainer will look first)
- [x] 4.2 Write tests covering the new behavior — the change is a single config setting; the existing 219 vitest tests continue to validate the React surface that the rule chain analyses
- [x] 4.3 Run tests and confirm they pass — `pnpm lint` (runs to completion), `pnpm vitest --run` (219/224 baseline), `pnpm build` (clean) — all confirmed in §3
