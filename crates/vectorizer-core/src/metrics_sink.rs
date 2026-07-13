//! `MetricsSink` — the trait that lets foundation-layer modules record
//! operational metrics without a compile-time dependency on the
//! service-layer `monitoring` module.
//!
//! Before this trait existed, `db/ttl_reaper.rs`, `cache/query_cache.rs`,
//! `hub/quota.rs`, and `auth/mod.rs` each reached directly into
//! `crate::monitoring` (the global Prometheus `METRICS` statics, or the
//! `ApiKeyUsageRecorder` ring buffer) to emit their metrics. That created
//! an upward dependency from foundation modules onto the service layer,
//! blocking the umbrella-crate split (2026-07-11 improvement analysis,
//! §1.1; phase41 §1).
//!
//! Every method has a no-op default body so a concrete sink only needs
//! to override the handful of operations it actually cares about —
//! `PrometheusMetricsSink` (in `vectorizer::monitoring`) overrides the
//! Prometheus-backed ones, while [`NoopMetricsSink`] overrides nothing
//! and serves as the default for tests and constructors that don't
//! inject a sink.

/// Abstraction over metrics emission for the four call sites cataloged
/// in the 2026-07-11 improvement analysis (§1.1): the TTL reaper, the
/// query cache, HiveHub quota checks, and per-API-key usage tracking.
///
/// Implementations MUST be cheap to call on the hot path: no I/O, and
/// no locking beyond whatever the underlying metrics backend already
/// does internally (e.g. a `prometheus::CounterVec`'s own atomics).
///
/// Requires `Debug` so consumers that store `Arc<dyn MetricsSink>`
/// (e.g. `QuotaManager`, `AuthManager`) can keep their own
/// `#[derive(Debug)]` without a manual impl.
pub trait MetricsSink: Send + Sync + std::fmt::Debug {
    /// Record the wake-up lag (in seconds) of a TTL reaper sweep for
    /// `collection` — how far past its scheduled interval the sweep
    /// actually started.
    fn ttl_reaper_lag_seconds(&self, _collection: &str, _lag_seconds: f64) {}

    /// Record that a TTL reaper sweep completed for `collection`.
    fn ttl_reaper_scan_completed(&self, _collection: &str) {}

    /// Record that `count` vectors were expired by the TTL reaper for
    /// `collection`.
    fn ttl_vectors_expired(&self, _collection: &str, _count: f64) {}

    /// Record a cache lookup outcome for `cache_type` (e.g. `"query"`).
    /// `hit` is `true` for a cache hit, `false` for a miss (including
    /// an expired entry treated as a miss).
    fn cache_request(&self, _cache_type: &str, _hit: bool) {}

    /// Record a HiveHub quota check outcome for `tenant_id`/`quota_type`.
    /// `allowed` is `false` when the quota was exceeded.
    fn hub_quota_check(&self, _tenant_id: &str, _quota_type: &str, _allowed: bool) {}

    /// Record the current usage gauge value for `tenant_id`/`quota_type`.
    fn hub_quota_usage(&self, _tenant_id: &str, _quota_type: &str, _usage: f64) {}

    /// Record the latency (in seconds) of a HiveHub quota check.
    fn hub_quota_check_latency(&self, _seconds: f64) {}

    /// Record one successful API key validation, for the per-key
    /// per-day usage tracker backing the dashboard sparkline and the
    /// `GET /auth/keys/{id}/usage` endpoint.
    fn api_key_validated(&self, _key_id: &str) {}
}

/// A [`MetricsSink`] that discards every call. The default for
/// consumers constructed without an explicit sink (unit tests,
/// programmatic embeddings that don't need instrumentation).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopMetricsSink;

impl MetricsSink for NoopMetricsSink {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn noop_sink_accepts_all_calls_without_panicking() {
        let sink: Arc<dyn MetricsSink> = Arc::new(NoopMetricsSink);
        sink.ttl_reaper_lag_seconds("test", 1.0);
        sink.ttl_reaper_scan_completed("test");
        sink.ttl_vectors_expired("test", 3.0);
        sink.cache_request("query", true);
        sink.hub_quota_check("tenant", "storage", false);
        sink.hub_quota_usage("tenant", "storage", 42.0);
        sink.hub_quota_check_latency(0.01);
        sink.api_key_validated("key-1");
    }
}
