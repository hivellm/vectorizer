# Proposal: phase8_publish-sdk-v3-to-npm

## Why

`@hivehub/vectorizer-sdk@3.0.0` is built + tested in
`sdks/typescript/` on `release/v3.0.0` but has NOT been published to
the npm registry. `pnpm view @hivehub/vectorizer-sdk versions` shows
the registry tops out at **2.2.0** (3 published versions).

This is the single largest blocker on v3.0.0 ship-readiness:

- `gui/package.json` pins `@hivehub/vectorizer-sdk@^3.0.0`.
  `pnpm install` in `gui/` currently **fails** with
  `ERR_PNPM_NO_MATCHING_VERSION` because the registry doesn't have
  it. Consequence: `gui/pnpm-lock.yaml` cannot be refreshed, the
  Electron 41 -> 42+ bump tracked in
  `phase7_frontend-major-migrations` can't run, the minimatch
  transitive alert at `electron-builder -> @electron/asar` stays
  open, and the GUI installer itself cannot be rebuilt from scratch
  on a clean CI runner.
- Downstream projects that consume `@hivehub/vectorizer-sdk` cannot
  test against the v3 wire shapes without pointing at a local
  checkout — a workflow that fails for CI + non-contributor users.
- `phase8_fix-python-sdk-wire-shapes` / `phase8_fix-rust-sdk-*` /
  `phase8_enable-typescript-sdk-live-tests` all want clean npm
  publishes of each SDK so version skew is testable.

Source: `docs/releases/v3.0.0-verification.md` (Section 4.1 + the
"Remaining open" note at the end).

## What Changes

Publish `@hivehub/vectorizer-sdk@3.0.0` (+ the C# NuGet package +
Python PyPI + Go module + Rust crate, one per SDK) so each language
ecosystem can resolve v3.

TypeScript (primary blocker):

1. `cd sdks/typescript && pnpm publish --access public` with the
   `@hivehub` scope token configured via
   `NPM_TOKEN` in the publish CI or a local `.npmrc`.
2. Confirm the published tarball by
   `npm view @hivehub/vectorizer-sdk@3.0.0` — check `main`, `types`,
   `exports`, `files` match the built `dist/`.
3. Run the `gui/` + downstream `pnpm install` round-trip to verify
   the lockfile resolves against the published SDK.

Companion publishes (independent, same playbook):

- Python: `cd sdks/python && python -m build && twine upload dist/*`.
- Go: tag `sdks/go` at `v3.0.0` on the go.mod path `github.com/hivellm/vectorizer-sdk-go`.
- Rust: `cd sdks/rust && cargo publish --token $CARGO_REGISTRY_TOKEN`
  (crate `vectorizer-sdk` on crates.io).
- C#: `dotnet nuget push bin/Release/Vectorizer.3.0.0.nupkg
  --source nuget.org --api-key $NUGET_API_KEY`.

Pre-publish quality gates per SDK are already captured by the
existing per-SDK follow-up tasks (`phase8_fix-python-sdk-wire-shapes`,
`phase8_fix-rust-sdk-integration-shapes`,
`phase8_fix-csharp-sdk-xunit-refs`,
`phase8_enable-typescript-sdk-live-tests`). This task is the publish
action itself, scheduled AFTER those gates go green.

## Impact

- Affected specs: `sdks/PUBLISHING.md` (publish playbook per SDK);
  `CHANGELOG.md` under `3.0.0 > Published`.
- Affected code: no source changes — pure publish action. Updates
  `sdks/PUBLISHING_STATUS.md` after each SDK lands.
- Breaking change: YES — v3 is a major-version bump with multiple
  documented BREAKING CHANGES blocks already in
  `CHANGELOG.md > 3.0.0`.
- User benefit: `pnpm install @hivehub/vectorizer-sdk@3` (plus
  equivalents) works for every downstream. Unblocks `gui/` lockfile
  refresh + Electron bump + the minimatch transitive alert.
