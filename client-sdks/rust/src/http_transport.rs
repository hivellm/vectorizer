//! HTTP transport implementation using reqwest

use crate::error::{VectorizerError, Result};
use crate::transport::{Protocol, Transport};
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde_json::Value;

/// HTTP transport client
pub struct HttpTransport {
    client: Client,
    base_url: String,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(base_url: &str, api_key: Option<&str>, timeout_secs: u64) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(key) = api_key {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", key))
                    .map_err(|e| VectorizerError::configuration(format!("Invalid API key: {}", e)))?,
            );
        }

        let client = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .default_headers(headers)
            .build()
            .map_err(|e| VectorizerError::configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    /// Make a generic request
    async fn request(&self, method: &str, path: &str, body: Option<&Value>) -> Result<String> {
        let url = format!("{}{}", self.base_url, path);

        let mut request = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(VectorizerError::configuration(format!("Unsupported HTTP method: {}", method))),
        };

        if let Some(data) = body {
            request = request.json(data);
        }

        let response = request.send().await
            .map_err(|e| VectorizerError::network(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(VectorizerError::server(format!("HTTP {}: {}", status, error_text)));
        }

        response.text().await
            .map_err(|e| VectorizerError::network(format!("Failed to read response: {}", e)))
    }
}

#[async_trait]
impl Transport for HttpTransport {
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
        Protocol::Http
    }
}

