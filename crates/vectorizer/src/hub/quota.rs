//! Quota management module for HiveHub integration
//!
//! Provides quota checking and enforcement for multi-tenant operations,
//! including storage limits, vector counts, and rate limiting.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};

use super::HubCacheConfig;
use super::client::HubClient;
use crate::error::{Result, VectorizerError};
use crate::monitoring::metrics::METRICS;

/// Types of quotas that can be checked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaType {
    /// Maximum storage in bytes
    Storage,
    /// Maximum number of vectors
    VectorCount,
    /// Maximum number of collections
    CollectionCount,
    /// Rate limit - requests per minute
    RequestsPerMinute,
    /// Rate limit - requests per hour
    RequestsPerHour,
    /// Rate limit - requests per day
    RequestsPerDay,
}

impl std::fmt::Display for QuotaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaType::Storage => write!(f, "storage"),
            QuotaType::VectorCount => write!(f, "vector_count"),
            QuotaType::CollectionCount => write!(f, "collection_count"),
            QuotaType::RequestsPerMinute => write!(f, "requests_per_minute"),
            QuotaType::RequestsPerHour => write!(f, "requests_per_hour"),
            QuotaType::RequestsPerDay => write!(f, "requests_per_day"),
        }
    }
}

/// Quota information for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaInfo {
    /// Tenant identifier
    pub tenant_id: String,

    /// Storage quota
    pub storage: StorageQuota,

    /// Vector quota
    pub vectors: VectorQuota,

    /// Collection quota
    pub collections: CollectionQuota,

    /// Rate limits
    pub rate_limits: RateLimitQuota,

    /// When this quota was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Storage quota details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuota {
    /// Maximum storage allowed in bytes
    pub limit: u64,
    /// Current usage in bytes
    pub used: u64,
    /// Whether additional storage can be allocated
    pub can_allocate: bool,
}

impl StorageQuota {
    /// Check if a certain amount of storage can be used
    pub fn can_use(&self, bytes: u64) -> bool {
        self.can_allocate && (self.used + bytes <= self.limit)
    }

    /// Get remaining storage in bytes
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Get usage percentage
    pub fn usage_percent(&self) -> f64 {
        if self.limit == 0 {
            100.0
        } else {
            (self.used as f64 / self.limit as f64) * 100.0
        }
    }
}

/// Vector quota details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorQuota {
    /// Maximum vectors allowed
    pub limit: u64,
    /// Current vector count
    pub used: u64,
    /// Whether additional vectors can be inserted
    pub can_insert: bool,
}

impl VectorQuota {
    /// Check if a certain number of vectors can be inserted
    pub fn can_use(&self, count: u64) -> bool {
        self.can_insert && (self.used + count <= self.limit)
    }

    /// Get remaining vector capacity
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }
}

/// Collection quota details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionQuota {
    /// Maximum collections allowed
    pub limit: u64,
    /// Current collection count
    pub used: u64,
    /// Whether additional collections can be created
    pub can_create: bool,
}

impl CollectionQuota {
    /// Check if a collection can be created
    pub fn can_use(&self) -> bool {
        self.can_create && (self.used < self.limit)
    }

    /// Get remaining collection capacity
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }
}

/// Rate limit quota details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitQuota {
    /// Requests per minute limit
    pub requests_per_minute: u32,
    /// Requests per hour limit
    pub requests_per_hour: u32,
    /// Requests per day limit
    pub requests_per_day: u32,
}

/// Cached quota entry
#[derive(Debug, Clone)]
struct CachedQuota {
    info: QuotaInfo,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedQuota {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// Rate limit tracking for a tenant
#[derive(Debug)]
struct RateLimitTracker {
    /// Requests in current minute
    minute_count: u32,
    minute_reset: Instant,
    /// Requests in current hour
    hour_count: u32,
    hour_reset: Instant,
    /// Requests in current day
    day_count: u32,
    day_reset: Instant,
}

impl RateLimitTracker {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            minute_count: 0,
            minute_reset: now,
            hour_count: 0,
            hour_reset: now,
            day_count: 0,
            day_reset: now,
        }
    }

    fn check_and_increment(&mut self, limits: &RateLimitQuota) -> Option<QuotaType> {
        let now = Instant::now();

        // Reset counters if needed
        if now.duration_since(self.minute_reset) >= Duration::from_secs(60) {
            self.minute_count = 0;
            self.minute_reset = now;
        }
        if now.duration_since(self.hour_reset) >= Duration::from_secs(3600) {
            self.hour_count = 0;
            self.hour_reset = now;
        }
        if now.duration_since(self.day_reset) >= Duration::from_secs(86400) {
            self.day_count = 0;
            self.day_reset = now;
        }

        // Check limits
        if self.minute_count >= limits.requests_per_minute {
            return Some(QuotaType::RequestsPerMinute);
        }
        if self.hour_count >= limits.requests_per_hour {
            return Some(QuotaType::RequestsPerHour);
        }
        if self.day_count >= limits.requests_per_day {
            return Some(QuotaType::RequestsPerDay);
        }

        // Increment counters
        self.minute_count += 1;
        self.hour_count += 1;
        self.day_count += 1;

        None
    }

    fn get_retry_after(&self, quota_type: QuotaType) -> Duration {
        let now = Instant::now();
        match quota_type {
            QuotaType::RequestsPerMinute => {
                Duration::from_secs(60) - now.duration_since(self.minute_reset)
            }
            QuotaType::RequestsPerHour => {
                Duration::from_secs(3600) - now.duration_since(self.hour_reset)
            }
            QuotaType::RequestsPerDay => {
                Duration::from_secs(86400) - now.duration_since(self.day_reset)
            }
            _ => Duration::from_secs(60),
        }
    }
}

/// Quota manager for HiveHub integration
#[derive(Debug)]
pub struct QuotaManager {
    /// HiveHub client
    client: Arc<HubClient>,
    /// Quota cache (tenant_id -> cached_quota)
    cache: Arc<RwLock<HashMap<String, CachedQuota>>>,
    /// Rate limit trackers (tenant_id -> tracker)
    rate_limits: Arc<RwLock<HashMap<String, RateLimitTracker>>>,
    /// Cache TTL
    cache_ttl: Duration,
    /// Maximum cache entries
    max_cache_entries: usize,
}

impl QuotaManager {
    /// Create a new QuotaManager
    pub fn new(client: Arc<HubClient>, cache_config: &HubCacheConfig) -> Self {
        Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(cache_config.quota_ttl_seconds),
            max_cache_entries: cache_config.max_entries,
        }
    }

    /// Check if a quota allows a specific operation
    pub async fn check_quota(
        &self,
        tenant_id: &str,
        quota_type: QuotaType,
        requested: u64,
    ) -> Result<bool> {
        let timer = METRICS.hub_quota_check_latency_seconds.start_timer();
        let quota_type_str = quota_type.to_string();

        let quota_info = self.get_quota(tenant_id).await?;

        let allowed = match quota_type {
            QuotaType::Storage => quota_info.storage.can_use(requested),
            QuotaType::VectorCount => quota_info.vectors.can_use(requested),
            QuotaType::CollectionCount => quota_info.collections.can_use(),
            QuotaType::RequestsPerMinute
            | QuotaType::RequestsPerHour
            | QuotaType::RequestsPerDay => {
                // Rate limits are checked separately
                self.check_rate_limit(tenant_id, &quota_info.rate_limits)?
            }
        };

        // Record metrics
        let result_label = if allowed { "allowed" } else { "denied" };
        METRICS
            .hub_quota_checks_total
            .with_label_values(&[tenant_id, &quota_type_str, result_label])
            .inc();

        if !allowed {
            METRICS
                .hub_quota_exceeded_total
                .with_label_values(&[tenant_id, &quota_type_str])
                .inc();
            debug!(
                "Quota check failed for tenant {}: {} (requested: {})",
                tenant_id, quota_type, requested
            );
        }

        // Update quota usage gauge
        let usage = match quota_type {
            QuotaType::Storage => quota_info.storage.used as f64,
            QuotaType::VectorCount => quota_info.vectors.used as f64,
            QuotaType::CollectionCount => quota_info.collections.used as f64,
            _ => 0.0,
        };
        METRICS
            .hub_quota_usage
            .with_label_values(&[tenant_id, &quota_type_str])
            .set(usage);

        drop(timer);
        Ok(allowed)
    }

    /// Get quota information for a tenant
    pub async fn get_quota(&self, tenant_id: &str) -> Result<QuotaInfo> {
        // Check cache first
        if let Some(cached) = self.get_cached(tenant_id) {
            trace!("Using cached quota for tenant {}", tenant_id);
            return Ok(cached);
        }

        // Fetch from HiveHub
        let quota = self.fetch_quota(tenant_id).await?;

        // Cache the result
        self.cache_quota(tenant_id, quota.clone());

        Ok(quota)
    }

    /// Check rate limit for a tenant
    fn check_rate_limit(&self, tenant_id: &str, limits: &RateLimitQuota) -> Result<bool> {
        let mut rate_limits = self.rate_limits.write();

        let tracker = rate_limits
            .entry(tenant_id.to_string())
            .or_insert_with(RateLimitTracker::new);

        if let Some(quota_type) = tracker.check_and_increment(limits) {
            let retry_after = tracker.get_retry_after(quota_type);
            warn!(
                "Rate limit exceeded for tenant {}: {} (retry after {:?})",
                tenant_id, quota_type, retry_after
            );
            return Err(VectorizerError::RateLimitExceeded {
                limit_type: quota_type.to_string(),
                limit: match quota_type {
                    QuotaType::RequestsPerMinute => limits.requests_per_minute,
                    QuotaType::RequestsPerHour => limits.requests_per_hour,
                    QuotaType::RequestsPerDay => limits.requests_per_day,
                    _ => 0,
                },
            });
        }

        Ok(true)
    }

    /// Enforce quota before an operation
    pub async fn enforce(
        &self,
        tenant_id: &str,
        quota_type: QuotaType,
        requested: u64,
    ) -> Result<()> {
        if !self.check_quota(tenant_id, quota_type, requested).await? {
            return Err(VectorizerError::RateLimitExceeded {
                limit_type: quota_type.to_string(),
                limit: 0,
            });
        }
        Ok(())
    }

    /// Get cached quota if valid
    fn get_cached(&self, tenant_id: &str) -> Option<QuotaInfo> {
        let cache = self.cache.read();
        cache.get(tenant_id).and_then(|cached| {
            if cached.is_expired() {
                None
            } else {
                Some(cached.info.clone())
            }
        })
    }

    /// Cache quota information
    fn cache_quota(&self, tenant_id: &str, info: QuotaInfo) {
        let mut cache = self.cache.write();

        // Evict expired entries if cache is full
        if cache.len() >= self.max_cache_entries {
            cache.retain(|_, v| !v.is_expired());
        }

        cache.insert(
            tenant_id.to_string(),
            CachedQuota {
                info,
                cached_at: Instant::now(),
                ttl: self.cache_ttl,
            },
        );
    }

    /// Invalidate cache for a tenant
    pub fn invalidate_cache(&self, tenant_id: &str) {
        self.cache.write().remove(tenant_id);
    }

    /// Clear entire cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
        self.rate_limits.write().clear();
    }

    /// Fetch quota from HiveHub
    ///
    /// Note: In cluster mode, the Vectorizer runs locally and quotas are
    /// managed by HiveHub. This uses the SDK's check_quota to verify
    /// if operations are allowed. The SDK returns simple allowed/remaining/limit
    /// fields, so we use these to build local quota info.
    async fn fetch_quota(&self, tenant_id: &str) -> Result<QuotaInfo> {
        use hivehub_internal_sdk::models::QuotaCheckRequest;

        // Use check_quota to verify connectivity and get basic quota status
        let request = QuotaCheckRequest {
            project_id: tenant_id.to_string(),
            operation: "status".to_string(),
            estimated_size: None,
        };

        match self.client.check_quota(&request).await {
            Ok(response) => {
                // Build quota info from response
                // The SDK check_quota returns allowed, remaining, and limit (as i64)
                // We use these for all quota types since the Hub manages
                // detailed quotas and the Vectorizer just needs to know if allowed
                let limit = response.limit.unwrap_or(i64::MAX).max(0) as u64;
                let remaining = response.remaining.unwrap_or(i64::MAX).max(0) as u64;
                let used = limit.saturating_sub(remaining);

                Ok(QuotaInfo {
                    tenant_id: tenant_id.to_string(),
                    storage: StorageQuota {
                        limit,
                        used,
                        can_allocate: response.allowed,
                    },
                    vectors: VectorQuota {
                        limit,
                        used,
                        can_insert: response.allowed,
                    },
                    collections: CollectionQuota {
                        limit,
                        used,
                        can_create: response.allowed,
                    },
                    rate_limits: RateLimitQuota {
                        // Default rate limits for local operations
                        requests_per_minute: 1000,
                        requests_per_hour: 10000,
                        requests_per_day: 100000,
                    },
                    updated_at: chrono::Utc::now(),
                })
            }
            Err(e) => {
                warn!("Failed to fetch quota from HiveHub: {}, using defaults", e);
                // Return permissive defaults when HiveHub is unavailable
                // The Hub handles actual enforcement
                Ok(QuotaInfo {
                    tenant_id: tenant_id.to_string(),
                    storage: StorageQuota {
                        limit: u64::MAX,
                        used: 0,
                        can_allocate: true,
                    },
                    vectors: VectorQuota {
                        limit: u64::MAX,
                        used: 0,
                        can_insert: true,
                    },
                    collections: CollectionQuota {
                        limit: u64::MAX,
                        used: 0,
                        can_create: true,
                    },
                    rate_limits: RateLimitQuota {
                        requests_per_minute: 1000,
                        requests_per_hour: 10000,
                        requests_per_day: 100000,
                    },
                    updated_at: chrono::Utc::now(),
                })
            }
        }
    }

    /// Update local usage tracking (optimistic update before HiveHub sync)
    pub fn update_local_usage(
        &self,
        tenant_id: &str,
        storage_delta: i64,
        vector_delta: i64,
        collection_delta: i64,
    ) {
        let mut cache = self.cache.write();

        if let Some(cached) = cache.get_mut(tenant_id) {
            // Update storage
            if storage_delta >= 0 {
                cached.info.storage.used = cached
                    .info
                    .storage
                    .used
                    .saturating_add(storage_delta as u64);
            } else {
                cached.info.storage.used = cached
                    .info
                    .storage
                    .used
                    .saturating_sub((-storage_delta) as u64);
            }
            cached.info.storage.can_allocate = cached.info.storage.used < cached.info.storage.limit;

            // Update vectors
            if vector_delta >= 0 {
                cached.info.vectors.used =
                    cached.info.vectors.used.saturating_add(vector_delta as u64);
            } else {
                cached.info.vectors.used = cached
                    .info
                    .vectors
                    .used
                    .saturating_sub((-vector_delta) as u64);
            }
            cached.info.vectors.can_insert = cached.info.vectors.used < cached.info.vectors.limit;

            // Update collections
            if collection_delta >= 0 {
                cached.info.collections.used = cached
                    .info
                    .collections
                    .used
                    .saturating_add(collection_delta as u64);
            } else {
                cached.info.collections.used = cached
                    .info
                    .collections
                    .used
                    .saturating_sub((-collection_delta) as u64);
            }
            cached.info.collections.can_create =
                cached.info.collections.used < cached.info.collections.limit;

            cached.info.updated_at = chrono::Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_quota_can_use() {
        let quota = StorageQuota {
            limit: 1000,
            used: 500,
            can_allocate: true,
        };

        assert!(quota.can_use(499)); // Within remaining
        assert!(quota.can_use(500)); // Exactly remaining
        assert!(!quota.can_use(501)); // Over limit

        let disabled = StorageQuota {
            limit: 1000,
            used: 500,
            can_allocate: false,
        };
        assert!(!disabled.can_use(1)); // Can't allocate even small amount
    }

    #[test]
    fn test_storage_quota_remaining() {
        let quota = StorageQuota {
            limit: 1000,
            used: 300,
            can_allocate: true,
        };
        assert_eq!(quota.remaining(), 700);

        let over_quota = StorageQuota {
            limit: 1000,
            used: 1500,
            can_allocate: false,
        };
        assert_eq!(over_quota.remaining(), 0);
    }

    #[test]
    fn test_storage_quota_usage_percent() {
        let quota = StorageQuota {
            limit: 1000,
            used: 250,
            can_allocate: true,
        };
        assert!((quota.usage_percent() - 25.0).abs() < 0.01);

        let empty = StorageQuota {
            limit: 0,
            used: 0,
            can_allocate: false,
        };
        assert!((empty.usage_percent() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_vector_quota_can_use() {
        let quota = VectorQuota {
            limit: 10000,
            used: 5000,
            can_insert: true,
        };

        assert!(quota.can_use(4999));
        assert!(quota.can_use(5000));
        assert!(!quota.can_use(5001));
    }

    #[test]
    fn test_collection_quota_can_use() {
        let quota = CollectionQuota {
            limit: 10,
            used: 9,
            can_create: true,
        };
        assert!(quota.can_use());

        let full = CollectionQuota {
            limit: 10,
            used: 10,
            can_create: true,
        };
        assert!(!full.can_use());
    }

    #[test]
    fn test_quota_type_display() {
        assert_eq!(QuotaType::Storage.to_string(), "storage");
        assert_eq!(QuotaType::VectorCount.to_string(), "vector_count");
        assert_eq!(QuotaType::CollectionCount.to_string(), "collection_count");
        assert_eq!(
            QuotaType::RequestsPerMinute.to_string(),
            "requests_per_minute"
        );
    }
}
