//! Rate Limiting Implementation
//!
//! This module provides rate limiting functionality to prevent API abuse.
//! It supports both global rate limiting and per-API-key limiting.

<<<<<<< HEAD
=======
use std::collections::HashMap;
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
use std::num::NonZeroU32;
use std::sync::Arc;

use axum::extract::Request;
<<<<<<< HEAD
use axum::http::StatusCode;
=======
use axum::http::{HeaderMap, HeaderValue, StatusCode};
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter as GovernorRateLimiter};
<<<<<<< HEAD
=======
use tokio::sync::RwLock;
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44

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

/// Rate limiter for the vectorizer API
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    global_limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
<<<<<<< HEAD
=======
    per_key_limiters: Arc<
        RwLock<HashMap<String, Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    >,
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
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
<<<<<<< HEAD
=======
            per_key_limiters: Arc::new(RwLock::new(HashMap::new())),
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
        }
    }

    /// Check if a request should be allowed (global limit)
    pub fn check_global(&self) -> bool {
        self.global_limiter.check().is_ok()
    }
<<<<<<< HEAD
=======

    /// Get or create a per-key rate limiter
    async fn get_per_key_limiter(
        &self,
        api_key: &str,
    ) -> Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        // Check if limiter already exists
        {
            let limiters = self.per_key_limiters.read().await;
            if let Some(limiter) = limiters.get(api_key) {
                return limiter.clone();
            }
        }

        // Create new limiter for this API key
        let quota = Quota::per_second(
            NonZeroU32::new(self.config.requests_per_second)
                .unwrap_or(NonZeroU32::new(100).unwrap()),
        )
        .allow_burst(
            NonZeroU32::new(self.config.burst_size).unwrap_or(NonZeroU32::new(200).unwrap()),
        );

        let new_limiter = Arc::new(GovernorRateLimiter::direct(quota));

        // Store the new limiter
        {
            let mut limiters = self.per_key_limiters.write().await;
            limiters.insert(api_key.to_string(), new_limiter.clone());
        }

        new_limiter
    }

    /// Check if a request is allowed for a specific API key
    pub async fn check_per_key_rate_limit(&self, api_key: &str) -> bool {
        let limiter = self.get_per_key_limiter(api_key).await;
        limiter.check().is_ok()
    }
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(req: Request, next: Next) -> Response {
    use crate::monitoring::metrics::METRICS;

<<<<<<< HEAD
    // Get rate limiter from request extensions
    // For now, we'll use a simple global rate limiter
    // In production, this should be per-API-key
=======
    // Extract API key from request headers
    let api_key = extract_api_key(&req);

    // Get rate limiter from request extensions or create a default one
    let rate_limiter = req
        .extensions()
        .get::<Arc<RateLimiter>>()
        .cloned()
        .unwrap_or_else(|| Arc::new(RateLimiter::default()));

    // Check rate limits
    let is_allowed = if let Some(key) = api_key {
        // Per-key rate limiting
        rate_limiter.check_per_key_rate_limit(&key).await
    } else {
        // Global rate limiting for requests without API key
        rate_limiter.check_global()
    };

    if !is_allowed {
        // Rate limit exceeded
        METRICS
            .api_errors_total
            .with_label_values(&["rate_limit", "exceeded", "429"])
            .inc();

        return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
    }
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44

    // Record the request attempt
    METRICS
        .api_errors_total
        .with_label_values(&["rate_limit", "check", "200"])
        .inc();

<<<<<<< HEAD
    // For MVP, we're not enforcing rate limits yet - just tracking
    // TODO: Extract API key and apply per-key rate limiting

    next.run(req).await
}

=======
    next.run(req).await
}

/// Extract API key from request headers
fn extract_api_key(req: &Request) -> Option<String> {
    let headers = req.headers();

    // Check for API key in Authorization header (Bearer token)
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    // Check for API key in X-API-Key header
    if let Some(api_key_header) = headers.get("X-API-Key") {
        if let Ok(api_key_str) = api_key_header.to_str() {
            return Some(api_key_str.to_string());
        }
    }

    None
}

>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
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
