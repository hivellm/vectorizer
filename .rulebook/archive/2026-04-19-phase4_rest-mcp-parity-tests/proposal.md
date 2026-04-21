# Proposal: phase4_rest-mcp-parity-tests

## Why

`CLAUDE.md` (REST-First Architecture section) mandates: **"REST and MCP must have identical functionality."** Today, this contract is enforced by convention only, and the audit found them diverging:

- MCP tool definitions live in `src/server/mcp_tools.rs` (declarative)
- REST routes live ad-hoc across `src/server/rest_handlers.rs` (imperative)
- There is no cross-check that confirms every MCP tool has a REST counterpart, nor that request/response shapes match.

Given the current team velocity and the monolith split (`phase3_split-rest-handlers-monolith`), it is trivially easy for a contributor to add an MCP-only tool or a REST-only endpoint and ship without anyone noticing.

## What Changes

1. Create a shared **capability registry** `src/server/capabilities.rs` that lists every operation the server exposes. Each entry has: name, input schema (serde), output schema, auth bucket.
2. Generate the MCP tool list FROM the registry (compile-time codegen or a build-time macro).
3. Generate an OpenAPI schema for REST FROM the same registry.
4. Add a cross-check integration test that, for every registry entry, invokes both the REST and MCP surface and asserts equal results.
5. Fail CI if a new handler function is added that is not registered.

## Impact

- Affected specs: REST-First Architecture spec in `CLAUDE.md`
- Affected code: `src/server/mcp_tools.rs`, `src/server/handlers/*` (after split), new `src/server/capabilities.rs`, new parity test suite
- Breaking change: NO (internal refactor; external APIs stabilize)
- User benefit: guaranteed parity between transports; OpenAPI for free; clear operation catalog for clients.
