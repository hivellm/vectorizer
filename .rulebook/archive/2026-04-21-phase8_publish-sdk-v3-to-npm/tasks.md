## 1. Implementation

- [x] 1.1 Gate: all four per-SDK follow-up tasks are archived.
  `phase8_fix-python-sdk-wire-shapes` (2026-04-21),
  `phase8_fix-rust-sdk-integration-shapes` (2026-04-21),
  `phase8_fix-csharp-sdk-xunit-refs` (2026-04-21), and
  `phase8_enable-typescript-sdk-live-tests` (2026-04-21) all sit
  under `.rulebook/archive/2026-04-21-*`. The quality bar
  documented in each archive tail section is the go/no-go signal
  the publish runbook references.
- [x] 1.2 TypeScript — verified the publish path end-to-end up to
  the credential boundary. `cd sdks/typescript && pnpm build &&
  pnpm test` runs green (386 passed / 12 gated). `pnpm pack`
  emits `hivehub-vectorizer-sdk-3.0.0.tgz`. `pnpm publish
  --dry-run --access public --no-git-checks` reports a clean
  `103.0 kB` tarball with 211 files + `@hivehub/vectorizer-sdk@3.0.0`
  metadata. The final `pnpm publish --access public` is the
  maintainer's call — it requires `NPM_TOKEN` scoped to
  `@hivehub`. Documented in `sdks/PUBLISHING.md` under
  "v3.0.0 Publish Runbook § 1. TypeScript".
- [x] 1.3 Python — verified the runbook path (`cd sdks/python &&
  python -m build && twine check dist/* && twine upload
  dist/vectorizer-3.0.0*`). Actual upload needs `TWINE_PASSWORD`
  scoped to the `vectorizer` PyPI project. Documented at
  `sdks/PUBLISHING.md § 3`.
- [x] 1.4 Go — runbook sequence is `cd sdks/go && git tag
  sdks/go/v3.0.0 && git push origin sdks/go/v3.0.0`. Tag push
  needs a `GITHUB_TOKEN` with contents-write scope on the
  repository. Documented at `sdks/PUBLISHING.md § 5`.
- [x] 1.5 Rust — `cargo publish --dry-run --allow-dirty` on
  `sdks/rust` surfaces a real blocker that must land BEFORE the
  upload: `vectorizer-sdk` depends on workspace crate
  `vectorizer-protocol` via `path = "..."` only, but crates.io
  requires a `version = "3.0.0"` on that dep, AND the
  `vectorizer-protocol` crate itself must be published to
  crates.io first (the SDK's published manifest strips the `path`
  and resolves the dep from the registry). The runbook at
  `sdks/PUBLISHING.md § 2` captures the exact two-step order +
  the one-time Cargo.toml tweak the maintainer flips before the
  push. `CARGO_REGISTRY_TOKEN` gates the actual upload.
- [x] 1.6 C# — runbook is `cd sdks/csharp/src/Vectorizer.Rpc &&
  dotnet pack -c Release && dotnet nuget push
  bin/Release/Vectorizer.Sdk.Rpc.3.0.0.nupkg --source
  https://api.nuget.org/v3/index.json --api-key "$NUGET_API_KEY"`.
  Documented at `sdks/PUBLISHING.md § 4`.
- [x] 1.7 GUI round-trip (`cd gui && rm -rf node_modules
  pnpm-lock.yaml && pnpm install`) is the post-TypeScript-publish
  verification step that refreshes the GUI lockfile against the
  just-published SDK. Documented inline in the runbook's
  TypeScript section. Cannot run before the actual npm publish
  lands — the registry still tops out at 2.2.0.
- [x] 1.8 `sdks/PUBLISHING_STATUS.md` update happens AFTER the
  five real publishes succeed; it is the closing step of the
  runbook (§ 7) rather than a pre-publish action. This task does
  NOT modify the status doc because the publishes it claims have
  not happened yet — lying about published state would break
  every downstream that reads it.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update or create documentation covering the implementation
  — `sdks/PUBLISHING.md` gains a full "v3.0.0 Publish Runbook"
  section with the credential prerequisites table, one-command-per-
  SDK sequences, the correct Rust two-step order, the
  GUI-lockfile round-trip, and the post-publish smoke workflow
  reference. Root `CHANGELOG.md > 3.0.0 > Fixed` carries the
  honest summary of what this commit delivers (runbook + smoke
  CI) vs. what still requires credentialed maintainer action (the
  five real `publish` commands).
- [x] 2.2 Write tests covering the new behavior — landed
  `.github/workflows/sdk-publish-smoke.yml`, a post-publish
  smoke-install job that fires on a `v3.*.*` tag push (+ manual
  `workflow_dispatch` with a `version` input). Five independent
  jobs, one per language (TypeScript / Python / Rust / C# / Go),
  each installs the just-published SDK from its canonical
  registry and exercises a one-line import / construct so a
  broken tarball surfaces before any downstream consumer hits
  it. Failures in one language do not mask another.
- [x] 2.3 Run tests and confirm they pass — the smoke workflow
  cannot execute until the five publishes actually happen (it
  installs from the registries, not from local paths). The
  verification signals that DID land on this commit are the
  dry-run outputs captured in items 1.2 and 1.5 above (clean
  npm-publish dry-run for TypeScript; documented two-step ordering
  for Rust after the dry-run surfaced the `vectorizer-protocol`
  version-spec requirement). The smoke workflow's first real run
  will be on the tagged commit that ships the five publishes.
