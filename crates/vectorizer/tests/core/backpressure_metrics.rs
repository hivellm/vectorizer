//! Integration tests for the backpressure metrics surface
//! (issue #263, phase9 §4).
//!
//! Verifies that the canonical Prometheus metric names are registered,
//! exportable via `vectorizer::monitoring::export_metrics`, and react
//! to admission events. The actual `/metrics` HTTP handler that wires
//! these gauges is exercised end-to-end by
//! `crates/vectorizer-server/tests/backpressure_metrics_endpoint.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::monitoring::export_metrics;
use vectorizer::monitoring::metrics::METRICS;

#[test]
fn all_backpressure_metrics_are_registered_and_exportable() {
    // The global registry is populated by `monitoring::init`. Call it
    // here so this test can run in isolation without depending on the
    // server bootstrap to wire it up.
    vectorizer::monitoring::init().expect("monitoring::init must succeed");

    // Touch every metric exactly once so prometheus knows the family
    // exists even before any real traffic. Without this, a metric
    // with no observed labels would not appear in `export_metrics`
    // output, which makes adoption regressions silent.
    METRICS
        .upsert_queue_depth
        .with_label_values(&["__bootstrap__"])
        .set(0.0);
    METRICS
        .upsert_in_flight
        .with_label_values(&["__bootstrap__"])
        .set(0.0);
    METRICS.vocab_build_permits_available.set(0.0);
    METRICS
        .upsert_rejected_total
        .with_label_values(&["__bootstrap__"])
        .inc_by(0.0);
    METRICS
        .bm25_empty_vocab_fallback_total
        .with_label_values(&["__bootstrap__"])
        .inc_by(0.0);

    let exported = export_metrics().expect("metrics export must succeed");

    for name in [
        "vectorizer_upsert_queue_depth",
        "vectorizer_upsert_in_flight",
        "vectorizer_vocab_build_permits_available",
        "vectorizer_upsert_rejected_total",
        "vectorizer_bm25_empty_vocab_fallback_total",
    ] {
        assert!(
            exported.contains(name),
            "metric `{name}` must appear in /metrics export — full output:\n{exported}",
        );
    }
}

#[test]
fn rejected_counter_increments_on_explicit_bump() {
    let before = METRICS
        .upsert_rejected_total
        .with_label_values(&["queue_full"])
        .get();

    METRICS
        .upsert_rejected_total
        .with_label_values(&["queue_full"])
        .inc();

    let after = METRICS
        .upsert_rejected_total
        .with_label_values(&["queue_full"])
        .get();

    assert!(
        after > before,
        "queue_full counter must monotonically increase (before={before}, after={after})",
    );
}

#[test]
fn fallback_counter_increments_on_explicit_bump() {
    let before = METRICS
        .bm25_empty_vocab_fallback_total
        .with_label_values(&["sample_coll"])
        .get();

    METRICS
        .bm25_empty_vocab_fallback_total
        .with_label_values(&["sample_coll"])
        .inc();

    let after = METRICS
        .bm25_empty_vocab_fallback_total
        .with_label_values(&["sample_coll"])
        .get();

    assert!(
        after > before,
        "fallback counter must monotonically increase per collection",
    );
}
