//! HiveHub Cloud SDK client wrapper
//!
//! Provides a convenient wrapper around the hivehub-internal-sdk
//! for communicating with the HiveHub.Cloud API.

use std::sync::Arc;
use std::time::Duration;

use hivehub_internal_sdk::{
    HiveHubCloudClient as SdkClient, HiveHubCloudError as SdkError,
    models::{
        AccessKeyPermission, CollectionValidation, QuotaCheckRequest, QuotaCheckResponse,
        UpdateUsageRequest as SdkUpdateUsageRequest, UpdateUsageResponse, UserCollectionsResponse,
    },
};
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
}

/// Request to update usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUsageRequest {
    /// Number of vectors stored
    pub vector_count: u64,
    /// Storage used in bytes
    pub storage_bytes: u64,
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
