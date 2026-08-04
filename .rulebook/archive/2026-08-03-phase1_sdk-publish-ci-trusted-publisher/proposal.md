# Proposal: phase1_sdk-publish-ci-trusted-publisher

## Why
SDK publishing today is manual (build.sh + twine/dotnet nuget push with a
long-lived API key from ~/.pypirc or NUGET_API_KEY). That is error-prone and
stores secrets. Move every SDK to release-triggered CI publishing via OIDC
"Trusted Publishing" — no stored registry tokens, short-lived credentials
minted per-run, and provenance attestations where supported.

## What Changes
One GitHub Actions workflow per SDK (or a matrix), triggered on `release:
published` (tag-gated to the SDK's version), each using OIDC Trusted Publishing:

- **Python (PyPI)** — `pypa/gh-action-pypi-publish` with `permissions:
  id-token: write`, no `password`. Registry-side: add a PyPI Trusted Publisher
  for repo hivellm/vectorizer + the workflow file + environment.
- **TypeScript (npm)** — `npm publish --provenance --access public` with
  `id-token: write` and npm Trusted Publishing configured for
  @hivehub/vectorizer-sdk. Requires npm CLI >= 11.5.1 on the runner.
- **C# (NuGet)** — NuGet.org Trusted Publishing: exchange the GitHub OIDC token
  for a short-lived NuGet API key (NuGet/login action) then `dotnet nuget push`
  for Vectorizer.Sdk + Vectorizer.Sdk.Rpc. No stored NUGET_API_KEY.
- **Rust (crates.io)** — crates.io Trusted Publishing:
  `rust-lang/crates-io-auth-action` (OIDC) then `cargo publish` for the SDK
  crate(s) in dependency order.
- **Go** — no registry token model: modules publish by tag. The workflow
  validates sdks/go/version.go matches the release tag and pings the module
  proxy (GOPROXY) so pkg.go.dev indexes the new version. (No "trusted
  publisher" concept applies; documented as such.)

Prerequisite (registry-side, done by a maintainer, documented in the task):
configure the Trusted Publisher / OIDC linkage on PyPI, npmjs.com, nuget.org,
and crates.io pointing at this repo + the exact workflow filenames.

Guardrails: publish jobs run only for the matching SDK's tag/paths, use pinned
action SHAs where practical, and gate on the SDK's tests passing first.

## Impact
- Affected specs: ci/release, sdk-publishing
- Affected code: .github/workflows/sdk-publish-*.yml (new), docs/development/
  sdk-publishing.md (new), removal of the manual token path from
  sdks/*/publish.{sh,ps1} once CI is proven
- Breaking change: NO (adds automated publishing; manual scripts can stay as fallback)
- User benefit: reproducible, tokenless, provenance-backed SDK releases straight from a GitHub release
