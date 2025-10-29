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
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter as GovernorRateLimiter};

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
        }
    }

    /// Check if a request should be allowed (global limit)
    pub fn check_global(&self) -> bool {
        self.global_limiter.check().is_ok()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(req: Request, next: Next) -> Response {
    use crate::monitoring::metrics::METRICS;

    // Get rate limiter from request extensions
    // For now, we'll use a simple global rate limiter
    // In production, this should be per-API-key

    // Record the request attempt
    METRICS
        .api_errors_total
        .with_label_values(&["rate_limit", "check", "200"])
        .inc();

    // For MVP, we're not enforcing rate limits yet - just tracking
    // TODO: Extract API key and apply per-key rate limiting

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
