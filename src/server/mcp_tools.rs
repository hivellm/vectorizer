//! MCP Tools definitions - Individual Focused Tools

use std::borrow::Cow;

use rmcp::model::{Tool, ToolAnnotations};
use serde_json::json;

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
                "List all available collections with metadata including vector count, dimension, and configuration."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 2. Create Collection
        Tool {
            name: Cow::Borrowed("create_collection"),
            title: Some("Create Collection".to_string()),
            description: Some(Cow::Borrowed(
                "Create a new vector collection with specified dimension and distance metric."
            )),
            input_schema: json!({
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
                    }
                },
                "required": ["name", "dimension"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },

        // 3. Get Collection Info
        Tool {
            name: Cow::Borrowed("get_collection_info"),
            title: Some("Get Collection Info".to_string()),
            description: Some(Cow::Borrowed(
                "Get detailed information about a specific collection including stats and configuration."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Collection name"
                    }
                },
                "required": ["name"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 4. Insert Text
        Tool {
            name: Cow::Borrowed("insert_text"),
            title: Some("Insert Text".to_string()),
            description: Some(Cow::Borrowed(
                "Insert a single text into a collection with automatic embedding generation."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },

        // 5. Get Vector
        Tool {
            name: Cow::Borrowed("get_vector"),
            title: Some("Get Vector".to_string()),
            description: Some(Cow::Borrowed(
                "Retrieve a specific vector by ID from a collection."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 6. Update Vector
        Tool {
            name: Cow::Borrowed("update_vector"),
            title: Some("Update Vector".to_string()),
            description: Some(Cow::Borrowed(
                "Update an existing vector with new text and/or metadata."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },

        // 7. Delete Vector
        Tool {
            name: Cow::Borrowed("delete_vector"),
            title: Some("Delete Vector".to_string()),
            description: Some(Cow::Borrowed(
                "Delete one or more vectors by ID from a collection."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },

        // 8. Multi-Collection Search
        Tool {
            name: Cow::Borrowed("multi_collection_search"),
            title: Some("Multi-Collection Search".to_string()),
            description: Some(Cow::Borrowed(
                "Search across multiple collections simultaneously with results from each."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 9. Basic Search
        Tool {
            name: Cow::Borrowed("search"),
            title: Some("Basic Vector Search".to_string()),
            description: Some(Cow::Borrowed(
                "Basic vector similarity search in a single collection."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // =============================================
        // Search Operations (3 tools)
        // =============================================

        // 10. Intelligent Search
        Tool {
            name: Cow::Borrowed("search_intelligent"),
            title: Some("Intelligent Search".to_string()),
            description: Some(Cow::Borrowed(
                "AI-powered search with automatic query expansion and result deduplication across collections."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 11. Semantic Search
        Tool {
            name: Cow::Borrowed("search_semantic"),
            title: Some("Semantic Search".to_string()),
            description: Some(Cow::Borrowed(
                "Semantic search with basic reranking for better relevance."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 12. Extra Search (Combined)
        Tool {
            name: Cow::Borrowed("search_extra"),
            title: Some("Combined Search".to_string()),
            description: Some(Cow::Borrowed(
                "Combined search that concatenates results from multiple search strategies (basic, intelligent, semantic)."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // =============================================
        // Discovery Operations (2 tools)
        // =============================================

        // 13. Filter Collections
        Tool {
            name: Cow::Borrowed("filter_collections"),
            title: Some("Filter Collections".to_string()),
            description: Some(Cow::Borrowed(
                "Filter collections by name patterns with include/exclude support."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 14. Expand Queries
        Tool {
            name: Cow::Borrowed("expand_queries"),
            title: Some("Expand Queries".to_string()),
            description: Some(Cow::Borrowed(
                "Generate query variations and expansions for broader search coverage."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // =============================================
        // File Operations (5 tools)
        // =============================================

        // 15. Get File Content
        Tool {
            name: Cow::Borrowed("get_file_content"),
            title: Some("Get File Content".to_string()),
            description: Some(Cow::Borrowed(
                "Retrieve complete file content from a collection."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 16. List Files
        Tool {
            name: Cow::Borrowed("list_files"),
            title: Some("List Files".to_string()),
            description: Some(Cow::Borrowed(
                "List all indexed files in a collection with metadata and filters."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 17. Get File Chunks
        Tool {
            name: Cow::Borrowed("get_file_chunks"),
            title: Some("Get File Chunks".to_string()),
            description: Some(Cow::Borrowed(
                "Retrieve file chunks in original order for progressive reading."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 18. Get Project Outline
        Tool {
            name: Cow::Borrowed("get_project_outline"),
            title: Some("Get Project Outline".to_string()),
            description: Some(Cow::Borrowed(
                "Generate hierarchical project structure overview from indexed files."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 19. Get Related Files
        Tool {
            name: Cow::Borrowed("get_related_files"),
            title: Some("Get Related Files".to_string()),
            description: Some(Cow::Borrowed(
                "Find semantically related files using vector similarity."
            )),
            input_schema: json!({
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
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
    ]
}
