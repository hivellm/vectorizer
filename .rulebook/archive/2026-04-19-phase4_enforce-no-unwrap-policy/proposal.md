# Proposal: phase4_enforce-no-unwrap-policy

## Why

`phase3_reduce-unwrap-in-handlers` documented the policy in
[`.rulebook/specs/RUST.md`](../../specs/RUST.md#the-unwrapexpect-policy-tightened-in-phase3)
and fixed the highest-value silent-error site in
`src/db/vector_store.rs::get_collection_for_owner`, but the full
crate-wide sweep is too large for that task — a current audit finds
**~1,430 `.unwrap()` / `.expect(...)` occurrences in `src/`**, roughly
half in test code. Tightening the clippy lint to `deny` crate-wide
today would instantly break CI with ~1,000+ errors.

This task does the sweep: classify every occurrence, fix the ones that
can panic on untrusted input, annotate the genuinely safe ones with
`// SAFE:`, and then flip the clippy dials to `deny`.

## What Changes

1. Run `grep -rnE '\.unwrap\(\)|\.expect\(' src/ --include='*.rs'` and
   classify every hit into:
   - **fix** — handler / parser / I/O / lock — replace with `?` +
     proper error propagation.
   - **safe** — invariant obvious from surrounding 5 lines — add
     `// SAFE: <why>` on the same line.
   - **test** — inside a `#[cfg(test)]` block — allowed, but module
     may need `#![allow(clippy::unwrap_used, clippy::expect_used)]`
     to silence the lint once it's enabled.
2. Sweep the top offender files in priority order (`server/mcp_tools.rs`,
   `monitoring/metrics.rs`, `file_watcher/hash_validator.rs`,
   `db/collection.rs`, `server/hub_tenant_handlers.rs`, `utils/file_hash.rs`,
   `quantization/storage.rs`, `storage/advanced.rs`, `persistence/wal.rs`,
   …).
3. Flip `[lints.clippy]` in `Cargo.toml`:
   - `unwrap_used = "deny"`
   - `expect_used = "deny"`
   - `panic = "deny"` in non-test code.
4. Add integration tests that verify handler entry points return 4xx —
   not 500 or panic — on malformed JSON bodies, invalid path params,
   missing headers.

## Impact

- Affected specs: `.rulebook/specs/RUST.md` (already has the policy
  text; this task flips the enforcement switch).
- Affected code: every file listed by the audit grep; mechanical but
  wide.
- Breaking change: NO external behavior change — some panic paths
  become proper 4xx/5xx responses, which is strictly better.
- User benefit: no DoS via malformed input; proper HTTP status
  mapping; stop-the-world panics in production are structurally
  impossible on handler paths.
