//! HiveHub integration configuration data.
//!
//! Plain serde types only — `HubManager` construction and business
//! logic live in `crate::hub`, which re-exports these types from here
//! (see `phase41_architecture-decoupling` §2).

use serde::{Deserialize, Serialize};

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
