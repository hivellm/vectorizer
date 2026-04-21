# Proposal: phase7_rmcp-1.x-migration

## Why

`rmcp` (the Rust SDK for the Model Context Protocol) went 0.10 → 1.x
upstream. We still ship against 0.10 because a straight bump surfaces
71 compile errors across 10 server-side handler files — the 1.x cut
reshaped the core data model (several structs are now
`#[non_exhaustive]`, `Implementation` gained a required `description`
field, `Tool` gained `execution`, `ListToolsResult` /
`ListResourcesResult` / `CallToolRequestParams` gained `meta`, …).

This is a real application-code migration, not a version bump. It
was carved out of `phase6_major-dep-migrations` §5.2 so the handler
rewrite can land as its own diff and be reverted surgically if the
MCP traffic regresses in production.

## What Changes

- Bump `rmcp = "0.10"` → `rmcp = "1.5"` in
  `crates/vectorizer-core/Cargo.toml`,
  `crates/vectorizer/Cargo.toml`, and
  `crates/vectorizer-server/Cargo.toml`.
- Update struct construction sites in these 10 files so every
  currently-failing call compiles:
  - `crates/vectorizer-server/src/server/mcp/handlers.rs`
  - `crates/vectorizer-server/src/server/mcp/tools.rs`
  - `crates/vectorizer-server/src/server/core/mcp_service.rs`
  - `crates/vectorizer-server/src/server/core/routing.rs`
  - `crates/vectorizer-server/src/server/graph_handlers.rs`
  - `crates/vectorizer-server/src/server/discovery_handlers.rs`
  - `crates/vectorizer-server/src/server/files/operations.rs`
  - `crates/vectorizer-server/src/umicp/handlers.rs`
  - `crates/vectorizer/tests/api/mcp/integration.rs`
  - `crates/vectorizer/tests/api/mcp/graph_integration.rs`
- Fill in the new required fields with meaningful values:
  - `Implementation { description: Some("...") }` — reuse the server
    version + a short human-readable string.
  - `Tool { execution: ... }` — investigate what v1 wants here (likely
    an execution policy enum); if the policy is request-scoped, move
    it into the handler dispatch.
  - `ListToolsResult { meta: None }`, same for other `meta` fields —
    `None` keeps the behaviour identical to 0.10.
- Rerun the MCP integration tests under
  `crates/vectorizer-server/tests/mcp/` and the golden-response tests
  against `cursor_client` + `claude_desktop` test fixtures.

## Impact

- Affected specs: `docs/specs/MCP.md` if the public tool-description
  payload changes shape.
- Affected code: 10 files (see above).
- Breaking change: POTENTIALLY YES for MCP clients that parse the
  wire format — the 1.x protocol version may advertise different
  capability flags. Verify against cursor + claude-desktop before
  merging.
- User benefit: access to the MCP stable API (notifications, typed
  errors, progress reporting, cancellation) that 0.10 doesn't ship.
