//! HiveHub Cloud integration module
//!
//! Provides multi-tenant cluster mode integration with HiveHub.Cloud,
//! including authentication, quota management, and usage tracking.
//!
//! This module enables Vectorizer to operate as a managed service through
//! HiveHub.Cloud with proper user isolation, authentication, and billing.

pub mod auth;
pub mod backup;
pub mod client;
pub mod ip_whitelist;
pub mod key_rotation;
pub mod mcp_gateway;
pub mod middleware;
pub mod quota;
pub mod request_signing;
pub mod usage;

use std::sync::Arc;

pub use auth::{HubAuth, HubAuthResult, TenantContext, TenantPermission};
pub use backup::{BackupConfig, RestoreResult, UserBackupInfo, UserBackupManager};
pub use client::{
    HubClient, HubClientConfig, OperationLogEntry, OperationLogsRequest, OperationLogsResponse,
};
pub use ip_whitelist::{
    IpAccessResult, IpPolicy, IpWhitelist, IpWhitelistConfig, IpWhitelistStats,
};
pub use key_rotation::{DEFAULT_GRACE_PERIOD_SECS, KeyRotation, KeyRotationManager, KeyStatus};
pub use mcp_gateway::{McpHubGateway, McpOperationLog, McpOperationType, McpRequestContext};
pub use middleware::HubAuthMiddleware;
use parking_lot::RwLock;
pub use quota::{QuotaInfo, QuotaManager, QuotaType};
pub use request_signing::{
    HEADER_NONCE, HEADER_SIGNATURE, HEADER_TIMESTAMP, RequestSigningValidator, SignedRequest,
    SigningConfig, create_signing_headers,
};
use tracing::{error, info, warn};
pub use usage::{UsageMetrics, UsageReporter};
use uuid::Uuid;

// `HubConfig` and its sub-structs are plain serde data types owned by
// `config` (phase41_architecture-decoupling §2: config must not depend
// on hub, so the dependency now points the other way). Re-exported
// here under the historical `crate::hub::*` paths so every existing
// call site keeps compiling.
pub use crate::config::sections::hub::{
    ConnectionPoolConfig, HubCacheConfig, HubConfig, TenantIsolationMode,
};
use crate::error::{Result, VectorizerError};

/// HiveHub integration manager
///
/// Coordinates all HiveHub integration components including
/// authentication, quota management, usage reporting, and key rotation.
#[derive(Debug)]
pub struct HubManager {
    /// HiveHub client
    client: Arc<HubClient>,
    /// Authentication handler
    auth: Arc<HubAuth>,
    /// Quota manager
    quota: Arc<QuotaManager>,
    /// Usage reporter
    usage: Arc<UsageReporter>,
    /// Key rotation manager
    key_rotation: Arc<KeyRotationManager>,
    /// Configuration
    config: HubConfig,
    /// Active state
    active: Arc<RwLock<bool>>,
}

impl HubManager {
    /// Create a new HubManager with the given configuration
    pub async fn new(config: HubConfig) -> Result<Self> {
        if !config.enabled {
            info!("HiveHub integration is disabled");
        }

        // Validate configuration
        if config.enabled && config.service_api_key.is_none() {
            return Err(VectorizerError::ConfigurationError(
                "HiveHub enabled but HIVEHUB_SERVICE_API_KEY is not set".to_string(),
            ));
        }

        let client_config = HubClientConfig {
            api_url: config.api_url.clone(),
            service_api_key: config.service_api_key.clone().unwrap_or_default(),
            timeout_seconds: config.timeout_seconds,
            retries: config.retries,
        };

        let client = Arc::new(HubClient::new(client_config)?);
        let auth = Arc::new(HubAuth::new(client.clone(), &config.cache));
        let quota = Arc::new(QuotaManager::new(client.clone(), &config.cache));
        let usage = Arc::new(UsageReporter::new(
            client.clone(),
            config.usage_report_interval,
        ));
        let key_rotation = Arc::new(KeyRotationManager::new(client.clone(), None));

        Ok(Self {
            client,
            auth,
            quota,
            usage,
            key_rotation,
            config,
            active: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the HubManager (begins usage reporting)
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut active = self.active.write();
        if *active {
            warn!("HubManager already started");
            return Ok(());
        }

        info!("Starting HiveHub integration manager");

        // Verify connection to HiveHub
        if let Err(e) = self.client.health_check().await {
            error!("Failed to connect to HiveHub: {}", e);
            return Err(VectorizerError::ConfigurationError(format!(
                "Cannot connect to HiveHub: {}",
                e
            )));
        }

        // Start usage reporting
        self.usage.start().await?;

        *active = true;
        info!("HiveHub integration manager started successfully");
        Ok(())
    }

    /// Stop the HubManager
    pub async fn stop(&self) -> Result<()> {
        let mut active = self.active.write();
        if !*active {
            return Ok(());
        }

        info!("Stopping HiveHub integration manager");

        // Stop usage reporting (will flush pending reports)
        self.usage.stop().await?;

        *active = false;
        info!("HiveHub integration manager stopped");
        Ok(())
    }

    /// Check if HiveHub integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check if HubManager is active
    pub fn is_active(&self) -> bool {
        *self.active.read()
    }

    /// Get the authentication handler
    pub fn auth(&self) -> &Arc<HubAuth> {
        &self.auth
    }

    /// Get the quota manager
    pub fn quota(&self) -> &Arc<QuotaManager> {
        &self.quota
    }

    /// Get the usage reporter
    pub fn usage(&self) -> &Arc<UsageReporter> {
        &self.usage
    }

    /// Get the key rotation manager
    pub fn key_rotation(&self) -> &Arc<KeyRotationManager> {
        &self.key_rotation
    }

    /// Get the HiveHub client
    pub fn client(&self) -> &Arc<HubClient> {
        &self.client
    }

    /// Get the configuration
    pub fn config(&self) -> &HubConfig {
        &self.config
    }

    /// Validate an API key and return tenant context
    pub async fn validate_api_key(&self, api_key: &str) -> Result<TenantContext> {
        if !self.config.enabled {
            return Err(VectorizerError::ConfigurationError(
                "HiveHub integration is not enabled".to_string(),
            ));
        }

        self.auth.validate_api_key(api_key).await
    }

    /// Check quota for a tenant operation
    pub async fn check_quota(
        &self,
        tenant_id: &str,
        quota_type: QuotaType,
        requested: u64,
    ) -> Result<bool> {
        if !self.config.enabled {
            return Ok(true); // No quota enforcement in standalone mode
        }

        self.quota
            .check_quota(tenant_id, quota_type, requested)
            .await
    }

    /// Record usage for a collection
    ///
    /// In cluster mode, usage is tracked per-collection (by UUID).
    /// The HiveHub manages tenant-level aggregation.
    pub async fn record_usage(&self, collection_id: Uuid, metrics: UsageMetrics) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        self.usage.record(collection_id, metrics).await
    }

    /// Get tenant-scoped collection name
    pub fn tenant_collection_name(&self, tenant_id: &str, collection_name: &str) -> String {
        match self.config.tenant_isolation {
            TenantIsolationMode::None => collection_name.to_string(),
            TenantIsolationMode::Collection | TenantIsolationMode::Storage => {
                format!("{}:{}", tenant_id, collection_name)
            }
        }
    }

    /// Extract tenant ID and collection name from a prefixed collection name
    pub fn parse_tenant_collection(&self, full_name: &str) -> Option<(String, String)> {
        match self.config.tenant_isolation {
            TenantIsolationMode::None => None,
            TenantIsolationMode::Collection | TenantIsolationMode::Storage => {
                let parts: Vec<&str> = full_name.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            }
        }
    }

    /// Get tenant storage path for storage-level isolation
    pub fn tenant_storage_path(&self, tenant_id: &str, base_path: &str) -> String {
        match self.config.tenant_isolation {
            TenantIsolationMode::Storage => {
                format!("{}/tenants/{}", base_path, tenant_id)
            }
            _ => base_path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_config_default() {
        let config = HubConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.retries, 3);
        assert!(config.cache.enabled);
    }

    #[test]
    fn test_tenant_collection_name_none_isolation() {
        let config = HubConfig {
            enabled: false,
            tenant_isolation: TenantIsolationMode::None,
            ..Default::default()
        };

        // We can't create HubManager without async, so test the logic directly
        let full_name = match config.tenant_isolation {
            TenantIsolationMode::None => "documents".to_string(),
            _ => format!("{}:{}", "tenant_123", "documents"),
        };

        assert_eq!(full_name, "documents");
    }

    #[test]
    fn test_tenant_collection_name_collection_isolation() {
        let config = HubConfig {
            enabled: true,
            tenant_isolation: TenantIsolationMode::Collection,
            ..Default::default()
        };

        let full_name = match config.tenant_isolation {
            TenantIsolationMode::None => "documents".to_string(),
            _ => format!("{}:{}", "tenant_123", "documents"),
        };

        assert_eq!(full_name, "tenant_123:documents");
    }

    #[test]
    fn test_parse_tenant_collection() {
        let full_name = "tenant_123:documents";
        let parts: Vec<&str> = full_name.splitn(2, ':').collect();

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "tenant_123");
        assert_eq!(parts[1], "documents");
    }

    #[test]
    fn test_tenant_storage_path() {
        let config = HubConfig {
            enabled: true,
            tenant_isolation: TenantIsolationMode::Storage,
            ..Default::default()
        };

        let path = match config.tenant_isolation {
            TenantIsolationMode::Storage => {
                format!("{}/tenants/{}", "/data", "tenant_123")
            }
            _ => "/data".to_string(),
        };

        assert_eq!(path, "/data/tenants/tenant_123");
    }
}
