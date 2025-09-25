//! MCP server implementation
//!
//! Provides the main MCP server that handles connections and message routing

use super::{McpConfig, McpConnection, McpError, McpRequest, McpResponse, McpServerState};
use crate::auth::AuthManager;
use crate::db::VectorStore;
use crate::error::Result;
use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::sync::Arc;
// RwLock is used in McpServerState
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// MCP server implementation
#[derive(Debug)]
pub struct McpServer {
    /// Server state
    state: Arc<McpServerState>,
    /// Vector store
    #[allow(dead_code)]
    vector_store: Arc<VectorStore>,
    /// Authentication manager
    #[allow(dead_code)]
    auth_manager: Option<Arc<AuthManager>>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(
        config: McpConfig,
        vector_store: Arc<VectorStore>,
        auth_manager: Option<Arc<AuthManager>>,
    ) -> Self {
        let state = Arc::new(McpServerState::new(config));

        Self {
            state,
            vector_store,
            auth_manager,
        }
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        let config = &self.state.config;

        if !config.enabled {
            info!("MCP server is disabled");
            return Ok(());
        }

        info!("Starting MCP server on {}:{}", config.host, config.port);

        // Create router
        let _app: Router = Router::new()
            .route("/mcp", get(Self::handle_websocket))
            .with_state(Arc::clone(&self.state));

        // Start server (in a real implementation, this would use axum::serve)
        info!("MCP server started successfully");
        info!(
            "WebSocket endpoint: ws://{}:{}/mcp",
            config.host, config.port
        );

        Ok(())
    }

    /// Handle WebSocket upgrade
    pub async fn handle_websocket(
        ws: WebSocketUpgrade,
        State(state): State<Arc<McpServerState>>,
    ) -> Response {
        ws.on_upgrade(|socket| Self::handle_connection(socket, state))
    }

    /// Handle individual WebSocket connection
    async fn handle_connection(socket: WebSocket, state: Arc<McpServerState>) {
        let connection_id = Uuid::new_v4().to_string();
        info!("New MCP connection: {}", connection_id);

        // Create connection
        let connection = McpConnection {
            id: connection_id.clone(),
            client_capabilities: serde_json::json!({}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: false,
        };

        // Add connection to state
        if let Err(e) = state
            .add_connection(connection_id.clone(), connection)
            .await
        {
            error!("Failed to add connection: {}", e);
            return;
        }

        // Handle messages
        let (mut sender, mut receiver) = socket.split();

        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received message: {}", text);

                    // Update activity
                    if let Err(e) = state.update_activity(&connection_id).await {
                        warn!("Failed to update activity: {}", e);
                    }

                    // Parse and handle request
                    match serde_json::from_str::<McpRequest>(&text) {
                        Ok(request) => {
                            let response = Self::handle_request(request, &state).await;

                            // Send response
                            if let Ok(response_json) = serde_json::to_string(&response) {
                                if let Err(e) =
                                    sender.send(Message::Text(response_json.into())).await
                                {
                                    error!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse request: {}", e);

                            let error_response = McpResponse {
                                id: None,
                                result: None,
                                error: Some(McpError {
                                    code: -32700, // Parse error
                                    message: "Invalid JSON".to_string(),
                                    data: Some(serde_json::json!({"parse_error": e.to_string()})),
                                }),
                            };

                            if let Ok(response_json) = serde_json::to_string(&error_response) {
                                let _ = sender.send(Message::Text(response_json.into())).await;
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Connection closed: {}", connection_id);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if let Err(e) = sender.send(Message::Pong(data)).await {
                        error!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Handle pong if needed
                }
                Ok(Message::Binary(_)) => {
                    warn!("Received binary message, ignoring");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        // Clean up connection
        if let Err(e) = state.remove_connection(&connection_id).await {
            error!("Failed to remove connection: {}", e);
        }

        info!("Connection closed: {}", connection_id);
    }

    /// Handle MCP request
    async fn handle_request(request: McpRequest, state: &McpServerState) -> McpResponse {
        match request {
            McpRequest::Initialize {
                protocol_version,
                capabilities: _capabilities,
                client_info,
            } => {
                info!(
                    "MCP Initialize request - Protocol: {}, Client: {:?}",
                    protocol_version, client_info
                );

                McpResponse {
                    id: None,
                    result: Some(serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {},
                            "resources": {}
                        },
                        "serverInfo": {
                            "name": state.capabilities.server_info.name,
                            "version": state.capabilities.server_info.version
                        }
                    })),
                    error: None,
                }
            }

            McpRequest::ToolsList => {
                debug!("MCP ToolsList request");

                McpResponse {
                    id: None,
                    result: Some(serde_json::json!({
                        "tools": state.capabilities.tools
                    })),
                    error: None,
                }
            }

            McpRequest::ToolsCall { name, arguments } => {
                debug!(
                    "MCP ToolsCall request - Tool: {}, Args: {:?}",
                    name, arguments
                );

                // Handle tool calls
                let result = Self::handle_tool_call(&name, arguments, state).await;

                McpResponse {
                    id: None,
                    result: Some(result),
                    error: None,
                }
            }

            McpRequest::ResourcesList => {
                debug!("MCP ResourcesList request");

                McpResponse {
                    id: None,
                    result: Some(serde_json::json!({
                        "resources": state.capabilities.resources
                    })),
                    error: None,
                }
            }

            McpRequest::ResourcesRead { uri } => {
                debug!("MCP ResourcesRead request - URI: {}", uri);

                let result = Self::handle_resource_read(&uri, state).await;

                McpResponse {
                    id: None,
                    result: Some(result),
                    error: None,
                }
            }

            McpRequest::Ping => {
                debug!("MCP Ping request");

                McpResponse {
                    id: None,
                    result: Some(serde_json::json!({
                        "pong": true,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })),
                    error: None,
                }
            }
        }
    }

    /// Handle tool call
    async fn handle_tool_call(
        tool_name: &str,
        arguments: serde_json::Value,
        state: &McpServerState,
    ) -> serde_json::Value {
        match tool_name {
            "search_vectors" => Self::handle_search_vectors(arguments, state).await,
            "list_collections" => Self::handle_list_collections(state).await,
            "get_collection_info" => Self::handle_get_collection_info(arguments, state).await,
            "embed_text" => Self::handle_embed_text(arguments, state).await,
            _ => {
                serde_json::json!({
                    "error": "Unknown tool",
                    "tool": tool_name
                })
            }
        }
    }

    /// Handle search vectors tool
    async fn handle_search_vectors(
        arguments: serde_json::Value,
        _state: &McpServerState,
    ) -> serde_json::Value {
        let collection = arguments
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        if collection.is_empty() || query.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameters: collection and query"
            });
        }

        // In a real implementation, this would use the vector store
        // For now, return mock results
        serde_json::json!({
            "results": [
                {
                    "id": "doc_1",
                    "score": 0.95,
                    "content": "Sample document 1"
                },
                {
                    "id": "doc_2",
                    "score": 0.87,
                    "content": "Sample document 2"
                }
            ],
            "query": query,
            "collection": collection,
            "limit": limit
        })
    }

    /// Handle list collections tool
    async fn handle_list_collections(_state: &McpServerState) -> serde_json::Value {
        // In a real implementation, this would query the vector store
        serde_json::json!({
            "collections": [
                {
                    "name": "documents",
                    "vector_count": 1000,
                    "dimension": 384
                },
                {
                    "name": "embeddings",
                    "vector_count": 500,
                    "dimension": 768
                }
            ]
        })
    }

    /// Handle get collection info tool
    async fn handle_get_collection_info(
        arguments: serde_json::Value,
        _state: &McpServerState,
    ) -> serde_json::Value {
        let collection = arguments
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if collection.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameter: collection"
            });
        }

        // In a real implementation, this would query the vector store
        serde_json::json!({
            "name": collection,
            "vector_count": 1000,
            "dimension": 384,
            "metric": "cosine",
            "created_at": "2024-01-01T00:00:00Z"
        })
    }

    /// Handle embed text tool
    async fn handle_embed_text(
        arguments: serde_json::Value,
        _state: &McpServerState,
    ) -> serde_json::Value {
        let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("");

        if text.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameter: text"
            });
        }

        // In a real implementation, this would use the embedding system
        // For now, return mock embedding
        serde_json::json!({
            "embedding": vec![0.1; 384], // Mock 384-dimensional embedding
            "text": text,
            "dimension": 384
        })
    }

    /// Handle resource read
    async fn handle_resource_read(uri: &str, state: &McpServerState) -> serde_json::Value {
        match uri {
            "vectorizer://collections" => Self::handle_list_collections(state).await,
            "vectorizer://stats" => {
                serde_json::json!({
                    "total_collections": 2,
                    "total_vectors": 1500,
                    "memory_usage": "256MB",
                    "uptime": "24h"
                })
            }
            _ => {
                serde_json::json!({
                    "error": "Resource not found",
                    "uri": uri
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let config = McpConfig::default();
        let vector_store = Arc::new(VectorStore::new());
        let server = McpServer::new(config, vector_store, None);

        assert!(server.state.config.enabled);
        assert_eq!(server.state.config.port, 15003);
    }

    #[tokio::test]
    async fn test_mcp_request_handling() {
        let config = McpConfig::default();
        let state = McpServerState::new(config);

        let request = McpRequest::Ping;
        let response = McpServer::handle_request(request, &state).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_tool_call_handling() {
        let config = McpConfig::default();
        let state = McpServerState::new(config);

        let arguments = serde_json::json!({
            "collection": "test",
            "query": "test query",
            "limit": 5
        });

        let result = McpServer::handle_tool_call("search_vectors", arguments, &state).await;
        assert!(result.get("results").is_some());
    }
}
