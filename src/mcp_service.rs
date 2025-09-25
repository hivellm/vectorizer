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

                _ => Err(ErrorData::invalid_params("Unknown tool", None)),
            }
        }
    }
}
