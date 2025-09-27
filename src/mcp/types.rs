//! MCP type definitions
//!
//! Common types and structures used throughout the MCP implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpMessage {
    Request(McpRequestMessage),
    Response(McpResponseMessage),
    Notification(McpNotificationMessage),
}

/// MCP request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequestMessage {
    /// Request ID
    pub id: String,
    /// Method name
    pub method: String,
    /// Method parameters
    pub params: serde_json::Value,
}

/// MCP response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponseMessage {
    /// Response ID (matches request ID)
    pub id: String,
    /// Response result
    pub result: Option<serde_json::Value>,
    /// Error information
    pub error: Option<McpErrorMessage>,
}

/// MCP notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotificationMessage {
    /// Method name
    pub method: String,
    /// Method parameters
    pub params: serde_json::Value,
}

/// MCP error message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpErrorMessage {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Error data
    pub data: Option<serde_json::Value>,
}

/// MCP client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientCapabilities {
    /// Experimental features
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    /// Sampling capabilities
    pub sampling: Option<McpSamplingCapabilities>,
}

/// MCP sampling capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSamplingCapabilities {
    /// Whether sampling is supported
    pub supported: bool,
}

/// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerCapabilities {
    /// Tool capabilities
    pub tools: Option<McpToolCapabilities>,
    /// Resource capabilities
    pub resources: Option<McpResourceCapabilities>,
    /// Prompt capabilities
    pub prompts: Option<McpPromptCapabilities>,
    /// Logging capabilities
    pub logging: Option<McpLoggingCapabilities>,
}

/// MCP tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCapabilities {
    /// Whether tools are supported
    pub supported: bool,
    /// Whether tool calls can be made
    pub call_tool: Option<bool>,
}

/// MCP resource capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceCapabilities {
    /// Whether resources are supported
    pub supported: bool,
    /// Whether resources can be subscribed to
    pub subscribe: Option<bool>,
    /// Whether resources can be listed
    pub list_changed: Option<bool>,
}

/// MCP prompt capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptCapabilities {
    /// Whether prompts are supported
    pub supported: bool,
}

/// MCP logging capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpLoggingCapabilities {
    /// Whether logging is supported
    pub supported: bool,
}

/// MCP initialization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInitializeParams {
    /// Protocol version
    pub protocol_version: String,
    /// Client capabilities
    pub capabilities: McpClientCapabilities,
    /// Client information
    pub client_info: McpClientInfo,
}

/// MCP client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
}

/// MCP initialization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInitializeResult {
    /// Protocol version
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: McpServerCapabilities,
    /// Server information
    pub server_info: McpServerInfo,
}

/// MCP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// Input schema
    pub input_schema: serde_json::Value,
}

/// MCP tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    /// Tool name
    pub name: String,
    /// Tool arguments
    pub arguments: serde_json::Value,
}

/// MCP tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    /// Tool name
    pub name: String,
    /// Tool result content
    pub content: Vec<McpContent>,
    /// Whether the tool call was successful
    pub is_error: Option<bool>,
}

/// MCP content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource {
        resource: McpResourceReference,
        text: Option<String>,
    },
}

/// MCP resource reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceReference {
    /// Resource URI
    pub uri: String,
    /// Resource text
    pub text: Option<String>,
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceDefinition {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: Option<String>,
    /// Resource MIME type
    pub mime_type: Option<String>,
}

/// MCP resource read result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceReadResult {
    /// Resource contents
    pub contents: Vec<McpContent>,
}

/// MCP prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptDefinition {
    /// Prompt name
    pub name: String,
    /// Prompt description
    pub description: Option<String>,
    /// Prompt arguments
    pub arguments: Option<Vec<McpPromptArgument>>,
}

/// MCP prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    /// Argument name
    pub name: String,
    /// Argument description
    pub description: Option<String>,
    /// Whether the argument is required
    pub required: Option<bool>,
}

/// MCP prompt result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptResult {
    /// Prompt name
    pub name: String,
    /// Prompt result content
    pub content: Vec<McpContent>,
    /// Whether the prompt was successful
    pub is_error: Option<bool>,
}

/// MCP log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpLogEntry {
    /// Log level
    pub level: McpLogLevel,
    /// Log data
    pub data: serde_json::Value,
    /// Log logger
    pub logger: Option<String>,
}

/// MCP log level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpLogLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

/// MCP connection state
#[derive(Debug, Clone)]
pub struct McpConnectionState {
    /// Connection ID
    pub id: String,
    /// Client information
    pub client_info: Option<McpClientInfo>,
    /// Client capabilities
    pub client_capabilities: Option<McpClientCapabilities>,
    /// Connection timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Authentication status
    pub authenticated: bool,
    /// Connection status
    pub status: McpConnectionStatus,
}

/// MCP connection status
#[derive(Debug, Clone, PartialEq)]
pub enum McpConnectionStatus {
    /// Connection is being established
    Connecting,
    /// Connection is active
    Connected,
    /// Connection is being closed
    Disconnecting,
    /// Connection is closed
    Disconnected,
}

impl Serialize for McpConnectionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            McpConnectionStatus::Connecting => serializer.serialize_str("connecting"),
            McpConnectionStatus::Connected => serializer.serialize_str("connected"),
            McpConnectionStatus::Disconnecting => serializer.serialize_str("disconnecting"),
            McpConnectionStatus::Disconnected => serializer.serialize_str("disconnected"),
        }
    }
}

impl<'de> Deserialize<'de> for McpConnectionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "connecting" => Ok(McpConnectionStatus::Connecting),
            "connected" => Ok(McpConnectionStatus::Connected),
            "disconnecting" => Ok(McpConnectionStatus::Disconnecting),
            "disconnected" => Ok(McpConnectionStatus::Disconnected),
            _ => Err(serde::de::Error::custom("Invalid connection status")),
        }
    }
}

// -----------------------------------------------------------------------------
// Test-facing MCP types (compat layer)
// These mirror the simplified types expected by the test suite.
// -----------------------------------------------------------------------------

/// MCP high-level request used by tests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum McpRequest {
    /// Search request
    Search(SearchRequest),
}

/// Search request payload (MCP tests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Natural language query
    pub query: String,
    /// Target collection name
    pub collection_name: String,
    /// Top-K results
    pub k: usize,
    /// Optional filter
    pub filter: Option<serde_json::Value>,
}

/// Search result item (MCP tests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Vector/document identifier
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Optional payload
    pub payload: Option<serde_json::Value>,
}

/// Search response (MCP tests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Results list
    pub results: Vec<SearchResult>,
    /// Collection searched
    pub collection_name: String,
}

/// MCP high-level response used by tests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum McpResponse {
    /// Response for search
    SearchResponse(SearchResponse),
    /// Error response
    Error(McpError),
}

/// Simplified MCP error (MCP tests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    /// Error code string
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional details
    pub details: Option<String>,
}

/// Tool call format (MCP tests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name
    pub tool_name: String,
    /// Tool arguments
    pub tool_args: serde_json::Value,
}

/// Tool output format (MCP tests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Tool name
    pub tool_name: String,
    /// Serialized output
    pub output: serde_json::Value,
    /// Optional error message
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_message_serialization() {
        let request = McpMessage::Request(McpRequestMessage {
            id: "1".to_string(),
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
        });

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("tools/list"));
        // Check if it's a valid JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_mcp_capabilities() {
        let capabilities = McpClientCapabilities {
            experimental: Some(HashMap::new()),
            sampling: Some(McpSamplingCapabilities { supported: true }),
        };

        let json = serde_json::to_string(&capabilities).unwrap();
        assert!(json.contains("sampling"));
    }

    #[test]
    fn test_mcp_connection_status() {
        let status = McpConnectionStatus::Connected;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"connected\"");
    }
}
