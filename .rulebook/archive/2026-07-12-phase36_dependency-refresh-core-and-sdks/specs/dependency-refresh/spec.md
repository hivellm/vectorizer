# Spec: dependency refresh — core + SDKs

## ADDED Requirements

### Requirement: Semver-major bumps MUST land as isolated compat commits

Each semver-major dependency bump (rmcp 2.x, candle 0.11,
aes-gcm 0.11 in this phase) SHALL land as its own commit
containing the bump plus every code change it forces, with the
focused test evidence named in the commit message. Batch merges
that bury a major among patch bumps are FORBIDDEN.

#### Scenario: rmcp major lands reviewably

Given the rmcp 1.7 → 2.1 bump requires handler changes
When the bump is committed
Then the commit contains both the Cargo.toml/lock change and the
  `server/mcp/*` compat fixes
And the commit message names the tests that were run
And no other dependency bump rides in the same commit

### Requirement: The candle crate family MUST move in lockstep

`candle-core`, `candle-nn`, and `candle-transformers` SHALL always
be pinned to the same version in `crates/vectorizer/Cargo.toml`.
A bump of one without the others is a build error waiting to
happen and MUST be rejected in review.

#### Scenario: candle family bump

Given candle-core moves 0.10.2 → 0.11.0
When the bump commit is prepared
Then candle-nn and candle-transformers move to 0.11.0 in the same
  commit
And `cargo check --features candle-models` passes

### Requirement: Workspace refresh MUST pass the full quality gate

After the compatible-range `cargo update`, the workspace SHALL
pass `cargo nextest run --workspace --no-fail-fast` (0 failed)
and `cargo clippy --workspace --all-targets --all-features
-- -D warnings` (0 warnings) before the refresh commit merges.
Any package that has to be rolled back gets a `=`-pinned entry in
Cargo.toml with an inline comment naming the breakage.

#### Scenario: refresh gate

Given `cargo update` has refreshed the lockfile
When the quality gate runs
Then nextest reports 0 failed
And clippy reports 0 warnings
And every rollback pin carries an inline justification comment

### Requirement: Every SDK ecosystem MUST be covered by dependabot

`.github/dependabot.yml` SHALL contain update entries for the
root cargo workspace AND each SDK ecosystem: npm
(sdks/typescript), pip (sdks/python), gomod (sdks/go), nuget
(sdks/csharp) — all at weekly cadence.

#### Scenario: SDK drift surfaces automatically

Given a new msgpack release ships for the TypeScript SDK
When the weekly dependabot run executes
Then a PR against `sdks/typescript/package.json` is opened
  without human intervention

### Requirement: Security override pins MUST survive the refresh

The pnpm `overrides` blocks added by the security phases (js-yaml
`>=4.2.0 <5`, esbuild `>=0.28.1`, vite `8.0.16`, undici, dompurify,
etc.) SHALL remain intact after the SDK refresh. A refresh that
silently drops an override re-opens a closed CVE.

#### Scenario: overrides intact post-refresh

Given the TypeScript SDK refresh has run
When `sdks/typescript/package.json` is diffed against the
  pre-refresh state
Then every `pnpm.overrides` entry present before the refresh is
  still present (values may only move forward, never disappear)
