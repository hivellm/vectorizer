# Proposal: phase8_fix-rust-sdk-integration-shapes

## Why

Probe 4.4 of `phase8_release-v3-runtime-verification` ran
`cargo test -p vectorizer-sdk --features rpc` against the live
`release/v3.0.0` binary. Result: 72 unit tests pass across sub-suites,
**3 failures in `sdks/rust/tests/integration_tests.rs`**:

- `test_list_collections` (line 54)
- `test_get_collection_info` (line 274)
- `test_serialization` (line 384)

All three panic on wire-shape assertions. Same root cause as probe
4.2 (Python) — the F1-F5 REST rewrites changed response shapes that
the SDK tests were pinned against:

- `list_collections` now returns `{collections: [...],
  total_collections: N}` (was `{collections: [...], total: N}` in
  some earlier version).
- `get_collection_info` response shape gained fields (normalization
  block, quantization block).
- `test_serialization` likely rejects the extra fields that the
  server now emits because the SDK deserializer was marked
  `#[serde(deny_unknown_fields)]` on a model that should tolerate
  them.

Source: `docs/releases/v3.0.0-verification.md` section 4.

## What Changes

Align the Rust SDK response models with the server's current emitters
and drop `deny_unknown_fields` on response types that should tolerate
future additions.

1. Rerun each of the 3 failing tests with the live server and capture
   the exact panic message + mismatched field.
2. Update `sdks/rust/src/{models,client}.rs` response models to
   accept the current field set.
3. Drop `#[serde(deny_unknown_fields)]` on response models in favor
   of `#[serde(default)]` for optional fields — request models keep
   the strict posture.
4. Re-run the suite. Acceptance: 0 failures; both HTTP and RPC
   transports exercise their respective live paths.

## Impact

- Affected specs: `docs/users/sdks/RUST.md` if documented;
  `sdks/rust/CHANGELOG.md` under `3.0.0 > Fixed`.
- Affected code:
  - `sdks/rust/src/models.rs` (response shapes)
  - `sdks/rust/src/client.rs` (parsers)
  - `sdks/rust/tests/integration_tests.rs` (the 3 failing cases)
- Breaking change: MAYBE (dropping `deny_unknown_fields` is a
  relaxation, not a break; new fields in the response shape may
  deserve an SDK minor-bump but no runtime break).
- User benefit: Rust SDK 3.0.0 works against the v3 server on both
  HTTP and RPC transports. Unblocks probe 4.4.
