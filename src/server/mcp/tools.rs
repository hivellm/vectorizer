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

pub fn get_mcp_tools() -> Vec<Tool> {
    vec![
        // =============================================
        // Core Collection/Vector Operations (9 tools)
        // =============================================

        // 1. List Collections
        Tool {
            name: Cow::Borrowed("list_collections"),
            title: Some("List Collections".to_string()),
            description: Some(Cow::Borrowed(
                "List all available collections with metadata including vector count, dimension, and configuration.",
            )),
            input_schema: schema(json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 2. Create Collection
        Tool {
            name: Cow::Borrowed("create_collection"),
            title: Some("Create Collection".to_string()),
            description: Some(Cow::Borrowed(
                "Create a new vector collection with specified dimension and distance metric.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
            meta: None,
        },
        // 3. Get Collection Info
        Tool {
            name: Cow::Borrowed("get_collection_info"),
            title: Some("Get Collection Info".to_string()),
            description: Some(Cow::Borrowed(
                "Get detailed information about a specific collection including stats and configuration.",
            )),
            input_schema: schema(json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["name"]
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 4. Insert Text
        Tool {
            name: Cow::Borrowed("insert_text"),
            title: Some("Insert Text".to_string()),
            description: Some(Cow::Borrowed(
                "Insert a single text into a collection with automatic embedding generation.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
            meta: None,
        },
        // 5. Get Vector
        Tool {
            name: Cow::Borrowed("get_vector"),
            title: Some("Get Vector".to_string()),
            description: Some(Cow::Borrowed(
                "Retrieve a specific vector by ID from a collection.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 6. Update Vector
        Tool {
            name: Cow::Borrowed("update_vector"),
            title: Some("Update Vector".to_string()),
            description: Some(Cow::Borrowed(
                "Update an existing vector with new text and/or metadata.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
            meta: None,
        },
        // 7. Delete Vector
        Tool {
            name: Cow::Borrowed("delete_vector"),
            title: Some("Delete Vector".to_string()),
            description: Some(Cow::Borrowed(
                "Delete one or more vectors by ID from a collection.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
            meta: None,
        },
        // 8. Multi-Collection Search
        Tool {
            name: Cow::Borrowed("multi_collection_search"),
            title: Some("Multi-Collection Search".to_string()),
            description: Some(Cow::Borrowed(
                "Search across multiple collections simultaneously with results from each.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 9. Basic Search
        Tool {
            name: Cow::Borrowed("search"),
            title: Some("Basic Vector Search".to_string()),
            description: Some(Cow::Borrowed(
                "Basic vector similarity search in a single collection.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // =============================================
        // Search Operations (3 tools)
        // =============================================

        // 10. Intelligent Search
        Tool {
            name: Cow::Borrowed("search_intelligent"),
            title: Some("Intelligent Search".to_string()),
            description: Some(Cow::Borrowed(
                "AI-powered search with automatic query expansion and result deduplication across collections.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 11. Semantic Search
        Tool {
            name: Cow::Borrowed("search_semantic"),
            title: Some("Semantic Search".to_string()),
            description: Some(Cow::Borrowed(
                "Semantic search with basic reranking for better relevance.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 12. Extra Search (Combined)
        Tool {
            name: Cow::Borrowed("search_extra"),
            title: Some("Combined Search".to_string()),
            description: Some(Cow::Borrowed(
                "Combined search that concatenates results from multiple search strategies (basic, intelligent, semantic).",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 13. Hybrid Search (Dense + Sparse)
        Tool {
            name: Cow::Borrowed("search_hybrid"),
            title: Some("Hybrid Search".to_string()),
            description: Some(Cow::Borrowed(
                "Hybrid search combining dense (HNSW) and sparse vector search for optimal results.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // =============================================
        // Discovery Operations (2 tools)
        // =============================================

        // 13. Filter Collections
        Tool {
            name: Cow::Borrowed("filter_collections"),
            title: Some("Filter Collections".to_string()),
            description: Some(Cow::Borrowed(
                "Filter collections by name patterns with include/exclude support.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 14. Expand Queries
        Tool {
            name: Cow::Borrowed("expand_queries"),
            title: Some("Expand Queries".to_string()),
            description: Some(Cow::Borrowed(
                "Generate query variations and expansions for broader search coverage.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // =============================================
        // File Operations (5 tools)
        // =============================================

        // 15. Get File Content
        Tool {
            name: Cow::Borrowed("get_file_content"),
            title: Some("Get File Content".to_string()),
            description: Some(Cow::Borrowed(
                "Retrieve complete file content from a collection.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 16. List Files
        Tool {
            name: Cow::Borrowed("list_files"),
            title: Some("List Files".to_string()),
            description: Some(Cow::Borrowed(
                "List all indexed files in a collection with metadata and filters.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 17. Get File Chunks
        Tool {
            name: Cow::Borrowed("get_file_chunks"),
            title: Some("Get File Chunks".to_string()),
            description: Some(Cow::Borrowed(
                "Retrieve file chunks in original order for progressive reading.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 18. Get Project Outline
        Tool {
            name: Cow::Borrowed("get_project_outline"),
            title: Some("Get Project Outline".to_string()),
            description: Some(Cow::Borrowed(
                "Generate hierarchical project structure overview from indexed files.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // 19. Get Related Files
        Tool {
            name: Cow::Borrowed("get_related_files"),
            title: Some("Get Related Files".to_string()),
            description: Some(Cow::Borrowed(
                "Find semantically related files using vector similarity.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // =============================================
        // Graph Operations (6 tools)
        // =============================================

        // Graph List Nodes
        Tool {
            name: Cow::Borrowed("graph_list_nodes"),
            title: Some("List Graph Nodes".to_string()),
            description: Some(Cow::Borrowed(
                "List all nodes in a collection's graph with their metadata.",
            )),
            input_schema: schema(json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["collection"]
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // Graph Get Neighbors
        Tool {
            name: Cow::Borrowed("graph_get_neighbors"),
            title: Some("Get Graph Node Neighbors".to_string()),
            description: Some(Cow::Borrowed(
                "Get all neighbors of a specific node in the graph with their relationships.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // Graph Find Related
        Tool {
            name: Cow::Borrowed("graph_find_related"),
            title: Some("Find Related Nodes".to_string()),
            description: Some(Cow::Borrowed(
                "Find all nodes related to a given node within N hops in the graph.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // Graph Find Path
        Tool {
            name: Cow::Borrowed("graph_find_path"),
            title: Some("Find Path Between Nodes".to_string()),
            description: Some(Cow::Borrowed(
                "Find the shortest path between two nodes in the graph.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // Graph Create Edge
        Tool {
            name: Cow::Borrowed("graph_create_edge"),
            title: Some("Create Graph Edge".to_string()),
            description: Some(Cow::Borrowed(
                "Create an explicit edge/relationship between two nodes in the graph.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
            meta: None,
        },
        // Graph Delete Edge
        Tool {
            name: Cow::Borrowed("graph_delete_edge"),
            title: Some("Delete Graph Edge".to_string()),
            description: Some(Cow::Borrowed("Delete an edge/relationship from the graph.")),
            input_schema: schema(json!({
                "type": "object",
                "properties": {
                    "edge_id": {
                        "type": "string",
                        "description": "Edge identifier to delete"
                    }
                },
                "required": ["edge_id"]
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
            meta: None,
        },
        // Graph Discover Edges
        Tool {
            name: Cow::Borrowed("graph_discover_edges"),
            title: Some("Discover Graph Edges".to_string()),
            description: Some(Cow::Borrowed(
                "Automatically discover and create SIMILAR_TO edges between nodes based on semantic similarity. Can discover for a specific node or entire collection.",
            )),
            input_schema: schema(json!({
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
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
            meta: None,
        },
        // Graph Discover Status
        Tool {
            name: Cow::Borrowed("graph_discover_status"),
            title: Some("Get Graph Discovery Status".to_string()),
            description: Some(Cow::Borrowed(
                "Get discovery status for a collection, showing how many nodes have edges and overall progress.",
            )),
            input_schema: schema(json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["collection"]
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true)),
            meta: None,
        },
        // =============================================
        // Collection Maintenance Tools (3 tools)
        // =============================================

        // List Empty Collections
        Tool {
            name: Cow::Borrowed("list_empty_collections"),
            title: Some("List Empty Collections".to_string()),
            description: Some(Cow::Borrowed(
                "List all collections that have zero vectors. Useful for identifying collections that can be cleaned up.",
            )),
            input_schema: schema(json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
        // Cleanup Empty Collections
        Tool {
            name: Cow::Borrowed("cleanup_empty_collections"),
            title: Some("Cleanup Empty Collections".to_string()),
            description: Some(Cow::Borrowed(
                "Delete all empty collections (collections with zero vectors). Use dry_run=true to preview what would be deleted without actually deleting.",
            )),
            input_schema: schema(json!({
                "type": "object",
                "properties": {
                    "dry_run": {
                        "type": "boolean",
                        "description": "If true, only shows what would be deleted without actually deleting. Default: false",
                        "default": false
                    }
                },
                "required": []
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
            meta: None,
        },
        // Get Collection Stats
        Tool {
            name: Cow::Borrowed("get_collection_stats"),
            title: Some("Get Collection Statistics".to_string()),
            description: Some(Cow::Borrowed(
                "Get detailed statistics for a collection including vector count, dimension, and whether it's empty.",
            )),
            input_schema: schema(json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["collection"]
            })),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
            meta: None,
        },
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
            Some(Tool {
                name: Cow::Owned(name.to_string()),
                title: None,
                description: Some(Cow::Owned(cap.summary.to_string())),
                input_schema: schema(schema_fn()),
                output_schema: None,
                icons: None,
                annotations: None,
                meta: None,
            })
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
