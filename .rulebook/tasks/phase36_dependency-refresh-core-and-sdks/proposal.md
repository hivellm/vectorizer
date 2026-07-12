# Proposal: phase36_dependency-refresh-core-and-sdks

Source: v3.5.0 release scope — second structural cleanup after
phase35 (image CVEs). Open dependabot queue at task-creation time:
https://github.com/hivellm/vectorizer/pulls (10 PRs, #336–#345).

## Why

Dependency drift is the second chronic problem this release
addresses. Every batch cycle repeats the same manual dance
(retarget dependabot PRs → merge cascade → fix breakage → bump
lockfiles), and between cycles the SDKs drift out of sync with the
core. Concretely, right now:

1. **10 open dependabot PRs against `main`** (#336–#345), three of
   which are semver-major or high-risk and will need code work,
   not just a lockfile merge:
   - `rmcp` 1.7.0 → **2.1.0** (#342) — MCP SDK major; server
     handlers + tool registration surface may change.
   - `candle-core` / `candle-transformers` 0.10.2 → **0.11.0**
     (#341, #337) — must bump together with `candle-nn` (same
     family, one lockstep pin).
   - `aes-gcm` 0.10.3 → **0.11.0** (#343) — crypto API surface;
     auth persistence + payload encryption call sites.
2. **SDK manifests only get versions bumps, not dependency
   refreshes.** The TS/Python/Go/C# SDKs carry their own dep trees
   (msgpack, reqwest-equivalents, test frameworks) that no
   dependabot config covers end-to-end; several are pinned to
   versions from the 3.0.x era.
3. **Cargo minor-drift**: `cargo update --verbose` reports 120+
   "unchanged behind latest" entries; the compatible-range refresh
   never runs because nothing forces it.

Doing this as one phase36 task (instead of ad-hoc batches) gives
the release a single reviewable dependency story and leaves the
tree at a known-current baseline before 3.5.0 ships.

## What Changes

1. **Land the dependabot queue** (#336–#345) onto `release/3.5.0`
   via the established retarget-and-cascade flow. Low-risk patch/
   minor bumps merge first (uuid, memmap2, bcrypt, xxhash-rust,
   transmutation, tracing-opentelemetry); the three majors (rmcp
   2.x, candle 0.11, aes-gcm 0.11) each get their own compat
   commit with code fixes and a focused test run.
2. **Compatible-range refresh of the core workspace**:
   `cargo update` across the workspace to pull every semver-
   compatible minor/patch that dependabot doesn't PR individually.
   Gate: full `cargo nextest run --workspace` + clippy `-D
   warnings` afterward.
3. **SDK dependency refresh** (each SDK gets its own commit):
   - **Rust SDK** — rides the workspace refresh (same lockfile).
   - **TypeScript** — `pnpm update --latest --interactive=false`
     within semver ranges + explicit review of `@msgpack/msgpack`,
     vitest, eslint majors; keep the security overrides from
     phase-security intact.
   - **Python** — refresh `pyproject.toml` optional/dev pins
     (pytest, httpx/requests line, msgpack) to current stable.
   - **Go** — `go get -u ./... && go mod tidy` within the module's
     major; verify with `go vet` + `go test -short`.
   - **C#** — bump NuGet refs (`MessagePack` already at 2.5.301
     from the security pass; refresh `Microsoft.Extensions.*` 8.x
     line to latest patch) and run `dotnet test`.
4. **Dependabot config hygiene** — extend `.github/dependabot.yml`
   so the SDK directories (sdks/typescript, sdks/python, sdks/go,
   sdks/csharp) are all covered with the same weekly cadence as
   the root Cargo ecosystem, preventing the SDK-drift class of
   problem from recurring.

## Impact

- Affected specs: `specs/phase36_dependency-refresh-core-and-sdks/`
- Affected code:
  - `Cargo.toml` + `Cargo.lock` (workspace) — rmcp/candle/aes-gcm
    majors may touch `crates/vectorizer-server/src/server/mcp/*`,
    `crates/vectorizer/src/embedding/*` (candle providers),
    `crates/vectorizer/src/auth/persistence.rs` +
    `src/security/payload_encryption.rs` (aes-gcm).
  - `sdks/typescript/{package.json,pnpm-lock.yaml}`
  - `sdks/python/pyproject.toml`
  - `sdks/go/go.mod` + `go.sum`
  - `sdks/csharp/**/*.csproj`
  - `.github/dependabot.yml`
- Breaking change: NO for consumers — public REST/RPC/MCP wire
  contracts unchanged. The rmcp 2.x major is internal to the
  server; MCP protocol version stays `2025-03-26` unless the bump
  forces otherwise (decision + fallback recorded in design.md if
  it does).
- User benefit:
  - 3.5.0 ships on a current, uniformly-refreshed dep baseline —
    fewer latent CVEs, fewer "requires rustc/tooling X" surprises.
  - SDK dep trees stop aging silently; dependabot watches all of
    them going forward.
  - The three risky majors land with dedicated compat commits and
    test evidence instead of being buried in a batch merge.
