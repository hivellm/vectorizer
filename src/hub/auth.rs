//! HiveHub authentication module
//!
//! Provides API key validation and tenant context management
//! for multi-tenant operations.
//!
//! Note: This module provides local API key validation with caching.
//! The actual key validation is done through the collection validation
//! endpoint when performing operations on collections.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use hivehub_internal_sdk::models::AccessKeyPermission;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use super::HubCacheConfig;
use super::client::HubClient;
use crate::error::{Result, VectorizerError};

/// API key format prefix
const API_KEY_PREFIX_LIVE: &str = "hh_live_";
const API_KEY_PREFIX_TEST: &str = "hh_test_";
const API_KEY_MIN_LENGTH: usize = 40;

/// Permission levels for API keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantPermission {
    /// Full administrative access
    Admin,
    /// Read and write data operations
    ReadWrite,
    /// Read-only access
    ReadOnly,
    /// MCP protocol access (limited)
    Mcp,
}

impl TenantPermission {
    /// Check if this permission allows the specified operation
    pub fn allows(&self, operation: &str) -> bool {
        match self {
            TenantPermission::Admin => true,
            TenantPermission::ReadWrite => {
                matches!(
                    operation,
                    "create_collection"
                        | "delete_collection"
                        | "list_collections"
                        | "insert_vectors"
                        | "update_vectors"
                        | "delete_vectors"
                        | "search"
                        | "get_collection"
                        | "get_vector"
                )
            }
            TenantPermission::ReadOnly => {
                matches!(
                    operation,
                    "list_collections" | "search" | "get_collection" | "get_vector"
                )
            }
            TenantPermission::Mcp => {
                matches!(
                    operation,
                    "list_collections" | "insert_vectors" | "update_vectors" | "search"
                )
            }
        }
    }

    /// Check if this permission level is admin
    pub fn is_admin(&self) -> bool {
        matches!(self, TenantPermission::Admin)
    }

    /// Check if this permission allows write operations
    pub fn can_write(&self) -> bool {
        matches!(
            self,
            TenantPermission::Admin | TenantPermission::ReadWrite | TenantPermission::Mcp
        )
    }

    /// Convert from SDK permission type
    pub fn from_sdk_permission(perm: &AccessKeyPermission) -> Self {
        match perm {
            AccessKeyPermission::Admin => TenantPermission::Admin,
            AccessKeyPermission::Write => TenantPermission::ReadWrite,
            AccessKeyPermission::Read => TenantPermission::ReadOnly,
            AccessKeyPermission::Delete => TenantPermission::ReadWrite,
        }
    }
}

impl std::fmt::Display for TenantPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantPermission::Admin => write!(f, "ADMIN"),
            TenantPermission::ReadWrite => write!(f, "READ_WRITE"),
            TenantPermission::ReadOnly => write!(f, "READ_ONLY"),
            TenantPermission::Mcp => write!(f, "MCP"),
        }
    }
}

/// Tenant context extracted from a validated API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    /// Unique tenant identifier
    pub tenant_id: String,
    /// Tenant display name
    pub tenant_name: String,
    /// API key identifier (for logging, not the actual key)
    pub api_key_id: String,
    /// Permissions granted to this API key
    pub permissions: Vec<TenantPermission>,
    /// Rate limit overrides (if any)
    pub rate_limits: Option<TenantRateLimits>,
    /// Validation timestamp
    pub validated_at: chrono::DateTime<chrono::Utc>,
    /// Whether this is a test key
    pub is_test: bool,
}

impl TenantContext {
    /// Get the tenant ID
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Check if tenant has a specific permission
    pub fn has_permission(&self, permission: TenantPermission) -> bool {
        self.permissions.contains(&permission)
            || self.permissions.contains(&TenantPermission::Admin)
    }

    /// Check if tenant is allowed to perform an operation
    pub fn can_perform(&self, operation: &str) -> bool {
        self.permissions.iter().any(|p| p.allows(operation))
    }

    /// Check if this is an admin tenant
    pub fn is_admin(&self) -> bool {
        self.permissions.contains(&TenantPermission::Admin)
    }

    /// Check if tenant has admin-level access
    pub fn can_admin(&self) -> bool {
        self.permissions.contains(&TenantPermission::Admin)
    }

    /// Check if tenant can perform write operations
    pub fn can_write(&self) -> bool {
        self.permissions.iter().any(|p| p.can_write())
    }

    /// Get the highest permission level
    pub fn highest_permission(&self) -> TenantPermission {
        if self.permissions.contains(&TenantPermission::Admin) {
            TenantPermission::Admin
        } else if self.permissions.contains(&TenantPermission::ReadWrite) {
            TenantPermission::ReadWrite
        } else if self.permissions.contains(&TenantPermission::Mcp) {
            TenantPermission::Mcp
        } else {
            TenantPermission::ReadOnly
        }
    }
}

/// Rate limit configuration for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRateLimits {
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    /// Requests per hour
    pub requests_per_hour: Option<u32>,
    /// Requests per day
    pub requests_per_day: Option<u32>,
}

/// Result of authentication attempt
#[derive(Debug, Clone)]
pub enum HubAuthResult {
    /// Authentication successful
    Success(TenantContext),
    /// Invalid API key format
    InvalidFormat,
    /// API key not found or revoked
    InvalidKey,
    /// API key expired
    Expired,
    /// Rate limited
    RateLimited { retry_after: Duration },
    /// Service unavailable
    ServiceUnavailable,
}

/// Cached authentication entry
#[derive(Debug, Clone)]
struct CachedAuth {
    context: TenantContext,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedAuth {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// HiveHub authentication handler
#[derive(Debug)]
pub struct HubAuth {
    /// HiveHub client
    client: Arc<HubClient>,
    /// Authentication cache (api_key_hash -> cached_auth)
    cache: Arc<RwLock<HashMap<String, CachedAuth>>>,
    /// Failed authentication tracking for brute force protection
    failed_attempts: Arc<RwLock<HashMap<String, FailedAttempts>>>,
    /// Cache TTL
    cache_ttl: Duration,
    /// Maximum cache entries
    max_cache_entries: usize,
}

/// Tracking for failed authentication attempts
#[derive(Debug, Clone)]
struct FailedAttempts {
    count: u32,
    first_attempt: Instant,
    last_attempt: Instant,
}

impl HubAuth {
    /// Maximum failed attempts before rate limiting
    const MAX_FAILED_ATTEMPTS: u32 = 5;
    /// Window for tracking failed attempts (60 seconds)
    const FAILED_ATTEMPT_WINDOW: Duration = Duration::from_secs(60);
    /// Block duration after max failures
    const BLOCK_DURATION: Duration = Duration::from_secs(300);

    /// Create a new HubAuth instance
    pub fn new(client: Arc<HubClient>, cache_config: &HubCacheConfig) -> Self {
        Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(cache_config.api_key_ttl_seconds),
            max_cache_entries: cache_config.max_entries,
        }
    }

    /// Validate an API key and return tenant context
    ///
    /// Note: For full validation, use validate_collection which validates
    /// both the key and collection ownership via the HiveHub API.
    pub async fn validate_api_key(&self, api_key: &str) -> Result<TenantContext> {
        // Validate format first (fast path)
        if !Self::validate_key_format(api_key) {
            warn!("Invalid API key format");
            return Err(VectorizerError::AuthenticationError(
                "Invalid API key format".to_string(),
            ));
        }

        // Hash the key for cache lookup and logging
        let key_hash = Self::hash_key(api_key);

        // Check for brute force protection
        if self.is_blocked(&key_hash) {
            warn!("API key blocked due to too many failed attempts");
            return Err(VectorizerError::RateLimitExceeded {
                limit_type: "auth_failure".to_string(),
                limit: Self::MAX_FAILED_ATTEMPTS,
            });
        }

        // Check cache first
        if let Some(cached) = self.get_cached(&key_hash) {
            trace!("Using cached authentication for key {}", &key_hash[..8]);
            return Ok(cached);
        }

        // For now, create a basic tenant context from the key format
        // Real validation happens during collection operations
        let context = self.create_context_from_key(api_key, &key_hash)?;

        // Cache the result
        self.cache_auth(&key_hash, context.clone());

        debug!(
            "API key validated for tenant: {} ({})",
            context.tenant_name, context.tenant_id
        );
        Ok(context)
    }

    /// Create tenant context from API key format
    fn create_context_from_key(&self, api_key: &str, key_hash: &str) -> Result<TenantContext> {
        let is_test = api_key.starts_with(API_KEY_PREFIX_TEST);

        // Extract tenant ID from key (first 8 chars of hash as identifier)
        let tenant_id = format!("tenant_{}", &key_hash[..16]);
        let api_key_id = format!("key_{}", &key_hash[..8]);

        // Default permissions for API keys
        let permissions = if is_test {
            vec![TenantPermission::ReadWrite]
        } else {
            vec![TenantPermission::ReadWrite]
        };

        Ok(TenantContext {
            tenant_id,
            tenant_name: format!("Tenant ({})", &key_hash[..8]),
            api_key_id,
            permissions,
            rate_limits: None,
            validated_at: chrono::Utc::now(),
            is_test,
        })
    }

    /// Validate a collection operation
    /// This performs full validation through the HiveHub API
    pub async fn validate_collection(
        &self,
        api_key: &str,
        collection_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<TenantContext> {
        // First validate the key format
        let context = self.validate_api_key(api_key).await?;

        // Then validate collection ownership via API
        let validation = self
            .client
            .validate_collection(collection_id, user_id)
            .await?;

        if !validation.valid {
            return Err(VectorizerError::AuthorizationError(format!(
                "Collection {} does not belong to user {}",
                collection_id, user_id
            )));
        }

        // Update context with validated user info
        Ok(TenantContext {
            tenant_id: validation.user_id.to_string(),
            tenant_name: context.tenant_name,
            api_key_id: context.api_key_id,
            permissions: context.permissions,
            rate_limits: context.rate_limits,
            validated_at: chrono::Utc::now(),
            is_test: context.is_test,
        })
    }

    /// Validate API key format
    fn validate_key_format(api_key: &str) -> bool {
        if api_key.len() < API_KEY_MIN_LENGTH {
            return false;
        }

        api_key.starts_with(API_KEY_PREFIX_LIVE) || api_key.starts_with(API_KEY_PREFIX_TEST)
    }

    /// Hash an API key for storage/logging
    fn hash_key(api_key: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get cached authentication if valid
    fn get_cached(&self, key_hash: &str) -> Option<TenantContext> {
        let cache = self.cache.read();
        cache.get(key_hash).and_then(|cached| {
            if cached.is_expired() {
                None
            } else {
                Some(cached.context.clone())
            }
        })
    }

    /// Cache authentication result
    fn cache_auth(&self, key_hash: &str, context: TenantContext) {
        let mut cache = self.cache.write();

        // Evict expired entries if cache is full
        if cache.len() >= self.max_cache_entries {
            cache.retain(|_, v| !v.is_expired());
        }

        // If still full, remove oldest entry
        if cache.len() >= self.max_cache_entries {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.cached_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(
            key_hash.to_string(),
            CachedAuth {
                context,
                cached_at: Instant::now(),
                ttl: self.cache_ttl,
            },
        );
    }

    /// Invalidate cache for a specific key
    pub fn invalidate_cache(&self, api_key: &str) {
        let key_hash = Self::hash_key(api_key);
        self.cache.write().remove(&key_hash);
    }

    /// Clear entire cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Check if an IP/key is blocked
    fn is_blocked(&self, key_hash: &str) -> bool {
        let attempts = self.failed_attempts.read();
        if let Some(failed) = attempts.get(key_hash) {
            if failed.count >= Self::MAX_FAILED_ATTEMPTS {
                // Check if still in block window
                if failed.last_attempt.elapsed() < Self::BLOCK_DURATION {
                    return true;
                }
            }
        }
        false
    }

    /// Record a failed authentication attempt
    fn record_failed_attempt(&self, key_hash: &str) {
        let mut attempts = self.failed_attempts.write();
        let now = Instant::now();

        let entry = attempts
            .entry(key_hash.to_string())
            .or_insert(FailedAttempts {
                count: 0,
                first_attempt: now,
                last_attempt: now,
            });

        // Reset if outside window
        if entry.first_attempt.elapsed() > Self::FAILED_ATTEMPT_WINDOW {
            entry.count = 0;
            entry.first_attempt = now;
        }

        entry.count += 1;
        entry.last_attempt = now;
    }

    /// Reset failed attempts for a key
    fn reset_failed_attempts(&self, key_hash: &str) {
        self.failed_attempts.write().remove(key_hash);
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read();
        let total = cache.len();
        let expired = cache.values().filter(|v| v.is_expired()).count();
        (total, total - expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_key_format() {
        // Valid formats
        assert!(HubAuth::validate_key_format(
            "hh_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"
        ));
        assert!(HubAuth::validate_key_format(
            "hh_test_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"
        ));

        // Invalid formats
        assert!(!HubAuth::validate_key_format("invalid_key"));
        assert!(!HubAuth::validate_key_format("hh_live_short"));
        assert!(!HubAuth::validate_key_format(""));
    }

    #[test]
    fn test_permission_allows() {
        // Admin allows everything
        assert!(TenantPermission::Admin.allows("create_collection"));
        assert!(TenantPermission::Admin.allows("search"));
        assert!(TenantPermission::Admin.allows("admin_endpoint"));

        // ReadWrite allows data operations
        assert!(TenantPermission::ReadWrite.allows("create_collection"));
        assert!(TenantPermission::ReadWrite.allows("search"));
        assert!(!TenantPermission::ReadWrite.allows("admin_endpoint"));

        // ReadOnly only allows reads
        assert!(!TenantPermission::ReadOnly.allows("create_collection"));
        assert!(TenantPermission::ReadOnly.allows("search"));
        assert!(TenantPermission::ReadOnly.allows("list_collections"));

        // MCP has limited write access
        assert!(!TenantPermission::Mcp.allows("create_collection"));
        assert!(!TenantPermission::Mcp.allows("delete_vectors"));
        assert!(TenantPermission::Mcp.allows("insert_vectors"));
        assert!(TenantPermission::Mcp.allows("search"));
    }

    #[test]
    fn test_tenant_context_permissions() {
        let context = TenantContext {
            tenant_id: "tenant_123".to_string(),
            tenant_name: "Test Tenant".to_string(),
            api_key_id: "key_abc".to_string(),
            permissions: vec![TenantPermission::ReadWrite],
            rate_limits: None,
            validated_at: chrono::Utc::now(),
            is_test: true,
        };

        assert!(context.has_permission(TenantPermission::ReadWrite));
        assert!(!context.has_permission(TenantPermission::Admin));
        assert!(context.can_perform("search"));
        assert!(context.can_perform("insert_vectors"));
        assert!(!context.is_admin());
        assert_eq!(context.highest_permission(), TenantPermission::ReadWrite);
    }

    #[test]
    fn test_hash_key() {
        let key1 = "hh_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6";
        let key2 = "hh_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6";
        let key3 = "hh_live_different_key_value_here_abcde";

        // Same keys should have same hash
        assert_eq!(HubAuth::hash_key(key1), HubAuth::hash_key(key2));

        // Different keys should have different hashes
        assert_ne!(HubAuth::hash_key(key1), HubAuth::hash_key(key3));
    }
}
