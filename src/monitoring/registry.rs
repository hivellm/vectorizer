//! Prometheus Registry Management
//!
//! This module manages the global Prometheus registry and provides
//! utilities for metric registration and collection.

use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use prometheus::Registry;

use super::metrics::METRICS;

/// Global Prometheus registry
static REGISTRY: Lazy<Arc<RwLock<Registry>>> = Lazy::new(|| Arc::new(RwLock::new(Registry::new())));

/// Initialize the metrics registry (idempotent)
pub fn init() -> Result<(), prometheus::Error> {
    use once_cell::sync::OnceCell;

    static INIT: OnceCell<()> = OnceCell::new();

    INIT.get_or_try_init(|| {
        let registry = REGISTRY.write();

        // Register all metrics
        METRICS.register(&registry)?;

        tracing::info!("Prometheus metrics registry initialized");
        Ok::<(), prometheus::Error>(())
    })?;

    Ok(())
}

/// Get a reference to the global registry
pub fn get_registry() -> Arc<RwLock<Registry>> {
    Arc::clone(&REGISTRY)
}

#[cfg(test)]
mod tests {
    use prometheus::Encoder;

    use super::*;

    #[test]
    fn test_registry_init_idempotent() {
        // Multiple calls should not fail
        init().unwrap();
        init().unwrap();
        init().unwrap();
    }

    #[test]
    fn test_registry_access() {
        // Ensure initialization
        init().unwrap();

        let registry = get_registry();
        let registry_guard = registry.read();

        let metric_families = registry_guard.gather();
        assert!(
            !metric_families.is_empty(),
            "Registry should contain metrics"
        );
    }

    #[test]
    fn test_metrics_export() {
        // Ensure initialization
        init().unwrap();

        // Record some metrics
        METRICS
            .search_requests_total
            .with_label_values(&["test_export", "basic", "success"])
            .inc();
        METRICS.vectors_total.set(42.0);

        // Export metrics
        let registry = get_registry();
        let registry_guard = registry.read();
        let metric_families = registry_guard.gather();

        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(
            output.contains("vectorizer_search_requests_total"),
            "Should contain search requests metric"
        );
        assert!(
            output.contains("vectorizer_vectors_total"),
            "Should contain vectors total metric"
        );
    }
}
