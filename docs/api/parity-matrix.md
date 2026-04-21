# REST/MCP Parity Matrix

This document is the human-readable view of
`src/server/capabilities::inventory()`. It tracks which operations are
exposed on both transports, which are intentionally one-sided, and the
rationale for every gap.

For the architectural background see
[`docs/architecture/capabilities.md`](../architecture/capabilities.md).

## Summary

| Transport tag | Count | Meaning |
|---|---|---|
| `Both` | 29 | Same conceptual operation reachable on REST + MCP |
| `McpOnly` | 2 | Live on MCP, no REST counterpart by design |
| `RestOnly` | 1 | Live on REST, no MCP counterpart by design |
| **Registry total** | **32** | |
| Untracked REST routes | ~100 | Auth, admin, qdrant compat, replication, monitoring, multipart, alt protocols (see below) |

## Data-plane operations (in registry)

### Both — must satisfy parity

| `id` | MCP tool | REST | Auth |
|---|---|---|---|
| `collection.list` | `list_collections` | `GET /collections` | User |
| `collection.create` | `create_collection` | `POST /collections` | User |
| `collection.get_info` | `get_collection_info` | `GET /collections/{name}` | User |
| `collection.list_empty` | `list_empty_collections` | `GET /collections/empty` | User |
| `collection.cleanup_empty` | `cleanup_empty_collections` | `DELETE /collections/cleanup` | Admin |
| `vector.insert_text` | `insert_text` | `POST /insert` | User |
| `vector.get` | `get_vector` | `POST /vector` | User |
| `vector.update` | `update_vector` | `POST /update` | User |
| `vector.delete` | `delete_vector` | `POST /delete` | User |
| `search.basic` | `search` | `POST /search` | User |
| `search.multi_collection` | `multi_collection_search` | `POST /multi_collection_search` | User |
| `search.intelligent` | `search_intelligent` | `POST /intelligent_search` | User |
| `search.semantic` | `search_semantic` | `POST /semantic_search` | User |
| `search.hybrid` | `search_hybrid` | `POST /collections/{name}/hybrid_search` | User |
| `discovery.filter_collections` | `filter_collections` | `POST /discovery/filter_collections` | User |
| `discovery.expand_queries` | `expand_queries` | `POST /discovery/expand_queries` | User |
| `file.get_content` | `get_file_content` | `POST /file/content` | User |
| `file.list` | `list_files` | `POST /file/list` | User |
| `file.get_chunks` | `get_file_chunks` | `POST /file/chunks` | User |
| `file.get_outline` | `get_project_outline` | `POST /file/outline` | User |
| `file.get_related` | `get_related_files` | `POST /file/related` | User |
| `graph.list_nodes` | `graph_list_nodes` | `GET /graph/nodes/{collection}` | User |
| `graph.get_neighbors` | `graph_get_neighbors` | `GET /graph/nodes/{collection}/{node_id}/neighbors` | User |
| `graph.find_related` | `graph_find_related` | `GET /graph/nodes/{collection}/{node_id}/related` | User |
| `graph.find_path` | `graph_find_path` | `POST /graph/path` | User |
| `graph.create_edge` | `graph_create_edge` | `POST /graph/edges` | User |
| `graph.delete_edge` | `graph_delete_edge` | `DELETE /graph/edges/{edge_id}` | User |
| `graph.discover_edges` | `graph_discover_edges` | `POST /graph/discover/{collection}` | User |
| `graph.discover_status` | `graph_discover_status` | `GET /graph/discover/{collection}/status` | User |

### `McpOnly` — documented gaps

| `id` | MCP tool | Why no REST |
|---|---|---|
| `search.extra_combined` | `search_extra` | Server-side fan-out over `search`, `search_intelligent`, `search_semantic`. A REST counterpart would just be a thin wrapper duplicating client-side composition; clients can compose the three calls themselves. |
| `collection.get_stats` | `get_collection_stats` | `GET /collections/{name}` returns broader info today; the dedicated stats shape is currently MCP-only. Adding a REST endpoint is tracked as a follow-up. |

### `RestOnly` — documented gaps

| `id` | REST | Why no MCP |
|---|---|---|
| `auth.login` | `POST /auth/login` | MCP clients attach pre-issued JWT tokens at the transport layer; the login + key + user lifecycle endpoints are HTTP-shaped (cookie, redirect, multipart) and would require re-inventing transport-level concerns inside MCP. |

## Untracked REST routes (intentional, not in registry)

The audit at the start of `phase4_rest-mcp-parity-tests` catalogued
~100 REST routes that are intentionally REST-only. They are NOT entered
into the capability registry because the parity test would either need
to skip them or fail spuriously. Listed here for visibility:

| Surface | Routes | Reason |
|---|---:|---|
| Auth (`/auth/*`) | 12 | session/key/user mgmt is HTTP-shaped (see `auth.login` rationale above) |
| Admin / Setup / Backups / Workspace | 18 | server lifecycle, file I/O, admin gates |
| Qdrant compatibility (`/qdrant/*`) | 40 | faithful Qdrant API replica; MCP clients should call MCP-native tools instead |
| Replication (`/replication/*`) | 4 | HA control plane for operators |
| Monitoring (`/metrics`, `/prometheus/metrics`, `/logs`, `/indexing/progress`, `/stats`, `/status`, `/health`) | 7 | scrape targets + ops dashboards |
| Multipart upload (`/files/upload`, `/files/config`) | 2 | HTTP multipart streaming; MCP would need base64 + chunking |
| Alternative protocols (`/graphql`, `/graphiql`, `/umicp`, `/umicp/health`, `/umicp/discover`, `/dashboard*`) | 8 | different query language / static assets |
| HiveHub multi-tenant (`/hub/backups/*`, `/hub/usage/*`) | 6 | tenant isolation, backup multipart restore |

## Outstanding REST data-ops not yet in the registry

There are 26 REST data-plane endpoints whose MCP counterpart has not
been added yet. Until they're either implemented on MCP or explicitly
marked `RestOnly`, the parity test will skip them.

The list is captured in the original phase-4 audit (see
`tests/api/parity.rs::SKIP_REST_DATA_OPS` once that test is added):

- Batch ops: `/batch_insert`, `/insert_texts`, `/batch_search`,
  `/batch_update`, `/batch_delete`
- Discovery pipeline stages:
  `/discovery/{score_collections,broad_discovery,semantic_focus,promote_readme,compress_evidence,build_answer_plan,render_llm_prompt}`
- Other: `/embed`, `/contextual_search`, `/file/summary`,
  `/file/search_by_type`, `/discover` (top-level orchestrator)

Adding these to MCP is tracked as a separate task so the registry
migration can land independently.

## How to keep this doc honest

The boot-time invariant in
`src/server/capabilities::assert_inventory_invariants()` enforces the
structural shape of the registry; the unit tests in
`src/server/mcp/tools.rs` enforce schema parity with the legacy MCP
list. This doc is **descriptive** — when you add or change a registry
entry, also update the matching row here. A future task will generate
this doc from the registry directly.
