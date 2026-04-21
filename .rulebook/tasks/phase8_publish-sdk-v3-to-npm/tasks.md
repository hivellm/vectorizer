## 1. Implementation

- [ ] 1.1 Gate: confirm `phase8_fix-python-sdk-wire-shapes`,
  `phase8_fix-rust-sdk-integration-shapes`,
  `phase8_fix-csharp-sdk-xunit-refs`, and
  `phase8_enable-typescript-sdk-live-tests` are all archived (go/no-go).
- [ ] 1.2 TypeScript publish:
  `cd sdks/typescript && pnpm build && pnpm test && pnpm pack` to
  verify the tarball, then `pnpm publish --access public`. Capture
  the published version + the tarball URL.
- [ ] 1.3 Python publish: `cd sdks/python && python -m build &&
  twine upload dist/vectorizer-3.0.0*`.
- [ ] 1.4 Go publish: tag `sdks/go` at `v3.0.0`
  (`git tag sdks/go/v3.0.0 && git push origin sdks/go/v3.0.0`).
  Verify via
  `go install github.com/hivellm/vectorizer-sdk-go@v3.0.0`.
- [ ] 1.5 Rust publish: `cd sdks/rust && cargo publish --dry-run`
  first, then `cargo publish`. Crate name `vectorizer-sdk` on
  crates.io.
- [ ] 1.6 C# publish: `cd sdks/csharp && dotnet pack -c Release &&
  dotnet nuget push bin/Release/Vectorizer.3.0.0.nupkg --source
  nuget.org --api-key $NUGET_API_KEY`.
- [ ] 1.7 Round-trip: `cd gui && rm -rf node_modules pnpm-lock.yaml
  && pnpm install` — confirm the new lockfile resolves against the
  published `@hivehub/vectorizer-sdk@3.0.0`.
- [ ] 1.8 Update `sdks/PUBLISHING_STATUS.md` with the published
  version + timestamp for each SDK.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (`sdks/PUBLISHING.md` one-command runbook; `CHANGELOG.md` under
  `3.0.0 > Published`).
- [ ] 2.2 Write tests covering the new behavior (post-publish
  smoke-install test in CI:
  `.github/workflows/sdk-publish-smoke.yml` that on the published
  tag runs `pnpm install @hivehub/vectorizer-sdk@3 && node -e
  'require("@hivehub/vectorizer-sdk")'` + equivalents per language
  to catch a broken tarball before any downstream hits it).
- [ ] 2.3 Run tests and confirm they pass (the post-publish CI
  workflow must go green on the tagged commit; `cd gui && pnpm
  install` round-trip must also succeed end-to-end).
