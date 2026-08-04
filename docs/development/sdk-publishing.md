# SDK publishing (OIDC Trusted Publishing)

All five SDKs publish from GitHub Actions on `release: published`, using OIDC
**Trusted Publishing** wherever the registry supports it — no long-lived
registry tokens are stored in the repo. Each run mints a short-lived,
workflow-scoped credential, and (npm, PyPI) attaches provenance.

| SDK | Package | Registry | Workflow | Auth model |
|-----|---------|----------|----------|------------|
| Python | `vectorizer_sdk` | PyPI | `sdk-publish-python.yml` | OIDC Trusted Publisher |
| TypeScript | `@hivehub/vectorizer-sdk` | npm | `sdk-publish-typescript.yml` | OIDC Trusted Publishing + provenance |
| C# | `Vectorizer.Sdk`, `Vectorizer.Sdk.Rpc` | NuGet.org | `sdk-publish-csharp.yml` | OIDC login → short-lived key |
| Rust | `vectorizer-sdk` | crates.io | `sdk-publish-rust.yml` | OIDC Trusted Publisher |
| Go | `github.com/hivellm/vectorizer-sdk-go` | proxy.golang.org | `sdk-publish-go.yml` | git tag (no token model) |

The workflows are version-gated: on a `vX.Y.Z` release each verifies the SDK's
manifest version equals `X.Y.Z` before publishing, and gates on the SDK's
tests/build passing first. Because every SDK is versioned in lockstep with the
repo, one `vX.Y.Z` GitHub release publishes them all at `X.Y.Z`.

## One-time registry-side setup (maintainer)

OIDC Trusted Publishing must be linked on each registry **before** the first
release-triggered run. This is done once per package by an account owner and
cannot be automated from CI.

### PyPI (Python)
1. Sign in to <https://pypi.org> → the `vectorizer-sdk` project → *Publishing*.
2. Add a **GitHub Actions** trusted publisher:
   - Owner: `hivellm`, Repository: `vectorizer`
   - Workflow filename: `sdk-publish-python.yml`
   - Environment: leave blank (or set one and add `environment:` to the job).
3. For dry-runs, add the same publisher on <https://test.pypi.org>.

### npm (TypeScript)
1. On <https://www.npmjs.com> → `@hivehub/vectorizer-sdk` → *Settings* →
   *Trusted Publishing*.
2. Add a GitHub Actions publisher: repo `hivellm/vectorizer`, workflow
   `sdk-publish-typescript.yml`.
3. The runner installs `npm@latest` (Trusted Publishing needs npm ≥ 11.5.1).

### NuGet.org (C#)
1. On <https://www.nuget.org> → account → *Trusted Publishing* → create a policy
   for each package (`Vectorizer.Sdk`, `Vectorizer.Sdk.Rpc`):
   - Repository owner `hivellm`, repository `vectorizer`,
     workflow `sdk-publish-csharp.yml`.
2. Set the repo variable **`NUGET_USER`** (Settings → Secrets and variables →
   Actions → Variables) to the nuget.org account that owns the policy — the
   `NuGet/login` action needs it.

### crates.io (Rust)
1. On <https://crates.io> → crate `vectorizer-sdk` → *Settings* →
   *Trusted Publishing*.
2. Add a GitHub publisher: repo `hivellm/vectorizer`, workflow
   `sdk-publish-rust.yml`.
3. `vectorizer-sdk` is the only crate this repo publishes — its RPC transport
   is the already-published `thunder-rpc`, so nothing has to be released
   ahead of it.

### Go (module proxy)
Go has no token model — publishing is a git tag. The Go SDK lives in its own
repo, `github.com/hivellm/vectorizer-go`, vendored here as the `sdks/go`
submodule. To release the Go SDK, push a matching `vX.Y.Z` tag **on the
vectorizer-go repo**; `sdk-publish-go.yml` then warms `proxy.golang.org` so
pkg.go.dev indexes it.

> **Known discrepancy:** the module path in `sdks/go/go.mod` is
> `github.com/hivellm/vectorizer-sdk-go`, but the hosting repo is
> `hivellm/vectorizer-go`. `go get` resolves the module path to a repo URL, so
> these must match (rename the repo to `vectorizer-sdk-go`, or set the module
> path / a vanity import to the real repo). Reconcile this in the vectorizer-go
> repo before the first Go release.

## Verifying OIDC end-to-end (dry-run)

After the registry-side setup, run each workflow via *Actions → Run workflow*
(`workflow_dispatch`) to prove the OIDC exchange without shipping a real
version:

- **Python** — `dry_run: true` uploads to **TestPyPI**.
- **TypeScript** — `dry_run: true` runs `npm publish --provenance --dry-run`.
- **C#** — `dry_run: true` packs the `.nupkg`s without pushing.
- **Rust** — `dry_run: true` runs `cargo publish --dry-run` for both crates.
- **Go** — `workflow_dispatch` validates the submodule version only.

## Releasing

1. Bump every SDK manifest to the new version (already lockstep with the repo).
2. Tag the vectorizer-go repo with the same `vX.Y.Z` (for the Go SDK).
3. Publish a GitHub release `vX.Y.Z`. The five workflows fire, each verifying
   its version against the tag, gating on tests, then publishing.

The pre-existing manual scripts (`sdks/*/build.sh`, `twine`,
`dotnet nuget push`) remain usable as a fallback but are no longer the primary
path.
