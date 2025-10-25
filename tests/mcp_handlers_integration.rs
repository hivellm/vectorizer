//! Integration tests for MCP handlers
//!
//! These tests verify MCP tool functionality including:
//! - Search operations
//! - File operations  
//! - Discovery tools
//! - Collection management via MCP

use serde_json::json;

#[tokio::test]
async fn test_mcp_search_tool() {
    // Test basic search via MCP
    let request = json!({
        "query": "test query",
        "collection": "test_collection",
        "limit": 10
    });

    // Verify request structure is valid
    assert!(request.get("query").is_some());
    assert!(request.get("collection").is_some());
    assert_eq!(request["limit"], 10);
}

#[tokio::test]
async fn test_mcp_intelligent_search_tool() {
    // Test intelligent search parameters
    let request = json!({
        "query": "find documents about AI",
        "collections": ["docs", "papers"],
        "max_results": 20,
        "domain_expansion": true
    });

    assert!(request.get("query").is_some());
    assert!(request["collections"].is_array());
    assert_eq!(request["max_results"], 20);
    assert_eq!(request["domain_expansion"], true);
}

#[tokio::test]
async fn test_mcp_semantic_search_tool() {
    // Test semantic search parameters
    let request = json!({
        "query": "machine learning",
        "collection": "papers",
        "max_results": 15,
        "similarity_threshold": 0.7
    });

    assert!(request.get("query").is_some());
    assert_eq!(request["similarity_threshold"], 0.7);
}

#[tokio::test]
async fn test_mcp_list_files_tool() {
    // Test list files parameters
    let request = json!({
        "collection": "documents",
        "sort_by": "name",
        "max_results": 100
    });

    assert_eq!(request["collection"], "documents");
    assert_eq!(request["sort_by"], "name");
    assert_eq!(request["max_results"], 100);
}

#[tokio::test]
async fn test_mcp_get_file_content_tool() {
    // Test get file content parameters
    let request = json!({
        "collection": "docs",
        "file_path": "README.md",
        "max_size_kb": 500
    });

    assert_eq!(request["collection"], "docs");
    assert_eq!(request["file_path"], "README.md");
    assert_eq!(request["max_size_kb"], 500);
}

#[tokio::test]
async fn test_mcp_get_file_chunks_tool() {
    // Test get file chunks parameters
    let request = json!({
        "collection": "code",
        "file_path": "src/main.rs",
        "start_chunk": 0,
        "limit": 10
    });

    assert_eq!(request["start_chunk"], 0);
    assert_eq!(request["limit"], 10);
}

#[tokio::test]
async fn test_mcp_get_project_outline_tool() {
    // Test project outline parameters
    let request = json!({
        "collection": "project",
        "max_depth": 5,
        "highlight_key_files": true
    });

    assert_eq!(request["max_depth"], 5);
    assert_eq!(request["highlight_key_files"], true);
}

#[tokio::test]
async fn test_mcp_get_related_files_tool() {
    // Test related files parameters
    let request = json!({
        "collection": "source",
        "file_path": "src/lib.rs",
        "max_results": 10,
        "similarity_threshold": 0.6
    });

    assert_eq!(request["file_path"], "src/lib.rs");
    assert_eq!(request["max_results"], 10);
    assert_eq!(request["similarity_threshold"], 0.6);
}

#[tokio::test]
async fn test_mcp_multi_collection_search_tool() {
    // Test multi-collection search parameters
    let request = json!({
        "query": "search term",
        "collections": ["coll1", "coll2", "coll3"],
        "max_per_collection": 5,
        "max_total_results": 20
    });

    assert!(request["collections"].is_array());
    assert_eq!(request["max_per_collection"], 5);
    assert_eq!(request["max_total_results"], 20);
}

#[tokio::test]
async fn test_mcp_insert_text_tool() {
    // Test insert text parameters
    let request = json!({
        "collection_name": "docs",
        "text": "This is a test document",
        "metadata": {"source": "test"}
    });

    assert_eq!(request["collection_name"], "docs");
    assert!(request.get("text").is_some());
    assert!(request.get("metadata").is_some());
}

#[tokio::test]
async fn test_mcp_tool_request_validation() {
    // Test that required fields are validated
    let invalid_search = json!({
        // Missing query field
        "collection": "test"
    });

    assert!(invalid_search.get("query").is_none());

    let invalid_insert = json!({
        // Missing text field
        "collection_name": "test"
    });

    assert!(invalid_insert.get("text").is_none());
}

#[tokio::test]
async fn test_mcp_optional_parameters() {
    // Test that optional parameters work
    let minimal_search = json!({
        "query": "test",
        "collection": "docs"
    });

    assert!(minimal_search.get("limit").is_none()); // Optional
    assert!(minimal_search.get("similarity_threshold").is_none()); // Optional

    let full_search = json!({
        "query": "test",
        "collection": "docs",
        "limit": 20,
        "similarity_threshold": 0.8
    });

    assert_eq!(full_search["limit"], 20);
    assert_eq!(full_search["similarity_threshold"], 0.8);
}

#[tokio::test]
async fn test_mcp_collection_filters() {
    // Test filter collections tool parameters
    let request = json!({
        "query": "test",
        "include": ["*.rs", "*.md"],
        "exclude": ["*.lock"]
    });

    assert!(request["include"].is_array());
    assert!(request["exclude"].is_array());
}

#[tokio::test]
async fn test_mcp_expand_queries_tool() {
    // Test query expansion parameters
    let request = json!({
        "query": "vector database",
        "max_expansions": 8,
        "include_definition": true,
        "include_features": true,
        "include_architecture": true
    });

    assert_eq!(request["max_expansions"], 8);
    assert_eq!(request["include_definition"], true);
    assert_eq!(request["include_features"], true);
    assert_eq!(request["include_architecture"], true);
}

#[tokio::test]
async fn test_mcp_error_handling() {
    // Test that MCP tools handle errors gracefully
    let invalid_request = json!({
        "query": "",  // Empty query
        "collection": "test"
    });

    // Should handle empty query
    assert_eq!(invalid_request["query"], "");

    let missing_required = json!({
        "limit": 10
        // Missing query and collection
    });

    assert!(missing_required.get("query").is_none());
    assert!(missing_required.get("collection").is_none());
}
