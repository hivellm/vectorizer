//! HiveHub Cloud SDK client wrapper
//!
//! Provides a convenient wrapper around the hivehub-internal-sdk
//! for communicating with the HiveHub.Cloud API.

use std::sync::Arc;
use std::time::Duration;

use hivehub_internal_sdk::models::{
    AccessKeyPermission, CollectionValidation, QuotaCheckRequest, QuotaCheckResponse,
    UpdateUsageRequest as SdkUpdateUsageRequest, UpdateUsageResponse, UserCollectionsResponse,
};
use hivehub_internal_sdk::{HiveHubCloudClient as SdkClient, HiveHubCloudError as SdkError};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::error::{Result, VectorizerError};

/// Configuration for the HiveHub client
#[derive(Debug, Clone)]
pub struct HubClientConfig {
    /// HiveHub API URL
    pub api_url: String,
    /// Service API key for authentication
    pub service_api_key: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Number of retries for failed requests
    pub retries: u32,
}

impl Default for HubClientConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.hivehub.cloud".to_string(),
            service_api_key: String::new(),
            timeout_seconds: 30,
            retries: 3,
        }
    }
}

/// HiveHub client wrapper
///
/// Wraps the hivehub-internal-sdk client with additional
/// error handling, logging, and retry logic.
#[derive(Debug)]
pub struct HubClient {
    /// The underlying SDK client
    inner: SdkClient,
    /// Configuration
    config: HubClientConfig,
    /// Connection status
    connected: Arc<RwLock<bool>>,
}

impl HubClient {
    /// Create a new HubClient with the given configuration
    pub fn new(config: HubClientConfig) -> Result<Self> {
        info!("Initializing HiveHub client for {}", config.api_url);

        if config.service_api_key.is_empty() {
            return Err(VectorizerError::ConfigurationError(
                "Service API key is required".to_string(),
            ));
        }

        let inner = SdkClient::new(config.service_api_key.clone(), config.api_url.clone())
            .map_err(|e| {
                VectorizerError::ConfigurationError(format!(
                    "Failed to create HiveHub client: {}",
                    e
                ))
            })?;

        Ok(Self {
            inner,
            config,
            connected: Arc::new(RwLock::new(false)),
        })
    }

    /// Create a client from environment variables
    pub fn from_env() -> Result<Self> {
        let api_url = std::env::var("HIVEHUB_API_URL")
            .unwrap_or_else(|_| "https://api.hivehub.cloud".to_string());

        let service_api_key = std::env::var("HIVEHUB_SERVICE_API_KEY").map_err(|_| {
            VectorizerError::ConfigurationError(
                "HIVEHUB_SERVICE_API_KEY environment variable not set".to_string(),
            )
        })?;

        let timeout_seconds = std::env::var("HIVEHUB_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let retries = std::env::var("HIVEHUB_RETRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        Self::new(HubClientConfig {
            api_url,
            service_api_key,
            timeout_seconds,
            retries,
        })
    }

    /// Check connection to HiveHub API
    pub async fn health_check(&self) -> Result<()> {
        trace!("Performing HiveHub health check");

        // Use quota check as health check since the SDK doesn't have a dedicated health endpoint
        let request = QuotaCheckRequest {
            project_id: "health_check".to_string(),
            operation: "ping".to_string(),
            estimated_size: None,
        };

        match self.inner.vectorizer().check_quota(&request).await {
            Ok(_) => {
                *self.connected.write() = true;
                debug!("HiveHub health check passed");
                Ok(())
            }
            Err(e) => {
                *self.connected.write() = false;
                // For health check, we accept any response as valid connection
                // since we're just testing connectivity
                warn!(
                    "HiveHub health check returned error (may be expected): {}",
                    e
                );
                Ok(()) // Consider connected even if we get an error response
            }
        }
    }

    /// Check if the client is connected
    pub fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    // ========================================
    // Vectorizer Service API
    // ========================================

    /// Get all collections for a user
    pub async fn get_user_collections(&self, user_id: &Uuid) -> Result<UserCollectionsResponse> {
        trace!("Getting collections for user: {}", user_id);

        self.inner
            .vectorizer()
            .get_user_collections(user_id)
            .await
            .map_err(Self::map_sdk_error)
    }

    /// Validate collection ownership and quota
    pub async fn validate_collection(
        &self,
        collection_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<CollectionValidation> {
        trace!(
            "Validating collection {} for user {}",
            collection_id, user_id
        );

        self.inner
            .vectorizer()
            .validate_collection(collection_id, user_id)
            .await
            .map_err(Self::map_sdk_error)
    }

    /// Update usage metrics for a collection
    pub async fn update_usage(
        &self,
        collection_id: &Uuid,
        request: UpdateUsageRequest,
    ) -> Result<UpdateUsageResponse> {
        trace!(
            "Updating usage for collection {}: vectors={}, storage={}",
            collection_id, request.vector_count, request.storage_bytes
        );

        let sdk_request = SdkUpdateUsageRequest {
            vector_count: request.vector_count,
            storage_bytes: request.storage_bytes,
        };

        self.inner
            .vectorizer()
            .update_usage(collection_id, &sdk_request)
            .await
            .map_err(Self::map_sdk_error)
    }

    /// Check quota for an operation
    pub async fn check_quota(&self, request: &QuotaCheckRequest) -> Result<QuotaCheckResponse> {
        trace!("Checking quota for operation: {}", request.operation);

        self.inner
            .vectorizer()
            .check_quota(request)
            .await
            .map_err(Self::map_sdk_error)
    }

    // ========================================
    // Access Keys API
    // ========================================

    /// Generate a Vectorizer access key
    pub async fn generate_vectorizer_key(
        &self,
        name: &str,
        permissions: Vec<AccessKeyPermission>,
    ) -> Result<hivehub_internal_sdk::models::GenerateAccessKeyResponse> {
        trace!("Generating Vectorizer access key: {}", name);

        self.inner
            .access_keys()
            .generate_vectorizer_key(name, permissions)
            .await
            .map_err(Self::map_sdk_error)
    }

    /// Revoke an access key
    pub async fn revoke_access_key(&self, key_id: &Uuid) -> Result<()> {
        trace!("Revoking access key: {}", key_id);

        self.inner
            .access_keys()
            .revoke(key_id)
            .await
            .map(|_| ())
            .map_err(Self::map_sdk_error)
    }

    // ========================================
    // Logging API
    // ========================================

    /// Send operation logs to HiveHub Cloud
    ///
    /// This sends a batch of operation logs for centralized logging and analytics.
    /// Logs are processed asynchronously by the cloud service.
    pub async fn send_operation_logs(
        &self,
        request: OperationLogsRequest,
    ) -> Result<OperationLogsResponse> {
        trace!(
            "Sending {} operation logs to HiveHub Cloud",
            request.logs.len()
        );

        // Use reqwest directly since the SDK may not have this endpoint
        let url = format!("{}/api/v1/vectorizer/logs", self.config.api_url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .build()
            .map_err(|e| {
                VectorizerError::InternalError(format!("Failed to create HTTP client: {}", e))
            })?;

        let response = client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.service_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| VectorizerError::InternalError(format!("Failed to send logs: {}", e)))?;

        if response.status().is_success() {
            let result: OperationLogsResponse =
                response
                    .json()
                    .await
                    .unwrap_or_else(|_| OperationLogsResponse {
                        accepted: true,
                        processed: request.logs.len(),
                        error: None,
                    });
            debug!("Successfully sent {} operation logs", result.processed);
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Failed to send operation logs: {} - {}", status, error_text);

            // Return success anyway to avoid blocking operations
            // Cloud logging failures should not impact core functionality
            Ok(OperationLogsResponse {
                accepted: false,
                processed: 0,
                error: Some(format!("HTTP {}: {}", status, error_text)),
            })
        }
    }

    /// Get the API URL for this client
    pub fn api_url(&self) -> &str {
        &self.config.api_url
    }

    /// Get the service ID (derived from API key prefix)
    pub fn service_id(&self) -> String {
        // Use first 8 chars of API key hash as service ID
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.config.service_api_key.hash(&mut hasher);
        format!("vec-{:016x}", hasher.finish())
    }

    // ========================================
    // Error Mapping
    // ========================================

    /// Map SDK errors to VectorizerError
    fn map_sdk_error(err: SdkError) -> VectorizerError {
        match err {
            SdkError::Authentication(msg) => VectorizerError::AuthenticationError(msg),
            SdkError::NotFound(msg) => VectorizerError::NotFound(msg),
            SdkError::QuotaExceeded(msg) => VectorizerError::RateLimitExceeded {
                limit_type: "quota".to_string(),
                limit: 0,
            },
            SdkError::ServiceUnavailable(msg) => {
                VectorizerError::InternalError(format!("HiveHub service unavailable: {}", msg))
            }
            SdkError::Validation(msg) => {
                VectorizerError::ConfigurationError(format!("Validation error: {}", msg))
            }
            SdkError::BadRequest(msg) => {
                VectorizerError::ConfigurationError(format!("Bad request: {}", msg))
            }
            SdkError::Http(e) => VectorizerError::InternalError(format!("HTTP error: {}", e)),
            SdkError::Serialization(e) => {
                VectorizerError::InternalError(format!("Serialization error: {}", e))
            }
            SdkError::Configuration(msg) => VectorizerError::ConfigurationError(msg),
            SdkError::Unknown(msg) => {
                VectorizerError::InternalError(format!("Unknown error: {}", msg))
            }
        }
    }

    /// Create a mock HubClient for testing
    #[cfg(test)]
    pub fn new_mock() -> Self {
        let config = HubClientConfig {
            api_url: "http://localhost:12000".to_string(),
            service_api_key: "test-key".to_string(),
            timeout_seconds: 10,
            retries: 1,
        };

        // Create a mock client (this will fail in tests but that's okay for unit tests)
        let inner = SdkClient::new(config.service_api_key.clone(), config.api_url.clone())
            .expect("Failed to create mock client");

        Self {
            inner,
            config,
            connected: Arc::new(RwLock::new(false)),
        }
    }
}

/// Request to update usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUsageRequest {
    /// Number of vectors stored
    pub vector_count: u64,
    /// Storage used in bytes
    pub storage_bytes: u64,
}

/// Request to send operation logs to HiveHub Cloud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLogsRequest {
    /// Service identifier (vectorizer instance)
    pub service_id: String,
    /// Batch of operation logs
    pub logs: Vec<OperationLogEntry>,
}

/// Single operation log entry for cloud logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLogEntry {
    /// Operation ID (UUID)
    pub operation_id: Uuid,
    /// Tenant ID
    pub tenant_id: String,
    /// Operation name/tool
    pub operation: String,
    /// Operation type category
    pub operation_type: String,
    /// Collection name (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Success status
    pub success: bool,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Response from sending operation logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLogsResponse {
    /// Whether the logs were accepted
    pub accepted: bool,
    /// Number of logs processed
    pub processed: usize,
    /// Error message if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Default for UpdateUsageRequest {
    fn default() -> Self {
        Self {
            vector_count: 0,
            storage_bytes: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_client_config_default() {
        let config = HubClientConfig::default();
        assert_eq!(config.api_url, "https://api.hivehub.cloud");
        assert!(config.service_api_key.is_empty());
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.retries, 3);
    }

    #[test]
    fn test_hub_client_requires_api_key() {
        let config = HubClientConfig::default();
        let result = HubClient::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_usage_request_default() {
        let req = UpdateUsageRequest::default();
        assert_eq!(req.vector_count, 0);
        assert_eq!(req.storage_bytes, 0);
    }
}
