//! Error handling tests for the Rust SDK
//! Tests for all VectorizerError variants and error mapping

use serde_json;
use vectorizer_sdk::*;

#[test]
fn test_authentication_error() {
    let error = VectorizerError::authentication("Invalid API key");
    assert!(matches!(
        error,
        VectorizerError::Authentication { message: _ }
    ));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Authentication failed"));
    assert!(error_msg.contains("Invalid API key"));
}

#[test]
fn test_collection_not_found_error() {
    let error = VectorizerError::collection_not_found("test_collection");
    assert!(
        matches!(error, VectorizerError::CollectionNotFound { ref collection } if collection == "test_collection")
    );

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Collection 'test_collection' not found"));
}

#[test]
fn test_vector_not_found_error() {
    let error = VectorizerError::vector_not_found("test_collection", "vector_123");
    assert!(
        matches!(error, VectorizerError::VectorNotFound { ref collection, ref vector_id } 
        if collection == "test_collection" && vector_id == "vector_123")
    );

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Vector 'vector_123' not found in collection 'test_collection'"));
}

#[test]
fn test_validation_error() {
    let error = VectorizerError::validation("Invalid input parameters");
    assert!(matches!(error, VectorizerError::Validation { ref message } 
        if message == "Invalid input parameters"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Validation error"));
    assert!(error_msg.contains("Invalid input parameters"));
}

#[test]
fn test_network_error() {
    let error = VectorizerError::network("Connection timeout");
    assert!(matches!(error, VectorizerError::Network { ref message } 
        if message == "Connection timeout"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Network error"));
    assert!(error_msg.contains("Connection timeout"));
}

#[test]
fn test_server_error() {
    let error = VectorizerError::server("Internal server error");
    assert!(matches!(error, VectorizerError::Server { ref message } 
        if message == "Internal server error"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Server error"));
    assert!(error_msg.contains("Internal server error"));
}

#[test]
fn test_timeout_error() {
    let error = VectorizerError::timeout(30);
    assert!(matches!(error, VectorizerError::Timeout { timeout_secs } 
        if timeout_secs == 30));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Request timeout after 30s"));
}

#[test]
fn test_rate_limit_error() {
    let error = VectorizerError::rate_limit("Too many requests");
    assert!(matches!(error, VectorizerError::RateLimit { ref message } 
        if message == "Too many requests"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Rate limit exceeded"));
    assert!(error_msg.contains("Too many requests"));
}

#[test]
fn test_configuration_error() {
    let error = VectorizerError::configuration("Invalid configuration");
    assert!(
        matches!(error, VectorizerError::Configuration { ref message } 
        if message == "Invalid configuration")
    );

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Configuration error"));
    assert!(error_msg.contains("Invalid configuration"));
}

#[test]
fn test_embedding_error() {
    let error = VectorizerError::embedding("Embedding generation failed");
    assert!(matches!(error, VectorizerError::Embedding { ref message } 
        if message == "Embedding generation failed"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Embedding generation failed"));
    assert!(error_msg.contains("Embedding generation failed"));
}

#[test]
fn test_search_error() {
    let error = VectorizerError::search("Search operation failed");
    assert!(matches!(error, VectorizerError::Search { ref message } 
        if message == "Search operation failed"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Search failed"));
    assert!(error_msg.contains("Search operation failed"));
}

#[test]
fn test_storage_error() {
    let error = VectorizerError::storage("Storage operation failed");
    assert!(matches!(error, VectorizerError::Storage { ref message } 
        if message == "Storage operation failed"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Storage error"));
    assert!(error_msg.contains("Storage operation failed"));
}

#[test]
fn test_batch_operation_error() {
    let error = VectorizerError::batch_operation("Batch operation failed");
    assert!(
        matches!(error, VectorizerError::BatchOperation { ref message } 
        if message == "Batch operation failed")
    );

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Batch operation failed"));
    assert!(error_msg.contains("Batch operation failed"));
}

#[test]
fn test_mcp_error() {
    let error = VectorizerError::mcp("MCP communication failed");
    assert!(matches!(error, VectorizerError::Mcp { ref message } 
        if message == "MCP communication failed"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("MCP error"));
    assert!(error_msg.contains("MCP communication failed"));
}

#[test]
fn test_serialization_error() {
    let error = VectorizerError::Serialization("JSON parsing failed".to_string());
    assert!(matches!(error, VectorizerError::Serialization(ref message) 
        if message == "JSON parsing failed"));

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Serialization error"));
    assert!(error_msg.contains("JSON parsing failed"));
}

#[tokio::test]
async fn test_http_error_conversion() {
    use reqwest::Error as ReqwestError;

    // Create a mock HTTP error (this is a simplified test)
    // In real scenarios, this would come from reqwest operations
    // Create a reqwest error by making a request that will fail
    let client = reqwest::Client::new();
    let result = client.get("http://localhost:9999/nonexistent").send().await;
    let reqwest_error = result.unwrap_err();
    let http_error = VectorizerError::Http(reqwest_error);

    let error_msg = format!("{}", http_error);
    assert!(error_msg.contains("HTTP error"));
}

#[test]
fn test_io_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let vectorizer_error = VectorizerError::from(io_error);

    assert!(matches!(vectorizer_error, VectorizerError::Io(_)));
    let error_msg = format!("{}", vectorizer_error);
    assert!(error_msg.contains("IO error"));
}

#[test]
fn test_serde_json_error_conversion() {
    let json_error = serde_json::Error::io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid JSON",
    ));
    let vectorizer_error = VectorizerError::from(json_error);

    assert!(matches!(
        vectorizer_error,
        VectorizerError::Serialization(_)
    ));
    let error_msg = format!("{}", vectorizer_error);
    assert!(error_msg.contains("Serialization error"));
}

#[test]
fn test_http_status_code_mapping() {
    // Test 401 Unauthorized
    let error_401 = error::map_http_error(401, Some("Unauthorized".to_string()));
    assert!(
        matches!(error_401, VectorizerError::Authentication { message } 
        if message == "Unauthorized")
    );

    // Test 403 Forbidden
    let error_403 = error::map_http_error(403, None);
    assert!(
        matches!(error_403, VectorizerError::Authentication { message } 
        if message == "Access forbidden")
    );

    // Test 404 Not Found
    let error_404 = error::map_http_error(404, Some("Not found".to_string()));
    assert!(matches!(error_404, VectorizerError::Server { message } 
        if message == "Resource not found"));

    // Test 429 Too Many Requests
    let error_429 = error::map_http_error(429, Some("Rate limit exceeded".to_string()));
    assert!(matches!(error_429, VectorizerError::RateLimit { message } 
        if message == "Rate limit exceeded"));

    // Test 500 Internal Server Error
    let error_500 = error::map_http_error(500, Some("Internal server error".to_string()));
    assert!(matches!(error_500, VectorizerError::Server { message } 
        if message == "Internal server error"));

    // Test 502 Bad Gateway
    let error_502 = error::map_http_error(502, None);
    assert!(matches!(error_502, VectorizerError::Server { message } 
        if message == "HTTP 502"));

    // Test unknown status code
    let error_unknown = error::map_http_error(999, Some("Unknown error".to_string()));
    assert!(matches!(error_unknown, VectorizerError::Server { message } 
        if message == "Unknown error"));
}

#[test]
fn test_error_display_formatting() {
    // Test various error display formats
    let auth_error = VectorizerError::authentication("Invalid credentials");
    assert_eq!(
        format!("{}", auth_error),
        "Authentication failed: Invalid credentials"
    );

    let collection_error = VectorizerError::collection_not_found("my_collection");
    assert_eq!(
        format!("{}", collection_error),
        "Collection 'my_collection' not found"
    );

    let vector_error = VectorizerError::vector_not_found("my_collection", "vector_123");
    assert_eq!(
        format!("{}", vector_error),
        "Vector 'vector_123' not found in collection 'my_collection'"
    );

    let validation_error = VectorizerError::validation("Invalid input");
    assert_eq!(
        format!("{}", validation_error),
        "Validation error: Invalid input"
    );

    let network_error = VectorizerError::network("Connection failed");
    assert_eq!(
        format!("{}", network_error),
        "Network error: Connection failed"
    );

    let server_error = VectorizerError::server("Server unavailable");
    assert_eq!(
        format!("{}", server_error),
        "Server error: Server unavailable"
    );

    let timeout_error = VectorizerError::timeout(60);
    assert_eq!(format!("{}", timeout_error), "Request timeout after 60s");

    let rate_limit_error = VectorizerError::rate_limit("Too many requests");
    assert_eq!(
        format!("{}", rate_limit_error),
        "Rate limit exceeded: Too many requests"
    );

    let config_error = VectorizerError::configuration("Invalid config");
    assert_eq!(
        format!("{}", config_error),
        "Configuration error: Invalid config"
    );

    let embedding_error = VectorizerError::embedding("Embedding failed");
    assert_eq!(
        format!("{}", embedding_error),
        "Embedding generation failed: Embedding failed"
    );

    let search_error = VectorizerError::search("Search failed");
    assert_eq!(format!("{}", search_error), "Search failed: Search failed");

    let storage_error = VectorizerError::storage("Storage failed");
    assert_eq!(
        format!("{}", storage_error),
        "Storage error: Storage failed"
    );

    let batch_error = VectorizerError::batch_operation("Batch failed");
    assert_eq!(
        format!("{}", batch_error),
        "Batch operation failed: Batch failed"
    );

    let mcp_error = VectorizerError::mcp("MCP failed");
    assert_eq!(format!("{}", mcp_error), "MCP error: MCP failed");

    let serialization_error = VectorizerError::Serialization("JSON failed".to_string());
    assert_eq!(
        format!("{}", serialization_error),
        "Serialization error: JSON failed"
    );
}

#[test]
fn test_error_debug_formatting() {
    let error = VectorizerError::authentication("Test error");
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Authentication"));
    assert!(debug_str.contains("Test error"));
}

#[test]
fn test_error_std_error_trait() {
    // Test that VectorizerError implements std::error::Error
    let error = VectorizerError::authentication("Test error");
    let error_ref: &dyn std::error::Error = &error;

    // This should compile and work
    let error_msg = error_ref.to_string();
    assert!(error_msg.contains("Authentication failed"));
}

#[test]
fn test_error_result_type_alias() {
    // Test that the Result type alias works correctly
    fn returns_result() -> Result<String> {
        Err(VectorizerError::validation("Test validation error"))
    }

    match returns_result() {
        Ok(_) => panic!("Should have returned an error"),
        Err(e) => {
            assert!(matches!(e, VectorizerError::Validation { message } 
                if message == "Test validation error"));
        }
    }
}

#[test]
fn test_error_chaining() {
    // Test error conversion chaining
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let vectorizer_error: VectorizerError = io_error.into();

    assert!(matches!(vectorizer_error, VectorizerError::Io(_)));

    // Test that we can convert back to string representation
    let error_msg = format!("{}", vectorizer_error);
    assert!(error_msg.contains("IO error"));
    assert!(error_msg.contains("File not found"));
}

#[test]
fn test_comprehensive_error_scenarios() {
    // Test multiple error types in a realistic scenario
    let errors = vec![
        VectorizerError::authentication("Invalid API key"),
        VectorizerError::collection_not_found("missing_collection"),
        VectorizerError::validation("Invalid dimension: must be > 0"),
        VectorizerError::network("Connection timeout"),
        VectorizerError::server("Internal server error"),
        VectorizerError::timeout(30),
        VectorizerError::rate_limit("Rate limit exceeded"),
        VectorizerError::configuration("Missing required config"),
        VectorizerError::embedding("Model not available"),
        VectorizerError::search("Index not ready"),
        VectorizerError::storage("Disk full"),
        VectorizerError::batch_operation("Partial batch failure"),
        VectorizerError::mcp("MCP server unavailable"),
        VectorizerError::network("Connection closed"),
    ];

    for error in errors {
        // Each error should have a meaningful string representation
        let error_msg = format!("{}", error);
        assert!(!error_msg.is_empty());
        assert!(error_msg.len() > 10); // Should have substantial content

        // Each error should be debuggable
        let debug_msg = format!("{:?}", error);
        assert!(!debug_msg.is_empty());

        // Each error should implement std::error::Error
        let error_ref: &dyn std::error::Error = &error;
        assert!(!error_ref.to_string().is_empty());
    }
}
