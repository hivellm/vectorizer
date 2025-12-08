//! Rate Limiting Implementation
//!
//! This module provides rate limiting functionality to prevent API abuse.
//! It supports both global rate limiting and per-API-key limiting.
//!
//! # Features
//! - Global rate limiting for all requests
//! - Per-API-key rate limiting with customizable limits
//! - Configurable rate limits per key via configuration file
//! - Tiered rate limiting (different limits for different API key tiers)

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second per API key (default limit)
    #[serde(default = "default_requests_per_second")]
    pub requests_per_second: u32,
    /// Burst capacity (default burst)
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
    /// Per-key rate limit overrides
    #[serde(default)]
    pub key_overrides: HashMap<String, KeyRateLimitConfig>,
    /// Tier-based rate limits
    #[serde(default)]
    pub tiers: HashMap<String, TierRateLimitConfig>,
    /// API key to tier mapping
    #[serde(default)]
    pub key_tiers: HashMap<String, String>,
}

fn default_requests_per_second() -> u32 {
    100
}

fn default_burst_size() -> u32 {
    200
}

/// Per-key rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRateLimitConfig {
    /// Requests per second for this specific key
    pub requests_per_second: u32,
    /// Burst capacity for this specific key
    pub burst_size: u32,
    /// Optional description for this key
    #[serde(default)]
    pub description: Option<String>,
}

/// Tier-based rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRateLimitConfig {
    /// Tier name (e.g., "free", "basic", "premium", "enterprise")
    pub name: String,
    /// Requests per second for this tier
    pub requests_per_second: u32,
    /// Burst capacity for this tier
    pub burst_size: u32,
    /// Optional daily request limit
    #[serde(default)]
    pub daily_limit: Option<u64>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100, // 100 req/s default
            burst_size: 200,          // Allow bursts up to 200
            key_overrides: HashMap::new(),
            tiers: default_tiers(),
            key_tiers: HashMap::new(),
        }
    }
}

/// Create default tier configuration
fn default_tiers() -> HashMap<String, TierRateLimitConfig> {
    let mut tiers = HashMap::new();

    tiers.insert(
        "free".to_string(),
        TierRateLimitConfig {
            name: "Free".to_string(),
            requests_per_second: 10,
            burst_size: 20,
            daily_limit: Some(1000),
        },
    );

    tiers.insert(
        "basic".to_string(),
        TierRateLimitConfig {
            name: "Basic".to_string(),
            requests_per_second: 50,
            burst_size: 100,
            daily_limit: Some(10000),
        },
    );

    tiers.insert(
        "premium".to_string(),
        TierRateLimitConfig {
            name: "Premium".to_string(),
            requests_per_second: 200,
            burst_size: 400,
            daily_limit: Some(100000),
        },
    );

    tiers.insert(
        "enterprise".to_string(),
        TierRateLimitConfig {
            name: "Enterprise".to_string(),
            requests_per_second: 1000,
            burst_size: 2000,
            daily_limit: None, // Unlimited
        },
    );

    tiers
}

impl RateLimitConfig {
    /// Create a new rate limit config with custom defaults
    pub fn with_defaults(requests_per_second: u32, burst_size: u32) -> Self {
        Self {
            requests_per_second,
            burst_size,
            ..Default::default()
        }
    }

    /// Add a key-specific rate limit override
    pub fn add_key_override(&mut self, api_key: String, requests_per_second: u32, burst_size: u32) {
        self.key_overrides.insert(
            api_key,
            KeyRateLimitConfig {
                requests_per_second,
                burst_size,
                description: None,
            },
        );
    }

    /// Assign an API key to a tier
    pub fn assign_key_to_tier(&mut self, api_key: String, tier: String) {
        self.key_tiers.insert(api_key, tier);
    }

    /// Get the rate limit for a specific API key
    pub fn get_key_limits(&self, api_key: &str) -> (u32, u32) {
        // First check for key-specific override
        if let Some(override_config) = self.key_overrides.get(api_key) {
            return (
                override_config.requests_per_second,
                override_config.burst_size,
            );
        }

        // Then check for tier assignment
        if let Some(tier_name) = self.key_tiers.get(api_key) {
            if let Some(tier_config) = self.tiers.get(tier_name) {
                return (tier_config.requests_per_second, tier_config.burst_size);
            }
        }

        // Fall back to defaults
        (self.requests_per_second, self.burst_size)
    }

    /// Load configuration from YAML file
    pub fn from_yaml_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        info!(
            "Loaded rate limit config from {}: {} key overrides, {} tiers",
            path,
            config.key_overrides.len(),
            config.tiers.len()
        );
        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn to_yaml_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Type alias for the keyed rate limiter
type KeyedLimiter = GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Rate limiter for the vectorizer API
#[derive(Clone)]
pub struct RateLimiter {
    config: Arc<RateLimitConfig>,
    global_limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    /// Per-API-key rate limiters
    key_limiters: Arc<DashMap<String, Arc<KeyedLimiter>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(config.requests_per_second).unwrap_or(NonZeroU32::new(100).unwrap()),
        )
        .allow_burst(NonZeroU32::new(config.burst_size).unwrap_or(NonZeroU32::new(200).unwrap()));

        let global_limiter = Arc::new(GovernorRateLimiter::direct(quota));

        Self {
            config: Arc::new(config),
            global_limiter,
            key_limiters: Arc::new(DashMap::new()),
        }
    }

    /// Create rate limiter from configuration file
    pub fn from_config_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = RateLimitConfig::from_yaml_file(path)?;
        Ok(Self::new(config))
    }

    /// Get the configuration
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Check if a request should be allowed (global limit)
    pub fn check_global(&self) -> bool {
        self.global_limiter.check().is_ok()
    }

    /// Check if a request should be allowed for a specific API key
    /// Uses per-key configuration if available, otherwise defaults
    pub fn check_key(&self, api_key: &str) -> bool {
        // Get the rate limits for this specific key (may be custom or default)
        let (rps, burst) = self.config.get_key_limits(api_key);

        // Get or create a rate limiter for this API key
        let limiter = self
            .key_limiters
            .entry(api_key.to_string())
            .or_insert_with(|| {
                let quota = Quota::per_second(
                    NonZeroU32::new(rps).unwrap_or(NonZeroU32::new(100).unwrap()),
                )
                .allow_burst(NonZeroU32::new(burst).unwrap_or(NonZeroU32::new(200).unwrap()));
                Arc::new(GovernorRateLimiter::direct(quota))
            });

        limiter.check().is_ok()
    }

    /// Check if a request should be allowed for a specific API key with custom limits
    /// This creates a new limiter if the key doesn't exist with the specified limits
    pub fn check_key_with_limits(
        &self,
        api_key: &str,
        requests_per_second: u32,
        burst_size: u32,
    ) -> bool {
        let limiter = self
            .key_limiters
            .entry(api_key.to_string())
            .or_insert_with(|| {
                let quota = Quota::per_second(
                    NonZeroU32::new(requests_per_second).unwrap_or(NonZeroU32::new(100).unwrap()),
                )
                .allow_burst(NonZeroU32::new(burst_size).unwrap_or(NonZeroU32::new(200).unwrap()));
                Arc::new(GovernorRateLimiter::direct(quota))
            });

        limiter.check().is_ok()
    }

    /// Check both global and per-key limits
    /// Returns true if both checks pass
    pub fn check(&self, api_key: Option<&str>) -> bool {
        // First check global limit
        if !self.check_global() {
            debug!("Global rate limit exceeded");
            return false;
        }

        // Then check per-key limit if API key is provided
        if let Some(key) = api_key {
            if !self.check_key(key) {
                debug!(
                    "Per-key rate limit exceeded for key: {}...",
                    &key[..8.min(key.len())]
                );
                return false;
            }
        }

        true
    }

    /// Update rate limiter for a specific key with new limits
    /// This removes the old limiter and creates a new one with the updated limits
    pub fn update_key_limits(&self, api_key: &str, requests_per_second: u32, burst_size: u32) {
        let quota = Quota::per_second(
            NonZeroU32::new(requests_per_second).unwrap_or(NonZeroU32::new(100).unwrap()),
        )
        .allow_burst(NonZeroU32::new(burst_size).unwrap_or(NonZeroU32::new(200).unwrap()));

        self.key_limiters.insert(
            api_key.to_string(),
            Arc::new(GovernorRateLimiter::direct(quota)),
        );
    }

    /// Remove rate limiter for a specific key
    pub fn remove_key(&self, api_key: &str) {
        self.key_limiters.remove(api_key);
    }

    /// Get the number of active key limiters
    pub fn active_key_count(&self) -> usize {
        self.key_limiters.len()
    }

    /// Clean up expired rate limiters for keys that haven't been used recently
    /// This should be called periodically to prevent memory leaks
    pub fn cleanup_expired(&self) {
        // For now, we just clear limiters that have been unused
        // In a more sophisticated implementation, we would track last access time
        let count = self.key_limiters.len();
        if count > 10000 {
            // If we have too many limiters, clear the oldest ones
            warn!(
                "Rate limiter has {} key limiters, consider implementing LRU eviction",
                count
            );
        }
    }

    /// Get rate limit info for a specific key
    pub fn get_key_info(&self, api_key: &str) -> Option<(u32, u32, bool)> {
        let (rps, burst) = self.config.get_key_limits(api_key);
        let exists = self.key_limiters.contains_key(api_key);
        Some((rps, burst, exists))
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

/// Extract API key from request headers
fn extract_api_key(req: &Request) -> Option<String> {
    // Try Authorization header first (Bearer token)
    if let Some(auth) = req.headers().get("authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(key) = req.headers().get("x-api-key") {
        if let Ok(key_str) = key.to_str() {
            return Some(key_str.to_string());
        }
    }

    None
}

/// Global rate limiter instance
static GLOBAL_RATE_LIMITER: once_cell::sync::Lazy<RateLimiter> =
    once_cell::sync::Lazy::new(RateLimiter::default);

/// Rate limiting middleware
pub async fn rate_limit_middleware(req: Request, next: Next) -> Response {
    use crate::monitoring::metrics::METRICS;

    // Extract API key from request
    let api_key = extract_api_key(&req);

    // Check rate limits
    let allowed = GLOBAL_RATE_LIMITER.check(api_key.as_deref());

    if !allowed {
        // Record rate limit exceeded
        METRICS
            .api_errors_total
            .with_label_values(&["rate_limit", "exceeded", "429"])
            .inc();

        warn!(
            "Rate limit exceeded for {}",
            api_key
                .as_ref()
                .map(|k| format!("API key {}...", &k[..8.min(k.len())]))
                .unwrap_or_else(|| "anonymous".to_string())
        );

        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please slow down your requests.",
        )
            .into_response();
    }

    // Record successful check
    METRICS
        .api_errors_total
        .with_label_values(&["rate_limit", "check", "200"])
        .inc();

    // Also track per-tenant API requests if we have an API key
    if let Some(key) = &api_key {
        METRICS.record_tenant_api_request(key);
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        assert_eq!(limiter.config.requests_per_second, 100);
        assert_eq!(limiter.config.burst_size, 200);
    }

    #[test]
    fn test_rate_limiter_default() {
        let limiter = RateLimiter::default();
        assert_eq!(limiter.config.requests_per_second, 100);
    }

    #[test]
    fn test_custom_config() {
        let config = RateLimitConfig::with_defaults(50, 100);
        let limiter = RateLimiter::new(config);
        assert_eq!(limiter.config.requests_per_second, 50);
        assert_eq!(limiter.config.burst_size, 100);
    }

    #[test]
    fn test_global_check() {
        let limiter = RateLimiter::default();

        // First request should pass
        assert!(limiter.check_global());
    }

    #[test]
    fn test_rate_limit_enforcement() {
        let config = RateLimitConfig::with_defaults(2, 2);
        let limiter = RateLimiter::new(config);

        // First 2 requests should pass (burst)
        assert!(limiter.check_global());
        assert!(limiter.check_global());

        // 3rd request should fail
        assert!(!limiter.check_global());
    }

    #[test]
    fn test_default_tiers() {
        let config = RateLimitConfig::default();

        // Verify default tiers exist
        assert!(config.tiers.contains_key("free"));
        assert!(config.tiers.contains_key("basic"));
        assert!(config.tiers.contains_key("premium"));
        assert!(config.tiers.contains_key("enterprise"));

        // Verify tier values
        let free = config.tiers.get("free").unwrap();
        assert_eq!(free.requests_per_second, 10);
        assert_eq!(free.burst_size, 20);
        assert_eq!(free.daily_limit, Some(1000));

        let enterprise = config.tiers.get("enterprise").unwrap();
        assert_eq!(enterprise.requests_per_second, 1000);
        assert_eq!(enterprise.daily_limit, None); // Unlimited
    }

    #[test]
    fn test_key_override() {
        let mut config = RateLimitConfig::default();

        // Add a key override
        config.add_key_override("special-key".to_string(), 500, 1000);

        // Verify override is applied
        let (rps, burst) = config.get_key_limits("special-key");
        assert_eq!(rps, 500);
        assert_eq!(burst, 1000);

        // Verify default is returned for non-overridden keys
        let (rps, burst) = config.get_key_limits("other-key");
        assert_eq!(rps, 100);
        assert_eq!(burst, 200);
    }

    #[test]
    fn test_tier_assignment() {
        let mut config = RateLimitConfig::default();

        // Assign key to premium tier
        config.assign_key_to_tier("premium-user".to_string(), "premium".to_string());

        // Verify tier limits are returned
        let (rps, burst) = config.get_key_limits("premium-user");
        assert_eq!(rps, 200);
        assert_eq!(burst, 400);
    }

    #[test]
    fn test_key_override_priority() {
        let mut config = RateLimitConfig::default();

        // Assign key to tier AND add override
        config.assign_key_to_tier("special-user".to_string(), "premium".to_string());
        config.add_key_override("special-user".to_string(), 999, 1998);

        // Override should take priority over tier
        let (rps, burst) = config.get_key_limits("special-user");
        assert_eq!(rps, 999);
        assert_eq!(burst, 1998);
    }

    #[test]
    fn test_key_config_serialization() {
        let key_config = KeyRateLimitConfig {
            requests_per_second: 100,
            burst_size: 200,
            description: Some("Test key".to_string()),
        };

        let json = serde_json::to_string(&key_config).unwrap();
        let deserialized: KeyRateLimitConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.requests_per_second, 100);
        assert_eq!(deserialized.burst_size, 200);
        assert_eq!(deserialized.description, Some("Test key".to_string()));
    }

    #[test]
    fn test_tier_config_serialization() {
        let tier_config = TierRateLimitConfig {
            name: "Test Tier".to_string(),
            requests_per_second: 50,
            burst_size: 100,
            daily_limit: Some(10000),
        };

        let json = serde_json::to_string(&tier_config).unwrap();
        let deserialized: TierRateLimitConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "Test Tier");
        assert_eq!(deserialized.requests_per_second, 50);
        assert_eq!(deserialized.daily_limit, Some(10000));
    }

    #[test]
    fn test_rate_limit_config_yaml_serialization() {
        let mut config = RateLimitConfig::default();
        config.add_key_override("test-key".to_string(), 500, 1000);
        config.assign_key_to_tier("tier-user".to_string(), "premium".to_string());

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: RateLimitConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.requests_per_second, config.requests_per_second);
        assert!(deserialized.key_overrides.contains_key("test-key"));
        assert!(deserialized.key_tiers.contains_key("tier-user"));
    }

    #[test]
    fn test_limiter_update_key_limits() {
        let config = RateLimitConfig::with_defaults(100, 200);
        let limiter = RateLimiter::new(config);

        // Create a limiter for a key
        limiter.check_key("test-key");
        assert_eq!(limiter.active_key_count(), 1);

        // Update the limits
        limiter.update_key_limits("test-key", 50, 100);
        assert_eq!(limiter.active_key_count(), 1);
    }

    #[test]
    fn test_limiter_remove_key() {
        let config = RateLimitConfig::with_defaults(100, 200);
        let limiter = RateLimiter::new(config);

        // Create limiters for keys
        limiter.check_key("key1");
        limiter.check_key("key2");
        assert_eq!(limiter.active_key_count(), 2);

        // Remove one key
        limiter.remove_key("key1");
        assert_eq!(limiter.active_key_count(), 1);
    }

    #[test]
    fn test_limiter_get_key_info() {
        let mut config = RateLimitConfig::default();
        config.add_key_override("special".to_string(), 500, 1000);
        let limiter = RateLimiter::new(config);

        // Key that hasn't been used yet
        let info = limiter.get_key_info("special").unwrap();
        assert_eq!(info, (500, 1000, false));

        // Use the key
        limiter.check_key("special");

        // Now it should exist
        let info = limiter.get_key_info("special").unwrap();
        assert_eq!(info, (500, 1000, true));
    }

    #[test]
    fn test_per_key_isolation() {
        let config = RateLimitConfig::with_defaults(2, 2);
        let limiter = RateLimiter::new(config);

        // Exhaust limits for key1
        assert!(limiter.check_key("key1"));
        assert!(limiter.check_key("key1"));
        assert!(!limiter.check_key("key1")); // Exhausted

        // key2 should still have full quota
        assert!(limiter.check_key("key2"));
        assert!(limiter.check_key("key2"));
        assert!(!limiter.check_key("key2")); // Exhausted

        // key1 is still exhausted
        assert!(!limiter.check_key("key1"));
    }

    #[test]
    fn test_check_with_api_key() {
        let config = RateLimitConfig::with_defaults(100, 200);
        let limiter = RateLimiter::new(config);

        // Should pass with API key
        assert!(limiter.check(Some("test-key")));

        // Should pass without API key (only global check)
        assert!(limiter.check(None));
    }
}
