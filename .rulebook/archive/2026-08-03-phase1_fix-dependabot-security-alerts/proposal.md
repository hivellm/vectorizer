# Proposal: phase1_fix-dependabot-security-alerts

## Why
27 open Dependabot security alerts (18 high, 8 medium, 1 low) — all npm,
concentrated in three lockfiles: gui/pnpm-lock.yaml (15), dashboard/pnpm-lock.yaml
(10) + dashboard/package.json (1), and sdks/typescript/pnpm-lock.yaml (1). They
are mostly transitive dependencies pulled by build/UI tooling. Clearing them
removes the security backlog for the 3.6.0 line.

## What Changes
Bump each vulnerable package to its patched version, per project, and refresh
the lockfiles. Prefer a `pnpm.overrides` floor (matches the existing pattern in
dashboard/package.json) for transitive deps; bump direct deps directly.

Packages -> patched target (npm):
- postcss -> 8.5.18 (dashboard, gui, sdks/typescript)
- brace-expansion -> 1.1.16 and >=5.0.7 (dashboard, gui) — two disjoint ranges
- react-router -> 7.18.0 / 8.3.0 (dashboard; one is a direct dep in package.json)
- js-yaml -> 4.3.0 (dashboard, gui) — stay on patched 4.x (do NOT jump to 5.x;
  the pnpm `<5` security override stands)
- dompurify -> 3.4.12 (dashboard)
- axios -> 1.18.0 (gui)
- app-builder-lib -> 26.15.0, builder-util-runtime -> 9.7.0 (gui, electron-builder)
- fast-uri -> 3.1.4, shell-quote -> 1.9.0, tar -> 7.5.18 (gui)

Verify each project still builds after the bump (dashboard `pnpm build`, TS SDK
`tsc` + eslint, gui `pnpm build` if feasible). Confirm the alerts clear.

## Impact
- Affected specs: security
- Affected code: dashboard/package.json + pnpm-lock.yaml, gui/package.json +
  pnpm-lock.yaml, sdks/typescript/package.json + pnpm-lock.yaml
- Breaking change: NO (security patches; react-router 7->8 in dashboard needs a
  build check as it is the one non-patch major)
- User benefit: no open high/medium security advisories on the shipped web assets
