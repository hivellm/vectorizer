# Proposal: phase1_deps-refresh-3.6.0

## Why
Opening the 3.6.0 cycle with 21 open dependabot PRs against the repo. Apply
the version bumps directly on `release/3.6.0` so the branch starts on current
dependencies, and resolve/close the PRs. Two bumps are NOT safe to apply blind
and are handled explicitly (js-yaml major, openraft alpha pin).

## What Changes
Apply the dependabot bumps by category:

**Cargo (lockfile / manifest):**
- Safe patch/minor (lockfile-only): thiserror 2.0.19, tokio 1.53.0,
  async-trait 0.1.91, anyhow 1.0.104, uuid 1.24.0, futures 0.3.33,
  serde 1.0.229, fastembed 5.17.3.
- lz4_flex 0.13 -> 0.14 (0.x breaking): both crate manifests + verified via the
  compression roundtrip tests (compress_prepend_size / decompress_size_prepended
  API unchanged; 28/28 compression tests pass).
- openraft 0.10.0-alpha.22 -> alpha.30 (#382): DEFERRED. It is deliberately
  pinned with `=` (comment in Cargo.toml) and a bump requires retesting the HA
  Raft path (tests/integration/cluster_ha.rs) with a live cluster. Not a blind
  bump; comment on the PR.

**NuGet (C# SDK csproj):** System.Text.Json 10.0.10, Microsoft.SourceLink.GitHub 10.0.301.

**npm (TS SDK):** @types/node 26.1.2, typescript-eslint 8.65.0, eslint 10.8.0.
- js-yaml 4 -> 5 (#388): REJECTED — 5.x is a breaking major and the pnpm
  security override pins the tree `<5`; js-yaml has no direct use. Close the PR.

**pip (Python SDK):** websockets requirement >=16.1 -> >=16.1.1.

**GitHub Actions:** setup-dotnet 4->6, setup-node 4->7, setup-python 5->7,
setup-go 5->7 (workflow action pins).

## Impact
- Affected specs: dependency-management
- Affected code: crates/*/Cargo.toml, Cargo.lock, sdks/**/{Cargo.toml,
  pyproject.toml,requirements.txt,package.json,*.csproj}, .github/workflows/*.yml
- Breaking change: NO (dependency refresh; lz4_flex format verified compatible)
- User benefit: current, patched dependencies for the 3.6.0 line; CI-noise PRs cleared
