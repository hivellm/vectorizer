## 1. Capability registry

- [x] 1.1 `graph.find_related` registry fixed to POST (router is source of truth)
- [x] 1.2 Missing REST endpoints registered (+592 lines in capabilities.rs: search/text, search/file, graph enable/status/edges, discovery, embed, delete_collection, ...) with explicit surface classification; exclusion set for the deliberate leftovers
- [x] 1.3 Parity test extended + new `capability_registry_route_reachability.rs` (3 tests): every registry path probed against the real in-process router — wrong method/absent route fails

## 2. MCP parity + error codes

- [x] 2.1 MCP tools added: `delete_collection`, `embed_text`, `contextual_search`, `get_database_stats` (+924 lines handlers.rs, +395 tools.rs)
- [x] 2.2 Discovery pipeline (8 ops) + `batch_*` MCP tools registered
- [x] 2.3 MCP handler errors routed through `VectorizerError` → `mcp_code()` — not-found surfaces as the mapped code, not −32603
- [x] 2.4 RPC error frames carry `VectorizerError::code()` (dispatch.rs)

## 3. Input hardening

- [x] 3.1 `MAX_SEARCH_LIMIT = 100` server-side clamp: REST search/text/vector, hybrid (dense_k/sparse_k/final_k), explain k, batch entries; MCP/RPC via their dispatch paths
- [x] 3.2 `list_vectors` offset capped (absurd-input guard; scan is already materialized so offset can't drive allocation)

## 4. GraphQL multi-tenancy

- [x] 4.1 Shared `tenant_collection_name` helper (schema/mod.rs); both `create_collection` and `upload_file` use it (colon form) — uploads land in the created collection; regression test proves create-then-upload targets the same collection (verified to fail with the bug reintroduced)
- [x] 4.2 `create_collection` metadata lookup uses the prefixed name

## 5. Error-shape unification

- [x] 5.1 Auth-middleware 401 bodies use the standard shape (`error_type`/`message`/`status_code`) in both prod-auth branches

## 6. Config + startup hardening

- [x] 6.1 `unknown_top_level_keys` allowlist check in `load_layered` → per-key warn (allowlist = modeled fields ∪ documented-but-unwired sections; full deny_unknown_fields would false-positive on the shipped default config)
- [x] 6.2 Config loaded ONCE into `Arc<VectorizerConfig>` in bootstrap; the four `.ok()` re-parses removed (max_request_size now a typed `api.rest` field; auth/hub from the loaded config; backpressure guard threaded through setup_handlers); parse failures warn loudly
- [x] 6.3 First boot succeeds with shipped defaults: empty jwt_secret → unconditional auto-generate (persisted via load_or_generate) with a prominent warn; legacy flag/env kept as no-ops; 0.0.0.0-without-auth rejection untouched (auth now legitimately stays on)

## 7. Docs

- [x] 7.1 API_REFERENCE tool count + route paths corrected (+68-line diff); undocumented endpoints noted

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 8.1 Update or create documentation covering the implementation — API_REFERENCE + CHANGELOG [3.5.0] + doc-comments at every changed site
- [x] 8.2 Write tests covering the new behavior — route-reachability suite (3), parity extensions (8 total), GraphQL tenant regression test, 11 new config/bootstrap unit tests (unknown keys, mode overlay, jwt resolution)
- [x] 8.3 Run tests and confirm they pass — server lib 218, MCP integration 30, parity 8, reachability 3, graphql+mcp+parity filter 43, config module 138; clippy -D warnings 0
