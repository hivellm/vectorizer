# Proposal: phase4_publish-3-6-1

Split out of [phase3_release-3-6-1](../phase3_release-3-6-1/proposal.md).

## Why

Everything 3.6.1 needs in the tree is done: twelve version carriers bumped,
the changelog written (including the `## [3.6.0]` section that release never
got), install snippets corrected, the quality gate green, and both Docker
images published and boot-tested.

What remains cannot be done from this working tree at all. It waits on two
external events:

1. **The commits reaching GitHub.** This shell has no SSH key, so it cannot
   push. Tagging `v3.6.1` before the commits land would publish a release that
   does not contain the fixes it claims.
2. **The registries serving 3.6.1.** `gui/package.json` cannot move to
   `^3.6.1` until npm has the package — `pnpm install` would fail to resolve,
   and committing a `package.json` whose lockfile disagrees breaks
   `--frozen-lockfile` in CI.

Keeping these inside phase3 would leave a finished piece of work permanently
open, and would blur what is actually blocked with what merely was not done.
The 3.6.0 cut is the cautionary example: it reached crates.io, NuGet, npm and
PyPI at different times with no single place recording which had landed.

## What Changes

No code. This task is the checklist for the publish itself and for verifying
it, plus the one tree change that depends on the publish having happened
(`gui/package.json`).

The pieces the tag drives — crates.io, npm, PyPI, NuGet via the OIDC publish
workflows — and the two the tag does **not**:

- **Docker.** Already published from phase3 (`3.6.1`, `3.6.1-fastembed`,
  `latest`), because `release-artifacts.yml` no longer contains a Docker job
  and never will publish them on its own. Listed here only so the verification
  is in one place.
- **The Go SDK.** `sdks/go` is a submodule with its own remote, and Go
  resolves a module version from a tag in *that* repository. The superproject
  tag does nothing for it.

## Impact

- Affected specs: none.
- Affected code: `gui/package.json` (+ its lockfile) only.
- Breaking change: NO.
- User benefit: the two fixes become installable from every registry rather
  than existing only in this repository — and the five SDKs land in lockstep,
  which the 3.6.0 cut did not manage.
