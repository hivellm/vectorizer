# Proposal: phase5_refresh-all-dependencies

## Why

Dependabot PRs were closed without merging, so every direct dependency
across every first-party manifest is behind the latest published
version. Stale deps accumulate CVEs, keep us off the bug-fix stream
for the libraries we build on, and drift us further from what the
ecosystem expects the stack to be running. The longer we wait, the
harder each individual upgrade becomes — batching a refresh now,
while we're mid-release on `v3.0.0` and can run the full test suite
against the result, is cheaper than chasing each library one-at-a-time
later.

## What Changes

Refresh direct dependencies across every first-party manifest to the
newest compatible version, split per ecosystem so any regression is
surgically revertable:

- **Rust workspace** (`Cargo.toml`, `crates/*/Cargo.toml`,
  `sdks/rust/Cargo.toml`) — `cargo update` for latest-in-range, then
  manual version bumps in `Cargo.toml` for any direct dep that has a
  newer compatible minor/patch or a safe major. Reject a bump only
  when the upstream changelog documents a breaking change we can't
  absorb in-scope.
- **TypeScript SDK** (`sdks/typescript/package.json`) — bump deps +
  devDeps via pnpm, rerun `pnpm test` + `pnpm build`.
- **Python SDK** (`sdks/python/pyproject.toml`) — update pins,
  rerun `pytest`.
- **Go SDK** (`sdks/go/go.mod`, `sdks/go/examples/go.mod`) — `go get
  -u ./...` + `go mod tidy`, rerun `go test ./...`.
- **C# SDK** (`sdks/csharp/*.csproj`) — bump `PackageReference`
  versions, rerun `dotnet test`.
- **GUI** (`gui/package.json`) — bump deps + devDeps, rerun the
  GUI test suite if any.
- **Dashboard** (`dashboard/package.json`) — same treatment.

Each ecosystem is its own commit. Each commit leaves the gate green
for that ecosystem before the next one starts. Major bumps that
require code changes are either absorbed in the same commit or
deferred to a follow-up task (never left as half-done).

## Impact

- Affected specs: none (dependency metadata only).
- Affected code: every first-party manifest; possibly small call-site
  edits to absorb major-version API changes.
- Breaking change: NO to downstream consumers (our own public API
  surface is unchanged). Some transitive bumps are MSRV-sensitive;
  we stay on the project's pinned toolchain.
- User benefit: closes known CVEs, picks up upstream bug fixes, and
  flattens the cost of the next refresh.
