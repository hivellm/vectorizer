//! MCP Tools definitions - Individual Focused Tools

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{Value, json};

/// Convert a static `json!({ ... })` literal into the `Arc<JsonObject>`
/// shape expected by `Tool::input_schema`.
///
/// Every call site in this module passes a top-level `{ "type": "object", ... }`
/// literal, so the `Value::Object(map)` arm is the only one that can fire at
/// runtime. The `unreachable!()` arm exists purely to satisfy the exhaustive
/// match — flipping `unwrap_used = "deny"` (phase4_enforce-no-unwrap-policy)
/// would otherwise reject the prior `.as_object().unwrap()` pattern that this
/// helper replaces in 30+ places.
fn schema(value: Value) -> Arc<serde_json::Map<String, Value>> {
    match value {
        Value::Object(map) => Arc::new(map),
        _ => unreachable!("schema() called with non-object json! literal"),
    }
}

/// Convenience wrapper around the rmcp 1.x `Tool` builder.
///
/// rmcp 1.5 marked `Tool` as `#[non_exhaustive]`, so struct-literal
/// construction (as we used under rmcp 0.10) is no longer legal from
/// outside the crate. This helper rebuilds the same shape via
/// `Tool::new(...).with_title(...).with_annotations(...)` so each of
/// the ~30 tool declarations below stays compact.
fn mk_tool(
    name: &'static str,
    title: &'static str,
    description: &'static str,
    input_schema: Value,
    annotations: ToolAnnotations,
) -> Tool {
    Tool::new(name, description, schema(input_schema))
        .with_title(title)
        .with_annotations(annotations)
}

pub fn get_mcp_tools() -> Vec<Tool> {
    vec![
        // =============================================
        // Core Collection/Vector Operations (9 tools)
        // =============================================

        // 1. List Collections
        mk_tool(
            "list_collections",
            "List Collections",
            "List all available collections with metadata including vector count, dimension, and configuration.",
            json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 2. Create Collection
        mk_tool(
            "create_collection",
            "Create Collection",
            "Create a new vector collection with specified dimension and distance metric.",
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "dimension": {
                        "type": "integer",
                        "description": "Vector dimension"
                    },
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
            }),
            ToolAnnotations::new().read_only(false),
        ),
        // 3. Get Collection Info
        mk_tool(
            "get_collection_info",
            "Get Collection Info",
            "Get detailed information about a specific collection including stats and configuration.",
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["name"]
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 4. Insert Text
        mk_tool(
            "insert_text",
            "Insert Text",
            "Insert a single text into a collection with automatic embedding generation.",
            json!({
                "type": "object",
                "properties": {
                    "collection_name": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to insert"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional metadata"
                    }
                },
                "required": ["collection_name", "text"]
            }),
            ToolAnnotations::new().read_only(false),
        ),
        // 5. Get Vector
        mk_tool(
            "get_vector",
            "Get Vector",
            "Retrieve a specific vector by ID from a collection.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "vector_id": {
                        "type": "string",
                        "description": "Vector ID"
                    }
                },
                "required": ["collection", "vector_id"]
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 6. Update Vector
        mk_tool(
            "update_vector",
            "Update Vector",
            "Update an existing vector with new text and/or metadata.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "vector_id": {
                        "type": "string",
                        "description": "Vector ID"
                    },
                    "text": {
                        "type": "string",
                        "description": "New text content"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional metadata"
                    }
                },
                "required": ["collection", "vector_id"]
            }),
            ToolAnnotations::new().read_only(false),
        ),
        // 7. Delete Vector
        mk_tool(
            "delete_vector",
            "Delete Vector",
            "Delete one or more vectors by ID from a collection.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "vector_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of vector IDs to delete"
                    }
                },
                "required": ["collection", "vector_ids"]
            }),
            ToolAnnotations::new().read_only(false),
        ),
        // 8. Multi-Collection Search
        mk_tool(
            "multi_collection_search",
            "Multi-Collection Search",
            "Search across multiple collections simultaneously with results from each.",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "collections": {
                        "type": "array",
                        "items": {"type": "string"},
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 9. Basic Search
        mk_tool(
            "search",
            "Basic Vector Search",
            "Basic vector similarity search in a single collection.",
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // =============================================
        // Search Operations (3 tools)
        // =============================================

        // 10. Intelligent Search
        mk_tool(
            "search_intelligent",
            "Intelligent Search",
            "AI-powered search with automatic query expansion and result deduplication across collections.",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "collections": {
                        "type": "array",
                        "items": {"type": "string"},
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 11. Semantic Search
        mk_tool(
            "search_semantic",
            "Semantic Search",
            "Semantic search with basic reranking for better relevance.",
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 12. Extra Search (Combined)
        mk_tool(
            "search_extra",
            "Combined Search",
            "Combined search that concatenates results from multiple search strategies (basic, intelligent, semantic).",
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
                    "strategies": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 13. Hybrid Search (Dense + Sparse)
        mk_tool(
            "search_hybrid",
            "Hybrid Search",
            "Hybrid search combining dense (HNSW) and sparse vector search for optimal results.",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (will be converted to dense embedding)"
                    },
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "query_sparse": {
                        "type": "object",
                        "description": "Optional sparse vector query (indices and values arrays)",
                        "properties": {
                            "indices": {
                                "type": "array",
                                "items": {"type": "integer"},
                                "description": "Non-zero indices"
                            },
                            "values": {
                                "type": "array",
                                "items": {"type": "number"},
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // =============================================
        // Discovery Operations (2 tools)
        // =============================================

        // 13. Filter Collections
        mk_tool(
            "filter_collections",
            "Filter Collections",
            "Filter collections by name patterns with include/exclude support.",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Query to filter collections (collection names or keywords)"
                    },
                    "include": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Include patterns (optional)"
                    },
                    "exclude": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Exclude patterns (optional)"
                    }
                },
                "required": ["query"]
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 14. Expand Queries
        mk_tool(
            "expand_queries",
            "Expand Queries",
            "Generate query variations and expansions for broader search coverage.",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Base query to expand"
                    },
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // =============================================
        // File Operations (5 tools)
        // =============================================

        // 15. Get File Content
        mk_tool(
            "get_file_content",
            "Get File Content",
            "Retrieve complete file content from a collection.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "File path"
                    },
                    "max_size_kb": {
                        "type": "integer",
                        "description": "Maximum file size in KB",
                        "default": 500,
                        "minimum": 1,
                        "maximum": 5000
                    }
                },
                "required": ["collection", "file_path"]
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 16. List Files
        mk_tool(
            "list_files",
            "List Files",
            "List all indexed files in a collection with metadata and filters.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "filter_by_type": {
                        "type": "array",
                        "items": {"type": "string"},
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 17. Get File Chunks
        mk_tool(
            "get_file_chunks",
            "Get File Chunks",
            "Retrieve file chunks in original order for progressive reading.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "File path"
                    },
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 18. Get Project Outline
        mk_tool(
            "get_project_outline",
            "Get Project Outline",
            "Generate hierarchical project structure overview from indexed files.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // 19. Get Related Files
        mk_tool(
            "get_related_files",
            "Get Related Files",
            "Find semantically related files using vector similarity.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "File path"
                    },
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // =============================================
        // Graph Operations (6 tools)
        // =============================================

        // Graph List Nodes
        mk_tool(
            "graph_list_nodes",
            "List Graph Nodes",
            "List all nodes in a collection's graph with their metadata.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["collection"]
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // Graph Get Neighbors
        mk_tool(
            "graph_get_neighbors",
            "Get Graph Node Neighbors",
            "Get all neighbors of a specific node in the graph with their relationships.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "node_id": {
                        "type": "string",
                        "description": "Node identifier"
                    }
                },
                "required": ["collection", "node_id"]
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // Graph Find Related
        mk_tool(
            "graph_find_related",
            "Find Related Nodes",
            "Find all nodes related to a given node within N hops in the graph.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "node_id": {
                        "type": "string",
                        "description": "Starting node identifier"
                    },
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
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // Graph Find Path
        mk_tool(
            "graph_find_path",
            "Find Path Between Nodes",
            "Find the shortest path between two nodes in the graph.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "source": {
                        "type": "string",
                        "description": "Source node identifier"
                    },
                    "target": {
                        "type": "string",
                        "description": "Target node identifier"
                    }
                },
                "required": ["collection", "source", "target"]
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // Graph Create Edge
        mk_tool(
            "graph_create_edge",
            "Create Graph Edge",
            "Create an explicit edge/relationship between two nodes in the graph.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "source": {
                        "type": "string",
                        "description": "Source node identifier"
                    },
                    "target": {
                        "type": "string",
                        "description": "Target node identifier"
                    },
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
            }),
            ToolAnnotations::new().read_only(false),
        ),
        // Graph Delete Edge
        mk_tool(
            "graph_delete_edge",
            "Delete Graph Edge",
            "Delete an edge/relationship from the graph.",
            json!({
                "type": "object",
                "properties": {
                    "edge_id": {
                        "type": "string",
                        "description": "Edge identifier to delete"
                    }
                },
                "required": ["edge_id"]
            }),
            ToolAnnotations::new().read_only(false),
        ),
        // Graph Discover Edges
        mk_tool(
            "graph_discover_edges",
            "Discover Graph Edges",
            "Automatically discover and create SIMILAR_TO edges between nodes based on semantic similarity. Can discover for a specific node or entire collection.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
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
            }),
            ToolAnnotations::new().read_only(false),
        ),
        // Graph Discover Status
        mk_tool(
            "graph_discover_status",
            "Get Graph Discovery Status",
            "Get discovery status for a collection, showing how many nodes have edges and overall progress.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["collection"]
            }),
            ToolAnnotations::new().read_only(true),
        ),
        // =============================================
        // Collection Maintenance Tools (3 tools)
        // =============================================

        // List Empty Collections
        mk_tool(
            "list_empty_collections",
            "List Empty Collections",
            "List all collections that have zero vectors. Useful for identifying collections that can be cleaned up.",
            json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
        // Cleanup Empty Collections
        mk_tool(
            "cleanup_empty_collections",
            "Cleanup Empty Collections",
            "Delete all empty collections (collections with zero vectors). Use dry_run=true to preview what would be deleted without actually deleting.",
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
            }),
            ToolAnnotations::new().read_only(false),
        ),
        // Get Collection Stats
        mk_tool(
            "get_collection_stats",
            "Get Collection Statistics",
            "Get detailed statistics for a collection including vector count, dimension, and whether it's empty.",
            json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["collection"]
            }),
            ToolAnnotations::new().read_only(true).idempotent(true),
        ),
    ]
}

/// Build the MCP tool list from the capability registry.
///
/// Returns one [`Tool`] per registry entry whose `mcp_tool_name` is
/// `Some` (i.e. `Transport::Both` or `Transport::McpOnly`). The hand-
/// written [`get_mcp_tools`] above is still the live source served to
/// clients — this helper exists so that, as the registry grows to cover
/// every entry, `get_mcp_tools` can be flipped over in a single change.
///
/// Until then, the parity test asserts that for every shared name, the
/// registry-built tool and the legacy tool agree on `input_schema` —
/// catching schema drift between the two sources.
pub fn tools_from_inventory() -> Vec<Tool> {
    use crate::server::capabilities::inventory;

    inventory()
        .into_iter()
        .filter_map(|cap| {
            let name = cap.mcp_tool_name?;
            let schema_fn = cap.mcp_input_schema?;
            // rmcp 1.x `Tool` is `#[non_exhaustive]`, so we can't use
            // struct-literal syntax. `Tool::new` accepts `Into<Cow<'static,
            // str>>` for name + description — `String` satisfies the bound
            // via `Cow::Owned`.
            Some(Tool::new(
                name.to_string(),
                cap.summary.to_string(),
                schema(schema_fn()),
            ))
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn registry_tools_are_a_subset_of_legacy_tools() {
        let legacy_names: std::collections::HashSet<String> = get_mcp_tools()
            .into_iter()
            .map(|t| t.name.into_owned())
            .collect();
        for tool in tools_from_inventory() {
            let name = tool.name.into_owned();
            assert!(
                legacy_names.contains(&name),
                "registry exposes MCP tool '{}' that no legacy entry covers \
                 — either add it to get_mcp_tools() (Stage C) or remove it \
                 from the registry",
                name
            );
        }
    }

    #[test]
    fn registry_and_legacy_agree_on_overlapping_input_schemas() {
        // Build a name -> input_schema lookup for the legacy list.
        let legacy: std::collections::HashMap<String, serde_json::Value> = get_mcp_tools()
            .into_iter()
            .map(|t| {
                let name = t.name.into_owned();
                let schema = serde_json::Value::Object((*t.input_schema).clone());
                (name, schema)
            })
            .collect();

        for tool in tools_from_inventory() {
            let name = tool.name.clone().into_owned();
            let registry_schema = serde_json::Value::Object((*tool.input_schema).clone());
            let legacy_schema = legacy
                .get(&name)
                .expect("subset test should have caught a missing legacy entry");
            assert_eq!(
                &registry_schema, legacy_schema,
                "MCP tool '{}': registry-built input schema diverges from \
                 legacy hand-written schema. Reconcile the two before \
                 flipping get_mcp_tools() to use the registry.",
                name
            );
        }
    }
}
