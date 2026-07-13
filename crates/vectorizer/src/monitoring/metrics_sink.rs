//! Production [`MetricsSink`] implementation — delegates to the global
//! Prometheus `METRICS` statics and (optionally) an
//! [`ApiKeyUsageRecorder`] ring buffer.
//!
//! This is the "provider" half of the phase41 §1 `MetricsSink`
//! inversion: `db/ttl_reaper.rs`, `cache/query_cache.rs`,
//! `hub/quota.rs`, and `auth/mod.rs` depend only on the trait
//! (defined in `vectorizer_core::metrics_sink`); this module supplies
//! the real implementation that bootstrap/wiring sites inject.

use std::sync::Arc;

use vectorizer_core::metrics_sink::MetricsSink;

use super::api_key_usage::ApiKeyUsageRecorder;
use super::metrics::METRICS;

/// [`MetricsSink`] backed by the global Prometheus `METRICS` registry.
///
/// The `api_key_usage` field is `None` for consumers that never call
/// [`MetricsSink::api_key_validated`] (the TTL reaper, the query
/// cache, and HiveHub quota checks all leave it unset); `AuthManager`
/// wiring supplies the same [`ApiKeyUsageRecorder`] instance the
/// server shares with its `GET /auth/keys/{id}/usage` handler so
/// writes and reads observe the same ring buffer.
#[derive(Debug, Clone, Default)]
pub struct PrometheusMetricsSink {
    api_key_usage: Option<Arc<ApiKeyUsageRecorder>>,
}

impl PrometheusMetricsSink {
    /// Create a sink with no API-key usage tracking. Used by
    /// consumers (TTL reaper, query cache, HiveHub quota) that only
    /// emit Prometheus counters/gauges/histograms.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a sink that also records API key validations into
    /// `recorder`. Used by `AuthManager` wiring so the ring buffer it
    /// writes to is the same one the server reads snapshots from.
    #[must_use]
    pub fn with_api_key_usage(recorder: Arc<ApiKeyUsageRecorder>) -> Self {
        Self {
            api_key_usage: Some(recorder),
        }
    }
}

impl MetricsSink for PrometheusMetricsSink {
    fn ttl_reaper_lag_seconds(&self, collection: &str, lag_seconds: f64) {
        METRICS
            .ttl_reaper_lag_secs
            .with_label_values(&[collection])
            .set(lag_seconds);
    }

    fn ttl_reaper_scan_completed(&self, collection: &str) {
        METRICS
            .ttl_reaper_scans_total
            .with_label_values(&[collection])
            .inc();
    }

    fn ttl_vectors_expired(&self, collection: &str, count: f64) {
        METRICS
            .ttl_vectors_expired_total
            .with_label_values(&[collection])
            .inc_by(count);
    }

    fn cache_request(&self, cache_type: &str, hit: bool) {
        let result = if hit { "hit" } else { "miss" };
        METRICS
            .cache_requests_total
            .with_label_values(&[cache_type, result])
            .inc();
    }

    fn hub_quota_check(&self, tenant_id: &str, quota_type: &str, allowed: bool) {
        let result_label = if allowed { "allowed" } else { "denied" };
        METRICS
            .hub_quota_checks_total
            .with_label_values(&[tenant_id, quota_type, result_label])
            .inc();
        if !allowed {
            METRICS
                .hub_quota_exceeded_total
                .with_label_values(&[tenant_id, quota_type])
                .inc();
        }
    }

    fn hub_quota_usage(&self, tenant_id: &str, quota_type: &str, usage: f64) {
        METRICS
            .hub_quota_usage
            .with_label_values(&[tenant_id, quota_type])
            .set(usage);
    }

    fn hub_quota_check_latency(&self, seconds: f64) {
        METRICS.hub_quota_check_latency_seconds.observe(seconds);
    }

    fn api_key_validated(&self, key_id: &str) {
        if let Some(recorder) = &self.api_key_usage {
            recorder.record(key_id);
        }
    }
}
