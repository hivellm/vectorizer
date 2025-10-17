//! MCP Tools definitions - Unified Interface

use std::borrow::Cow;

use rmcp::model::{Tool, ToolAnnotations};
use serde_json::json;

pub fn get_mcp_tools() -> Vec<Tool> {
    vec![
        // 1. Unified Search Tool
        Tool {
            name: Cow::Borrowed("search"),
            title: Some("Unified Search".to_string()),
            description: Some(Cow::Borrowed(
                "Unified search interface supporting multiple search strategies. \n\n\
                Available types:\n\
                - 'basic': Simple vector search with similarity ranking\n\
                - 'intelligent': AI-powered search with query expansion, MMR diversification, and deduplication\n\
                - 'semantic': Advanced semantic search with reranking algorithms and similarity thresholds\n\
                - 'contextual': Context-aware search with metadata filtering and contextual reranking\n\
                - 'multi_collection': Search across multiple collections with cross-collection ranking\n\
                - 'batch': Execute multiple search queries in a single batch operation\n\
                - 'by_file_type': Semantic search filtered by specific file extensions\n\n\
                Each type has specific parameters. See schema for details."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "search_type": {
                        "type": "string",
                        "enum": ["basic", "intelligent", "semantic", "contextual", "multi_collection", "batch", "by_file_type"],
                        "description": "Search strategy to use"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query (required for all types except batch)"
                    },
                    "collection": {
                        "type": "string",
                        "description": "Collection name (required for basic, semantic, contextual, batch, by_file_type)"
                    },
                    "collections": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of collection names (for intelligent, multi_collection)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results to return",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 100
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum results (for intelligent, semantic, contextual)",
                        "default": 10
                    },
                    "max_per_collection": {
                        "type": "integer",
                        "description": "Max results per collection (for multi_collection)",
                        "default": 5
                    },
                    "max_total_results": {
                        "type": "integer",
                        "description": "Total max results (for multi_collection)",
                        "default": 20
                    },
                    "domain_expansion": {
                        "type": "boolean",
                        "description": "Enable domain-specific query expansion (intelligent)",
                        "default": true
                    },
                    "technical_focus": {
                        "type": "boolean",
                        "description": "Prioritize technical content (intelligent)",
                        "default": true
                    },
                    "mmr_enabled": {
                        "type": "boolean",
                        "description": "Enable MMR diversification (intelligent)",
                        "default": true
                    },
                    "mmr_lambda": {
                        "type": "number",
                        "description": "MMR balance parameter 0.0-1.0 (intelligent)",
                        "default": 0.7
                    },
                    "semantic_reranking": {
                        "type": "boolean",
                        "description": "Enable semantic reranking (semantic)",
                        "default": true
                    },
                    "cross_encoder_reranking": {
                        "type": "boolean",
                        "description": "Enable cross-encoder reranking (semantic)",
                        "default": false
                    },
                    "similarity_threshold": {
                        "type": "number",
                        "description": "Minimum similarity score 0.0-1.0 (semantic)",
                        "default": 0.5
                    },
                    "context_filters": {
                        "type": "object",
                        "description": "Metadata filters (contextual)"
                    },
                    "context_reranking": {
                        "type": "boolean",
                        "description": "Enable context-aware reranking (contextual)",
                        "default": true
                    },
                    "context_weight": {
                        "type": "number",
                        "description": "Weight of context factors 0.0-1.0 (contextual)",
                        "default": 0.3
                    },
                    "cross_collection_reranking": {
                        "type": "boolean",
                        "description": "Enable cross-collection reranking (multi_collection)",
                        "default": true
                    },
                    "queries": {
                        "type": "array",
                        "description": "Array of query objects for batch search",
                        "items": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string"},
                                "limit": {"type": "integer", "default": 10}
                            }
                        }
                    },
                    "file_types": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File extensions to search (by_file_type)"
                    },
                    "return_full_files": {
                        "type": "boolean",
                        "description": "Return complete file content (by_file_type)",
                        "default": false
                    }
                },
                "required": ["search_type"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 2. Unified Collection Tool
        Tool {
            name: Cow::Borrowed("collection"),
            title: Some("Collection Management".to_string()),
            description: Some(Cow::Borrowed(
                "Unified interface for collection management operations.\n\n\
                Available types:\n\
                - 'list': List all available collections with metadata\n\
                - 'create': Create a new vector collection\n\
                - 'get_info': Get detailed information about a specific collection\n\
                - 'delete': Delete a collection permanently\n\n\
                Each type has specific parameters. See schema for details."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["list", "create", "get_info", "delete"],
                        "description": "Collection operation to perform"
                    },
                    "name": {
                        "type": "string",
                        "description": "Collection name (required for create, get_info, delete)"
                    },
                    "dimension": {
                        "type": "integer",
                        "description": "Vector dimension (required for create)"
                    },
                    "metric": {
                        "type": "string",
                        "enum": ["cosine", "euclidean"],
                        "description": "Distance metric (for create)",
                        "default": "cosine"
                    }
                },
                "required": ["operation"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },

        // 3. Unified Vector Tool
        Tool {
            name: Cow::Borrowed("vector"),
            title: Some("Vector Operations".to_string()),
            description: Some(Cow::Borrowed(
                "Unified interface for vector CRUD operations.\n\n\
                Available types:\n\
                - 'get': Retrieve a specific vector by ID\n\
                - 'update': Update an existing vector with new text/metadata\n\
                - 'delete': Delete specific vectors by their IDs\n\n\
                Each type has specific parameters. See schema for details."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["get", "update", "delete"],
                        "description": "Vector operation to perform"
                    },
                    "collection": {
                        "type": "string",
                        "description": "Collection name (required for all operations)"
                    },
                    "vector_id": {
                        "type": "string",
                        "description": "Vector ID (required for get, update)"
                    },
                    "vector_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of vector IDs (required for delete)"
                    },
                    "text": {
                        "type": "string",
                        "description": "New text content (for update)"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional metadata (for update)"
                    }
                },
                "required": ["operation", "collection"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },

        // 4. Unified Insert Tool
        Tool {
            name: Cow::Borrowed("insert"),
            title: Some("Insert Operations".to_string()),
            description: Some(Cow::Borrowed(
                "Unified interface for inserting data into collections.\n\n\
                Available types:\n\
                - 'single': Insert a single text with automatic embedding\n\
                - 'batch': Insert multiple texts in a batch operation\n\
                - 'structured': Insert texts with explicit IDs and metadata\n\n\
                Each type has specific parameters. See schema for details."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "insert_type": {
                        "type": "string",
                        "enum": ["single", "batch", "structured"],
                        "description": "Type of insert operation"
                    },
                    "collection": {
                        "type": "string",
                        "description": "Collection name (required for all types)"
                    },
                    "collection_name": {
                        "type": "string",
                        "description": "Collection name (alias for single type)"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to insert (required for single)"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional metadata (for single)"
                    },
                    "texts": {
                        "type": "array",
                        "description": "Array of texts (for batch) or objects (for structured)",
                        "items": {
                            "oneOf": [
                                {"type": "string"},
                                {
                                    "type": "object",
                                    "properties": {
                                        "id": {"type": "string"},
                                        "text": {"type": "string"},
                                        "metadata": {"type": "object"}
                                    },
                                    "required": ["id", "text"]
                                }
                            ]
                        }
                    }
                },
                "required": ["insert_type"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },

        // 5. Unified Batch Operations Tool
        Tool {
            name: Cow::Borrowed("batch_operations"),
            title: Some("Batch Operations".to_string()),
            description: Some(Cow::Borrowed(
                "Unified interface for batch vector operations.\n\n\
                Available types:\n\
                - 'update': Update multiple vectors in a single operation\n\
                - 'delete': Delete multiple vectors in a single operation\n\
                - 'search': Execute multiple search queries in a batch\n\n\
                Each type has specific parameters. See schema for details."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "batch_type": {
                        "type": "string",
                        "enum": ["update", "delete", "search"],
                        "description": "Type of batch operation"
                    },
                    "collection": {
                        "type": "string",
                        "description": "Collection name (required for all types)"
                    },
                    "updates": {
                        "type": "array",
                        "description": "Array of update objects (for update)",
                        "items": {
                            "type": "object",
                            "properties": {
                                "vector_id": {"type": "string"},
                                "text": {"type": "string"},
                                "metadata": {"type": "object"}
                            },
                            "required": ["vector_id"]
                        }
                    },
                    "vector_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of vector IDs to delete (for delete)"
                    },
                    "queries": {
                        "type": "array",
                        "description": "Array of query objects (for search)",
                        "items": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string"},
                                "limit": {"type": "integer", "default": 10}
                            },
                            "required": ["query"]
                        }
                    }
                },
                "required": ["batch_type", "collection"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },

        // 6. Unified Discovery Tool
        Tool {
            name: Cow::Borrowed("discovery"),
            title: Some("Discovery Pipeline".to_string()),
            description: Some(Cow::Borrowed(
                "Unified interface for discovery pipeline and utilities.\n\n\
                Available types:\n\
                - 'full_pipeline': Complete discovery pipeline with filtering, scoring, expansion, search, ranking, compression, and prompt generation\n\
                - 'filter_collections': Pre-filter collections by name patterns with stopword removal\n\
                - 'score_collections': Rank collections by relevance using name match, term boost, and signals\n\
                - 'expand_queries': Generate query variations (definition, features, architecture, API, performance)\n\
                - 'broad_discovery': Multi-query broad search with MMR diversification and deduplication\n\
                - 'semantic_focus': Deep semantic search in specific collection with reranking\n\
                - 'promote_readme': Boost README files to the top of search results\n\
                - 'compress_evidence': Extract key sentences with citations from chunks\n\
                - 'build_answer_plan': Organize bullets into structured sections\n\
                - 'render_llm_prompt': Generate compact, structured prompt for LLM\n\n\
                Each type has specific parameters. See schema for details."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "discovery_type": {
                        "type": "string",
                        "enum": ["full_pipeline", "filter_collections", "score_collections", "expand_queries", "broad_discovery", "semantic_focus", "promote_readme", "compress_evidence", "build_answer_plan", "render_llm_prompt"],
                        "description": "Type of discovery operation"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query (required for most types)"
                    },
                    "include_collections": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Collections to include (for full_pipeline, filter_collections)"
                    },
                    "exclude_collections": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Collections to exclude (for full_pipeline, filter_collections)"
                    },
                    "max_bullets": {
                        "type": "integer",
                        "default": 20,
                        "description": "Maximum evidence bullets (for full_pipeline, compress_evidence)"
                    },
                    "broad_k": {
                        "type": "integer",
                        "default": 50,
                        "description": "Broad search results (for full_pipeline)"
                    },
                    "focus_k": {
                        "type": "integer",
                        "default": 15,
                        "description": "Focus search results per collection (for full_pipeline)"
                    },
                    "include": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Include patterns (for filter_collections)"
                    },
                    "exclude": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Exclude patterns (for filter_collections)"
                    },
                    "name_match_weight": {
                        "type": "number",
                        "default": 0.4,
                        "description": "Weight for name matching (for score_collections)"
                    },
                    "term_boost_weight": {
                        "type": "number",
                        "default": 0.3,
                        "description": "Weight for term boost (for score_collections)"
                    },
                    "signal_boost_weight": {
                        "type": "number",
                        "default": 0.3,
                        "description": "Weight for signals (for score_collections)"
                    },
                    "max_expansions": {
                        "type": "integer",
                        "default": 8,
                        "description": "Maximum expansions (for expand_queries)"
                    },
                    "include_definition": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include definition queries (for expand_queries)"
                    },
                    "include_features": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include features queries (for expand_queries)"
                    },
                    "include_architecture": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include architecture queries (for expand_queries)"
                    },
                    "queries": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Array of search queries (for broad_discovery, semantic_focus)"
                    },
                    "k": {
                        "type": "integer",
                        "description": "Maximum results (for broad_discovery, semantic_focus)"
                    },
                    "collection": {
                        "type": "string",
                        "description": "Target collection name (for semantic_focus)"
                    },
                    "chunks": {
                        "type": "array",
                        "description": "Array of scored chunks (for promote_readme, compress_evidence)"
                    },
                    "max_per_doc": {
                        "type": "integer",
                        "default": 3,
                        "description": "Max bullets per document (for compress_evidence)"
                    },
                    "bullets": {
                        "type": "array",
                        "description": "Array of evidence bullets (for build_answer_plan)"
                    },
                    "plan": {
                        "type": "object",
                        "description": "Answer plan with sections and bullets (for render_llm_prompt)"
                    }
                },
                "required": ["discovery_type"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },

        // 7. Unified File Operations Tool
        Tool {
            name: Cow::Borrowed("file_operations"),
            title: Some("File Operations".to_string()),
            description: Some(Cow::Borrowed(
                "Unified interface for file operations in collections.\n\n\
                Available types:\n\
                - 'get_content': Retrieve complete file content from a collection\n\
                - 'list_files': List all indexed files in a collection with metadata\n\
                - 'get_summary': Get extractive or structural summary of an indexed file\n\
                - 'get_chunks': Retrieve chunks in original file order for progressive reading\n\
                - 'get_outline': Generate hierarchical project structure overview\n\
                - 'get_related': Find semantically related files using vector similarity\n\n\
                Each type has specific parameters. See schema for details."
            )),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation_type": {
                        "type": "string",
                        "enum": ["get_content", "list_files", "get_summary", "get_chunks", "get_outline", "get_related"],
                        "description": "Type of file operation"
                    },
                    "collection": {
                        "type": "string",
                        "description": "Collection name (required for all types)"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "File path (required for get_content, get_summary, get_chunks, get_related)"
                    },
                    "max_size_kb": {
                        "type": "integer",
                        "description": "Maximum file size in KB (for get_content)",
                        "default": 500,
                        "minimum": 1,
                        "maximum": 5000
                    },
                    "filter_by_type": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Filter by file types (for list_files)"
                    },
                    "min_chunks": {
                        "type": "integer",
                        "description": "Minimum number of chunks (for list_files)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results (for list_files, get_related)",
                        "default": 100
                    },
                    "sort_by": {
                        "type": "string",
                        "enum": ["name", "size", "chunks", "recent"],
                        "description": "Sort order (for list_files)",
                        "default": "name"
                    },
                    "summary_type": {
                        "type": "string",
                        "enum": ["extractive", "structural", "both"],
                        "description": "Type of summary (for get_summary)",
                        "default": "both"
                    },
                    "max_sentences": {
                        "type": "integer",
                        "description": "Maximum sentences for extractive summary (for get_summary)",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 20
                    },
                    "start_chunk": {
                        "type": "integer",
                        "description": "Starting chunk index (for get_chunks)",
                        "default": 0,
                        "minimum": 0
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Number of chunks to retrieve (for get_chunks)",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 50
                    },
                    "include_context": {
                        "type": "boolean",
                        "description": "Include prev/next chunk hints (for get_chunks)",
                        "default": false
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum directory depth (for get_outline)",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 10
                    },
                    "include_summaries": {
                        "type": "boolean",
                        "description": "Include file summaries in outline (for get_outline)",
                        "default": false
                    },
                    "highlight_key_files": {
                        "type": "boolean",
                        "description": "Highlight important files like README (for get_outline)",
                        "default": true
                    },
                    "similarity_threshold": {
                        "type": "number",
                        "description": "Minimum similarity score 0.0-1.0 (for get_related)",
                        "default": 0.6,
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "include_reason": {
                        "type": "boolean",
                        "description": "Include explanation of why files are related (for get_related)",
                        "default": true
                    }
                },
                "required": ["operation_type", "collection"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
    ]
}
