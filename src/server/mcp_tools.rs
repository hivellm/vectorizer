//! MCP Tools definitions

use rmcp::model::{Tool, ToolAnnotations};
use serde_json::json;
use std::borrow::Cow;

pub fn get_mcp_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: Cow::Borrowed("search_vectors"),
            title: Some("Search Vectors".to_string()),
            description: Some(Cow::Borrowed("Search for semantically similar content in a vector collection")),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "collection": {"type": "string", "description": "Collection name"},
                    "query": {"type": "string", "description": "Search query text"},
                    "limit": {"type": "integer", "description": "Maximum number of results", "default": 10}
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
            description: Some(Cow::Borrowed("List all available vector collections")),
            input_schema: json!({"type": "object", "properties": {}}).as_object().unwrap().clone().into(),
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
    ]
}
