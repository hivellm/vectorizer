//! UMICP transport implementation using umicp-core

#[cfg(feature = "umicp")]
use async_trait::async_trait;
#[cfg(feature = "umicp")]
use serde_json::Value;

#[cfg(feature = "umicp")]
use crate::error::{Result, VectorizerError};
#[cfg(feature = "umicp")]
use crate::transport::{Protocol, Transport};

#[cfg(feature = "umicp")]
/// UMICP transport client
pub struct UmicpTransport {
    host: String,
    port: u16,
    api_key: Option<String>,
    timeout_secs: u64,
}

#[cfg(feature = "umicp")]
impl UmicpTransport {
    /// Create a new UMICP transport
    pub fn new(host: &str, port: u16, api_key: Option<&str>, timeout_secs: u64) -> Result<Self> {
        Ok(Self {
            host: host.to_string(),
            port,
            api_key: api_key.map(|s| s.to_string()),
            timeout_secs,
        })
    }

    /// Make a generic request via UMICP
    async fn request(&self, method: &str, path: &str, body: Option<&Value>) -> Result<String> {
        // Note: This is a simplified implementation.
        // For a full UMICP implementation, you would use the umicp-core crate
        // to create proper UMICP envelopes and establish connections.

        // Since umicp-core doesn't have high-level HTTP client, we use a hybrid approach:
        // Use HTTP with UMICP protocol headers

        use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
        use reqwest::{Client, ClientBuilder};

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("X-UMICP-Protocol", HeaderValue::from_static("true"));

        if let Some(key) = &self.api_key {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {key}"))
                    .map_err(|e| VectorizerError::configuration(format!("Invalid API key: {e}")))?,
            );
        }

        let client = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .default_headers(headers)
            .build()
            .map_err(|e| VectorizerError::configuration(format!("Failed to create client: {e}")))?;

        let url = format!("http://{}:{}{}", self.host, self.port, path); // Multiple variables, keep as is

        let mut request = match method {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => {
                return Err(VectorizerError::configuration(format!(
                    "Unsupported method: {method}"
                )));
            }
        };

        if let Some(data) = body {
            request = request.json(data);
        }

        let response = request
            .send()
            .await
            .map_err(|e| VectorizerError::network(format!("UMICP request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VectorizerError::server(format!(
                "UMICP HTTP {status}: {error_text}"
            )));
        }

        response
            .text()
            .await
            .map_err(|e| VectorizerError::network(format!("Failed to read response: {e}")))
    }
}

#[cfg(feature = "umicp")]
#[async_trait]
impl Transport for UmicpTransport {
    async fn get(&self, path: &str) -> Result<String> {
        self.request("GET", path, None).await
    }

    async fn post(&self, path: &str, data: Option<&Value>) -> Result<String> {
        self.request("POST", path, data).await
    }

    async fn put(&self, path: &str, data: Option<&Value>) -> Result<String> {
        self.request("PUT", path, data).await
    }

    async fn delete(&self, path: &str) -> Result<String> {
        self.request("DELETE", path, None).await
    }

    fn protocol(&self) -> Protocol {
        Protocol::Umicp
    }
}
