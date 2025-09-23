//! Model Context Protocol (MCP) integration for Vectorizer
//! 
//! Provides MCP server implementation for IDE integration, allowing AI models
//! to interact with the vector database through a standardized protocol.

pub mod server;
pub mod tools;
pub mod types;
pub mod handlers;

pub use server::McpServer;
pub use tools::*;
pub use types::{
    McpMessage,
    McpRequestMessage,
    McpResponseMessage,
    McpNotificationMessage,
    McpErrorMessage,
};
pub use handlers::*;

use crate::error::{Result, VectorizerError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Enable MCP server
    pub enabled: bool,
    /// MCP server port
    pub port: u16,
    /// MCP server host
    pub host: String,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Enable authentication for MCP
    pub auth_required: bool,
    /// Allowed API keys for MCP access
    pub allowed_api_keys: Vec<String>,
    /// Server information
    pub server_info: McpServerInfo,
    /// Available tools
    pub tools: Vec<McpTool>,
    /// Available resources
    pub resources: Vec<McpResource>,
    /// Performance settings
    pub performance: McpPerformanceConfig,
    /// Logging settings
    pub logging: McpLoggingConfig,
}

/// MCP performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPerformanceConfig {
    /// Enable connection pooling
    pub connection_pooling: bool,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Cleanup interval for inactive connections in seconds
    pub cleanup_interval: u64,
}

/// MCP logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpLoggingConfig {
    /// Log level
    pub level: String,
    /// Log requests
    pub log_requests: bool,
    /// Log responses
    pub log_responses: bool,
    /// Log errors
    pub log_errors: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 15003,
            host: "127.0.0.1".to_string(),
            max_connections: 10,
            connection_timeout: 300, // 5 minutes
            auth_required: true,
            allowed_api_keys: vec![],
            server_info: McpServerInfo {
                name: "Vectorizer MCP Server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: "Model Context Protocol server for Vectorizer vector database".to_string(),
            },
            tools: vec![
                McpTool {
                    name: "search_vectors".to_string(),
                    description: "Search for similar vectors in a collection".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "collection": {"type": "string", "description": "Collection name"},
                            "query": {"type": "string", "description": "Search query"},
                            "limit": {"type": "integer", "description": "Maximum number of results", "default": 10}
                        },
                        "required": ["collection", "query"]
                    }),
                },
                McpTool {
                    name: "list_collections".to_string(),
                    description: "List all available collections".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
                McpTool {
                    name: "get_collection_info".to_string(),
                    description: "Get information about a specific collection".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "collection": {"type": "string", "description": "Collection name"}
                        },
                        "required": ["collection"]
                    }),
                },
                McpTool {
                    name: "embed_text".to_string(),
                    description: "Generate embeddings for text using the default embedding model".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "text": {"type": "string", "description": "Text to embed"}
                        },
                        "required": ["text"]
                    }),
                },
            ],
            resources: vec![
                McpResource {
                    uri: "vectorizer://collections".to_string(),
                    name: "Collections".to_string(),
                    description: "List of all collections in the vector database".to_string(),
                    mime_type: "application/json".to_string(),
                },
                McpResource {
                    uri: "vectorizer://stats".to_string(),
                    name: "Database Statistics".to_string(),
                    description: "Current database statistics and performance metrics".to_string(),
                    mime_type: "application/json".to_string(),
                },
            ],
            performance: McpPerformanceConfig {
                connection_pooling: true,
                max_message_size: 1048576, // 1MB
                heartbeat_interval: 30,
                cleanup_interval: 300, // 5 minutes
            },
            logging: McpLoggingConfig {
                level: "info".to_string(),
                log_requests: true,
                log_responses: false,
                log_errors: true,
            },
        }
    }
}

/// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    /// Server information
    pub server_info: McpServerInfo,
    /// Available tools
    pub tools: Vec<McpTool>,
    /// Available resources
    pub resources: Vec<McpResource>,
}

/// MCP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Server description
    pub description: String,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema
    pub input_schema: serde_json::Value,
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: String,
    /// Resource MIME type
    pub mime_type: String,
}

/// MCP request types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    /// Initialize connection
    Initialize {
        protocol_version: String,
        capabilities: serde_json::Value,
        client_info: serde_json::Value,
    },
    /// List available tools
    ToolsList,
    /// Call a tool
    ToolsCall {
        name: String,
        arguments: serde_json::Value,
    },
    /// List available resources
    ResourcesList,
    /// Read a resource
    ResourcesRead {
        uri: String,
    },
    /// Ping request
    Ping,
}

/// MCP response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// Response ID
    pub id: Option<String>,
    /// Response result
    pub result: Option<serde_json::Value>,
    /// Error information
    pub error: Option<McpError>,
}

/// MCP error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Error data
    pub data: Option<serde_json::Value>,
}

/// MCP connection state
#[derive(Debug)]
pub struct McpConnection {
    /// Connection ID
    pub id: String,
    /// Client capabilities
    pub client_capabilities: serde_json::Value,
    /// Connection timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Authentication status
    pub authenticated: bool,
}

/// MCP server state
#[derive(Debug)]
pub struct McpServerState {
    /// Active connections
    pub connections: Arc<RwLock<HashMap<String, McpConnection>>>,
    /// Server configuration
    pub config: McpConfig,
    /// Server capabilities
    pub capabilities: McpCapabilities,
}

impl McpServerState {
    /// Create new MCP server state
    pub fn new(config: McpConfig) -> Self {
        let capabilities = McpCapabilities {
            server_info: McpServerInfo {
                name: "Vectorizer MCP Server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: "Model Context Protocol server for Vectorizer vector database".to_string(),
            },
            tools: vec![
                McpTool {
                    name: "search_vectors".to_string(),
                    description: "Search for similar vectors in a collection".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "collection": {"type": "string", "description": "Collection name"},
                            "query": {"type": "string", "description": "Search query"},
                            "limit": {"type": "integer", "description": "Maximum number of results", "default": 10}
                        },
                        "required": ["collection", "query"]
                    }),
                },
                McpTool {
                    name: "list_collections".to_string(),
                    description: "List all available collections".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
                McpTool {
                    name: "get_collection_info".to_string(),
                    description: "Get information about a specific collection".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "collection": {"type": "string", "description": "Collection name"}
                        },
                        "required": ["collection"]
                    }),
                },
                McpTool {
                    name: "embed_text".to_string(),
                    description: "Generate embeddings for text using the default embedding model".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "text": {"type": "string", "description": "Text to embed"}
                        },
                        "required": ["text"]
                    }),
                },
            ],
            resources: vec![
                McpResource {
                    uri: "vectorizer://collections".to_string(),
                    name: "Collections".to_string(),
                    description: "List of all collections in the vector database".to_string(),
                    mime_type: "application/json".to_string(),
                },
                McpResource {
                    uri: "vectorizer://stats".to_string(),
                    name: "Database Statistics".to_string(),
                    description: "Current database statistics and performance metrics".to_string(),
                    mime_type: "application/json".to_string(),
                },
            ],
        };

        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
            capabilities,
        }
    }

    /// Add a new connection
    pub async fn add_connection(&self, connection_id: String, connection: McpConnection) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if connections.len() >= self.config.max_connections {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Maximum connections exceeded".to_string(),
            });
        }
        
        connections.insert(connection_id, connection);
        Ok(())
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);
        Ok(())
    }

    /// Update connection activity
    pub async fn update_activity(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.last_activity = chrono::Utc::now();
        }
        
        Ok(())
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Clean up inactive connections
    pub async fn cleanup_inactive_connections(&self) -> Result<usize> {
        let mut connections = self.connections.write().await;
        let now = chrono::Utc::now();
        let timeout_duration = chrono::Duration::seconds(self.config.connection_timeout as i64);
        
        let inactive_connections: Vec<String> = connections
            .iter()
            .filter(|(_, conn)| now - conn.last_activity > timeout_duration)
            .map(|(id, _)| id.clone())
            .collect();
        
        for connection_id in &inactive_connections {
            connections.remove(connection_id);
        }
        
        Ok(inactive_connections.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_default() {
        let config = McpConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 15003);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_mcp_server_state_creation() {
        let config = McpConfig::default();
        let state = McpServerState::new(config);
        
        assert_eq!(state.capabilities.server_info.name, "Vectorizer MCP Server");
        assert!(!state.capabilities.tools.is_empty());
        assert!(!state.capabilities.resources.is_empty());
    }

    #[tokio::test]
    async fn test_mcp_connection_management() {
        let config = McpConfig::default();
        let state = McpServerState::new(config);
        
        let connection = McpConnection {
            id: "test_connection".to_string(),
            client_capabilities: serde_json::json!({}),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            authenticated: false,
        };
        
        // Add connection
        state.add_connection("test_connection".to_string(), connection).await.unwrap();
        assert_eq!(state.connection_count().await, 1);
        
        // Update activity
        state.update_activity("test_connection").await.unwrap();
        
        // Remove connection
        state.remove_connection("test_connection").await.unwrap();
        assert_eq!(state.connection_count().await, 0);
    }
}
