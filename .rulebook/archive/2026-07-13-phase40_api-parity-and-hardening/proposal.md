# Proposal: phase40_api-parity-and-hardening

Source: docs/analysis/2026-07-11-improvement-analysis/ (§2, §5.1-5.3)

## Why

The 2026-07-11 improvement analysis found the API surface split into
four transports (REST, MCP, RPC, gRPC) with no superset, an
incomplete source-of-truth registry, and several hardening gaps:

1. **Registry drift**: `capabilities.rs` claims source-of-truth but
   omits live REST endpoints (`/contextual_search`, `/discover`, 7
   discovery routes, `/embed`, `DELETE /collections/{name}`, graph
   routes), mismatches HTTP methods (`graph.find_related` declared
   GET at `capabilities.rs:328`, router registers POST at
   `api/graph.rs:64`), and models neither RPC nor gRPC.
2. **MCP gaps**: `contextual_search`, `delete_collection`,
   `embed_text`, `get_database_stats`, the 8-op discovery pipeline
   and `batch_*` are REST/RPC-only. MCP errors bypass
   `mapping::mcp_code()` — handlers hardcode `internal_error`
   (`mcp/handlers.rs:180,360,518,522`), so "not found" surfaces as
   −32603 Internal.
3. **Unbounded input**: REST `/search`, `/search/text` and MCP
   `search` accept unbounded `limit` (`search.rs:59,467`,
   `mcp/handlers.rs:175`) — memory-DoS vector. `list_vectors` offset
   unbounded.
4. **GraphQL multi-tenant bug**: `create_collection` prefixes
   `user_{id}:{name}` (`mutation.rs:92`) while `upload_file` uses
   `user_{id}_{name}` (`:767`) — uploads land in a different
   collection; `create_collection` also fetches metadata by the
   unprefixed name (`:123-126`).
5. **Error-shape drift**: prod-auth middleware emits ad-hoc
   `{"error": …}` (`routing.rs:1064-1093`); RPC returns free-form
   strings with no stable code.
6. **Config hardening** (from §5): typos silently ignored (no
   `deny_unknown_fields`); `bootstrap.rs` re-parses `config.yml` 4×
   with `.ok()`, bypassing the layered loader so mode overrides never
   reach auth/hub; default config (`0.0.0.0` + auth + empty secret)
   fails first boot.

## What Changes

- Fix the `graph.find_related` method mismatch; add all missing REST
  endpoints to the registry (or mark exclusions explicitly); extend
  the parity test to fail on route/registry drift.
- Add the missing MCP tools (`delete_collection`, `embed_text`,
  `contextual_search`, `get_database_stats`, discovery pipeline,
  `batch_*`) per the REST-first rule that MCP mirrors REST.
- Route MCP handler errors through `VectorizerError` → `mcp_code()`;
  carry `VectorizerError::code()` into RPC error frames.
- Clamp `limit` server-side to the schema max on REST, MCP, and RPC;
  cap `list_vectors` offset.
- Single tenant-prefix helper used by all GraphQL resolvers; fix the
  unprefixed metadata lookup.
- Replace ad-hoc auth-middleware error JSON with the standard
  `ErrorResponse` shape.
- Config hardening: reject unknown top-level keys (or warn loudly),
  load config once through the layered loader in bootstrap, and make
  first boot succeed by defaulting to auto-generated JWT secret (or
  `127.0.0.1` host).
- Correct API_REFERENCE tool count and route paths.

## Impact

- Affected specs: `specs/api-parity/spec.md` (new, in this task)
- Affected code: `crates/vectorizer-server/src/server/capabilities.rs`,
  `crates/vectorizer-server/src/server/mcp/handlers.rs`,
  `crates/vectorizer-server/src/server/rest_handlers/{search,vectors}.rs`,
  `crates/vectorizer-server/src/protocol/rpc/dispatch.rs`,
  `crates/vectorizer-server/src/api/graphql/schema/mutation.rs`,
  `crates/vectorizer-server/src/server/core/{routing,bootstrap}.rs`,
  `crates/vectorizer/src/config/`, `docs/users/api/API_REFERENCE.md`
- Breaking change: NO for REST/MCP/gRPC responses; GraphQL uploads
  previously landing in mis-prefixed collections will now target the
  created collection (bug-fix behavior change)
- User benefit: clients can trust the capability registry; MCP error
  codes meaningful; DoS vector closed; multi-tenant GraphQL works
