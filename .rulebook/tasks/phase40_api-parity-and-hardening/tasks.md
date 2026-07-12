## 1. Capability registry

- [ ] 1.1 Fix `graph.find_related` method mismatch (`capabilities.rs:328` GET vs `api/graph.rs:64` POST)
- [ ] 1.2 Add missing REST endpoints to the registry (`/contextual_search`, `/discover`, `/discovery/*`, `/embed`, `DELETE /collections/{name}`, `/collections/{name}/search/{text,file}`, graph enable/status/edges) or mark them explicitly excluded
- [ ] 1.3 Extend `tests/api/parity.rs` to diff the axum router's actual routes against the registry and fail on drift

## 2. MCP parity + error codes

- [ ] 2.1 Add MCP tools: `delete_collection`, `embed_text`, `contextual_search`, `get_database_stats`
- [ ] 2.2 Add MCP tools for the discovery pipeline (8 ops) and `batch_*` operations
- [ ] 2.3 Route MCP handler errors through `VectorizerError` → `mapping::mcp_code()` (replace hardcoded `internal_error` at `handlers.rs:180,360,518,522`)
- [ ] 2.4 Carry `VectorizerError::code()` into RPC error frames (`rpc/dispatch.rs`)

## 3. Input hardening

- [ ] 3.1 Clamp `limit` to schema max (100) in REST `/search`, `/search/text`, MCP `search`, and RPC search dispatch
- [ ] 3.2 Cap `list_vectors` offset (`vectors.rs:42-45`)

## 4. GraphQL multi-tenancy

- [ ] 4.1 Single tenant-prefix helper; unify `mutation.rs:92` (colon) vs `:767` (underscore)
- [ ] 4.2 Fix `create_collection` metadata lookup to use the prefixed name (`mutation.rs:123-126`)

## 5. Error-shape unification

- [ ] 5.1 Replace ad-hoc auth-middleware JSON (`routing.rs:1064-1093`) with the standard `ErrorResponse` shape

## 6. Config + startup hardening

- [ ] 6.1 Reject or loudly warn on unknown config keys (layered loader / `deny_unknown_fields` strategy)
- [ ] 6.2 Load config once via the layered loader in bootstrap; remove the four `serde_yaml::from_str(...).ok()` re-parses (`bootstrap.rs:1341,1362,1438`, `setup_handlers.rs:26`)
- [ ] 6.3 Make first boot succeed with the shipped default config (auto-generate JWT secret by default or default host to 127.0.0.1)

## 7. Docs

- [ ] 7.1 Correct API_REFERENCE tool count (32, not "38+") and wrong route paths; document registered-but-undocumented endpoints

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Update or create documentation covering the implementation
- [ ] 8.2 Write tests covering the new behavior
- [ ] 8.3 Run tests and confirm they pass
