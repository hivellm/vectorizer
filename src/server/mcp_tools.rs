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
    ]
}
