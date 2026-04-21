# Proposal: phase8_dep-audit-npm-security-sweep

## Why

After the 2026-04-21 security-alert triage
(commit `10b01ed2` on `release/v3.0.0`), Dependabot still has a pile
of open npm / pnpm advisories against the three live lockfiles. They
are all **development-scope** or **transitive through upstream
packages we cannot patch directly** — no runtime risk to the shipped
server binary or the published SDKs — so the current triage
deliberately deferred them. This task is the focused sweep that
closes the cluster.

Current open count after triage (2026-04-21):

| Path | Cluster | Count | Scope |
|------|---------|-------|-------|
| `gui/pnpm-lock.yaml` | Electron 41 family | ~22 | dev |
| `gui/pnpm-lock.yaml` | electron-builder -> `@electron/asar` -> minimatch | 1 | dev |
| `dashboard/pnpm-lock.yaml` | monaco-editor -> dompurify | 5 | dev |
| `dashboard/pnpm-lock.yaml` | Vite path traversal / `server.fs.deny` bypass | 3 | dev |
| `dashboard/pnpm-lock.yaml` | lodash `_.template` + prototype pollution | 2 | runtime (transitive) |
| `sdks/typescript/pnpm-lock.yaml` | Vite / picomatch / flatted / serialize-javascript | ~7 | dev |

The lodash runtime pair on `dashboard/` is the only one with nominal
runtime scope; in practice lodash is bundled into the dashboard SPA,
which loads inside an authenticated operator session, so the attack
surface is the operator's own browser. Still worth closing.

Two upstream blockers delay individual fixes:

1. **Electron 41 -> 42+** — the `gui/` Electron upgrade is
   manifest-only on v3.0.0 (`phase7_frontend-major-migrations`)
   because it depends on the `@hivehub/vectorizer-sdk@3.0.0` npm
   publish landing first. When the SDK publishes, `pnpm install`
   can pull a newer Electron line that clears ~22 alerts.
2. **monaco-editor -> dompurify** — monaco has not shipped a patched
   release that pins a non-vulnerable dompurify. Either wait for
   monaco upstream or replace monaco with a different editor
   (`@codemirror/view` is the usual swap).

## What Changes

Run each lockfile through `pnpm update --latest` inside a dedicated
worktree, verify the resulting build + test passes, and commit the
refreshed lockfile per path. Where the upstream blocker prevents a
clean bump, document the workaround (version pin + issue link) in
the lockfile's surrounding `package.json` and open upstream issues
if none exist.

Order of operations (independent, safe to split across PRs):

1. `sdks/typescript/` — Vite 7 -> 8, picomatch 2 -> 4, flatted 3.2 ->
   3.3, serialize-javascript 6 -> 7. All dev-tool bumps; expect
   `pnpm build && pnpm lint && pnpm test` to stay green. Closes
   ~7 alerts.
2. `dashboard/` — Vite 7 -> 8 + lodash 4.18.1 -> latest patch. Test
   under Playwright + vitest. Closes ~5 alerts. Leave monaco-editor
   alone until the upstream dompurify bump lands.
3. `gui/` — picomatch + minimatch transitive bumps via `pnpm update`
   once the electron-builder chain cooperates. Electron 41 -> 42+
   stays blocked on the `@hivehub/vectorizer-sdk@3.0.0` publish; link
   `phase7_frontend-major-migrations` and move the bulk-Electron
   bump into the follow-up that the SDK publish unblocks.

Each step runs the project's lint + build + test commands before the
lockfile is committed so we catch transitive breakage (Vite 8 needs
Node 22.12+, for example) at the boundary.

## Impact

- Affected specs: `docs/deployment/configuration.md` (if the Vite
  bump changes an operator-visible flag); `CHANGELOG.md` under
  `3.0.0 > Security`.
- Affected code:
  - `sdks/typescript/package.json` + `pnpm-lock.yaml`
  - `dashboard/package.json` + `pnpm-lock.yaml`
  - `gui/package.json` + `pnpm-lock.yaml`
  - `.github/workflows/*` if the Vite 8 Node floor bumps the CI
    matrix.
- Breaking change: NO (all bumps stay on their current major line
  where possible; Vite 8 is a single-major jump inside the Vite 7 ->
  8 migration already scoped in `phase7_frontend-major-migrations`).
- User benefit: closes ~35 of the remaining open Dependabot alerts
  on `release/v3.0.0` + `main`, matching the 0 / 0 / 0 baseline the
  v3.0.0 release should ship with.
