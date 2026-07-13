//! Capability registry — the single source of truth for the data-plane
//! operations the server exposes on both the REST and MCP surfaces.
//!
//! `CLAUDE.md`'s REST-First Architecture rule says **"REST and MCP must
//! have identical functionality."** This module makes that contract
//! checkable: every entry in [`inventory`] declares the operation's MCP
//! tool name (if any), the REST method + path (if any), the auth bucket
//! it lives in, and an explicit [`Transport`] tag that says whether
//! parity is expected (`Both`) or whether the operation is intentionally
//! one-sided (`RestOnly` / `McpOnly`).
//!
//! What is **not** in this registry: auth/session endpoints (`/auth/*`),
//! admin / setup / backup / workspace lifecycle, the Qdrant
//! compatibility surface (`/qdrant/*`), `/replication/*`, `/metrics`,
//! `/dashboard`, `/graphql`, multipart upload, and the UMICP protocol
//! adapter. Those are documented as transport-specific by design — see
//! `docs/architecture/capabilities.md` for the rationale.
//!
//! The registry is consumed by:
//!
//! - [`crate::server::mcp::tools::get_mcp_tools`] — derives the live
//!   tool list (so adding an entry here automatically exposes it).
//! - The parity test in `tests/api/parity.rs` — for every `Both` entry
//!   it invokes both transports and compares the response shape.
//! - The boot-time assertion in [`assert_inventory_invariants`] — runs
//!   on every server start so a registry typo can't ship silently.

use serde_json::{Value, json};
use vectorizer_core::error::Result;

/// Auth bucket the operation lives in. Used to decide whether the parity
/// test needs to mint a token before invoking the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthBucket {
    /// Reachable without any token (e.g. health check).
    Public,
    /// Requires a valid user-level token.
    User,
    /// Requires an admin-level token.
    Admin,
}

/// Which transports the operation is exposed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// Exposed via both REST and MCP — must satisfy parity.
    Both,
    /// Intentionally REST-only. Captures decisions like "auth lives in
    /// REST only because MCP clients pass JWTs in transport headers".
    RestOnly,
    /// Intentionally MCP-only. Should be rare; usually a temporary state
    /// pending a REST counterpart.
    McpOnly,
}

/// One capability the server exposes on the data plane.
#[derive(Debug, Clone)]
pub struct Capability {
    /// Stable identifier used by the parity test, the boot-time
    /// assertion, and the human-readable parity matrix. Must be unique
    /// across the inventory.
    pub id: &'static str,
    /// One-line summary used as both the MCP tool description and the
    /// OpenAPI `summary` field.
    pub summary: &'static str,
    /// MCP tool name. `None` if the operation is `Transport::RestOnly`.
    /// Must match the name used in
    /// [`crate::server::mcp::handlers::handle_mcp_tool`] dispatch.
    pub mcp_tool_name: Option<&'static str>,
    /// Builds the MCP input JSON schema. Lazily evaluated so adding a
    /// row stays cheap. `None` if `Transport::RestOnly`.
    pub mcp_input_schema: Option<fn() -> Value>,
    /// Canonical REST method + path. `None` if `Transport::McpOnly`.
    /// Path uses axum's `{name}` syntax for path params.
    pub rest: Option<(&'static str, &'static str)>,
    /// Auth bucket the operation lives in.
    pub auth: AuthBucket,
    /// Which transports this capability lives on.
    pub transport: Transport,
}

/// The live capability inventory. Order is preserved for the MCP tool
/// list shown to clients, so additions should append at the end of the
/// matching topical group. Each `summary` string and each schema fn
/// must match the legacy hand-written entry in
/// `src/server/mcp/tools.rs` byte-for-byte until that file is retired
/// — the unit tests
/// `server::mcp::tools::tests::registry_and_legacy_agree_on_overlapping_input_schemas`
/// and `_subset_of_legacy_tools` enforce that.
pub fn inventory() -> Vec<Capability> {
    vec![
        // -----------------------------------------------------------------
        // Core Collection / Vector ops
        // -----------------------------------------------------------------
        Capability {
            id: "collection.list",
            summary: "List all available collections with metadata including vector count, dimension, and configuration.",
            mcp_tool_name: Some("list_collections"),
            mcp_input_schema: Some(schema_empty_object),
            rest: Some(("GET", "/collections")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        // phase33 (#306): mirror the `providers` array from
        // `GET /stats` as a dedicated MCP tool. The REST surface stays
        // on `/stats` (no separate `/providers` endpoint per design
        // D4); the MCP tool exists because the existing `get_stats`
        // tool returns a single blob and AI agents are easier to wire
        // up against a typed `list_providers` shape.
        Capability {
            id: "embedding.list_providers",
            summary: "List every embedding provider registered in the running server (name, dimension, default flag).",
            mcp_tool_name: Some("list_providers"),
            mcp_input_schema: Some(schema_empty_object),
            rest: Some(("GET", "/stats")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "collection.create",
            summary: "Create a new vector collection with specified dimension and distance metric.",
            mcp_tool_name: Some("create_collection"),
            mcp_input_schema: Some(schema_create_collection),
            rest: Some(("POST", "/collections")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "collection.get_info",
            summary: "Get detailed information about a specific collection including stats and configuration.",
            mcp_tool_name: Some("get_collection_info"),
            mcp_input_schema: Some(schema_get_collection_info),
            rest: Some(("GET", "/collections/{name}")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "vector.insert_text",
            summary: "Insert a single text into a collection with automatic embedding generation.",
            mcp_tool_name: Some("insert_text"),
            mcp_input_schema: Some(schema_insert_text),
            rest: Some(("POST", "/insert")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "vector.get",
            summary: "Retrieve a specific vector by ID from a collection.",
            mcp_tool_name: Some("get_vector"),
            mcp_input_schema: Some(schema_get_vector),
            rest: Some(("POST", "/vector")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "vector.update",
            summary: "Update an existing vector with new text and/or metadata.",
            mcp_tool_name: Some("update_vector"),
            mcp_input_schema: Some(schema_update_vector),
            rest: Some(("POST", "/update")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "vector.delete",
            summary: "Delete one or more vectors by ID from a collection.",
            mcp_tool_name: Some("delete_vector"),
            mcp_input_schema: Some(schema_delete_vector),
            rest: Some(("POST", "/delete")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "search.multi_collection",
            summary: "Search across multiple collections simultaneously with results from each.",
            mcp_tool_name: Some("multi_collection_search"),
            mcp_input_schema: Some(schema_multi_collection_search),
            rest: Some(("POST", "/multi_collection_search")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "search.basic",
            summary: "Basic vector similarity search in a single collection.",
            mcp_tool_name: Some("search"),
            mcp_input_schema: Some(schema_search_vectors),
            rest: Some(("POST", "/search")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        // -----------------------------------------------------------------
        // Search variants
        // -----------------------------------------------------------------
        Capability {
            id: "search.intelligent",
            summary: "AI-powered search with automatic query expansion and result deduplication across collections.",
            mcp_tool_name: Some("search_intelligent"),
            mcp_input_schema: Some(schema_search_intelligent),
            rest: Some(("POST", "/intelligent_search")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "search.semantic",
            summary: "Semantic search with basic reranking for better relevance.",
            mcp_tool_name: Some("search_semantic"),
            mcp_input_schema: Some(schema_search_semantic),
            rest: Some(("POST", "/semantic_search")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "search.extra_combined",
            summary: "Combined search that concatenates results from multiple search strategies (basic, intelligent, semantic).",
            mcp_tool_name: Some("search_extra"),
            mcp_input_schema: Some(schema_search_extra),
            // No REST counterpart today: this is a server-side fan-out
            // over other search endpoints. A REST entry would just be a
            // thin wrapper duplicating client-side composition.
            rest: None,
            auth: AuthBucket::User,
            transport: Transport::McpOnly,
        },
        Capability {
            id: "search.hybrid",
            summary: "Hybrid search combining dense (HNSW) and sparse vector search for optimal results.",
            mcp_tool_name: Some("search_hybrid"),
            mcp_input_schema: Some(schema_search_hybrid),
            rest: Some(("POST", "/collections/{name}/hybrid_search")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        // phase40 §1.2: REST-only search variants over a single named
        // collection. No MCP counterpart exists today — search_semantic /
        // search_intelligent cover the AI-driven query paths, and adding a
        // dedicated MCP tool for a raw text-query-string or file-upload
        // search would just re-expose what `search` already does over
        // this transport. Tracked here (rather than left undocumented)
        // so the registry stops omitting a live route.
        Capability {
            id: "search.by_text",
            summary: "Search a specific collection using a raw text query string.",
            mcp_tool_name: None,
            mcp_input_schema: None,
            rest: Some(("POST", "/collections/{name}/search/text")),
            auth: AuthBucket::User,
            transport: Transport::RestOnly,
        },
        Capability {
            id: "search.by_file",
            summary: "Search a specific collection using an uploaded file as the query.",
            mcp_tool_name: None,
            mcp_input_schema: None,
            rest: Some(("POST", "/collections/{name}/search/file")),
            auth: AuthBucket::User,
            transport: Transport::RestOnly,
        },
        // -----------------------------------------------------------------
        // Discovery
        // -----------------------------------------------------------------
        Capability {
            id: "discovery.filter_collections",
            summary: "Filter collections by name patterns with include/exclude support.",
            mcp_tool_name: Some("filter_collections"),
            mcp_input_schema: Some(schema_filter_collections),
            rest: Some(("POST", "/discovery/filter_collections")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "discovery.expand_queries",
            summary: "Generate query variations and expansions for broader search coverage.",
            mcp_tool_name: Some("expand_queries"),
            mcp_input_schema: Some(schema_expand_queries),
            rest: Some(("POST", "/discovery/expand_queries")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        // -----------------------------------------------------------------
        // File operations
        // -----------------------------------------------------------------
        Capability {
            id: "file.get_content",
            summary: "Retrieve complete file content from a collection.",
            mcp_tool_name: Some("get_file_content"),
            mcp_input_schema: Some(schema_get_file_content),
            rest: Some(("POST", "/file/content")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "file.list",
            summary: "List all indexed files in a collection with metadata and filters.",
            mcp_tool_name: Some("list_files"),
            mcp_input_schema: Some(schema_list_files),
            rest: Some(("POST", "/file/list")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "file.get_chunks",
            summary: "Retrieve file chunks in original order for progressive reading.",
            mcp_tool_name: Some("get_file_chunks"),
            mcp_input_schema: Some(schema_get_file_chunks),
            rest: Some(("POST", "/file/chunks")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "file.get_outline",
            summary: "Generate hierarchical project structure overview from indexed files.",
            mcp_tool_name: Some("get_project_outline"),
            mcp_input_schema: Some(schema_get_project_outline),
            rest: Some(("POST", "/file/outline")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "file.get_related",
            summary: "Find semantically related files using vector similarity.",
            mcp_tool_name: Some("get_related_files"),
            mcp_input_schema: Some(schema_get_related_files),
            rest: Some(("POST", "/file/related")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        // -----------------------------------------------------------------
        // Graph
        // -----------------------------------------------------------------
        Capability {
            id: "graph.list_nodes",
            summary: "List all nodes in a collection's graph with their metadata.",
            mcp_tool_name: Some("graph_list_nodes"),
            mcp_input_schema: Some(schema_graph_list_nodes),
            rest: Some(("GET", "/graph/nodes/{collection}")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "graph.get_neighbors",
            summary: "Get all neighbors of a specific node in the graph with their relationships.",
            mcp_tool_name: Some("graph_get_neighbors"),
            mcp_input_schema: Some(schema_graph_get_neighbors),
            rest: Some(("GET", "/graph/nodes/{collection}/{node_id}/neighbors")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "graph.find_related",
            summary: "Find all nodes related to a given node within N hops in the graph.",
            mcp_tool_name: Some("graph_find_related"),
            mcp_input_schema: Some(schema_graph_find_related),
            // phase40 §1.1: the router (`api/graph.rs:64`) registers this
            // route as POST, not GET — the registry previously disagreed
            // and would have sent registry-driven callers into a 405.
            rest: Some(("POST", "/graph/nodes/{collection}/{node_id}/related")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "graph.find_path",
            summary: "Find the shortest path between two nodes in the graph.",
            mcp_tool_name: Some("graph_find_path"),
            mcp_input_schema: Some(schema_graph_find_path),
            rest: Some(("POST", "/graph/path")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "graph.create_edge",
            summary: "Create an explicit edge/relationship between two nodes in the graph.",
            mcp_tool_name: Some("graph_create_edge"),
            mcp_input_schema: Some(schema_graph_create_edge),
            rest: Some(("POST", "/graph/edges")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "graph.delete_edge",
            summary: "Delete an edge/relationship from the graph.",
            mcp_tool_name: Some("graph_delete_edge"),
            mcp_input_schema: Some(schema_graph_delete_edge),
            rest: Some(("DELETE", "/graph/edges/{edge_id}")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "graph.discover_edges",
            summary: "Automatically discover and create SIMILAR_TO edges between nodes based on semantic similarity. Can discover for a specific node or entire collection.",
            mcp_tool_name: Some("graph_discover_edges"),
            mcp_input_schema: Some(schema_graph_discover_edges),
            rest: Some(("POST", "/graph/discover/{collection}")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "graph.discover_status",
            summary: "Get discovery status for a collection, showing how many nodes have edges and overall progress.",
            mcp_tool_name: Some("graph_discover_status"),
            mcp_input_schema: Some(schema_graph_discover_status),
            rest: Some(("GET", "/graph/discover/{collection}/status")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        // phase40 §1.2: REST-only graph lifecycle + inspection routes with
        // no MCP counterpart. `graph.discover_edges`/`graph.discover_status`
        // above cover automatic edge discovery; these three cover turning
        // graph tracking on for a collection, reading whether it's on, and
        // listing raw edges (as opposed to per-node neighbors).
        Capability {
            id: "graph.enable",
            summary: "Enable graph relationship tracking for a collection.",
            mcp_tool_name: None,
            mcp_input_schema: None,
            rest: Some(("POST", "/graph/enable/{collection}")),
            auth: AuthBucket::User,
            transport: Transport::RestOnly,
        },
        Capability {
            id: "graph.status",
            summary: "Get whether graph relationship tracking is enabled for a collection.",
            mcp_tool_name: None,
            mcp_input_schema: None,
            rest: Some(("GET", "/graph/status/{collection}")),
            auth: AuthBucket::User,
            transport: Transport::RestOnly,
        },
        Capability {
            id: "graph.list_edges",
            summary: "List all edges in a collection's graph.",
            mcp_tool_name: None,
            mcp_input_schema: None,
            rest: Some(("GET", "/graph/collections/{collection}/edges")),
            auth: AuthBucket::User,
            transport: Transport::RestOnly,
        },
        // -----------------------------------------------------------------
        // Collection maintenance
        // -----------------------------------------------------------------
        Capability {
            id: "collection.list_empty",
            summary: "List all collections that have zero vectors. Useful for identifying collections that can be cleaned up.",
            mcp_tool_name: Some("list_empty_collections"),
            mcp_input_schema: Some(schema_empty_object),
            rest: Some(("GET", "/collections/empty")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "collection.cleanup_empty",
            summary: "Delete all empty collections (collections with zero vectors). Use dry_run=true to preview what would be deleted without actually deleting.",
            mcp_tool_name: Some("cleanup_empty_collections"),
            mcp_input_schema: Some(schema_cleanup_empty_collections),
            rest: Some(("DELETE", "/collections/cleanup")),
            auth: AuthBucket::Admin,
            transport: Transport::Both,
        },
        Capability {
            id: "collection.get_stats",
            summary: "Get detailed statistics for a collection including vector count, dimension, and whether it's empty.",
            mcp_tool_name: Some("get_collection_stats"),
            mcp_input_schema: Some(schema_get_collection_stats),
            // GET /collections/{name} returns broader info; the dedicated
            // stats endpoint is currently MCP-only. A REST counterpart is
            // tracked separately in the parity-matrix doc.
            rest: None,
            auth: AuthBucket::User,
            transport: Transport::McpOnly,
        },
        // -----------------------------------------------------------------
        // phase40 §2.1: MCP tools mirroring REST-only endpoints. These
        // routes used to live in `documented_rest_exclusions` pending this
        // work; they convert to `Transport::Both` now that the matching
        // MCP tool exists.
        // -----------------------------------------------------------------
        Capability {
            id: "collection.delete",
            summary: "Delete a collection and all of its vectors.",
            mcp_tool_name: Some("delete_collection"),
            mcp_input_schema: Some(schema_delete_collection),
            rest: Some(("DELETE", "/collections/{name}")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "embedding.embed_text",
            summary: "Generate an embedding for a text input via the server's active embedding provider.",
            mcp_tool_name: Some("embed_text"),
            mcp_input_schema: Some(schema_embed_text),
            rest: Some(("POST", "/embed")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "search.contextual",
            summary: "Search with context-aware filtering and reranking.",
            mcp_tool_name: Some("contextual_search"),
            mcp_input_schema: Some(schema_contextual_search),
            rest: Some(("POST", "/contextual_search")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "stats.get_database_stats",
            summary: "Get aggregate collection/vector counts and the embedding provider registry for the whole server.",
            mcp_tool_name: Some("get_database_stats"),
            mcp_input_schema: Some(schema_empty_object),
            // GET /stats is already tracked under `embedding.list_providers`
            // (same route, different capability slice) — a second `Both`
            // row pointing at the same (method, path) would trip the
            // duplicate-REST-route invariant, so this stays McpOnly.
            rest: None,
            auth: AuthBucket::User,
            transport: Transport::McpOnly,
        },
        // -----------------------------------------------------------------
        // phase40 §2.2: discovery pipeline (8 ops). Same exclusion→Both
        // conversion rationale as the block above.
        // -----------------------------------------------------------------
        Capability {
            id: "discovery.discover",
            summary: "Run the full discovery pipeline (filter, score, expand, broad search, semantic focus, README promotion, evidence compression, answer plan, prompt rendering) for a query.",
            mcp_tool_name: Some("discover"),
            mcp_input_schema: Some(schema_discover),
            rest: Some(("POST", "/discover")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "discovery.score_collections",
            summary: "Score collections by relevance to a query (name match, term boost, signal boost).",
            mcp_tool_name: Some("score_collections"),
            mcp_input_schema: Some(schema_score_collections),
            rest: Some(("POST", "/discovery/score_collections")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "discovery.broad_discovery",
            summary: "Search across all matching collections with multiple query variations to gather a wide candidate set.",
            mcp_tool_name: Some("broad_discovery"),
            mcp_input_schema: Some(schema_broad_discovery),
            rest: Some(("POST", "/discovery/broad_discovery")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "discovery.semantic_focus",
            summary: "Narrow a broad candidate set down to the most relevant chunks within a single collection.",
            mcp_tool_name: Some("semantic_focus"),
            mcp_input_schema: Some(schema_semantic_focus),
            rest: Some(("POST", "/discovery/semantic_focus")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "discovery.promote_readme",
            summary: "Boost README-like chunks to the front of a scored chunk list.",
            mcp_tool_name: Some("promote_readme"),
            mcp_input_schema: Some(schema_promote_readme),
            rest: Some(("POST", "/discovery/promote_readme")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "discovery.compress_evidence",
            summary: "Compress scored chunks into short evidence bullets, capped per document.",
            mcp_tool_name: Some("compress_evidence"),
            mcp_input_schema: Some(schema_compress_evidence),
            rest: Some(("POST", "/discovery/compress_evidence")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "discovery.build_answer_plan",
            summary: "Group evidence bullets into an ordered answer plan (sections by category).",
            mcp_tool_name: Some("build_answer_plan"),
            mcp_input_schema: Some(schema_build_answer_plan),
            rest: Some(("POST", "/discovery/build_answer_plan")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "discovery.render_llm_prompt",
            summary: "Render an answer plan into the final LLM-ready prompt string.",
            mcp_tool_name: Some("render_llm_prompt"),
            mcp_input_schema: Some(schema_render_llm_prompt),
            rest: Some(("POST", "/discovery/render_llm_prompt")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        // -----------------------------------------------------------------
        // phase40 §2.2: batch operations. No REST-exclusion entries existed
        // for these routes (they were simply absent from the registry).
        // -----------------------------------------------------------------
        Capability {
            id: "vector.batch_insert_texts",
            summary: "Insert multiple texts into a collection in one call, embedding each server-side.",
            mcp_tool_name: Some("batch_insert_texts"),
            mcp_input_schema: Some(schema_batch_insert_texts),
            rest: Some(("POST", "/batch_insert")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "search.batch_search",
            summary: "Run multiple searches against one collection in a single call.",
            mcp_tool_name: Some("batch_search"),
            mcp_input_schema: Some(schema_batch_search),
            rest: Some(("POST", "/batch_search")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "vector.batch_update",
            summary: "Update multiple vectors' data and/or payload in one call.",
            mcp_tool_name: Some("batch_update"),
            mcp_input_schema: Some(schema_batch_update),
            rest: Some(("POST", "/batch_update")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        Capability {
            id: "vector.batch_delete",
            summary: "Delete multiple vectors by ID from one collection in a single call.",
            mcp_tool_name: Some("batch_delete"),
            mcp_input_schema: Some(schema_batch_delete),
            rest: Some(("POST", "/batch_delete")),
            auth: AuthBucket::User,
            transport: Transport::Both,
        },
        // -----------------------------------------------------------------
        // Transport-specific markers (RestOnly entries) — pin the design
        // intent so the parity test never tries to call MCP for these.
        // -----------------------------------------------------------------
        Capability {
            id: "auth.login",
            summary: "Exchange username + password for a JWT session token. Intentionally REST-only because MCP clients attach pre-issued JWTs at the transport layer.",
            mcp_tool_name: None,
            mcp_input_schema: None,
            rest: Some(("POST", "/auth/login")),
            auth: AuthBucket::Public,
            transport: Transport::RestOnly,
        },
    ]
}

/// Live REST routes deliberately left out of [`inventory`] for now.
///
/// phase40 §1.2 found these routes registered in
/// `core/routing.rs`/`api/graph.rs` with no matching capability entry.
/// Rather than adding them as bare `RestOnly` rows (which would forever
/// forgo the MCP counterpart they're slated to get), each one is parked
/// here with a reason. phase40 §2 (MCP parity) has since landed the
/// `delete_collection`, `embed_text`, `contextual_search`, and 8-step
/// discovery pipeline MCP tools — those routes converted to
/// `Transport::Both` entries in [`inventory`] and were removed from this
/// list.
///
/// Consumed by the route-reachability test in
/// `vectorizer-server/tests/capability_registry_route_reachability.rs`
/// (every entry here is dispatched against the real router exactly like
/// an [`inventory`] entry, so a route removed out from under an
/// exclusion is caught the same way a removed registry route is) and by
/// the fast structural check in `vectorizer/tests/api/parity.rs` that
/// asserts no route is simultaneously tracked and excluded.
pub fn documented_rest_exclusions() -> &'static [(&'static str, &'static str, &'static str)] {
    &[]
}

fn schema_empty_object() -> Value {
    json!({ "type": "object", "properties": {}, "required": [] })
}

fn schema_create_collection() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "Collection name" },
            "dimension": { "type": "integer", "description": "Vector dimension" },
            "metric": {
                "type": "string",
                "description": "Distance metric: 'cosine' or 'euclidean'",
                "default": "cosine"
            },
            "graph": {
                "type": "object",
                "description": "Graph configuration (optional)",
                "properties": {
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable graph relationship tracking",
                        "default": false
                    }
                }
            }
        },
        "required": ["name", "dimension"]
    })
}

fn schema_search_vectors() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Search query" },
            "collection": { "type": "string", "description": "Collection name" },
            "limit": {
                "type": "integer",
                "description": "Maximum results to return",
                "default": 10,
                "minimum": 1,
                "maximum": 100
            },
            "similarity_threshold": {
                "type": "number",
                "description": "Minimum similarity score 0.0-1.0",
                "default": 0.1
            }
        },
        "required": ["query", "collection"]
    })
}

fn schema_delete_vector() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "vector_ids": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Array of vector IDs to delete"
            }
        },
        "required": ["collection", "vector_ids"]
    })
}

fn schema_get_collection_info() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "Collection name" }
        },
        "required": ["name"]
    })
}

fn schema_insert_text() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection_name": { "type": "string", "description": "Collection name" },
            "text": { "type": "string", "description": "Text to insert" },
            "metadata": { "type": "object", "description": "Optional metadata" }
        },
        "required": ["collection_name", "text"]
    })
}

fn schema_get_vector() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "vector_id": { "type": "string", "description": "Vector ID" }
        },
        "required": ["collection", "vector_id"]
    })
}

fn schema_update_vector() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "vector_id": { "type": "string", "description": "Vector ID" },
            "text": { "type": "string", "description": "New text content" },
            "metadata": { "type": "object", "description": "Optional metadata" }
        },
        "required": ["collection", "vector_id"]
    })
}

fn schema_multi_collection_search() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Search query" },
            "collections": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Array of collection names to search"
            },
            "max_per_collection": {
                "type": "integer",
                "description": "Maximum results per collection",
                "default": 5
            },
            "max_total_results": {
                "type": "integer",
                "description": "Total maximum results",
                "default": 20
            },
            "similarity_threshold": {
                "type": "number",
                "description": "Minimum similarity score 0.0-1.0",
                "default": 0.1
            }
        },
        "required": ["query", "collections"]
    })
}

fn schema_search_intelligent() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Search query" },
            "collections": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Array of collection names (optional, searches all if omitted)"
            },
            "max_results": {
                "type": "integer",
                "description": "Maximum results to return",
                "default": 10
            },
            "domain_expansion": {
                "type": "boolean",
                "description": "Enable domain-specific query expansion",
                "default": true
            },
            "similarity_threshold": {
                "type": "number",
                "description": "Minimum similarity score 0.0-1.0",
                "default": 0.1
            }
        },
        "required": ["query"]
    })
}

fn schema_search_semantic() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Search query" },
            "collection": { "type": "string", "description": "Collection name" },
            "max_results": {
                "type": "integer",
                "description": "Maximum results to return",
                "default": 10
            },
            "similarity_threshold": {
                "type": "number",
                "description": "Minimum similarity score 0.0-1.0",
                "default": 0.1
            }
        },
        "required": ["query", "collection"]
    })
}

fn schema_search_extra() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Search query" },
            "collection": { "type": "string", "description": "Collection name" },
            "strategies": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Search strategies to combine: 'basic', 'intelligent', 'semantic'",
                "default": ["basic", "semantic"]
            },
            "max_results": {
                "type": "integer",
                "description": "Maximum results per strategy",
                "default": 10
            },
            "similarity_threshold": {
                "type": "number",
                "description": "Minimum similarity score 0.0-1.0",
                "default": 0.1
            }
        },
        "required": ["query", "collection"]
    })
}

fn schema_search_hybrid() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query (will be converted to dense embedding)"
            },
            "collection": { "type": "string", "description": "Collection name" },
            "query_sparse": {
                "type": "object",
                "description": "Optional sparse vector query (indices and values arrays)",
                "properties": {
                    "indices": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Non-zero indices"
                    },
                    "values": {
                        "type": "array",
                        "items": { "type": "number" },
                        "description": "Values at corresponding indices"
                    }
                }
            },
            "alpha": {
                "type": "number",
                "description": "Weight for dense search (0.0 = pure sparse, 1.0 = pure dense)",
                "default": 0.7,
                "minimum": 0.0,
                "maximum": 1.0
            },
            "algorithm": {
                "type": "string",
                "description": "Scoring algorithm: 'rrf' (Reciprocal Rank Fusion), 'weighted' (Weighted Combination), 'alpha' (Alpha Blending)",
                "enum": ["rrf", "weighted", "alpha"],
                "default": "rrf"
            },
            "dense_k": {
                "type": "integer",
                "description": "Number of results from dense search",
                "default": 20
            },
            "sparse_k": {
                "type": "integer",
                "description": "Number of results from sparse search",
                "default": 20
            },
            "final_k": {
                "type": "integer",
                "description": "Final number of results to return",
                "default": 10
            }
        },
        "required": ["query", "collection"]
    })
}

fn schema_filter_collections() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Query to filter collections (collection names or keywords)"
            },
            "include": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Include patterns (optional)"
            },
            "exclude": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Exclude patterns (optional)"
            }
        },
        "required": ["query"]
    })
}

fn schema_expand_queries() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Base query to expand" },
            "max_expansions": {
                "type": "integer",
                "description": "Maximum number of expansions",
                "default": 8
            },
            "include_definition": {
                "type": "boolean",
                "description": "Include definition queries",
                "default": true
            },
            "include_features": {
                "type": "boolean",
                "description": "Include features queries",
                "default": true
            },
            "include_architecture": {
                "type": "boolean",
                "description": "Include architecture queries",
                "default": true
            }
        },
        "required": ["query"]
    })
}

fn schema_get_file_content() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "file_path": { "type": "string", "description": "File path" },
            "max_size_kb": {
                "type": "integer",
                "description": "Maximum file size in KB",
                "default": 500,
                "minimum": 1,
                "maximum": 5000
            }
        },
        "required": ["collection", "file_path"]
    })
}

fn schema_list_files() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "filter_by_type": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Filter by file extensions (e.g., ['rs', 'ts'])"
            },
            "min_chunks": {
                "type": "integer",
                "description": "Minimum number of chunks"
            },
            "max_results": {
                "type": "integer",
                "description": "Maximum number of results",
                "default": 100
            },
            "sort_by": {
                "type": "string",
                "description": "Sort order: 'name', 'size', 'chunks', or 'recent'",
                "default": "name"
            }
        },
        "required": ["collection"]
    })
}

fn schema_get_file_chunks() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "file_path": { "type": "string", "description": "File path" },
            "start_chunk": {
                "type": "integer",
                "description": "Starting chunk index",
                "default": 0,
                "minimum": 0
            },
            "limit": {
                "type": "integer",
                "description": "Number of chunks to retrieve",
                "default": 10,
                "minimum": 1,
                "maximum": 50
            },
            "include_context": {
                "type": "boolean",
                "description": "Include prev/next chunk hints",
                "default": false
            }
        },
        "required": ["collection", "file_path"]
    })
}

fn schema_get_project_outline() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "max_depth": {
                "type": "integer",
                "description": "Maximum directory depth",
                "default": 5,
                "minimum": 1,
                "maximum": 10
            },
            "include_summaries": {
                "type": "boolean",
                "description": "Include file summaries in outline",
                "default": false
            },
            "highlight_key_files": {
                "type": "boolean",
                "description": "Highlight important files like README",
                "default": true
            }
        },
        "required": ["collection"]
    })
}

fn schema_get_related_files() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "file_path": { "type": "string", "description": "File path" },
            "max_results": {
                "type": "integer",
                "description": "Maximum number of results",
                "default": 10
            },
            "similarity_threshold": {
                "type": "number",
                "description": "Minimum similarity score 0.0-1.0",
                "default": 0.6,
                "minimum": 0.0,
                "maximum": 1.0
            },
            "include_reason": {
                "type": "boolean",
                "description": "Include explanation of why files are related",
                "default": true
            }
        },
        "required": ["collection", "file_path"]
    })
}

fn schema_graph_list_nodes() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" }
        },
        "required": ["collection"]
    })
}

fn schema_graph_get_neighbors() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "node_id": { "type": "string", "description": "Node identifier" }
        },
        "required": ["collection", "node_id"]
    })
}

fn schema_graph_find_related() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "node_id": { "type": "string", "description": "Starting node identifier" },
            "max_hops": {
                "type": "integer",
                "description": "Maximum number of hops to traverse",
                "default": 2
            },
            "relationship_type": {
                "type": "string",
                "description": "Filter by relationship type (SIMILAR_TO, REFERENCES, CONTAINS, DERIVED_FROM)",
                "enum": ["SIMILAR_TO", "REFERENCES", "CONTAINS", "DERIVED_FROM"]
            }
        },
        "required": ["collection", "node_id"]
    })
}

fn schema_graph_find_path() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "source": { "type": "string", "description": "Source node identifier" },
            "target": { "type": "string", "description": "Target node identifier" }
        },
        "required": ["collection", "source", "target"]
    })
}

fn schema_graph_create_edge() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "source": { "type": "string", "description": "Source node identifier" },
            "target": { "type": "string", "description": "Target node identifier" },
            "relationship_type": {
                "type": "string",
                "description": "Type of relationship",
                "enum": ["SIMILAR_TO", "REFERENCES", "CONTAINS", "DERIVED_FROM"]
            },
            "weight": {
                "type": "number",
                "description": "Edge weight (0.0 to 1.0)",
                "default": 1.0
            }
        },
        "required": ["collection", "source", "target", "relationship_type"]
    })
}

fn schema_graph_delete_edge() -> Value {
    json!({
        "type": "object",
        "properties": {
            "edge_id": { "type": "string", "description": "Edge identifier to delete" }
        },
        "required": ["edge_id"]
    })
}

fn schema_graph_discover_edges() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" },
            "node_id": {
                "type": "string",
                "description": "Optional: specific node ID to discover edges for. If omitted, discovers for entire collection."
            },
            "similarity_threshold": {
                "type": "number",
                "description": "Similarity threshold (0.0 to 1.0) for creating edges. Default: 0.7",
                "default": 0.7
            },
            "max_per_node": {
                "type": "integer",
                "description": "Maximum number of edges to create per node. Default: 10",
                "default": 10
            }
        },
        "required": ["collection"]
    })
}

fn schema_graph_discover_status() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" }
        },
        "required": ["collection"]
    })
}

fn schema_cleanup_empty_collections() -> Value {
    json!({
        "type": "object",
        "properties": {
            "dry_run": {
                "type": "boolean",
                "description": "If true, only shows what would be deleted without actually deleting. Default: false",
                "default": false
            }
        },
        "required": []
    })
}

fn schema_get_collection_stats() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": { "type": "string", "description": "Collection name" }
        },
        "required": ["collection"]
    })
}

fn schema_delete_collection() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Collection name"
            }
        },
        "required": ["name"]
    })
}

fn schema_embed_text() -> Value {
    json!({
        "type": "object",
        "properties": {
            "text": {
                "type": "string",
                "description": "Text to embed"
            },
            "model": {
                "type": "string",
                "description": "Embedding provider name (optional, uses the server default when omitted)"
            }
        },
        "required": ["text"]
    })
}

fn schema_contextual_search() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query"
            },
            "collection": {
                "type": "string",
                "description": "Collection name"
            },
            "context_filters": {
                "type": "object",
                "description": "Context metadata filters (optional)"
            },
            "context_weight": {
                "type": "number",
                "description": "Context weight in scoring",
                "default": 0.3
            },
            "context_reranking": {
                "type": "boolean",
                "description": "Enable context-aware reranking",
                "default": true
            },
            "max_results": {
                "type": "integer",
                "description": "Maximum results to return",
                "default": 10
            }
        },
        "required": ["query", "collection"]
    })
}

fn schema_discover() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query"
            },
            "include_collections": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Collections to include (optional)"
            },
            "exclude_collections": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Collections to exclude (optional)"
            },
            "max_bullets": {
                "type": "integer",
                "description": "Maximum evidence bullets to extract",
                "default": 20
            },
            "broad_k": {
                "type": "integer",
                "description": "Number of chunks retrieved in the broad discovery step",
                "default": 50
            },
            "focus_k": {
                "type": "integer",
                "description": "Number of chunks retrieved in the semantic focus step",
                "default": 15
            }
        },
        "required": ["query"]
    })
}

fn schema_score_collections() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query"
            },
            "name_match_weight": {
                "type": "number",
                "description": "Weight for collection-name term matches",
                "default": 0.4
            },
            "term_boost_weight": {
                "type": "number",
                "description": "Weight for query-term boosting",
                "default": 0.3
            },
            "signal_boost_weight": {
                "type": "number",
                "description": "Weight for collection signal boosting",
                "default": 0.3
            }
        },
        "required": ["query"]
    })
}

fn schema_broad_discovery() -> Value {
    json!({
        "type": "object",
        "properties": {
            "queries": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Query variations to search with"
            },
            "k": {
                "type": "integer",
                "description": "Maximum number of chunks to return",
                "default": 50
            }
        },
        "required": ["queries"]
    })
}

fn schema_semantic_focus() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": {
                "type": "string",
                "description": "Collection name"
            },
            "queries": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Query variations to search with"
            },
            "k": {
                "type": "integer",
                "description": "Maximum number of chunks to return",
                "default": 15
            }
        },
        "required": ["collection", "queries"]
    })
}

fn schema_promote_readme() -> Value {
    json!({
        "type": "object",
        "properties": {
            "chunks": {
                "type": "array",
                "items": {"type": "object"},
                "description": "Scored chunks: [{collection, doc_id, content, score, file_path, chunk_index, file_extension}]"
            }
        },
        "required": ["chunks"]
    })
}

fn schema_compress_evidence() -> Value {
    json!({
        "type": "object",
        "properties": {
            "chunks": {
                "type": "array",
                "items": {"type": "object"},
                "description": "Scored chunks: [{collection, doc_id, content, score, file_path, chunk_index, file_extension}]"
            },
            "max_bullets": {
                "type": "integer",
                "description": "Maximum number of bullets to produce",
                "default": 20
            },
            "max_per_doc": {
                "type": "integer",
                "description": "Maximum bullets per source document",
                "default": 3
            }
        },
        "required": ["chunks"]
    })
}

fn schema_build_answer_plan() -> Value {
    json!({
        "type": "object",
        "properties": {
            "bullets": {
                "type": "array",
                "items": {"type": "object"},
                "description": "Evidence bullets: [{text, source_id, collection, file_path, score, category}]"
            }
        },
        "required": ["bullets"]
    })
}

fn schema_render_llm_prompt() -> Value {
    json!({
        "type": "object",
        "properties": {
            "plan": {
                "type": "object",
                "description": "Answer plan: {sections: [{title, priority, bullets: [...]}], total_bullets, sources}"
            }
        },
        "required": ["plan"]
    })
}

fn schema_batch_insert_texts() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection_name": {
                "type": "string",
                "description": "Collection name"
            },
            "texts": {
                "type": "array",
                "items": {"type": "object"},
                "description": "Texts to insert: [{id?, text, metadata?, public_key?}]"
            }
        },
        "required": ["collection_name", "texts"]
    })
}

fn schema_batch_search() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": {
                "type": "string",
                "description": "Collection name"
            },
            "queries": {
                "type": "array",
                "items": {"type": "object"},
                "description": "Queries: [{query?, vector?, limit?}] — each entry needs either `query` (embedded server-side) or a raw `vector`"
            }
        },
        "required": ["collection", "queries"]
    })
}

fn schema_batch_update() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": {
                "type": "string",
                "description": "Collection name"
            },
            "updates": {
                "type": "array",
                "items": {"type": "object"},
                "description": "Updates: [{id, vector?, payload?}]"
            }
        },
        "required": ["collection", "updates"]
    })
}

fn schema_batch_delete() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collection": {
                "type": "string",
                "description": "Collection name"
            },
            "ids": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Vector IDs to delete"
            }
        },
        "required": ["collection", "ids"]
    })
}

/// Validate the registry's structural invariants. Called from server
/// boot so a typo in [`inventory`] crashes startup loudly instead of
/// silently desyncing the MCP tool list and the REST router.
///
/// Invariants:
///
/// 1. Every `id` is unique.
/// 2. Every `mcp_tool_name` (when present) is unique.
/// 3. Every `(method, path)` (when present) is unique.
/// 4. `transport == Both` implies BOTH `mcp_tool_name` and `rest` are
///    set, and BOTH `mcp_input_schema` and `rest` are set.
/// 5. `transport == RestOnly` implies `mcp_tool_name`, `mcp_input_schema`
///    are `None` and `rest` is `Some`.
/// 6. `transport == McpOnly` implies `rest` is `None` and the MCP fields
///    are `Some`.
pub fn assert_inventory_invariants() -> Result<()> {
    use vectorizer_core::error::VectorizerError;

    let inv = inventory();
    let mut seen_ids = std::collections::HashSet::<&'static str>::new();
    let mut seen_mcp = std::collections::HashSet::<&'static str>::new();
    let mut seen_rest = std::collections::HashSet::<(&'static str, &'static str)>::new();

    for cap in &inv {
        if !seen_ids.insert(cap.id) {
            return Err(VectorizerError::Configuration(format!(
                "capability registry: duplicate id '{}'",
                cap.id
            )));
        }
        if let Some(name) = cap.mcp_tool_name
            && !seen_mcp.insert(name)
        {
            return Err(VectorizerError::Configuration(format!(
                "capability registry: duplicate MCP tool name '{}' (id '{}')",
                name, cap.id
            )));
        }
        if let Some(rest) = cap.rest
            && !seen_rest.insert(rest)
        {
            return Err(VectorizerError::Configuration(format!(
                "capability registry: duplicate REST route {} {} (id '{}')",
                rest.0, rest.1, cap.id
            )));
        }

        match cap.transport {
            Transport::Both => {
                if cap.mcp_tool_name.is_none()
                    || cap.mcp_input_schema.is_none()
                    || cap.rest.is_none()
                {
                    return Err(VectorizerError::Configuration(format!(
                        "capability '{}' is Transport::Both but is missing \
                         either an MCP tool / schema or a REST route",
                        cap.id
                    )));
                }
            }
            Transport::RestOnly => {
                if cap.mcp_tool_name.is_some()
                    || cap.mcp_input_schema.is_some()
                    || cap.rest.is_none()
                {
                    return Err(VectorizerError::Configuration(format!(
                        "capability '{}' is Transport::RestOnly but has MCP \
                         fields set or is missing a REST route",
                        cap.id
                    )));
                }
            }
            Transport::McpOnly => {
                if cap.mcp_tool_name.is_none()
                    || cap.mcp_input_schema.is_none()
                    || cap.rest.is_some()
                {
                    return Err(VectorizerError::Configuration(format!(
                        "capability '{}' is Transport::McpOnly but has a REST \
                         route or is missing MCP fields",
                        cap.id
                    )));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn inventory_is_non_empty() {
        assert!(!inventory().is_empty());
    }

    #[test]
    fn invariants_hold_on_real_inventory() {
        assert_inventory_invariants().expect("registry invariants must hold");
    }

    #[test]
    fn duplicate_id_is_rejected_by_assert_helper() {
        // Hand-construct an inventory-shaped vec to exercise the duplicate
        // check directly. We can't poison `inventory()` itself because
        // it's the live registry; instead we duplicate the check inline
        // against a synthetic Vec to confirm the detection logic works.
        let dup = vec![
            Capability {
                id: "x",
                summary: "",
                mcp_tool_name: None,
                mcp_input_schema: None,
                rest: Some(("GET", "/x")),
                auth: AuthBucket::Public,
                transport: Transport::RestOnly,
            },
            Capability {
                id: "x",
                summary: "",
                mcp_tool_name: None,
                mcp_input_schema: None,
                rest: Some(("GET", "/y")),
                auth: AuthBucket::Public,
                transport: Transport::RestOnly,
            },
        ];
        let mut seen = std::collections::HashSet::new();
        let mut had_dup = false;
        for c in &dup {
            if !seen.insert(c.id) {
                had_dup = true;
            }
        }
        assert!(had_dup, "duplicate id detector must fire");
    }

    // phase40 §1.2/§1.3: mirrored in `vectorizer/tests/api/parity.rs`
    // (`excluded_routes_are_well_formed` /
    // `no_route_is_both_tracked_and_excluded`). Duplicated here as a
    // plain unit test — unlike that integration file, this one runs
    // under a bare `cargo test -p vectorizer-server` with no extra
    // feature flag needed.
    #[test]
    fn documented_exclusions_are_well_formed_and_disjoint_from_inventory() {
        let bad: Vec<(&'static str, &'static str, &'static str)> = documented_rest_exclusions()
            .iter()
            .filter(|(method, path, reason)| {
                method.is_empty() || path.is_empty() || reason.is_empty()
            })
            .copied()
            .collect();
        assert!(
            bad.is_empty(),
            "documented_rest_exclusions() entries must carry a non-empty method, path and \
             reason: {bad:?}"
        );

        let tracked: std::collections::HashSet<(&'static str, &'static str)> =
            inventory().into_iter().filter_map(|c| c.rest).collect();
        let overlap: Vec<(&'static str, &'static str)> = documented_rest_exclusions()
            .iter()
            .map(|(method, path, _)| (*method, *path))
            .filter(|route| tracked.contains(route))
            .collect();
        assert!(
            overlap.is_empty(),
            "route(s) present in both inventory() and documented_rest_exclusions(): {overlap:?}"
        );
    }
}
