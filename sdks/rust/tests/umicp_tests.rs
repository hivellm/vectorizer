//! Tests for UMICP transport

use vectorizer_sdk::{VectorizerClient, ClientConfig, Protocol, parse_connection_string};

#[cfg(feature = "umicp")]
use vectorizer_sdk::UmicpConfig;

#[cfg(feature = "umicp")]
#[tokio::test]
async fn test_umicp_client_creation() {
    let client = VectorizerClient::new(ClientConfig {
        protocol: Some(Protocol::Umicp),
        api_key: Some("test-key".to_string()),
        umicp: Some(UmicpConfig {
            host: "localhost".to_string(),
            port: 15003,
        }),
        ..Default::default()
    });

    assert!(client.is_ok());
    let client = client.unwrap();
    assert_eq!(client.protocol(), Protocol::Umicp);
}

#[cfg(feature = "umicp")]
#[tokio::test]
async fn test_umicp_from_connection_string() {
    let client = VectorizerClient::from_connection_string("umicp://localhost:15003", Some("test-key"));
    
    assert!(client.is_ok());
    let client = client.unwrap();
    assert_eq!(client.protocol(), Protocol::Umicp);
}

#[cfg(feature = "umicp")]
#[test]
fn test_parse_umicp_connection_string() {
    let result = parse_connection_string("umicp://localhost:15003");
    assert!(result.is_ok());
    
    let (protocol, host, port) = result.unwrap();
    assert_eq!(protocol, Protocol::Umicp);
    assert_eq!(host, "localhost");
    assert_eq!(port, Some(15003));
}

#[test]
fn test_parse_http_connection_string() {
    let result = parse_connection_string("http://localhost:15002");
    assert!(result.is_ok());
    
    let (protocol, url, _port) = result.unwrap();
    assert_eq!(protocol, Protocol::Http);
    assert_eq!(url, "http://localhost:15002");
}

#[test]
fn test_parse_https_connection_string() {
    let result = parse_connection_string("https://api.example.com");
    assert!(result.is_ok());
    
    let (protocol, url, _port) = result.unwrap();
    assert_eq!(protocol, Protocol::Http);
    assert_eq!(url, "https://api.example.com");
}

#[test]
fn test_parse_invalid_protocol() {
    let result = parse_connection_string("ftp://localhost");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_http_client_backward_compatibility() {
    let client = VectorizerClient::new_default();
    assert!(client.is_ok());
    
    let client = client.unwrap();
    assert_eq!(client.protocol(), Protocol::Http);
}

#[tokio::test]
async fn test_http_client_with_url() {
    let client = VectorizerClient::new_with_url("http://localhost:8080");
    assert!(client.is_ok());
    
    let client = client.unwrap();
    assert_eq!(client.protocol(), Protocol::Http);
}

#[tokio::test]
async fn test_http_client_with_api_key() {
    let client = VectorizerClient::new_with_api_key("http://localhost:15002", "test-key");
    assert!(client.is_ok());
    
    let client = client.unwrap();
    assert_eq!(client.protocol(), Protocol::Http);
}

#[cfg(feature = "umicp")]
#[tokio::test]
async fn test_umicp_requires_config() {
    let result = VectorizerClient::new(ClientConfig {
        protocol: Some(Protocol::Umicp),
        api_key: Some("test-key".to_string()),
        ..Default::default()
    });

    assert!(result.is_err());
}

