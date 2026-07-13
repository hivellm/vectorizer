# §2 — API Surface Consistency

> Scope: `crates/vectorizer-server/src/{server/rest_handlers,server/mcp,grpc,api/graphql,protocol/rpc}`
> + capability inventory (`server/capabilities.rs`). Cross-checked
> against the 2026-04-24 gap analyses.

## 2.1 Prior findings — resolved vs still open

The 2026-04-24 docs are largely **superseded by new infrastructure**:
`capabilities.rs` (registry + boot assertion
`assert_inventory_invariants` + `tests/api/parity.rs`) did not exist
then.

**Resolved since:**
- MCP tool renames all live (`search_vectors→search`,
  `insert_texts→insert_text`, `delete_vectors→delete_vector`,
  `intelligent_search→search_intelligent`,
  `semantic_search→search_semantic`) — `mcp/handlers.rs:100-148`.
- The 8 `graph_*` tools have full MCP+REST parity.
- `list_providers` (phase33) and `get_collection_stats` added.
- Centralized error mapping (`error/mapping.rs`,
  `VectorizerError::code()`) backs REST + gRPC.

**Still unresolved:**
- `contextual_search` absent from MCP (REST route exists,
  `routing.rs:426`).
- `delete_collection` absent from MCP (exists in REST/gRPC/RPC).
- `embed_text` absent from MCP; global `get_database_stats` absent
  from MCP.
- The 8-step discovery pipeline (`discover`, `score_collections`,
  `broad_discovery`, `semantic_focus`, `promote_readme`,
  `compress_evidence`, `build_answer_plan`, `render_llm_prompt`) is
  REST+RPC-only.
- `batch_*` MCP tools still absent.
- API_REFERENCE still says "38+ tools" (actual: 32).

## 2.2 Findings

| Sev | Location | Description | Suggested fix |
|---|---|---|---|
| **HIGH** | `capabilities.rs:328` vs `api/graph.rs:64` | Registry declares `graph.find_related` REST as **GET** `/graph/nodes/{c}/{id}/related`; router registers **POST**. Clients following the registry get 405. | Align registry to `POST` (or router to GET). |
| **HIGH** | `capabilities.rs:90-424` | Registry claims source-of-truth for REST+MCP but omits live REST endpoints: `/contextual_search`, `/discover`, `/discovery/*` (7 routes), `/embed`, `DELETE /collections/{name}`, `/collections/{name}/search/{text,file}`, `/graph/{enable,status,collections/{c}/edges}`. RPC & gRPC not modeled at all. | Add missing capabilities or mark `RestOnly` explicitly; extend registry to RPC/gRPC or document exclusion. |
| **HIGH** | `mcp/handlers.rs:180,360,518,522` | MCP handlers hardcode `ErrorData::internal_error` (−32603) for not-found/bad-input, bypassing `mapping::mcp_code()`. "Collection not found" returns Internal instead of NotFound (−32601). | Route MCP errors through `VectorizerError` → `mcp_code()`. |
| **HIGH** | `protocol/rpc/dispatch.rs:351,487,508,642…` | RPC returns free-form `Response::err(id, String)` — no stable error code/type, no parity with REST `error_type` / gRPC `Code`. | Carry `VectorizerError::code()` into RPC error frames. |
| **MED** | `core/routing.rs:1064-1071,1086-1093` | Global prod-auth middleware emits ad-hoc `{"error":"unauthorized",…}` — field `error`, not the `error_type/status_code/details` shape every other REST error uses. | Return `ErrorResponse::from` / `create_error_response`. |
| **MED** | `search.rs:59,467`; `mcp/handlers.rs:175`; `rpc/dispatch.rs:536` | **Unbounded `limit`**: REST `/search`, `/search/text` and MCP `search` use `unwrap_or(10)` with no upper cap (schema says max 100; handler ignores it). RPC caps only `.max(1)`. Memory-DoS vector. | Clamp `limit.min(MAX)` server-side; enforce schema bounds. |
| **MED** | `rest_handlers/vectors.rs:42-45` | `offset` in `list_vectors` unbounded (limit correctly capped at 50). | Cap offset or move to cursor pagination. |
| **MED** | MCP `handlers.rs:201-208` vs REST `search.rs:112-118` | Response drift: MCP search returns `{results:[{id,score,payload}], total}` (no `vector`); REST returns `…vector…, total_results`. Insert: `vector_id`/`status` vs RPC `id`/`success`. Cleanup: `deleted_count` vs RPC `removed`. | Standardize field names across transports; deep-compare in the parity test. |
| **MED** | `graphql/schema/query.rs:146-170` | GraphQL `search` accepts raw vector only — no text, semantic, intelligent, hybrid, multi-collection, discovery, file-ops, or batch. GraphQL is CRUD + graph + workspace/upload only. | Add `searchText`/hybrid/semantic resolvers or document GraphQL as CRUD-only. |
| **MED** | `graphql/schema/mutation.rs:92` vs `:767` | **Tenant-prefix inconsistency**: `create_collection` uses `user_{id}:{name}` (colon); `upload_file` uses `user_{id}_{name}` (underscore) → uploads target a different collection than created. | Unify via a single tenant-prefix helper. |
| **MED** | `graphql/schema/mutation.rs:123-126` | `create_collection` creates the tenant-prefixed name but fetches metadata by unprefixed `input.name` → errors/empty in multi-tenant mode. | Look up by `collection_name`. |
| **LOW** | `docs/users/api/API_REFERENCE.md:784,242,520,558,618` | "38+ MCP tools" (actual **32**). Documented paths wrong: `/collections/{name}/insert` etc. — real routes are body-based `/insert`, `/intelligent_search`, `/semantic_search`, `/batch_insert` (`routing.rs:331,418,425,410`). | Correct count + paths. |
| **LOW** | API_REFERENCE (absent entries) | Registered-but-undocumented: `/embed`, `/contextual_search`, `/discover`, `/discovery/*`, `/file/*`, `/graph/*`, `/replication/*`, `/cluster/*`, `/collections/{name}/{rename,reindex,snapshot,explain,ttl,reencode}`, `/slow_queries`, `/graphql`. | Document or mark internal. |

## 2.3 Parity matrix (conceptual op × transport)

- `delete_collection`, `contextual_search`, `embed`, the discovery
  pipeline (8 ops), and `batch_*` exist in **REST+RPC** but are
  **missing from MCP** and (except discovery) from `capabilities.rs`.
- gRPC is a **strict subset** (14 methods: CRUD + search /
  batch_search / hybrid_search / stats / health — no semantic /
  intelligent / multi / discovery / file / graph).
- `search_extra` and `get_collection_stats` are intentionally
  MCP-only.

**Net: four different surfaces, no transport is a superset, and the
registry that claims to enforce REST↔MCP parity models neither RPC
nor gRPC.** (→ phase40)
