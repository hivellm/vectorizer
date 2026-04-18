# Proposal: phase5_review-candle-bcrypt-bumps

## Why

Three Dependabot PRs are 0.x minor bumps where SemVer treats the change as potentially breaking, even though our tests pass today:

- #248 `candle-core` 0.9.2 → 0.10.2
- #245 `candle-transformers` 0.9.2 → 0.10.2
- #242 `bcrypt` 0.17.1 → 0.19.0 (skipped 0.18)

The tests pass, but the API surface of these crates is large (candle = tensor ops, bcrypt = password hashing) and green tests only prove "what we call, keeps working" — they don't catch silent semantic changes (e.g., different default cost factors on bcrypt).

## What Changes

For each PR:

1. Read the upstream CHANGELOG between the two versions.
2. Confirm no breaking semantic changes affect our usage.
3. For `bcrypt`: confirm cost factor default is unchanged, or adjust our `HashConfig::default()` to pin it.
4. For `candle-*`: run the existing embedding/quantization test suite at release mode; spot-check numerical equivalence on a small fixture (tolerance ≤ 1e-5).
5. Merge once confident.

## Impact

- Affected specs: embedding spec, auth spec
- Affected code: `Cargo.toml`, possibly `src/auth/` (if bcrypt default cost changes), possibly `src/embedding/` (if candle numeric changes)
- Breaking change: NO expected after review
- User benefit: stay current on ML and crypto libraries without surprise semantic drift.
