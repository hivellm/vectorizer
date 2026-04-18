# Proposal: phase4_refactor-tests-examples-after-mcp-api-change

## Why

Two regions of the codebase carry commented-out tests/examples with markers because the MCP API they exercised changed:

- `src/discovery/tests.rs:8` — `Discovery::new` now requires `VectorStore` and `EmbeddingManager`; the integration tests are commented out awaiting a refactor.
- `src/intelligent_search/examples.rs:311, 325, 344` — `MCPToolHandler` and `MCPServerIntegration::new` now require constructor arguments; three example tests are commented out.

Leaving them commented violates the Tier-1 "no deferred tasks" rule and means we have no coverage of the MCP-integrated flows for discovery or intelligent search.

## What Changes

1. Re-enable `src/discovery/tests.rs` integration tests, passing an in-memory `VectorStore` and a lightweight test `EmbeddingManager`.
2. Re-enable the three `intelligent_search/examples.rs` tests, providing the arguments `MCPServerIntegration::new` now expects.
3. Where the test is genuinely obsolete (feature removed), delete it outright rather than patching; no commented-out code should remain.

## Impact

- Affected specs: none.
- Affected code: `src/discovery/tests.rs`, `src/intelligent_search/examples.rs`, plus test-support helpers if a shared in-memory MCP test harness ends up extracted.
- Breaking change: NO — test code only.
- User benefit: restored test coverage on two critical MCP-integrated flows.
