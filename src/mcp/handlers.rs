//! MCP request handlers
//!
//! Handles specific MCP protocol requests and responses

use super::{McpRequest, McpResponse, McpServerState};
use crate::db::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::grpc::client::VectorizerGrpcClient;
use serde_json;
use tracing::{debug, info};
use std::sync::Arc;
use tokio::sync::Mutex;

/// MCP request handler
pub struct McpHandler;

impl McpHandler {
    /// Handle MCP request
    pub async fn handle_request(
        request: McpRequest,
        state: &McpServerState,
        vector_store: &VectorStore,
        embedding_manager: &EmbeddingManager,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> McpResponse {
        match request {
            McpRequest::Initialize {
                protocol_version,
                capabilities,
                client_info,
            } => Self::handle_initialize(protocol_version, capabilities, client_info, state).await,
            McpRequest::ToolsList => Self::handle_tools_list(state).await,
            McpRequest::ToolsCall { name, arguments } => {
                Self::handle_tools_call(name, arguments, state, vector_store, embedding_manager, grpc_client)
                    .await
            }
            McpRequest::ResourcesList => Self::handle_resources_list(state).await,
            McpRequest::ResourcesRead { uri } => {
                Self::handle_resources_read(uri, state, vector_store, grpc_client).await
            }
            McpRequest::Ping => Self::handle_ping().await,
        }
    }

    /// Handle initialize request
    async fn handle_initialize(
        protocol_version: String,
        _capabilities: serde_json::Value,
        client_info: serde_json::Value,
        state: &McpServerState,
    ) -> McpResponse {
        info!(
            "MCP Initialize - Protocol: {}, Client: {:?}",
            protocol_version, client_info
        );

        let result = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "supported": true,
                    "callTool": true
                },
                "resources": {
                    "supported": true,
                    "subscribe": false,
                    "listChanged": false
                },
                "prompts": {
                    "supported": false
                },
                "logging": {
                    "supported": true
                }
            },
            "serverInfo": {
                "name": state.capabilities.server_info.name,
                "version": state.capabilities.server_info.version
            }
        });

        McpResponse {
            id: None,
            result: Some(result),
            error: None,
        }
    }

    /// Handle tools list request
    async fn handle_tools_list(state: &McpServerState) -> McpResponse {
        debug!("MCP ToolsList request");

        let tools: Vec<serde_json::Value> = state
            .capabilities
            .tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema
                })
            })
            .collect();

        McpResponse {
            id: None,
            result: Some(serde_json::json!({
                "tools": tools
            })),
            error: None,
        }
    }

    /// Handle tools call request
    async fn handle_tools_call(
        name: String,
        arguments: serde_json::Value,
        _state: &McpServerState,
        vector_store: &VectorStore,
        embedding_manager: &EmbeddingManager,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> McpResponse {
        debug!("MCP ToolsCall - Tool: {}, Args: {:?}", name, arguments);

        let result = match name.as_str() {
            "search_vectors" => {
                Self::handle_search_vectors_tool(arguments, vector_store, embedding_manager, grpc_client).await
            }
            "list_collections" => Self::handle_list_collections_tool(vector_store, grpc_client).await,
            "get_collection_info" => {
                Self::handle_get_collection_info_tool(arguments, vector_store, grpc_client).await
            }
            "embed_text" => Self::handle_embed_text_tool(arguments, embedding_manager).await,
            "insert_vectors" => Self::handle_insert_vectors_tool(arguments, vector_store).await,
            "delete_vectors" => Self::handle_delete_vectors_tool(arguments, vector_store).await,
            "get_vector" => Self::handle_get_vector_tool(arguments, vector_store).await,
            "create_collection" => {
                Self::handle_create_collection_tool(arguments, vector_store).await
            }
            "delete_collection" => {
                Self::handle_delete_collection_tool(arguments, vector_store).await
            }
            "get_database_stats" => Self::handle_get_database_stats_tool(vector_store).await,
            // GRPC-specific tools
            "create_collection_grpc" => {
                Self::handle_create_collection_grpc_tool(arguments, grpc_client).await
            }
            "delete_collection_grpc" => {
                Self::handle_delete_collection_grpc_tool(arguments, grpc_client).await
            }
            "insert_vectors_grpc" => {
                Self::handle_insert_vectors_grpc_tool(arguments, grpc_client).await
            }
            "delete_vectors_grpc" => {
                Self::handle_delete_vectors_grpc_tool(arguments, grpc_client).await
            }
            "get_vector_grpc" => {
                Self::handle_get_vector_grpc_tool(arguments, grpc_client).await
            }
            "get_collection_info_grpc" => {
                Self::handle_get_collection_info_grpc_tool(arguments, grpc_client).await
            }
            "get_indexing_progress_grpc" => {
                Self::handle_get_indexing_progress_grpc_tool(grpc_client).await
            }
            "health_check_grpc" => {
                Self::handle_health_check_grpc_tool(grpc_client).await
            }
            _ => {
                serde_json::json!({
                    "error": "Unknown tool",
                    "tool": name
                })
            }
        };

        McpResponse {
            id: None,
            result: Some(result),
            error: None,
        }
    }

    /// Handle resources list request
    async fn handle_resources_list(state: &McpServerState) -> McpResponse {
        debug!("MCP ResourcesList request");

        let resources: Vec<serde_json::Value> = state
            .capabilities
            .resources
            .iter()
            .map(|resource| {
                serde_json::json!({
                    "uri": resource.uri,
                    "name": resource.name,
                    "description": resource.description,
                    "mimeType": resource.mime_type
                })
            })
            .collect();

        McpResponse {
            id: None,
            result: Some(serde_json::json!({
                "resources": resources
            })),
            error: None,
        }
    }

    /// Handle resources read request
    async fn handle_resources_read(
        uri: String,
        _state: &McpServerState,
        vector_store: &VectorStore,
        grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> McpResponse {
        debug!("MCP ResourcesRead - URI: {}", uri);

        let result = match uri.as_str() {
            "vectorizer://collections" => Self::handle_list_collections_tool(vector_store, grpc_client).await,
            "vectorizer://stats" => Self::handle_get_database_stats_tool(vector_store).await,
            _ => {
                serde_json::json!({
                    "error": "Resource not found",
                    "uri": uri
                })
            }
        };

        McpResponse {
            id: None,
            result: Some(result),
            error: None,
        }
    }

    /// Handle ping request
    async fn handle_ping() -> McpResponse {
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

    // Tool-specific handlers

    async fn handle_search_vectors_tool(
        arguments: serde_json::Value,
        vector_store: &VectorStore,
        embedding_manager: &EmbeddingManager,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
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

        // Try GRPC first, fallback to local vector store
        if let Some(ref mut client) = grpc_client {
            match client.search(collection.to_string(), query.to_string(), limit as i32).await {
                Ok(grpc_response) => {
                    let results: Vec<serde_json::Value> = grpc_response.results
                        .into_iter()
                        .map(|r| {
                            serde_json::json!({
                                "id": r.id,
                                "content": r.content,
                                "score": r.score,
                                "metadata": r.metadata
                            })
                        })
                        .collect();

                    return serde_json::json!({
                        "results": results,
                        "total_found": grpc_response.total_found,
                        "search_time_ms": grpc_response.search_time_ms
                    });
                }
                Err(e) => {
                    debug!("GRPC search failed, falling back to local store: {}", e);
                }
            }
        }

        // Fallback to local vector store
        match crate::mcp::tools::McpTools::search_vectors(
            collection,
            query,
            limit,
            vector_store,
            embedding_manager,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    async fn handle_list_collections_tool(vector_store: &VectorStore, mut grpc_client: Option<&mut VectorizerGrpcClient>) -> serde_json::Value {
        // Try GRPC first
        if let Some(ref mut client) = grpc_client {
            match client.list_collections().await {
                Ok(grpc_response) => {
                    let collections: Vec<serde_json::Value> = grpc_response.collections
                        .into_iter()
                        .map(|c| {
                            serde_json::json!({
                                "name": c.name,
                                "vector_count": c.vector_count,
                                "document_count": c.document_count,
                                "dimension": c.dimension,
                                "status": c.status
                            })
                        })
                        .collect();

                    return serde_json::json!({
                        "collections": collections,
                        "total": collections.len()
                    });
                }
                Err(e) => {
                    debug!("GRPC list_collections failed, falling back to local store: {}", e);
                }
            }
        }

        // Fallback to local vector store
        match crate::mcp::tools::McpTools::list_collections(vector_store).await {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    async fn handle_get_collection_info_tool(
        arguments: serde_json::Value,
        vector_store: &VectorStore,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
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

        // Note: get_collection_info uses local store as GRPC doesn't have this endpoint yet
        match crate::mcp::tools::McpTools::get_collection_info(collection, vector_store).await {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    async fn handle_embed_text_tool(
        arguments: serde_json::Value,
        embedding_manager: &EmbeddingManager,
    ) -> serde_json::Value {
        let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("");

        if text.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameter: text"
            });
        }

        match crate::mcp::tools::McpTools::embed_text(text, embedding_manager).await {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    async fn handle_insert_vectors_tool(
        arguments: serde_json::Value,
        vector_store: &VectorStore,
    ) -> serde_json::Value {
        let collection = arguments
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let empty_vec = vec![];
        let vectors = arguments
            .get("vectors")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        if collection.is_empty() || vectors.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameters: collection and vectors"
            });
        }

        // Parse vectors
        let parsed_vectors: std::result::Result<
            Vec<(String, Vec<f32>, Option<serde_json::Value>)>,
            _,
        > = vectors
            .iter()
            .map(|v| {
                let id = v
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let data = v
                    .get("data")
                    .and_then(|x| x.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect()
                    })
                    .unwrap_or_default();
                let payload = v.get("payload").cloned();
                Ok::<(String, Vec<f32>, Option<serde_json::Value>), String>((id, data, payload))
            })
            .collect();

        match parsed_vectors {
            Ok(vectors_data) => {
                match crate::mcp::tools::McpTools::insert_vectors(
                    collection,
                    vectors_data,
                    vector_store,
                )
                .await
                {
                    Ok(result) => result,
                    Err(e) => serde_json::json!({
                        "error": e.to_string()
                    }),
                }
            }
            Err(e) => serde_json::json!({
                "error": format!("Failed to parse vectors: {}", e)
            }),
        }
    }

    async fn handle_delete_vectors_tool(
        arguments: serde_json::Value,
        vector_store: &VectorStore,
    ) -> serde_json::Value {
        let collection = arguments
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let vector_ids: Vec<String> = arguments
            .get("vector_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if collection.is_empty() || vector_ids.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameters: collection and vector_ids"
            });
        }

        match crate::mcp::tools::McpTools::delete_vectors(collection, vector_ids, vector_store)
            .await
        {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    async fn handle_get_vector_tool(
        arguments: serde_json::Value,
        vector_store: &VectorStore,
    ) -> serde_json::Value {
        let collection = arguments
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let vector_id = arguments
            .get("vector_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if collection.is_empty() || vector_id.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameters: collection and vector_id"
            });
        }

        match crate::mcp::tools::McpTools::get_vector(collection, vector_id, vector_store).await {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    async fn handle_create_collection_tool(
        arguments: serde_json::Value,
        vector_store: &VectorStore,
    ) -> serde_json::Value {
        let name = arguments.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let dimension = arguments
            .get("dimension")
            .and_then(|v| v.as_u64())
            .unwrap_or(384) as usize;
        let metric = arguments
            .get("metric")
            .and_then(|v| v.as_str())
            .unwrap_or("cosine");

        if name.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameter: name"
            });
        }

        match crate::mcp::tools::McpTools::create_collection(name, dimension, metric, vector_store)
            .await
        {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    async fn handle_create_collection_grpc_tool(
        arguments: serde_json::Value,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> serde_json::Value {
        let name = arguments.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let dimension = arguments
            .get("dimension")
            .and_then(|v| v.as_u64())
            .unwrap_or(384) as i32;
        let metric = arguments
            .get("metric")
            .and_then(|v| v.as_str())
            .unwrap_or("cosine");

        if name.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameter: name"
            });
        }

        if let Some(ref mut client) = grpc_client {
            match client.create_collection(name.to_string(), dimension, metric.to_string()).await {
                Ok(response) => {
                    serde_json::json!({
                        "name": response.name,
                        "dimension": response.dimension,
                        "similarity_metric": response.similarity_metric,
                        "status": response.status,
                        "message": response.message
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "error": format!("GRPC create_collection failed: {}", e)
                    })
                }
            }
        } else {
            serde_json::json!({
                "error": "GRPC client not available"
            })
        }
    }

    async fn handle_delete_collection_tool(
        arguments: serde_json::Value,
        vector_store: &VectorStore,
    ) -> serde_json::Value {
        let name = arguments.get("name").and_then(|v| v.as_str()).unwrap_or("");

        if name.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameter: name"
            });
        }

        match crate::mcp::tools::McpTools::delete_collection(name, vector_store).await {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    async fn handle_get_database_stats_tool(vector_store: &VectorStore) -> serde_json::Value {
        match crate::mcp::tools::McpTools::get_database_stats(vector_store).await {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": e.to_string()
            }),
        }
    }

    // GRPC-specific handlers for new operations

    async fn handle_delete_collection_grpc_tool(
        arguments: serde_json::Value,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> serde_json::Value {
        let name = arguments.get("name").and_then(|v| v.as_str()).unwrap_or("");

        if name.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameter: name"
            });
        }

        if let Some(ref mut client) = grpc_client {
            match client.delete_collection(name.to_string()).await {
                Ok(response) => {
                    serde_json::json!({
                        "collection_name": response.collection_name,
                        "status": response.status,
                        "message": response.message
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "error": format!("GRPC delete_collection failed: {}", e)
                    })
                }
            }
        } else {
            serde_json::json!({
                "error": "GRPC client not available"
            })
        }
    }

    async fn handle_insert_vectors_grpc_tool(
        arguments: serde_json::Value,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> serde_json::Value {
        let collection = arguments
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let empty_vec = vec![];
        let vectors = arguments
            .get("vectors")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        if collection.is_empty() || vectors.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameters: collection and vectors"
            });
        }

        // Parse vectors
        let parsed_vectors: std::result::Result<
            Vec<(String, Vec<f32>, Option<std::collections::HashMap<String, String>>)>,
            _,
        > = vectors
            .iter()
            .map(|v| {
                let id = v
                    .get("id")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let data = v
                    .get("data")
                    .and_then(|x| x.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect()
                    })
                    .unwrap_or_default();
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
                Ok::<(String, Vec<f32>, Option<std::collections::HashMap<String, String>>), String>((id, data, metadata))
            })
            .collect();

        match parsed_vectors {
            Ok(vectors_data) => {
                if let Some(ref mut client) = grpc_client {
                    match client.insert_vectors(collection.to_string(), vectors_data).await {
                        Ok(response) => {
                            serde_json::json!({
                                "collection": response.collection,
                                "inserted_count": response.inserted_count,
                                "status": response.status,
                                "message": response.message
                            })
                        }
                        Err(e) => {
                            serde_json::json!({
                                "error": format!("GRPC insert_vectors failed: {}", e)
                            })
                        }
                    }
                } else {
                    serde_json::json!({
                        "error": "GRPC client not available"
                    })
                }
            }
            Err(e) => serde_json::json!({
                "error": format!("Failed to parse vectors: {}", e)
            }),
        }
    }

    async fn handle_delete_vectors_grpc_tool(
        arguments: serde_json::Value,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> serde_json::Value {
        let collection = arguments
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let vector_ids: Vec<String> = arguments
            .get("vector_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if collection.is_empty() || vector_ids.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameters: collection and vector_ids"
            });
        }

        if let Some(ref mut client) = grpc_client {
            match client.delete_vectors(collection.to_string(), vector_ids).await {
                Ok(response) => {
                    serde_json::json!({
                        "collection": response.collection,
                        "deleted_count": response.deleted_count,
                        "status": response.status,
                        "message": response.message
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "error": format!("GRPC delete_vectors failed: {}", e)
                    })
                }
            }
        } else {
            serde_json::json!({
                "error": "GRPC client not available"
            })
        }
    }

    async fn handle_get_vector_grpc_tool(
        arguments: serde_json::Value,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> serde_json::Value {
        let collection = arguments
            .get("collection")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let vector_id = arguments
            .get("vector_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if collection.is_empty() || vector_id.is_empty() {
            return serde_json::json!({
                "error": "Missing required parameters: collection and vector_id"
            });
        }

        if let Some(ref mut client) = grpc_client {
            match client.get_vector(collection.to_string(), vector_id.to_string()).await {
                Ok(response) => {
                    serde_json::json!({
                        "id": response.id,
                        "data": response.data,
                        "metadata": response.metadata,
                        "collection": response.collection,
                        "status": response.status
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "error": format!("GRPC get_vector failed: {}", e)
                    })
                }
            }
        } else {
            serde_json::json!({
                "error": "GRPC client not available"
            })
        }
    }

    async fn handle_get_collection_info_grpc_tool(
        arguments: serde_json::Value,
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
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

        if let Some(ref mut client) = grpc_client {
            match client.get_collection_info(collection.to_string()).await {
                Ok(response) => {
                    serde_json::json!({
                        "name": response.name,
                        "vector_count": response.vector_count,
                        "document_count": response.document_count,
                        "dimension": response.dimension,
                        "similarity_metric": response.similarity_metric,
                        "status": response.status,
                        "last_updated": response.last_updated,
                        "error_message": response.error_message
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "error": format!("GRPC get_collection_info failed: {}", e)
                    })
                }
            }
        } else {
            serde_json::json!({
                "error": "GRPC client not available"
            })
        }
    }

    async fn handle_get_indexing_progress_grpc_tool(
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> serde_json::Value {
        if let Some(ref mut client) = grpc_client {
            match client.get_indexing_progress().await {
                Ok(response) => {
                    let collections: Vec<serde_json::Value> = response.collections
                        .into_iter()
                        .map(|c| {
                            serde_json::json!({
                                "collection_name": c.collection_name,
                                "status": c.status,
                                "progress": c.progress,
                                "vector_count": c.vector_count,
                                "error_message": c.error_message,
                                "last_updated": c.last_updated
                            })
                        })
                        .collect();

                    serde_json::json!({
                        "collections": collections,
                        "is_indexing": response.is_indexing,
                        "overall_status": response.overall_status
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "error": format!("GRPC get_indexing_progress failed: {}", e)
                    })
                }
            }
        } else {
            serde_json::json!({
                "error": "GRPC client not available"
            })
        }
    }

    async fn handle_health_check_grpc_tool(
        mut grpc_client: Option<&mut VectorizerGrpcClient>,
    ) -> serde_json::Value {
        if let Some(ref mut client) = grpc_client {
            match client.health_check().await {
                Ok(response) => {
                    serde_json::json!({
                        "status": response.status,
                        "service": response.service,
                        "version": response.version,
                        "timestamp": response.timestamp,
                        "error_message": response.error_message
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "error": format!("GRPC health_check failed: {}", e)
                    })
                }
            }
        } else {
            serde_json::json!({
                "error": "GRPC client not available"
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_ping() {
        let response = McpHandler::handle_ping().await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert!(result.get("pong").unwrap().as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let config = crate::mcp::McpConfig::default();
        let state = McpServerState::new(config);

        let response = McpHandler::handle_tools_list(&state).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert!(result.get("tools").unwrap().is_array());
    }

    #[tokio::test]
    async fn test_handle_resources_list() {
        let config = crate::mcp::McpConfig::default();
        let state = McpServerState::new(config);

        let response = McpHandler::handle_resources_list(&state).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert!(result.get("resources").unwrap().is_array());
    }
}
