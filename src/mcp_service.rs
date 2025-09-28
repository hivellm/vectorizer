use rmcp::model::{
    CallToolRequestParam, CallToolResult, ErrorData, Implementation, ListToolsResult,
    PaginatedRequestParam, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler};
use serde_json::json;
use std::borrow::Cow;
use std::future::Future;
use std::sync::Arc;
use crate::grpc::client::VectorizerGrpcClient;
use crate::config::GrpcConfig;

#[derive(Clone)]
pub struct VectorizerService {
    grpc_server_url: String,
}

impl VectorizerService {
    pub fn new(_grpc_server_url: String) -> Self {
        Self {
            grpc_server_url: String::new(), // Not used anymore, config handles this
        }
    }

    async fn get_grpc_client(&self) -> Result<VectorizerGrpcClient, ErrorData> {
        let config = GrpcConfig::from_env();
        VectorizerGrpcClient::new(config.client)
            .await
            .map_err(|e| ErrorData::internal_error(format!("Failed to create GRPC client: {}", e), None))
    }
}

impl ServerHandler for VectorizerService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "vectorizer-mcp-server".to_string(),
                title: Some("Vectorizer MCP Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                website_url: None,
                icons: None,
            },
            instructions: Some("This server provides vector search capabilities. You can search vectors, list collections, and generate embeddings.".to_string()),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async move {
            let tools = vec![
                // Core vector operations
                Tool {
                    name: Cow::Borrowed("search_vectors"),
                    title: None,
                    description: Some(Cow::Borrowed("Search for similar vectors in a collection")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "query": {
                                "type": "string",
                                "description": "Search query text"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results",
                                "default": 10
                            }
                        },
                        "required": ["collection", "query"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("list_collections"),
                    title: None,
                    description: Some(Cow::Borrowed("List all available collections")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("get_collection_info"),
                    title: None,
                    description: Some(Cow::Borrowed("Get information about a specific collection")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            }
                        },
                        "required": ["collection"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("embed_text"),
                    title: None,
                    description: Some(Cow::Borrowed("Generate embeddings for text")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Text to embed"
                            }
                        },
                        "required": ["text"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                // Collection management
                Tool {
                    name: Cow::Borrowed("create_collection"),
                    title: None,
                    description: Some(Cow::Borrowed("Create a new collection")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "dimension": {
                                "type": "integer",
                                "description": "Vector dimension",
                                "default": 384
                            },
                            "metric": {
                                "type": "string",
                                "description": "Distance metric",
                                "default": "cosine"
                            }
                        },
                        "required": ["name"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("delete_collection"),
                    title: None,
                    description: Some(Cow::Borrowed("Delete a collection")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Collection name"
                            }
                        },
                        "required": ["name"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                // Vector operations
                Tool {
                    name: Cow::Borrowed("insert_texts"),
                    title: None,
                    description: Some(Cow::Borrowed("Insert texts into a collection (embeddings generated automatically)")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "vectors": {
                                "type": "array",
                                "description": "Array of vectors to insert",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {
                                            "type": "string",
                                            "description": "Vector ID"
                                        },
                                        "data": {
                                            "type": "array",
                                            "description": "Vector data",
                                            "items": {"type": "number"}
                                        },
                                        "metadata": {
                                            "type": "object",
                                            "description": "Optional metadata"
                                        }
                                    },
                                    "required": ["id", "data"]
                                }
                            }
                        },
                        "required": ["collection", "vectors"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("delete_vectors"),
                    title: None,
                    description: Some(Cow::Borrowed("Delete vectors from a collection")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "vector_ids": {
                                "type": "array",
                                "description": "Array of vector IDs to delete",
                                "items": {"type": "string"}
                            }
                        },
                        "required": ["collection", "vector_ids"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("get_vector"),
                    title: None,
                    description: Some(Cow::Borrowed("Get a specific vector by ID")),
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
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                // Monitoring
                Tool {
                    name: Cow::Borrowed("get_indexing_progress"),
                    title: None,
                    description: Some(Cow::Borrowed("Get indexing progress")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("health_check"),
                    title: None,
                    description: Some(Cow::Borrowed("Check service health")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                // Batch operations
                Tool {
                    name: Cow::Borrowed("batch_insert_texts"),
                    title: None,
                    description: Some(Cow::Borrowed("Batch insert texts with automatic embedding generation")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "texts": {
                                "type": "array",
                                "description": "Array of texts to insert",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {
                                            "type": "string",
                                            "description": "Text ID"
                                        },
                                        "text": {
                                            "type": "string",
                                            "description": "Text content"
                                        },
                                        "metadata": {
                                            "type": "object",
                                            "description": "Optional metadata"
                                        }
                                    },
                                    "required": ["id", "text"]
                                }
                            },
                            "provider": {
                                "type": "string",
                                "description": "Embedding provider",
                                "default": "bm25"
                            }
                        },
                        "required": ["collection", "texts"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("batch_search_vectors"),
                    title: None,
                    description: Some(Cow::Borrowed("Batch search with multiple queries")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "queries": {
                                "type": "array",
                                "description": "Array of search queries",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "query": {
                                            "type": "string",
                                            "description": "Search query text"
                                        },
                                        "limit": {
                                            "type": "integer",
                                            "description": "Maximum number of results",
                                            "default": 10
                                        },
                                        "score_threshold": {
                                            "type": "number",
                                            "description": "Minimum score threshold"
                                        }
                                    },
                                    "required": ["query"]
                                }
                            },
                            "provider": {
                                "type": "string",
                                "description": "Embedding provider",
                                "default": "bm25"
                            }
                        },
                        "required": ["collection", "queries"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("batch_update_vectors"),
                    title: None,
                    description: Some(Cow::Borrowed("Batch update existing vectors")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "updates": {
                                "type": "array",
                                "description": "Array of vector updates",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {
                                            "type": "string",
                                            "description": "Vector ID"
                                        },
                                        "text": {
                                            "type": "string",
                                            "description": "New text content (optional)"
                                        },
                                        "metadata": {
                                            "type": "object",
                                            "description": "New metadata (optional)"
                                        }
                                    },
                                    "required": ["id"]
                                }
                            },
                            "provider": {
                                "type": "string",
                                "description": "Embedding provider",
                                "default": "bm25"
                            }
                        },
                        "required": ["collection", "updates"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("batch_delete_vectors"),
                    title: None,
                    description: Some(Cow::Borrowed("Batch delete vectors by ID")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": {
                                "type": "string",
                                "description": "Collection name"
                            },
                            "vector_ids": {
                                "type": "array",
                                "description": "Array of vector IDs to delete",
                                "items": {"type": "string"}
                            }
                        },
                        "required": ["collection", "vector_ids"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                // Summarization tools
                Tool {
                    name: Cow::Borrowed("summarize_text"),
                    title: Some("Summarize Text".to_string()),
                    description: Some(Cow::Borrowed("Summarize text using various methods (extractive, keyword, sentence, abstractive)")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Text to summarize"
                            },
                            "method": {
                                "type": "string",
                                "description": "Summarization method",
                                "enum": ["extractive", "keyword", "sentence", "abstractive"],
                                "default": "extractive"
                            },
                            "max_length": {
                                "type": "integer",
                                "description": "Maximum summary length (optional)"
                            },
                            "compression_ratio": {
                                "type": "number",
                                "description": "Compression ratio (0.1-0.9, optional)"
                            },
                            "language": {
                                "type": "string",
                                "description": "Language code (optional)"
                            }
                        },
                        "required": ["text"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("summarize_context"),
                    title: Some("Summarize Context".to_string()),
                    description: Some(Cow::Borrowed("Summarize context for AI models to understand project content")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "context": {
                                "type": "string",
                                "description": "Context to summarize"
                            },
                            "method": {
                                "type": "string",
                                "description": "Summarization method",
                                "enum": ["extractive", "keyword", "sentence", "abstractive"],
                                "default": "extractive"
                            },
                            "max_length": {
                                "type": "integer",
                                "description": "Maximum summary length (optional)"
                            },
                            "compression_ratio": {
                                "type": "number",
                                "description": "Compression ratio (0.1-0.9, optional)"
                            },
                            "language": {
                                "type": "string",
                                "description": "Language code (optional)"
                            }
                        },
                        "required": ["context"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("get_summary"),
                    title: Some("Get Summary".to_string()),
                    description: Some(Cow::Borrowed("Get a specific summary by ID")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "summary_id": {
                                "type": "string",
                                "description": "Summary ID"
                            }
                        },
                        "required": ["summary_id"]
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
                Tool {
                    name: Cow::Borrowed("list_summaries"),
                    title: Some("List Summaries".to_string()),
                    description: Some(Cow::Borrowed("List all available summaries with optional filtering")),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "method": {
                                "type": "string",
                                "description": "Filter by summarization method (optional)"
                            },
                            "language": {
                                "type": "string",
                                "description": "Filter by language (optional)"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of summaries to return (optional)"
                            },
                            "offset": {
                                "type": "integer",
                                "description": "Offset for pagination (optional)"
                            }
                        }
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into(),
                    output_schema: None,
                    icons: None,
                    annotations: None,
                },
            ];

            Ok(ListToolsResult {
                tools,
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        async move {
            match request.name.as_ref() {
                "search_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection =
                        args.get("collection")
                            .and_then(|c| c.as_str())
                            .ok_or_else(|| {
                                ErrorData::invalid_params("Missing collection parameter", None)
                            })?;

                    let query = args.get("query").and_then(|q| q.as_str()).ok_or_else(|| {
                        ErrorData::invalid_params("Missing query parameter", None)
                    })?;

                    let limit = args.get("limit").and_then(|l| l.as_u64()).unwrap_or(10) as i32;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let search_response = grpc_client
                        .search(collection.to_string(), query.to_string(), limit)
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC search failed: {}", e), None))?;

                    let results: Vec<serde_json::Value> = search_response.results
                        .into_iter()
                        .map(|result| json!({
                            "id": result.id,
                            "content": result.content,
                            "score": result.score,
                            "metadata": result.metadata
                        }))
                        .collect();

                    let result_text = json!({
                        "results": results,
                        "total_found": search_response.total_found,
                        "search_time_ms": search_response.search_time_ms
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "list_collections" => {
                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let collections_response = grpc_client
                        .list_collections()
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC list collections failed: {}", e), None))?;

                    let collections: Vec<serde_json::Value> = collections_response.collections
                        .into_iter()
                        .map(|collection| json!({
                            "name": collection.name,
                            "vector_count": collection.vector_count,
                            "dimension": collection.dimension,
                            "similarity_metric": collection.similarity_metric,
                            "status": collection.status,
                            "last_updated": collection.last_updated,
                            "error_message": collection.error_message
                        }))
                        .collect();

                    let result_text = json!({
                        "collections": collections,
                        "total_collections": collections_response.total_collections
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "embed_text" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let text = args
                        .get("text")
                        .and_then(|t| t.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing text parameter", None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let embed_response = grpc_client
                        .embed_text(text.to_string(), "bm25".to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC embed failed: {}", e), None))?;

                    let result_text = json!({
                        "embedding": embed_response.embedding,
                        "dimension": embed_response.dimension,
                        "provider": embed_response.provider
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "get_collection_info" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let collection_info = grpc_client
                        .get_collection_info(collection.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC get_collection_info failed: {}", e), None))?;

                    let result_text = json!({
                        "name": collection_info.name,
                        "vector_count": collection_info.vector_count,
                        "document_count": collection_info.document_count,
                        "dimension": collection_info.dimension,
                        "similarity_metric": collection_info.similarity_metric,
                        "status": collection_info.status,
                        "last_updated": collection_info.last_updated,
                        "error_message": collection_info.error_message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "create_collection" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let name = args
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing name parameter", None))?;

                    let dimension = args.get("dimension").and_then(|d| d.as_u64()).unwrap_or(384) as i32;
                    let metric = args.get("metric").and_then(|m| m.as_str()).unwrap_or("cosine");

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .create_collection(name.to_string(), dimension, metric.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC create_collection failed: {}", e), None))?;

                    let result_text = json!({
                        "name": response.name,
                        "dimension": response.dimension,
                        "similarity_metric": response.similarity_metric,
                        "status": response.status,
                        "message": response.message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "delete_collection" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let name = args
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing name parameter", None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .delete_collection(name.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC delete_collection failed: {}", e), None))?;

                    let result_text = json!({
                        "collection_name": response.collection_name,
                        "status": response.status,
                        "message": response.message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "insert_texts" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    // Parse vectors as texts for GRPC embedding
                    let parsed_texts: std::result::Result<
                        Vec<(String, String, Option<std::collections::HashMap<String, String>>)>,
                        &str,
                    > = args
                        .get("vectors")
                        .and_then(|v| v.as_array())
                        .ok_or("Missing vectors array")
                        .and_then(|vectors| {
                            vectors
                                .iter()
                                .map(|v| {
                                    let id = v
                                        .get("id")
                                        .and_then(|x| x.as_str())
                                        .ok_or("Missing vector id")
                                        .map(|s| s.to_string())?;
                                    
                                    // Extrair texto do payload ou usar dados como fallback
                                    // Primeiro tentar extrair texto diretamente
                                    let text = if let Some(text_str) = v.get("text").and_then(|x| x.as_str()) {
                                        text_str.to_string()
                                    } else {
                                        // Fallback: converter dados de vetor para string se texto não estiver disponível
                                        v.get("data")
                                            .and_then(|x| x.as_array())
                                            .map(|arr| {
                                                arr.iter()
                                                    .filter_map(|x| x.as_f64())
                                                    .map(|f| f.to_string())
                                                    .collect::<Vec<String>>()
                                                    .join(",")
                                            })
                                            .ok_or("Missing text or data for embedding")?
                                    };
                                    
                                    let metadata = v
                                        .get("metadata")
                                        .and_then(|x| x.as_object())
                                        .map(|obj| {
                                            obj.iter()
                                                .filter_map(|(k, v)| {
                                                    v.as_str().map(|s| (k.clone(), s.to_string()))
                                                })
                                                .collect()
                                        });
                                    Ok::<(String, String, Option<std::collections::HashMap<String, String>>), &str>((id, text, metadata))
                                })
                                .collect()
                        });

                    let texts_data = parsed_texts
                        .map_err(|e| ErrorData::invalid_params(format!("Failed to parse texts: {}", e), None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .insert_texts(collection.to_string(), texts_data, "bm25".to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC insert_texts failed: {}", e), None))?;

                    let result_text = json!({
                        "collection": response.collection,
                        "inserted_count": response.inserted_count,
                        "status": response.status,
                        "message": response.message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "delete_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let vector_ids = args
                        .get("vector_ids")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| ErrorData::invalid_params("Missing vector_ids parameter", None))?
                        .iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect();

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .delete_vectors(collection.to_string(), vector_ids)
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC delete_vectors failed: {}", e), None))?;

                    let result_text = json!({
                        "collection": response.collection,
                        "deleted_count": response.deleted_count,
                        "status": response.status,
                        "message": response.message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "get_vector" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let vector_id = args
                        .get("vector_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing vector_id parameter", None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .get_vector(collection.to_string(), vector_id.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC get_vector failed: {}", e), None))?;

                    let result_text = json!({
                        "id": response.id,
                        "data": response.data,
                        "metadata": response.metadata,
                        "collection": response.collection,
                        "status": response.status
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "get_indexing_progress" => {
                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .get_indexing_progress()
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC get_indexing_progress failed: {}", e), None))?;

                    let collections: Vec<serde_json::Value> = response.collections
                        .into_iter()
                        .map(|c| json!({
                            "collection_name": c.collection_name,
                            "status": c.status,
                            "progress": c.progress,
                            "vector_count": c.vector_count,
                            "error_message": c.error_message,
                            "last_updated": c.last_updated
                        }))
                        .collect();

                    let result_text = json!({
                        "collections": collections,
                        "is_indexing": response.is_indexing,
                        "overall_status": response.overall_status
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "health_check" => {
                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .health_check()
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC health_check failed: {}", e), None))?;

                    let result_text = json!({
                        "status": response.status,
                        "service": response.service,
                        "version": response.version,
                        "timestamp": response.timestamp,
                        "error_message": response.error_message
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "batch_insert_texts" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let provider = args
                        .get("provider")
                        .and_then(|p| p.as_str())
                        .unwrap_or("bm25");

                    // Parse texts for GRPC
                    let parsed_texts: std::result::Result<
                        Vec<(String, String, Option<std::collections::HashMap<String, String>>)>,
                        &str,
                    > = args
                        .get("texts")
                        .and_then(|v| v.as_array())
                        .ok_or("Missing texts array")
                        .and_then(|texts| {
                            texts
                                .iter()
                                .map(|v| {
                                    let id = v
                                        .get("id")
                                        .and_then(|x| x.as_str())
                                        .ok_or("Missing text id")
                                        .map(|s| s.to_string())?;
                                    
                                    let text = v
                                        .get("text")
                                        .and_then(|x| x.as_str())
                                        .ok_or("Missing text content")
                                        .map(|s| s.to_string())?;
                                    
                                    let metadata = v
                                        .get("metadata")
                                        .and_then(|x| x.as_object())
                                        .map(|obj| {
                                            obj.iter()
                                                .filter_map(|(k, v)| {
                                                    v.as_str().map(|s| (k.clone(), s.to_string()))
                                                })
                                                .collect()
                                        });
                                    
                                    Ok::<(String, String, Option<std::collections::HashMap<String, String>>), &str>((id, text, metadata))
                                })
                                .collect()
                        });

                    let texts_data = parsed_texts
                        .map_err(|e| ErrorData::invalid_params(format!("Failed to parse texts: {}", e), None))?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .insert_texts(collection.to_string(), texts_data, provider.to_string())
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC batch insert_texts failed: {}", e), None))?;

                    let result_text = json!({
                        "success": true,
                        "collection": response.collection,
                        "inserted_count": response.inserted_count,
                        "status": response.status,
                        "message": response.message,
                        "operation": "batch_insert_texts"
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "batch_search_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let provider = args
                        .get("provider")
                        .and_then(|p| p.as_str())
                        .unwrap_or("bm25");

                    // Parse queries
                    let queries = args
                        .get("queries")
                        .and_then(|q| q.as_array())
                        .ok_or_else(|| ErrorData::invalid_params("Missing queries array", None))?;

                    let mut grpc_client = self.get_grpc_client().await?;
                    let mut batch_results = Vec::new();

                    for (i, query_obj) in queries.iter().enumerate() {
                        let query_text = query_obj
                            .get("query")
                            .and_then(|q| q.as_str())
                            .ok_or_else(|| ErrorData::invalid_params(format!("Missing query in item {}", i), None))?;

                        let limit = query_obj
                            .get("limit")
                            .and_then(|l| l.as_u64())
                            .unwrap_or(10) as i32;

                        let score_threshold = query_obj
                            .get("score_threshold")
                            .and_then(|s| s.as_f64());

                        // Make GRPC search request
                        match grpc_client
                            .search(collection.to_string(), query_text.to_string(), limit)
                            .await
                        {
                            Ok(search_response) => {
                                let results: Vec<serde_json::Value> = search_response.results
                                    .into_iter()
                                    .filter(|result| {
                                        if let Some(threshold) = score_threshold {
                                            result.score >= threshold as f32
                                        } else {
                                            true
                                        }
                                    })
                                    .map(|result| json!({
                                        "id": result.id,
                                        "content": result.content,
                                        "score": result.score,
                                        "metadata": result.metadata
                                    }))
                                    .collect();

                                batch_results.push(json!({
                                    "query": query_text,
                                    "query_index": i,
                                    "results": results,
                                    "total_found": search_response.total_found,
                                    "search_time_ms": search_response.search_time_ms
                                }));
                            }
                            Err(e) => {
                                batch_results.push(json!({
                                    "query": query_text,
                                    "query_index": i,
                                    "error": format!("Search failed: {}", e),
                                    "results": [],
                                    "total_found": 0,
                                    "search_time_ms": 0
                                }));
                            }
                        }
                    }

                    let result_text = json!({
                        "success": true,
                        "collection": collection,
                        "total_queries": queries.len(),
                        "batch_results": batch_results,
                        "operation": "batch_search_vectors"
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "batch_update_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let provider = args
                        .get("provider")
                        .and_then(|p| p.as_str())
                        .unwrap_or("bm25");

                    // Parse updates
                    let updates = args
                        .get("updates")
                        .and_then(|u| u.as_array())
                        .ok_or_else(|| ErrorData::invalid_params("Missing updates array", None))?;

                    let mut grpc_client = self.get_grpc_client().await?;
                    let mut batch_results = Vec::new();
                    let mut successful_updates = 0;
                    let mut failed_updates = 0;

                    for (i, update_obj) in updates.iter().enumerate() {
                        let vector_id = update_obj
                            .get("id")
                            .and_then(|id| id.as_str())
                            .ok_or_else(|| ErrorData::invalid_params(format!("Missing id in update item {}", i), None))?;

                        // Check if we have new text to update
                        if let Some(text) = update_obj.get("text").and_then(|t| t.as_str()) {
                            // Parse metadata
                            let metadata = update_obj
                                .get("metadata")
                                .and_then(|x| x.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .filter_map(|(k, v)| {
                                            v.as_str().map(|s| (k.clone(), s.to_string()))
                                        })
                                        .collect()
                                });

                            // Create update data
                            let update_data = vec![(vector_id.to_string(), text.to_string(), metadata)];

                            // Make GRPC update request (using insert_texts as update mechanism)
                            match grpc_client
                                .insert_texts(collection.to_string(), update_data, provider.to_string())
                                .await
                            {
                                Ok(response) => {
                                    batch_results.push(json!({
                                        "id": vector_id,
                                        "update_index": i,
                                        "status": "success",
                                        "message": response.message
                                    }));
                                    successful_updates += 1;
                                }
                                Err(e) => {
                                    batch_results.push(json!({
                                        "id": vector_id,
                                        "update_index": i,
                                        "status": "error",
                                        "error": format!("Update failed: {}", e)
                                    }));
                                    failed_updates += 1;
                                }
                            }
                        } else {
                            // No text update, just metadata update (not supported in current GRPC)
                            batch_results.push(json!({
                                "id": vector_id,
                                "update_index": i,
                                "status": "skipped",
                                "message": "No text content provided for update"
                            }));
                        }
                    }

                    let result_text = json!({
                        "success": true,
                        "collection": collection,
                        "total_updates": updates.len(),
                        "successful_updates": successful_updates,
                        "failed_updates": failed_updates,
                        "batch_results": batch_results,
                        "operation": "batch_update_vectors"
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "batch_delete_vectors" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let collection = args
                        .get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;

                    let vector_ids = args
                        .get("vector_ids")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| ErrorData::invalid_params("Missing vector_ids parameter", None))?
                        .iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect();

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .delete_vectors(collection.to_string(), vector_ids)
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC batch delete_vectors failed: {}", e), None))?;

                    let result_text = json!({
                        "success": true,
                        "collection": response.collection,
                        "deleted_count": response.deleted_count,
                        "status": response.status,
                        "message": response.message,
                        "operation": "batch_delete_vectors"
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                // Summarization tools
                "summarize_text" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let text = args.get("text").and_then(|t| t.as_str()).ok_or_else(|| {
                        ErrorData::invalid_params("Missing text parameter", None)
                    })?;

                    let method = args.get("method").and_then(|m| m.as_str()).unwrap_or("extractive");
                    let max_length = args.get("max_length").and_then(|l| l.as_i64()).map(|l| l as i32);
                    let compression_ratio = args.get("compression_ratio").and_then(|r| r.as_f64()).map(|r| r as f32);
                    let language = args.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .summarize_text(crate::grpc::vectorizer::SummarizeTextRequest {
                            text: text.to_string(),
                            method: method.to_string(),
                            max_length,
                            compression_ratio,
                            language,
                            metadata: std::collections::HashMap::new(),
                        })
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC summarize_text failed: {}", e), None))?;

                    let result_text = json!({
                        "summary_id": response.summary_id,
                        "original_text": response.original_text,
                        "summary": response.summary,
                        "method": response.method,
                        "original_length": response.original_length,
                        "summary_length": response.summary_length,
                        "compression_ratio": response.compression_ratio,
                        "language": response.language,
                        "status": response.status,
                        "message": response.message,
                        "metadata": response.metadata
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "summarize_context" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let context = args.get("context").and_then(|c| c.as_str()).ok_or_else(|| {
                        ErrorData::invalid_params("Missing context parameter", None)
                    })?;

                    let method = args.get("method").and_then(|m| m.as_str()).unwrap_or("extractive");
                    let max_length = args.get("max_length").and_then(|l| l.as_i64()).map(|l| l as i32);
                    let compression_ratio = args.get("compression_ratio").and_then(|r| r.as_f64()).map(|r| r as f32);
                    let language = args.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .summarize_context(crate::grpc::vectorizer::SummarizeContextRequest {
                            context: context.to_string(),
                            method: method.to_string(),
                            max_length,
                            compression_ratio,
                            language,
                            metadata: std::collections::HashMap::new(),
                        })
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC summarize_context failed: {}", e), None))?;

                    let result_text = json!({
                        "summary_id": response.summary_id,
                        "original_context": response.original_context,
                        "summary": response.summary,
                        "method": response.method,
                        "original_length": response.original_length,
                        "summary_length": response.summary_length,
                        "compression_ratio": response.compression_ratio,
                        "language": response.language,
                        "status": response.status,
                        "message": response.message,
                        "metadata": response.metadata
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "get_summary" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let summary_id = args.get("summary_id").and_then(|s| s.as_str()).ok_or_else(|| {
                        ErrorData::invalid_params("Missing summary_id parameter", None)
                    })?;

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .get_summary(crate::grpc::vectorizer::GetSummaryRequest {
                            summary_id: summary_id.to_string(),
                        })
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC get_summary failed: {}", e), None))?;

                    let result_text = json!({
                        "summary_id": response.summary_id,
                        "original_text": response.original_text,
                        "summary": response.summary,
                        "method": response.method,
                        "original_length": response.original_length,
                        "summary_length": response.summary_length,
                        "compression_ratio": response.compression_ratio,
                        "language": response.language,
                        "created_at": response.created_at,
                        "metadata": response.metadata,
                        "status": response.status
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                "list_summaries" => {
                    let args = request
                        .arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;

                    let method = args.get("method").and_then(|m| m.as_str()).map(|s| s.to_string());
                    let language = args.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());
                    let limit = args.get("limit").and_then(|l| l.as_i64()).map(|l| l as i32);
                    let offset = args.get("offset").and_then(|o| o.as_i64()).map(|o| o as i32);

                    // Make GRPC request
                    let mut grpc_client = self.get_grpc_client().await?;
                    let response = grpc_client
                        .list_summaries(crate::grpc::vectorizer::ListSummariesRequest {
                            method,
                            language,
                            limit,
                            offset,
                        })
                        .await
                        .map_err(|e| ErrorData::internal_error(format!("GRPC list_summaries failed: {}", e), None))?;

                    let summaries: Vec<serde_json::Value> = response.summaries
                        .into_iter()
                        .map(|summary| json!({
                            "summary_id": summary.summary_id,
                            "method": summary.method,
                            "language": summary.language,
                            "original_length": summary.original_length,
                            "summary_length": summary.summary_length,
                            "compression_ratio": summary.compression_ratio,
                            "created_at": summary.created_at,
                            "metadata": summary.metadata
                        }))
                        .collect();

                    let result_text = json!({
                        "summaries": summaries,
                        "total_count": response.total_count,
                        "status": response.status
                    }).to_string();

                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }

                _ => Err(ErrorData::invalid_params("Unknown tool", None)),
            }
        }
    }
}
