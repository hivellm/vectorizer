## 1. Prerequisites (registry-side, maintainer)
- [x] 1.1 Document + configure Trusted Publisher / OIDC on PyPI, npmjs.com, nuget.org, crates.io pointing at hivellm/vectorizer + the exact workflow filenames + environments (documented in docs/development/sdk-publishing.md; the registry-account config itself is a maintainer step)

## 2. Python (PyPI)
- [x] 2.1 sdk-publish-python.yml: build + pypa/gh-action-pypi-publish via OIDC (id-token: write), tag-gated, tests-gated

## 3. TypeScript (npm)
- [x] 3.1 sdk-publish-typescript.yml: npm publish --provenance via OIDC (npm >= 11.5.1), tag-gated, build+lint-gated

## 4. C# (NuGet)
- [x] 4.1 sdk-publish-csharp.yml: NuGet OIDC login -> short-lived key -> dotnet nuget push for Vectorizer.Sdk + Vectorizer.Sdk.Rpc

## 5. Rust (crates.io)
- [x] 5.1 sdk-publish-rust.yml: crates-io-auth-action (OIDC) -> cargo publish for the SDK crate(s) in dependency order

## 6. Go (tag-based)
- [x] 6.1 sdk-publish-go.yml: validate sdks/go/version.go matches the tag + ping GOPROXY so pkg.go.dev indexes it (no token) — adapted for the vectorizer-go submodule

## 7. Verify
- [x] 7.1 Each workflow exposes a workflow_dispatch dry-run path (TestPyPI / npm --dry-run / NuGet pack-only / crates.io --dry-run); the live OIDC round-trip runs once the maintainer completes the registry-side linkage in 1.1

## 8. Tail (docs + tests — check or waive with tailWaiver)
- [x] 8.1 Update or create documentation covering the implementation
- [ ] 8.2 Write tests covering the new behavior
- [ ] 8.3 Run tests and confirm they pass

<!-- tail-waiver: CI workflow definitions have no unit-testable behavior; verification is YAML-schema validation (all 5 parse, jobs present) plus each workflow's built-in workflow_dispatch dry-run path (TestPyPI / npm --dry-run / NuGet pack-only / cargo --dry-run). The live OIDC round-trip depends on the maintainer completing the registry-side Trusted Publisher linkage (item 1.1), which requires registry-account admin access the agent does not have. Docs delivered in docs/development/sdk-publishing.md. -->
