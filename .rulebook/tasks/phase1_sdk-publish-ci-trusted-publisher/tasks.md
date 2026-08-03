## 1. Prerequisites (registry-side, maintainer)
- [ ] 1.1 Document + configure Trusted Publisher / OIDC on PyPI, npmjs.com, nuget.org, crates.io pointing at hivellm/vectorizer + the exact workflow filenames + environments

## 2. Python (PyPI)
- [ ] 2.1 sdk-publish-python.yml: build + pypa/gh-action-pypi-publish via OIDC (id-token: write), tag-gated, tests-gated

## 3. TypeScript (npm)
- [ ] 3.1 sdk-publish-typescript.yml: npm publish --provenance via OIDC (npm >= 11.5.1), tag-gated, build+lint-gated

## 4. C# (NuGet)
- [ ] 4.1 sdk-publish-csharp.yml: NuGet OIDC login -> short-lived key -> dotnet nuget push for Vectorizer.Sdk + Vectorizer.Sdk.Rpc

## 5. Rust (crates.io)
- [ ] 5.1 sdk-publish-rust.yml: crates-io-auth-action (OIDC) -> cargo publish for the SDK crate(s) in dependency order

## 6. Go (tag-based)
- [ ] 6.1 sdk-publish-go.yml: validate sdks/go/version.go matches the tag + ping GOPROXY so pkg.go.dev indexes it (no token)

## 7. Verify
- [ ] 7.1 Dry-run / test-publish each (TestPyPI, npm --dry-run, NuGet int, crates.io dry-run) and confirm OIDC auth works end-to-end

## 8. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 8.1 Update or create documentation covering the implementation
- [ ] 8.2 Write tests covering the new behavior
- [ ] 8.3 Run tests and confirm they pass
