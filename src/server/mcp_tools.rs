//! MCP Tools definitions

use rmcp::model::{Tool, ToolAnnotations};
use serde_json::json;
use std::borrow::Cow;

pub fn get_mcp_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: Cow::Borrowed("search_vectors"),
            title: Some("Search Vectors".to_string()),
            description: Some(Cow::Borrowed("Search for semantically similar content in a vector collection using advanced similarity algorithms. Returns ranked results with similarity scores and metadata. Supports configurable result limits and similarity thresholds.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string", 
                        "description": "Name of the vector collection to search in. Must be an existing collection.",
                        "examples": ["cmmv-core-docs", "vectorizer-docs"]
                    },
                    "query": {
                        "type": "string", 
                        "description": "Natural language search query. Will be automatically embedded using the collection's embedding model.",
                        "examples": ["CMMV framework architecture", "authentication system", "API documentation"]
                    },
                    "limit": {
                        "type": "integer", 
                        "description": "Maximum number of results to return. Higher values may impact performance.",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 100
                    }
                },
                "required": ["collection", "query"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("list_collections"),
            title: Some("List Collections".to_string()),
            description: Some(Cow::Borrowed("Retrieve a comprehensive list of all available vector collections in the system. Returns collection names, metadata, and basic statistics. Useful for discovering available data sources before performing searches.")),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "description": "No parameters required. Returns all collections regardless of filters."
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("create_collection"),
            title: Some("Create Collection".to_string()),
            description: Some(Cow::Borrowed("Create a new vector collection")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Collection name"},
                    "dimension": {"type": "integer", "description": "Vector dimension"},
                    "metric": {"type": "string", "enum": ["cosine", "euclidean"], "default": "cosine"}
                },
                "required": ["name", "dimension"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },
        Tool {
            name: Cow::Borrowed("get_collection_info"),
            title: Some("Get Collection Info".to_string()),
            description: Some(Cow::Borrowed("Get information about a specific collection")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Collection name"}
                },
                "required": ["name"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("delete_collection"),
            title: Some("Delete Collection".to_string()),
            description: Some(Cow::Borrowed("Delete a vector collection")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Collection name"}
                },
                "required": ["name"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false).destructive(true)),
        },
        Tool {
            name: Cow::Borrowed("insert_text"),
            title: Some("Insert Text".to_string()),
            description: Some(Cow::Borrowed("Insert text into a collection with automatic embedding")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection_name": {"type": "string", "description": "Collection name"},
                    "text": {"type": "string", "description": "Text to insert"},
                    "metadata": {"type": "object", "description": "Optional metadata"}
                },
                "required": ["collection_name", "text"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },
        Tool {
            name: Cow::Borrowed("batch_insert_texts"),
            title: Some("Batch Insert Texts".to_string()),
            description: Some(Cow::Borrowed("Insert multiple texts into a collection")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection_name": {"type": "string", "description": "Collection name"},
                    "texts": {"type": "array", "items": {"type": "string"}, "description": "Array of texts"}
                },
                "required": ["collection_name", "texts"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },
        Tool {
            name: Cow::Borrowed("embed_text"),
            title: Some("Embed Text".to_string()),
            description: Some(Cow::Borrowed("Generate vector embeddings for input text")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "Text to embed"}
                },
                "required": ["text"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("get_vector"),
            title: Some("Get Vector".to_string()),
            description: Some(Cow::Borrowed("Retrieve a specific vector by ID")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Collection name"},
                    "vector_id": {"type": "string", "description": "Vector ID"}
                },
                "required": ["collection", "vector_id"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("delete_vectors"),
            title: Some("Delete Vectors".to_string()),
            description: Some(Cow::Borrowed("Delete specific vectors from a collection by their IDs")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Collection name"},
                    "vector_ids": {"type": "array", "items": {"type": "string"}, "description": "Array of vector IDs"}
                },
                "required": ["collection", "vector_ids"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false).destructive(true)),
        },
        Tool {
            name: Cow::Borrowed("update_vector"),
            title: Some("Update Vector".to_string()),
            description: Some(Cow::Borrowed("Update an existing vector with new text and/or metadata")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Collection name"},
                    "vector_id": {"type": "string", "description": "Vector ID"},
                    "text": {"type": "string", "description": "New text content"},
                    "metadata": {"type": "object", "description": "Optional metadata"}
                },
                "required": ["collection", "vector_id"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },
        Tool {
            name: Cow::Borrowed("health_check"),
            title: Some("Health Check".to_string()),
            description: Some(Cow::Borrowed("Check the health and status of the vectorizer service")),
            input_schema: json!({"type": "object", "properties": {}}).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("insert_texts"),
            title: Some("Insert Texts".to_string()),
            description: Some(Cow::Borrowed("Insert text content into a collection with automatic embedding generation")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Collection name"},
                    "texts": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string", "description": "Unique ID for the text"},
                                "text": {"type": "string", "description": "Text content"},
                                "metadata": {"type": "object", "description": "Optional metadata"}
                            },
                            "required": ["id", "text"]
                        }
                    }
                },
                "required": ["collection", "texts"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },
        Tool {
            name: Cow::Borrowed("batch_search_vectors"),
            title: Some("Batch Search Vectors".to_string()),
            description: Some(Cow::Borrowed("Execute multiple semantic search queries in a single batch operation")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Collection name"},
                    "queries": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string", "description": "Search query text"},
                                "limit": {"type": "integer", "description": "Maximum results", "default": 10}
                            },
                            "required": ["query"]
                        }
                    }
                },
                "required": ["collection", "queries"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("batch_update_vectors"),
            title: Some("Batch Update Vectors".to_string()),
            description: Some(Cow::Borrowed("Update multiple existing vectors in a collection efficiently")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Collection name"},
                    "updates": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "vector_id": {"type": "string", "description": "Vector ID to update"},
                                "text": {"type": "string", "description": "New text content"},
                                "metadata": {"type": "object", "description": "Optional metadata"}
                            },
                            "required": ["vector_id"]
                        }
                    }
                },
                "required": ["collection", "updates"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false)),
        },
        Tool {
            name: Cow::Borrowed("batch_delete_vectors"),
            title: Some("Batch Delete Vectors".to_string()),
            description: Some(Cow::Borrowed("Delete multiple vectors from a collection in a single operation")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Collection name"},
                    "vector_ids": {"type": "array", "items": {"type": "string"}, "description": "Array of vector IDs to delete"}
                },
                "required": ["collection", "vector_ids"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(false).destructive(true)),
        },
        Tool {
            name: Cow::Borrowed("get_indexing_progress"),
            title: Some("Get Indexing Progress".to_string()),
            description: Some(Cow::Borrowed("Monitor the progress of ongoing indexing operations across all collections")),
            input_schema: json!({"type": "object", "properties": {}}).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        // Intelligent Search Tools
        Tool {
            name: Cow::Borrowed("intelligent_search"),
            title: Some("Intelligent Search".to_string()),
            description: Some(Cow::Borrowed("Advanced semantic search with AI-powered query expansion, intelligent deduplication, and Maximal Marginal Relevance (MMR) diversification. Automatically generates multiple query variations, applies domain-specific knowledge, and ensures diverse, high-quality results. Ideal for complex research queries and comprehensive information retrieval.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string", 
                        "description": "Primary search query in natural language. Will be automatically expanded with related terms and domain-specific variations.",
                        "examples": ["CMMV framework architecture", "authentication best practices", "API design patterns"]
                    },
                    "collections": {
                        "type": "array", 
                        "items": {"type": "string"}, 
                        "description": "Specific collections to search in. If not provided, searches all available collections.",
                        "examples": [["cmmv-core-docs", "cmmv-admin-docs"], ["vectorizer-docs"]]
                    },
                    "max_results": {
                        "type": "integer", 
                        "description": "Maximum number of results to return after deduplication and MMR diversification.",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 50
                    },
                    "domain_expansion": {
                        "type": "boolean", 
                        "description": "Enable automatic domain-specific query expansion. Generates related technical terms and synonyms.",
                        "default": true
                    },
                    "technical_focus": {
                        "type": "boolean", 
                        "description": "Prioritize technical content and boost scores for API documentation, code examples, and implementation details.",
                        "default": true
                    },
                    "mmr_enabled": {
                        "type": "boolean", 
                        "description": "Enable Maximal Marginal Relevance diversification to ensure result diversity and avoid redundant information.",
                        "default": true
                    },
                    "mmr_lambda": {
                        "type": "number", 
                        "description": "MMR balance parameter: 0.0 = maximum diversity, 1.0 = maximum relevance. Controls the trade-off between relevance and diversity.",
                        "default": 0.7,
                        "minimum": 0.0,
                        "maximum": 1.0
                    }
                },
                "required": ["query"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("multi_collection_search"),
            title: Some("Multi Collection Search".to_string()),
            description: Some(Cow::Borrowed("Search across multiple vector collections simultaneously with intelligent cross-collection ranking and result aggregation. Automatically balances results from different collections, applies collection-specific scoring bonuses, and provides unified ranking across all sources. Perfect for comprehensive research across multiple data sources.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string", 
                        "description": "Search query in natural language. Will be embedded and searched across all specified collections.",
                        "examples": ["CMMV framework components", "authentication patterns", "API design best practices"]
                    },
                    "collections": {
                        "type": "array", 
                        "items": {"type": "string"}, 
                        "description": "List of collection names to search across. Each collection will be searched independently.",
                        "examples": [["cmmv-core-docs", "cmmv-admin-docs", "cmmv-formbuilder-docs"], ["vectorizer-docs", "performance-docs"]]
                    },
                    "max_per_collection": {
                        "type": "integer", 
                        "description": "Maximum number of results to retrieve from each individual collection before cross-collection ranking.",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 20
                    },
                    "max_total_results": {
                        "type": "integer", 
                        "description": "Maximum total number of results to return after cross-collection reranking and aggregation.",
                        "default": 20,
                        "minimum": 1,
                        "maximum": 100
                    },
                    "cross_collection_reranking": {
                        "type": "boolean", 
                        "description": "Enable intelligent cross-collection reranking to ensure optimal result distribution and relevance across collections.",
                        "default": true
                    }
                },
                "required": ["query", "collections"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("semantic_search"),
            title: Some("Semantic Search".to_string()),
            description: Some(Cow::Borrowed("Advanced semantic search with sophisticated reranking algorithms including semantic similarity scoring, cross-encoder reranking, and similarity threshold filtering. Provides highly accurate results by understanding context and meaning beyond simple keyword matching. Ideal for precise information retrieval with quality control.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string", 
                        "description": "Search query in natural language. Will be semantically analyzed and embedded for context-aware search.",
                        "examples": ["authentication middleware implementation", "database connection pooling", "error handling best practices"]
                    },
                    "collection": {
                        "type": "string", 
                        "description": "Name of the vector collection to search in. Must be an existing collection.",
                        "examples": ["cmmv-core-docs", "vectorizer-docs", "api-documentation"]
                    },
                    "max_results": {
                        "type": "integer", 
                        "description": "Maximum number of results to return after semantic reranking and filtering.",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 50
                    },
                    "semantic_reranking": {
                        "type": "boolean", 
                        "description": "Enable advanced semantic reranking using multiple similarity algorithms and context analysis.",
                        "default": true
                    },
                    "cross_encoder_reranking": {
                        "type": "boolean", 
                        "description": "Enable cross-encoder reranking for maximum precision. Uses advanced neural models for query-document matching.",
                        "default": false
                    },
                    "similarity_threshold": {
                        "type": "number", 
                        "description": "Minimum similarity score threshold (0.0-1.0). Results below this threshold will be filtered out.",
                        "default": 0.5,
                        "minimum": 0.0,
                        "maximum": 1.0
                    }
                },
                "required": ["query", "collection"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("contextual_search"),
            title: Some("Contextual Search".to_string()),
            description: Some(Cow::Borrowed("Context-aware semantic search with intelligent metadata filtering and contextual reranking. Combines semantic similarity with metadata-based filtering to provide highly relevant results based on specific context requirements. Supports complex filtering criteria and context-weighted scoring for precise information retrieval.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string", 
                        "description": "Search query in natural language. Will be analyzed in context of provided filters.",
                        "examples": ["API documentation", "authentication examples", "configuration settings"]
                    },
                    "collection": {
                        "type": "string", 
                        "description": "Name of the vector collection to search in. Must be an existing collection.",
                        "examples": ["cmmv-core-docs", "api-documentation", "configuration-docs"]
                    },
                    "context_filters": {
                        "type": "object", 
                        "description": "Metadata-based context filters to narrow down results. Supports nested filtering and multiple criteria.",
                        "examples": [
                            {"author": "John Doe", "version": "1.0"},
                            {"category": "authentication", "priority": "high"},
                            {"language": "typescript", "framework": "cmmv"}
                        ]
                    },
                    "max_results": {
                        "type": "integer", 
                        "description": "Maximum number of results to return after context filtering and reranking.",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 50
                    },
                    "context_reranking": {
                        "type": "boolean", 
                        "description": "Enable context-aware reranking that considers metadata relevance alongside semantic similarity.",
                        "default": true
                    },
                    "context_weight": {
                        "type": "number", 
                        "description": "Weight of context factors in final scoring (0.0-1.0). Higher values prioritize metadata matches over semantic similarity.",
                        "default": 0.3,
                        "minimum": 0.0,
                        "maximum": 1.0
                    }
                },
                "required": ["query", "collection"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        // Discovery System Tools (9 functions + full pipeline)
        Tool {
            name: Cow::Borrowed("discover"),
            title: Some("Discovery Pipeline".to_string()),
            description: Some(Cow::Borrowed("Complete discovery pipeline that chains filtering, scoring, expansion, search, ranking, compression, and prompt generation. Returns a structured LLM-ready prompt with evidence.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "User question or search query"},
                    "include_collections": {"type": "array", "items": {"type": "string"}, "description": "Collections to include (glob patterns like 'vectorizer*')"},
                    "exclude_collections": {"type": "array", "items": {"type": "string"}, "description": "Collections to exclude"},
                    "max_bullets": {"type": "integer", "default": 20, "description": "Maximum evidence bullets"},
                    "broad_k": {"type": "integer", "default": 50, "description": "Broad search results"},
                    "focus_k": {"type": "integer", "default": 15, "description": "Focus search results per collection"}
                },
                "required": ["query"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("filter_collections"),
            title: Some("Filter Collections".to_string()),
            description: Some(Cow::Borrowed("Pre-filter collections by name patterns with stopword removal from query")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query for filtering"},
                    "include": {"type": "array", "items": {"type": "string"}, "description": "Include patterns (e.g., ['vectorizer*', '*-docs'])"},
                    "exclude": {"type": "array", "items": {"type": "string"}, "description": "Exclude patterns (e.g., ['*-test'])"}
                },
                "required": ["query"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("score_collections"),
            title: Some("Score Collections".to_string()),
            description: Some(Cow::Borrowed("Rank collections by relevance using name match, term boost, and signal boost (size, recency, tags)")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query for scoring"},
                    "name_match_weight": {"type": "number", "default": 0.4, "description": "Weight for name matching"},
                    "term_boost_weight": {"type": "number", "default": 0.3, "description": "Weight for term boost"},
                    "signal_boost_weight": {"type": "number", "default": 0.3, "description": "Weight for signals"}
                },
                "required": ["query"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("expand_queries"),
            title: Some("Expand Queries".to_string()),
            description: Some(Cow::Borrowed("Generate query variations (definition, features, architecture, API, performance, use cases)")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Original query to expand"},
                    "max_expansions": {"type": "integer", "default": 8, "description": "Maximum expansions"},
                    "include_definition": {"type": "boolean", "default": true},
                    "include_features": {"type": "boolean", "default": true},
                    "include_architecture": {"type": "boolean", "default": true}
                },
                "required": ["query"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("broad_discovery"),
            title: Some("Broad Discovery".to_string()),
            description: Some(Cow::Borrowed("Multi-query broad search with MMR diversification and deduplication")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "queries": {"type": "array", "items": {"type": "string"}, "description": "Array of search queries"},
                    "k": {"type": "integer", "default": 50, "description": "Maximum results"}
                },
                "required": ["queries"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("semantic_focus"),
            title: Some("Semantic Focus".to_string()),
            description: Some(Cow::Borrowed("Deep semantic search in specific collection with reranking and context window")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Target collection name"},
                    "queries": {"type": "array", "items": {"type": "string"}, "description": "Array of search queries"},
                    "k": {"type": "integer", "default": 15, "description": "Maximum results"}
                },
                "required": ["collection", "queries"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("promote_readme"),
            title: Some("Promote README".to_string()),
            description: Some(Cow::Borrowed("Boost README files to the top of search results")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chunks": {"type": "array", "description": "Array of scored chunks to promote"}
                },
                "required": ["chunks"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("compress_evidence"),
            title: Some("Compress Evidence".to_string()),
            description: Some(Cow::Borrowed("Extract key sentences (8-30 words) with citations from chunks")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "chunks": {"type": "array", "description": "Array of scored chunks"},
                    "max_bullets": {"type": "integer", "default": 20, "description": "Max bullets to extract"},
                    "max_per_doc": {"type": "integer", "default": 3, "description": "Max bullets per document"}
                },
                "required": ["chunks"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("build_answer_plan"),
            title: Some("Build Answer Plan".to_string()),
            description: Some(Cow::Borrowed("Organize bullets into structured sections (Definition, Features, Architecture, Performance, Integrations, Use Cases)")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "bullets": {"type": "array", "description": "Array of evidence bullets"}
                },
                "required": ["bullets"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("render_llm_prompt"),
            title: Some("Render LLM Prompt".to_string()),
            description: Some(Cow::Borrowed("Generate compact, structured prompt for LLM with instructions, evidence, and citations")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plan": {"type": "object", "description": "Answer plan with sections and bullets"}
                },
                "required": ["plan"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        // File Operations Tools (Priority 1)
        Tool {
            name: Cow::Borrowed("get_file_content"),
            title: Some("Get File Content".to_string()),
            description: Some(Cow::Borrowed("Retrieve complete file content from a collection. Use this instead of read_file for indexed files. Faster and provides metadata.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name (e.g., 'vectorizer-source', 'vectorizer-docs')"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Relative file path within collection (e.g., 'src/main.rs', 'README.md')"
                    },
                    "max_size_kb": {
                        "type": "integer",
                        "description": "Maximum file size in KB (default: 500, max: 5000)",
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
        Tool {
            name: Cow::Borrowed("list_files_in_collection"),
            title: Some("List Files in Collection".to_string()),
            description: Some(Cow::Borrowed("List all indexed files in a collection with metadata. Use this to explore project structure and discover available files.")),
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
                        "description": "Filter by file types (e.g., ['rs', 'md', 'toml'])"
                    },
                    "min_chunks": {
                        "type": "integer",
                        "description": "Minimum number of chunks (filters out small files)"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 100)",
                        "default": 100
                    },
                    "sort_by": {
                        "type": "string",
                        "enum": ["name", "size", "chunks", "recent"],
                        "description": "Sort order (default: 'name')",
                        "default": "name"
                    }
                },
                "required": ["collection"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("get_file_summary"),
            title: Some("Get File Summary".to_string()),
            description: Some(Cow::Borrowed("Get extractive or structural summary of an indexed file. More efficient than reading the entire file. Provides key points and outline.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Relative file path within collection"
                    },
                    "summary_type": {
                        "type": "string",
                        "enum": ["extractive", "structural", "both"],
                        "description": "Type of summary (default: 'both')",
                        "default": "both"
                    },
                    "max_sentences": {
                        "type": "integer",
                        "description": "Maximum sentences for extractive summary (default: 5)",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 20
                    }
                },
                "required": ["collection", "file_path"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("get_file_chunks_ordered"),
            title: Some("Get File Chunks Ordered".to_string()),
            description: Some(Cow::Borrowed("Retrieve chunks in original file order for progressive reading. Useful for streaming large files without loading everything at once.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Relative file path within collection"
                    },
                    "start_chunk": {
                        "type": "integer",
                        "description": "Starting chunk index (default: 0)",
                        "default": 0,
                        "minimum": 0
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Number of chunks to retrieve (default: 10)",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 50
                    },
                    "include_context": {
                        "type": "boolean",
                        "description": "Include prev/next chunk hints (default: false)",
                        "default": false
                    }
                },
                "required": ["collection", "file_path"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("get_project_outline"),
            title: Some("Get Project Outline".to_string()),
            description: Some(Cow::Borrowed("Generate hierarchical project structure overview. Shows directory tree, key files, and statistics. Perfect for understanding codebase organization.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum directory depth (default: 5)",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 10
                    },
                    "include_summaries": {
                        "type": "boolean",
                        "description": "Include file summaries in outline (default: false)",
                        "default": false
                    },
                    "highlight_key_files": {
                        "type": "boolean",
                        "description": "Highlight important files like README (default: true)",
                        "default": true
                    }
                },
                "required": ["collection"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("get_related_files"),
            title: Some("Get Related Files".to_string()),
            description: Some(Cow::Borrowed("Find semantically related files using vector similarity. Navigate codebase by discovering related modules, dependencies, and similar implementations.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Reference file path"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of related files (default: 5)",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 20
                    },
                    "similarity_threshold": {
                        "type": "number",
                        "description": "Minimum similarity score 0.0-1.0 (default: 0.6)",
                        "default": 0.6,
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "include_reason": {
                        "type": "boolean",
                        "description": "Include explanation of why files are related (default: true)",
                        "default": true
                    }
                },
                "required": ["collection", "file_path"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
        Tool {
            name: Cow::Borrowed("search_by_file_type"),
            title: Some("Search by File Type".to_string()),
            description: Some(Cow::Borrowed("Semantic search filtered by file type. Find configuration files, documentation, or code by specific extensions. Optionally return full file contents.")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "file_types": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "File extensions to search (e.g., ['yaml', 'toml', 'json'])"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results (default: 10)",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 50
                    },
                    "return_full_files": {
                        "type": "boolean",
                        "description": "Return complete file content (default: false)",
                        "default": false
                    }
                },
                "required": ["collection", "query", "file_types"]
            }).as_object().unwrap().clone().into(),
            output_schema: None,
            icons: None,
            annotations: Some(ToolAnnotations::new().read_only(true).idempotent(true)),
        },
    ]
}
