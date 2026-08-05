# Proposal: phase3_release-3-6-1

Ships the fixes from [phase1](../phase1_collections-list-hides-lazy-load-progress/proposal.md)
([#391](https://github.com/hivellm/vectorizer/issues/391)) and
[phase2](../phase2_rest-client-accepts-rpc-scheme/proposal.md)
([#392](https://github.com/hivellm/vectorizer/issues/392)) as 3.6.1.

Runs **last**: cutting the version before both fixes land would publish a
3.6.1 that does not contain them.

## Why

Both issues were found against published 3.6.0 artifacts — the server image
and the Rust SDK — so neither fix reaches a user until a new version is
published. Every publishing surface has to move together: a release where the
crate says 3.6.1 and PyPI still says 3.6.0 is the exact drift that made the
3.6.0 cut painful.

## What Changes

Version bump to 3.6.1 across every artifact that carries one:

| Surface | File |
|---|---|
| Core crates | `crates/vectorizer{,-server,-core,-grpc,-cli}/Cargo.toml` |
| Rust SDK | `sdks/rust/Cargo.toml` |
| TypeScript SDK | `sdks/typescript/package.json` |
| Python SDK | `sdks/python/pyproject.toml` + `sdks/python/__init__.py` (`__version__`) |
| C# SDK | `sdks/csharp/Vectorizer.csproj`, `sdks/csharp/src/Vectorizer.Rpc/Vectorizer.Rpc.csproj` |
| Go SDK | git tag on the `sdks/go` submodule (Go publishes by tag, no manifest) |
| Lockfile | `Cargo.lock` (regenerated, not hand-edited) |

Three `3.6.0` matches in the tree are **not** version carriers and must stay
untouched: `dashboard/package.json` (`tailwind-merge: ^3.6.0`, an unrelated
dependency) and the prose references in `sdk-python-test.yml` /
`sdk-publish-typescript.yml`, which describe what happened during the v3.6.0
cut. `gui/package.json` pins `@hivehub/vectorizer-sdk: ^3.6.0`, whose caret
already admits 3.6.1 — bump it for clarity, not necessity.

Then publish. Note that publishing is **not** uniformly automated:

- Tag `v3.6.1` drives the SDK publish workflows (crates.io, npm, PyPI, NuGet
  via OIDC Trusted Publishing).
- Docker is **manual** — `release-artifacts.yml` no longer contains a single
  occurrence of "docker", so the tag publishes no image. Both variants must be
  cut locally with `build-push.ps1` (`-Tag 3.6.1` and `-Tag 3.6.1 -Fastembed`),
  per `docs/development/docker-builds.md`.

## Impact

- Affected specs: none.
- Affected code: manifests only — no behavior change in this task.
- Breaking change: NO — patch release.
- User benefit: the two fixes become installable. Specifically, an operator
  upgrading no longer reads normal warm-up as data loss (#391), and an SDK
  user who crosses the transports gets told so at construction (#392).
