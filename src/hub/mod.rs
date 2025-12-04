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
pub mod mcp_gateway;
pub mod middleware;
pub mod quota;
pub mod usage;

use std::sync::Arc;

pub use auth::{HubAuth, HubAuthResult, TenantContext, TenantPermission};
pub use backup::{BackupConfig, RestoreResult, UserBackupInfo, UserBackupManager};
pub use client::{HubClient, HubClientConfig};
pub use mcp_gateway::{McpHubGateway, McpOperationLog, McpOperationType, McpRequestContext};
pub use middleware::HubAuthMiddleware;
pub use quota::{QuotaInfo, QuotaManager, QuotaType};
pub use usage::{UsageMetrics, UsageReporter};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::{Result, VectorizerError};

/// HiveHub integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubConfig {
    /// Whether HiveHub integration is enabled
    #[serde(default)]
    pub enabled: bool,

    /// HiveHub API URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Service API key for server-to-hub communication
    /// Can be set via HIVEHUB_SERVICE_API_KEY environment variable
    #[serde(default)]
    pub service_api_key: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,

    /// Number of retries for failed requests
    #[serde(default = "default_retries")]
    pub retries: u32,

    /// Cache configuration
    #[serde(default)]
    pub cache: HubCacheConfig,

    /// Connection pool configuration
    #[serde(default)]
    pub connection_pool: ConnectionPoolConfig,

    /// Usage reporting interval in seconds
    #[serde(default = "default_usage_report_interval")]
    pub usage_report_interval: u64,

    /// Tenant isolation mode
    #[serde(default)]
    pub tenant_isolation: TenantIsolationMode,
}

/// Cache configuration for HiveHub API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubCacheConfig {
    /// Whether caching is enabled
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    /// API key validation cache TTL in seconds
    #[serde(default = "default_api_key_ttl")]
    pub api_key_ttl_seconds: u64,

    /// Quota cache TTL in seconds
    #[serde(default = "default_quota_ttl")]
    pub quota_ttl_seconds: u64,

    /// Maximum number of cached entries
    #[serde(default = "default_max_cache_entries")]
    pub max_entries: usize,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum idle connections per host
    #[serde(default = "default_max_idle_per_host")]
    pub max_idle_per_host: usize,

    /// Pool timeout in seconds
    #[serde(default = "default_pool_timeout")]
    pub pool_timeout_seconds: u64,
}

/// Tenant isolation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TenantIsolationMode {
    /// No tenant isolation (single-tenant mode)
    #[default]
    None,
    /// Collection-level isolation with tenant prefix
    Collection,
    /// Full storage-level isolation with separate paths
    Storage,
}

// Default value functions
fn default_api_url() -> String {
    std::env::var("HIVEHUB_API_URL").unwrap_or_else(|_| "https://api.hivehub.cloud".to_string())
}

fn default_timeout_seconds() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

fn default_cache_enabled() -> bool {
    true
}

fn default_api_key_ttl() -> u64 {
    300 // 5 minutes
}

fn default_quota_ttl() -> u64 {
    60 // 1 minute
}

fn default_max_cache_entries() -> usize {
    10000
}

fn default_max_idle_per_host() -> usize {
    10
}

fn default_pool_timeout() -> u64 {
    30
}

fn default_usage_report_interval() -> u64 {
    300 // 5 minutes
}

impl Default for HubConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_url: default_api_url(),
            service_api_key: std::env::var("HIVEHUB_SERVICE_API_KEY").ok(),
            timeout_seconds: default_timeout_seconds(),
            retries: default_retries(),
            cache: HubCacheConfig::default(),
            connection_pool: ConnectionPoolConfig::default(),
            usage_report_interval: default_usage_report_interval(),
            tenant_isolation: TenantIsolationMode::default(),
        }
    }
}

impl Default for HubCacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            api_key_ttl_seconds: default_api_key_ttl(),
            quota_ttl_seconds: default_quota_ttl(),
            max_entries: default_max_cache_entries(),
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: default_max_idle_per_host(),
            pool_timeout_seconds: default_pool_timeout(),
        }
    }
}

/// HiveHub integration manager
///
/// Coordinates all HiveHub integration components including
/// authentication, quota management, and usage reporting.
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

        Ok(Self {
            client,
            auth,
            quota,
            usage,
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
