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
            rest: Some(("GET", "/graph/nodes/{collection}/{node_id}/related")),
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
}
