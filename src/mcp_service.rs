use std::sync::Arc;
use std::borrow::Cow;
use std::future::Future;
use tokio::sync::Mutex;
use rmcp::{ServerHandler, RoleServer};
use rmcp::service::RequestContext;
use rmcp::model::{
    Tool, ListToolsResult, CallToolResult, PaginatedRequestParam,
    CallToolRequestParam, ErrorData, ServerInfo, ServerCapabilities,
    ProtocolVersion, Implementation
};
use serde_json::json;
use crate::embedding::EmbeddingManager;
use crate::VectorStore;

pub struct VectorizerService {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<Mutex<EmbeddingManager>>,
}

impl VectorizerService {
    pub fn new(
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<Mutex<EmbeddingManager>>,
    ) -> Self {
        Self {
            vector_store,
            embedding_manager,
        }
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
                    }).as_object().unwrap().clone().into(),
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
                    }).as_object().unwrap().clone().into(),
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
                    }).as_object().unwrap().clone().into(),
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
                    let args = request.arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
                    
                    let collection = args.get("collection")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing collection parameter", None))?;
                    
                    let query = args.get("query")
                        .and_then(|q| q.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing query parameter", None))?;
                    
                    let limit = args.get("limit")
                        .and_then(|l| l.as_u64())
                        .unwrap_or(10) as usize;
                    
                    // Perform search
                    let embedding = self.embedding_manager.lock().await
                        .embed_with_provider("bm25", query)
                        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                    
                    let results = self.vector_store.search(collection, &embedding, limit)
                        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                    
                    let result_text = serde_json::to_string(&results)
                        .unwrap_or_else(|_| "[]".to_string());
                    
                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }
                
                "list_collections" => {
                    let collections = self.vector_store.list_collections();
                    let result_text = serde_json::to_string(&collections)
                        .unwrap_or_else(|_| "[]".to_string());
                    
                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(result_text)],
                        structured_content: None,
                        is_error: Some(false),
                        meta: None,
                    })
                }
                
                "embed_text" => {
                    let args = request.arguments
                        .as_ref()
                        .ok_or_else(|| ErrorData::invalid_params("Missing arguments", None))?;
                    
                    let text = args.get("text")
                        .and_then(|t| t.as_str())
                        .ok_or_else(|| ErrorData::invalid_params("Missing text parameter", None))?;
                    
                    let embedding = self.embedding_manager.lock().await
                        .embed_with_provider("bm25", text)
                        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                    
                    let result = json!({
                        "embedding": embedding,
                        "dimension": embedding.len(),
                        "provider": "bm25"
                    });
                    
                    Ok(CallToolResult {
                        content: vec![rmcp::model::Content::text(
                            serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
                        )],
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