# Proposal: phase4_reenable-go-sdk-ci

## Why

`.github/workflows/sdk-go-test.yml.disabled` — the Go SDK CI workflow carries a `.disabled` suffix, meaning GitHub Actions never triggers it. Consequence:

- The Go SDK published in `sdks/go/` (if present) is **not validated** by any automated process.
- Users of the Go SDK might file bugs against behavior the project no longer supports.
- It is an orphan artifact that can silently break with any REST/gRPC change.

The project ships five SDKs (Python, JS, TS, Rust, C#) all actively tested — Go is the lone exception.

## What Changes

Decide between two options, record the decision via `rulebook_decision_create`:

**Option A — Re-enable and fix.** Rename the workflow file back to `sdk-go-test.yml`, fix any outstanding test failures, add it to required checks.

**Option B — Officially deprecate.** Delete the workflow, delete `sdks/go/`, update README/docs to remove the Go SDK from the supported list, add a migration note for existing Go users.

## Impact

- Affected specs: SDK spec, documentation
- Affected code: `.github/workflows/sdk-go-test.yml.disabled`, `sdks/go/`, README
- Breaking change: Option B is breaking for Go consumers
- User benefit: either a tested Go SDK or clarity that it's no longer supported — no more silent rot.
