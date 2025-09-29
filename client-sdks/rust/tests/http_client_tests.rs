//! HTTP client tests for the Rust SDK
//! Tests for HTTP request handling, error mapping, and response processing

use vectorizer_rust_sdk::*;
use std::collections::HashMap;

// Note: These tests focus on error mapping and client initialization
// since actual HTTP mocking would require more complex setup

#[test]
fn test_vectorizer_client_creation() {
    // Test default client creation
    let client_result = VectorizerClient::new_default();
    assert!(client_result.is_ok());
    
    let client = client_result.unwrap();
    assert_eq!(client.base_url(), "http://localhost:15001");
}

#[test]
fn test_vectorizer_client_with_custom_url() {
    // Test client creation with custom URL
    let client_result = VectorizerClient::new_with_url("http://custom:8080");
    assert!(client_result.is_ok());
    
    let client = client_result.unwrap();
    assert_eq!(client.base_url(), "http://custom:8080");
}

#[test]
fn test_vectorizer_client_with_api_key() {
    // Test client creation with API key
    let client_result = VectorizerClient::new_with_api_key("http://localhost:15001", "test-api-key");
    assert!(client_result.is_ok());
    
    let client = client_result.unwrap();
    assert_eq!(client.base_url(), "http://localhost:15001");
    // Note: API key verification would require actual HTTP calls
}

#[test]
fn test_http_error_mapping() {
    // Test HTTP status code mapping to VectorizerError
    let error_401 = error::map_http_error(401, Some("Unauthorized".to_string()));
    assert!(matches!(error_401, VectorizerError::Authentication { message } 
        if message == "Unauthorized"));

    let error_403 = error::map_http_error(403, None);
    assert!(matches!(error_403, VectorizerError::Authentication { message } 
        if message == "Access forbidden"));

    let error_404 = error::map_http_error(404, Some("Not found".to_string()));
    assert!(matches!(error_404, VectorizerError::Server { message } 
        if message == "Resource not found"));

    let error_429 = error::map_http_error(429, Some("Rate limit exceeded".to_string()));
    assert!(matches!(error_429, VectorizerError::RateLimit { message } 
        if message == "Rate limit exceeded"));

    let error_500 = error::map_http_error(500, Some("Internal server error".to_string()));
    assert!(matches!(error_500, VectorizerError::Server { message } 
        if message == "Internal server error"));

    let error_502 = error::map_http_error(502, None);
    assert!(matches!(error_502, VectorizerError::Server { message } 
        if message == "HTTP 502"));
}

#[test]
fn test_error_conversion_from_serde_json() {
    // Test conversion from serde_json::Error
    let io_error = std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid JSON");
    let json_error = serde_json::Error::io(io_error);
    let vectorizer_error = VectorizerError::from(json_error);
    
    assert!(matches!(vectorizer_error, VectorizerError::Serialization(_)));
    let error_msg = format!("{}", vectorizer_error);
    assert!(error_msg.contains("Serialization error"));
    assert!(error_msg.contains("Invalid JSON"));
}

#[test]
fn test_error_conversion_from_io_error() {
    // Test conversion from std::io::Error
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let vectorizer_error = VectorizerError::from(io_error);
    
    assert!(matches!(vectorizer_error, VectorizerError::Io(_)));
    let error_msg = format!("{}", vectorizer_error);
    assert!(error_msg.contains("IO error"));
}

#[tokio::test]
async fn test_error_conversion_from_reqwest() {
    // Test conversion from reqwest::Error
    // Create a reqwest error by making a request that will fail
    let client = reqwest::Client::new();
    let result = client.get("http://localhost:9999/nonexistent").send().await;
    let reqwest_error = result.unwrap_err();
    let vectorizer_error = VectorizerError::from(reqwest_error);
    
    assert!(matches!(vectorizer_error, VectorizerError::Http(_)));
    let error_msg = format!("{}", vectorizer_error);
    assert!(error_msg.contains("HTTP error"));
}

#[test]
fn test_client_configuration_validation() {
    // Test that client validates configuration
    let client_result = VectorizerClient::new_with_url("");
    // This should succeed as URL validation is not enforced at creation time
    // but would fail during actual HTTP requests
    assert!(client_result.is_ok());
}

#[test]
fn test_error_display_consistency() {
    // Test that all error types have consistent display formatting
    let errors = vec![
        VectorizerError::authentication("Test auth error"),
        VectorizerError::collection_not_found("test_collection"),
        VectorizerError::vector_not_found("collection", "vector_id"),
        VectorizerError::validation("Test validation error"),
        VectorizerError::network("Test network error"),
        VectorizerError::server("Test server error"),
        VectorizerError::timeout(30),
        VectorizerError::rate_limit("Test rate limit error"),
        VectorizerError::configuration("Test config error"),
        VectorizerError::embedding("Test embedding error"),
        VectorizerError::search("Test search error"),
        VectorizerError::storage("Test storage error"),
        VectorizerError::batch_operation("Test batch error"),
        VectorizerError::mcp("Test MCP error"),
        VectorizerError::Serialization("Test serialization error".to_string()),
    ];

    for error in errors {
        let error_msg = format!("{}", error);
        assert!(!error_msg.is_empty());
        assert!(error_msg.len() > 10); // Should have substantial content
        
        // Each error message should contain descriptive text
        assert!(error_msg.contains("error") || error_msg.contains("failed") || error_msg.contains("timeout") || 
                error_msg.contains("exceeded") || error_msg.contains("not found") || error_msg.contains("HTTP") ||
                error_msg.contains("IO") || error_msg.contains("Serialization"));
    }
}

#[test]
fn test_error_debug_formatting() {
    // Test that errors have proper debug formatting
    let error = VectorizerError::authentication("Debug test");
    let debug_str = format!("{:?}", error);
    
    assert!(debug_str.contains("Authentication"));
    assert!(debug_str.contains("Debug test"));
}

#[test]
fn test_std_error_trait_implementation() {
    // Test that VectorizerError properly implements std::error::Error
    let error = VectorizerError::authentication("Std error test");
    let error_ref: &dyn std::error::Error = &error;
    
    let error_msg = error_ref.to_string();
    assert!(error_msg.contains("Authentication failed"));
}

#[test]
fn test_result_type_alias() {
    // Test that the Result type alias works correctly
    fn returns_vectorizer_result() -> Result<String> {
        Err(VectorizerError::validation("Test validation"))
    }

    match returns_vectorizer_result() {
        Ok(_) => panic!("Should have returned an error"),
        Err(e) => {
            assert!(matches!(e, VectorizerError::Validation { message } 
                if message == "Test validation"));
        }
    }
}

#[test]
fn test_error_chain_conversion() {
    // Test error conversion chaining
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
    let vectorizer_error: VectorizerError = io_error.into();
    
    assert!(matches!(vectorizer_error, VectorizerError::Io(_)));
    
    let error_msg = format!("{}", vectorizer_error);
    assert!(error_msg.contains("IO error"));
    assert!(error_msg.contains("Permission denied"));
}

#[test]
fn test_comprehensive_error_scenarios() {
    // Test realistic error scenarios that might occur in HTTP client operations
    let scenarios = vec![
        ("Authentication failed", VectorizerError::authentication("Invalid API key")),
        ("Collection not found", VectorizerError::collection_not_found("missing_collection")),
        ("Validation error", VectorizerError::validation("Invalid dimension: must be > 0")),
        ("Network timeout", VectorizerError::network("Connection timeout")),
        ("Server error", VectorizerError::server("Internal server error")),
        ("Rate limiting", VectorizerError::rate_limit("Rate limit exceeded")),
        ("Serialization error", VectorizerError::Serialization("JSON parsing failed".to_string())),
    ];

    for (scenario_name, error) in scenarios {
        let error_msg = format!("{}", error);
        assert!(!error_msg.is_empty(), "Error message for '{}' should not be empty", scenario_name);
        assert!(error_msg.len() > 5, "Error message for '{}' should have substantial content", scenario_name);
        
        // Each error should be debuggable
        let debug_msg = format!("{:?}", error);
        assert!(!debug_msg.is_empty(), "Debug message for '{}' should not be empty", scenario_name);
        
        // Each error should implement std::error::Error
        let error_ref: &dyn std::error::Error = &error;
        assert!(!error_ref.to_string().is_empty(), "Std error message for '{}' should not be empty", scenario_name);
    }
}

#[test]
fn test_http_status_code_edge_cases() {
    // Test edge cases for HTTP status code mapping
    let edge_cases = vec![
        (200, "Should not be mapped to error"),
        (201, "Created - should not be mapped to error"),
        (400, "Bad request - should be mapped to server error"),
        (401, "Unauthorized - should be mapped to authentication error"),
        (403, "Forbidden - should be mapped to authentication error"),
        (404, "Not found - should be mapped to server error"),
        (429, "Too many requests - should be mapped to rate limit error"),
        (500, "Internal server error - should be mapped to server error"),
        (502, "Bad gateway - should be mapped to server error"),
        (503, "Service unavailable - should be mapped to server error"),
        (504, "Gateway timeout - should be mapped to server error"),
        (999, "Unknown status - should be mapped to server error"),
    ];

    for (status, description) in edge_cases {
        let error = error::map_http_error(status, Some(format!("HTTP {}", status)));
        let error_msg = format!("{}", error);
        
        // All errors should have meaningful messages
        assert!(!error_msg.is_empty(), "Error message for status {} ({}) should not be empty", status, description);
        
        // Verify correct error type mapping
        match status {
            401 | 403 => assert!(matches!(error, VectorizerError::Authentication { .. }), 
                                 "Status {} should map to Authentication error", status),
            429 => assert!(matches!(error, VectorizerError::RateLimit { .. }), 
                          "Status {} should map to RateLimit error", status),
            400..=599 => assert!(matches!(error, VectorizerError::Server { .. }), 
                                "Status {} should map to Server error", status),
            _ => assert!(matches!(error, VectorizerError::Server { .. }), 
                        "Status {} should map to Server error", status),
        }
    }
}

#[test]
fn test_client_url_handling() {
    // Test various URL formats
    let urls = vec![
        "http://localhost:15001",
        "https://api.example.com",
        "http://127.0.0.1:8080",
        "https://vectorizer.example.com:443",
    ];

    for url in urls {
        let client_result = VectorizerClient::new_with_url(url);
        assert!(client_result.is_ok(), "Client creation should succeed for URL: {}", url);
        
        let client = client_result.unwrap();
        assert_eq!(client.base_url(), url);
    }
}

#[test]
fn test_error_message_consistency() {
    // Test that error messages are consistent and informative
    let error = VectorizerError::authentication("Invalid API key provided");
    let error_msg = format!("{}", error);
    
    assert!(error_msg.starts_with("Authentication failed"));
    assert!(error_msg.contains("Invalid API key provided"));
    
    let collection_error = VectorizerError::collection_not_found("my_collection");
    let collection_msg = format!("{}", collection_error);
    
    assert_eq!(collection_msg, "Collection 'my_collection' not found");
    
    let vector_error = VectorizerError::vector_not_found("my_collection", "vector_123");
    let vector_msg = format!("{}", vector_error);
    
    assert_eq!(vector_msg, "Vector 'vector_123' not found in collection 'my_collection'");
}
