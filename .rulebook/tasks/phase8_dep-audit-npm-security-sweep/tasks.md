## 1. Implementation

- [ ] 1.1 Sweep `sdks/typescript/`:
  `cd sdks/typescript && pnpm update --latest` targeting Vite,
  picomatch, flatted, serialize-javascript. Verify `pnpm build &&
  pnpm lint && pnpm test` stays green (352 tests passing +
  46 pre-existing skips). Commit the refreshed `pnpm-lock.yaml`.
- [ ] 1.2 Sweep `dashboard/`: bump Vite to the patched release of
  its current major line, bump lodash to the latest patch version.
  Verify `pnpm build && pnpm lint && pnpm test` and the Playwright
  e2e suite still pass. Leave monaco-editor alone; document the
  upstream dompurify blocker inline in `package.json` with a
  `// reason` comment.
- [ ] 1.3 Sweep `gui/`: `pnpm update` for picomatch + minimatch
  transitive bumps via the electron-builder chain. Confirm
  `pnpm build` + `pnpm electron:pack` still produce a working
  unsigned installer on Windows. Electron 41 -> 42+ stays blocked on
  the `@hivehub/vectorizer-sdk@3.0.0` publish — link
  `phase7_frontend-major-migrations` and note the unblocker in the
  lockfile's surrounding `package.json`.
- [ ] 1.4 Re-check `pnpm audit` in every touched dir after the
  sweep. Acceptance: each `pnpm audit` either returns 0 alerts or
  only transitive alerts with a documented upstream blocker noted
  in this task's resolution log.
- [ ] 1.5 Re-check the GitHub Dependabot dashboard after the PR
  merges. Acceptance: the open-alert count drops from ~35 (post-
  triage baseline) by at least the 3 clusters covered in steps 1.1,
  1.2, 1.3.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (CHANGELOG entry under `3.0.0 > Security` listing each cluster
  closed; if a Vite-8 bump changes an operator-visible flag, note
  the rename in `docs/deployment/configuration.md`).
- [ ] 2.2 Write tests covering the new behavior (no new tests
  required for a pure lockfile sweep — the project's existing
  lint + build + unit + e2e suites are the acceptance gate, and
  step 1.1 / 1.2 / 1.3 runs them per path).
- [ ] 2.3 Run tests and confirm they pass (per-path commands noted
  in 1.1 / 1.2 / 1.3 plus `cargo test --workspace --lib` for the
  Rust side to prove no transitive npm bump cascaded into a Rust
  rebuild).
