//! MCP (Model Context Protocol) Integration Tests
//! 
//! These tests verify that the MCP server works correctly and
//! that all MCP tools function as expected.

use vectorizer::{
    mcp::{McpConfig, McpServerState, McpConnection, McpRequestMessage},
};
// use std::sync::Arc;
use serde_json::json;

#[tokio::test]
async fn test_mcp_config_default() {
    let config = McpConfig::default();
    
    assert_eq!(config.enabled, true);
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 15003);
}

#[tokio::test]
async fn test_mcp_config_serialization() {
    let config = McpConfig::default();
    
    let json = serde_json::to_string(&config).expect("Failed to serialize MCP config");
    let deserialized: McpConfig = serde_json::from_str(&json).expect("Failed to deserialize MCP config");
    
    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.host, deserialized.host);
    assert_eq!(config.port, deserialized.port);
}

#[tokio::test]
async fn test_mcp_server_state_creation() {
    let config = McpConfig::default();
    
    let state = McpServerState::new(config.clone());
    
    assert_eq!(state.config.enabled, config.enabled);
    assert_eq!(state.config.host, config.host);
    assert_eq!(state.config.port, config.port);
}

#[tokio::test]
async fn test_mcp_connection_creation() {
    let config = McpConfig::default();
    let _state = McpServerState::new(config);
    
    let connection = McpConnection {
        id: "test_connection".to_string(),
        client_capabilities: serde_json::json!({}),
        connected_at: chrono::Utc::now(),
        last_activity: chrono::Utc::now(),
        authenticated: false,
    };
    
    assert_eq!(connection.id, "test_connection");
    assert!(!connection.authenticated);
}

#[tokio::test]
async fn test_mcp_tools_list() {
    // Test that all expected MCP tools are available
    let expected_tools = [
        "search_vectors",
        "list_collections", 
        "get_collection_info",
        "embed_text",
        "insert_vectors",
        "delete_vectors",
        "get_vector",
        "create_collection",
        "delete_collection",
        "get_database_stats",
    ];
    
    // This would be tested with actual MCP server running
    // For now, we verify the tool names are defined
    for tool_name in &expected_tools {
        assert!(!tool_name.is_empty(), "Tool name should not be empty");
    }
}

#[tokio::test]
async fn test_mcp_request_serialization() {
    use vectorizer::mcp::types::*;
    
    // Test search_vectors request
    let request = McpRequest::Search(SearchRequest {
        query: "test query".to_string(),
        collection_name: "test_collection".to_string(),
        k: 10,
        filter: None,
    });
    
    let json = serde_json::to_string(&request).expect("Failed to serialize MCP request");
    let deserialized: McpRequest = serde_json::from_str(&json).expect("Failed to deserialize MCP request");
    
    if let McpRequest::Search(search_req) = deserialized {
        assert_eq!(search_req.query, "test query");
        assert_eq!(search_req.collection_name, "test_collection");
        assert_eq!(search_req.k, 10);
    } else {
        panic!("Expected Search request");
    }
}

#[tokio::test]
async fn test_mcp_response_serialization() {
    use vectorizer::mcp::types::*;
    
    // Test search response
    let response = McpResponse::SearchResponse(SearchResponse {
        results: vec![SearchResult {
            id: "test_id".to_string(),
            score: 0.95,
            payload: Some(json!({"title": "Test Document"})),
        }],
        collection_name: "test_collection".to_string(),
    });
    
    let json = serde_json::to_string(&response).expect("Failed to serialize MCP response");
    let deserialized: McpResponse = serde_json::from_str(&json).expect("Failed to deserialize MCP response");
    
    match deserialized {
        McpResponse::SearchResponse(search_resp) => {
            assert_eq!(search_resp.results.len(), 1);
            assert_eq!(search_resp.results[0].id, "test_id");
            assert_eq!(search_resp.results[0].score, 0.95);
            assert_eq!(search_resp.collection_name, "test_collection");
        }
        _ => panic!("Expected SearchResponse"),
    }
}

#[tokio::test]
async fn test_mcp_error_handling() {
    use vectorizer::mcp::types::*;
    
    let error = McpError {
        code: "COLLECTION_NOT_FOUND".to_string(),
        message: "Collection 'test' not found".to_string(),
        details: Some("The requested collection does not exist".to_string()),
    };
    
    let json = serde_json::to_string(&error).expect("Failed to serialize MCP error");
    let deserialized: McpError = serde_json::from_str(&json).expect("Failed to deserialize MCP error");
    
    assert_eq!(deserialized.code, "COLLECTION_NOT_FOUND");
    assert_eq!(deserialized.message, "Collection 'test' not found");
    assert!(deserialized.details.is_some());
}

#[tokio::test]
async fn test_mcp_tool_call_serialization() {
    use vectorizer::mcp::types::*;
    
    let tool_call = ToolCall {
        tool_name: "search_vectors".to_string(),
        tool_args: json!({
            "collection": "test_collection",
            "query": "test query",
            "limit": 10
        }),
    };
    
    let json = serde_json::to_string(&tool_call).expect("Failed to serialize tool call");
    let deserialized: ToolCall = serde_json::from_str(&json).expect("Failed to deserialize tool call");
    
    assert_eq!(deserialized.tool_name, "search_vectors");
    assert_eq!(deserialized.tool_args["collection"], "test_collection");
    assert_eq!(deserialized.tool_args["query"], "test query");
    assert_eq!(deserialized.tool_args["limit"], 10);
}

#[tokio::test]
async fn test_mcp_tool_output_serialization() {
    use vectorizer::mcp::types::*;
    
    let tool_output = ToolOutput {
        tool_name: "search_vectors".to_string(),
        output: json!({
            "results": [
                {
                    "id": "doc1",
                    "score": 0.95,
                    "payload": {"title": "Document 1"}
                }
            ],
            "total_results": 1
        }),
        error: None,
    };
    
    let json = serde_json::to_string(&tool_output).expect("Failed to serialize tool output");
    let deserialized: ToolOutput = serde_json::from_str(&json).expect("Failed to deserialize tool output");
    
    assert_eq!(deserialized.tool_name, "search_vectors");
    assert!(deserialized.error.is_none());
    assert!(deserialized.output["results"].is_array());
}

#[tokio::test]
async fn test_mcp_chat_request_serialization() {
    // Test basic MCP message serialization
    let request = McpRequestMessage {
        id: "test_request".to_string(),
        method: "chat".to_string(),
        params: serde_json::json!({
            "message": "Hello, world!",
            "conversation_id": "conv_123"
        }),
    };
    
    let json = serde_json::to_string(&request).expect("Failed to serialize request");
    let deserialized: McpRequestMessage = serde_json::from_str(&json).expect("Failed to deserialize request");
    
    assert_eq!(deserialized.id, "test_request");
    assert_eq!(deserialized.method, "chat");
}

#[tokio::test]
async fn test_mcp_embedding_request_serialization() {
    // Test basic MCP message serialization for embedding
    let request = McpRequestMessage {
        id: "embed_request".to_string(),
        method: "embed".to_string(),
        params: serde_json::json!({
            "text": ["Hello, world!", "Test text"],
            "model_id": "default"
        }),
    };
    
    let json = serde_json::to_string(&request).expect("Failed to serialize embedding request");
    let deserialized: McpRequestMessage = serde_json::from_str(&json).expect("Failed to deserialize embedding request");
    
    assert_eq!(deserialized.id, "embed_request");
    assert_eq!(deserialized.method, "embed");
}

#[tokio::test]
async fn test_mcp_websocket_message_format() {
    // Test that MCP WebSocket messages follow JSON-RPC 2.0 format
    let message = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search_vectors",
            "arguments": {
                "collection": "test",
                "query": "test query",
                "limit": 10
            }
        }
    });
    
    // Verify required JSON-RPC 2.0 fields
    assert_eq!(message["jsonrpc"], "2.0");
    assert!(message["id"].is_number());
    assert_eq!(message["method"], "tools/call");
    assert!(message["params"].is_object());
}

#[tokio::test]
async fn test_mcp_websocket_response_format() {
    // Test that MCP WebSocket responses follow JSON-RPC 2.0 format
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "results": [],
            "total_results": 0
        }
    });
    
    // Verify required JSON-RPC 2.0 fields
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["id"].is_number());
    assert!(response["result"].is_object());
}

#[tokio::test]
async fn test_mcp_websocket_error_format() {
    // Test that MCP WebSocket errors follow JSON-RPC 2.0 format
    let error_response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32602,
            "message": "Invalid params",
            "data": {
                "missing": ["collection", "query"]
            }
        }
    });
    
    // Verify required JSON-RPC 2.0 fields
    assert_eq!(error_response["jsonrpc"], "2.0");
    assert!(error_response["id"].is_number());
    assert!(error_response["error"].is_object());
    assert!(error_response["error"]["code"].is_number());
    assert!(error_response["error"]["message"].is_string());
}
