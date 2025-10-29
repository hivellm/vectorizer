//! MCP Integration Tests
//!
//! This module contains integration tests for the MCP server functionality.

use std::sync::Arc;

use rmcp::model::CallToolRequestParam;
use vectorizer::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::server::mcp_handlers::handle_mcp_tool;

/// Test MCP tool handling for basic operations
#[tokio::test]
async fn test_mcp_tool_handling() {
    // Create test vector store and embedding manager
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Test list collections tool
    let request = CallToolRequestParam {
        name: "list_collections".to_string().into(),
        arguments: Some(serde_json::Map::new()),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone()).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert!(!call_result.content.is_empty());

    // Test create collection tool
    let mut args = serde_json::Map::new();
    args.insert(
        "name".to_string(),
        serde_json::Value::String("test_collection".to_string()),
    );
    args.insert(
        "dimension".to_string(),
        serde_json::Value::Number(serde_json::Number::from(128)),
    );
    args.insert(
        "metric".to_string(),
        serde_json::Value::String("cosine".to_string()),
    );

    let request = CallToolRequestParam {
        name: "create_collection".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone()).await;
    assert!(result.is_ok());

    // Test get collection info tool
    let mut args = serde_json::Map::new();
    args.insert(
        "name".to_string(),
        serde_json::Value::String("test_collection".to_string()),
    );

    let request = CallToolRequestParam {
        name: "get_collection_info".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone()).await;
    assert!(result.is_ok());
}

/// Test MCP tool error handling
#[tokio::test]
async fn test_mcp_tool_error_handling() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Test unknown tool
    let request = CallToolRequestParam {
        name: "unknown_tool".to_string().into(),
        arguments: Some(serde_json::Map::new()),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone()).await;
    assert!(result.is_err());

    // Test missing arguments
    let request = CallToolRequestParam {
        name: "create_collection".to_string().into(),
        arguments: None,
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone()).await;
    assert!(result.is_err());
}

// Removed obsolete test_mcp_performance_tools - those MCP tools no longer exist
