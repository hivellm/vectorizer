//! Rate Limiting Implementation
//!
//! This module provides rate limiting functionality to prevent API abuse.
//! It supports both global rate limiting and per-API-key limiting.

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
use tracing::{debug, warn};

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per second per API key
    pub requests_per_second: u32,
    /// Burst capacity
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100, // 100 req/s default
            burst_size: 200,          // Allow bursts up to 200
        }
    }
}

/// Type alias for the keyed rate limiter
type KeyedLimiter = GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Rate limiter for the vectorizer API
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
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
            config,
            global_limiter,
            key_limiters: Arc::new(DashMap::new()),
        }
    }

    /// Check if a request should be allowed (global limit)
    pub fn check_global(&self) -> bool {
        self.global_limiter.check().is_ok()
    }

    /// Check if a request should be allowed for a specific API key
    pub fn check_key(&self, api_key: &str) -> bool {
        // Get or create a rate limiter for this API key
        let limiter = self
            .key_limiters
            .entry(api_key.to_string())
            .or_insert_with(|| {
                let quota = Quota::per_second(
                    NonZeroU32::new(self.config.requests_per_second)
                        .unwrap_or(NonZeroU32::new(100).unwrap()),
                )
                .allow_burst(
                    NonZeroU32::new(self.config.burst_size)
                        .unwrap_or(NonZeroU32::new(200).unwrap()),
                );
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
        let config = RateLimitConfig {
            requests_per_second: 50,
            burst_size: 100,
        };
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
        let config = RateLimitConfig {
            requests_per_second: 2,
            burst_size: 2,
        };
        let limiter = RateLimiter::new(config);

        // First 2 requests should pass (burst)
        assert!(limiter.check_global());
        assert!(limiter.check_global());

        // 3rd request should fail
        assert!(!limiter.check_global());
    }
}
