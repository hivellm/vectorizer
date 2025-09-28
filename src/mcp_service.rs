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

                _ => Err(ErrorData::invalid_params("Unknown tool", None)),
            }
        }
    }
}
