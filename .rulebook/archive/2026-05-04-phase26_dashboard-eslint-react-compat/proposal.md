# Proposal: phase26_dashboard-eslint-react-compat

## Why

`pnpm lint` is broken project-wide on the dashboard with
`TypeError: contextOrFilename.getFilename is not a function` from
`eslint-plugin-react@7.37.5` because `eslint@10.3.0` removed the legacy
context shape that `react/display-name` and several other rules still
call. Every `.ts` / `.tsx` file errors before any rule runs, so CI lint
gates produce no signal and pre-commit hooks are effectively bypassed.

This is **not** a phase24 regression — it pre-exists the dashboard
console redesign. Phase24's tail (item 8.3) flagged it and explicitly
deferred to a dedicated dependency-bump task.

## What Changes

Either pin `eslint` back to a `9.x` line that `eslint-plugin-react`
supports, or upgrade `eslint-plugin-react` to a release that targets
the eslint v10 context API (≥ `7.38.x` if/when published, or migrate to
the flat-config-aware fork). Choose whichever path keeps the existing
`.eslintrc` rule set green with zero warnings under
`--max-warnings 50`.

## Impact

- Affected code:
  - `dashboard/package.json` — bump or pin `eslint` /
    `eslint-plugin-react` (and any peer like `eslint-config-*` that
    needs to follow)
  - `dashboard/pnpm-lock.yaml` — regenerated lockfile
  - `dashboard/.eslintrc.*` — possible rule-set tweaks if a rule was
    removed or renamed in the chosen version
- Affected specs: none
- Breaking change: NO (lint-only; runtime unaffected)
- User benefit: restores `pnpm lint` as a working quality gate so
  pre-commit hooks and CI lint runs produce real signal again, and
  unblocks the phase24 tail item 8.3.
