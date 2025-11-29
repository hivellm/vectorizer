//! Transport abstraction layer for Vectorizer client.
//!
//! Supports multiple transport protocols:
//! - HTTP/HTTPS (default)
//! - UMICP (Universal Messaging and Inter-process Communication Protocol)

use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Transport protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// HTTP/HTTPS protocol
    Http,
    /// UMICP protocol
    #[cfg(feature = "umicp")]
    Umicp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Http => write!(f, "http"),
            #[cfg(feature = "umicp")]
            Protocol::Umicp => write!(f, "umicp"),
        }
    }
}

/// Transport trait for making requests
#[async_trait]
pub trait Transport: Send + Sync {
    /// Make a GET request
    async fn get(&self, path: &str) -> Result<String>;

    /// Make a POST request
    async fn post(&self, path: &str, data: Option<&Value>) -> Result<String>;

    /// Make a PUT request
    async fn put(&self, path: &str, data: Option<&Value>) -> Result<String>;

    /// Make a DELETE request
    async fn delete(&self, path: &str) -> Result<String>;

    /// Get the protocol being used
    fn protocol(&self) -> Protocol;
}

/// Parse a connection string into protocol and connection details
///
/// Examples:
/// - "http://localhost:15002" -> HTTP transport
/// - "https://api.example.com" -> HTTPS transport
/// - "umicp://localhost:15003" -> UMICP transport
pub fn parse_connection_string(connection_string: &str) -> Result<(Protocol, String, Option<u16>)> {
    // Simple manual parsing
    let parts: Vec<&str> = connection_string.split("://").collect();

    if parts.len() != 2 {
        return Err(crate::error::VectorizerError::configuration(
            "Invalid connection string format. Expected protocol://host[:port]",
        ));
    }

    let scheme = parts[0];
    let authority = parts[1];

    // Parse host and port
    #[allow(unused_variables)]
    let (host, port) = if authority.contains(':') {
        let host_port: Vec<&str> = authority.split(':').collect();
        if host_port.len() != 2 {
            return Err(crate::error::VectorizerError::configuration(
                "Invalid host:port format",
            ));
        }
        let port = host_port[1]
            .parse::<u16>()
            .map_err(|_| crate::error::VectorizerError::configuration("Invalid port number"))?;
        (host_port[0].to_string(), Some(port))
    } else {
        (authority.to_string(), None)
    };

    match scheme {
        "http" => Ok((Protocol::Http, format!("http://{}", authority), None)),
        "https" => Ok((Protocol::Http, format!("https://{}", authority), None)),
        #[cfg(feature = "umicp")]
        "umicp" => Ok((Protocol::Umicp, host, port)),
        _ => Err(crate::error::VectorizerError::configuration(format!(
            "Unsupported protocol: {}",
            scheme
        ))),
    }
}
