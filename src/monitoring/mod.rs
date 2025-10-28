//! Monitoring and Observability Module
//!
//! This module provides comprehensive monitoring capabilities including:
//! - Prometheus metrics export
//! - OpenTelemetry distributed tracing
//! - Structured logging with correlation IDs
//!
//! # Architecture
//!
//! The monitoring system follows a layered approach:
//! 1. **Metrics Collection**: Low-overhead instrumentation using Prometheus
//! 2. **Metrics Registry**: Centralized registry for all metrics
//! 3. **HTTP Export**: `/metrics` endpoint for Prometheus scraping
//! 4. **Distributed Tracing**: OpenTelemetry for request tracing
//! 5. **Structured Logging**: JSON logs with correlation IDs
//!
//! # Usage
//!
//! ```rust,ignore
//! use vectorizer::monitoring::Metrics;
//!
//! // Initialize monitoring
//! let metrics = Metrics::new();
//!
//! // Record metrics (example - actual API may differ)
//! // See metrics module for correct usage
//! ```

pub mod correlation;
pub mod metrics;
pub mod performance;
pub mod registry;
pub mod system_collector;
pub mod telemetry;

use anyhow::Result;
pub use correlation::{
    CORRELATION_ID_HEADER, correlation_middleware, current_correlation_id, generate_correlation_id,
};
pub use metrics::Metrics;
use prometheus::{Encoder, TextEncoder};
pub use system_collector::{SystemCollector, SystemCollectorConfig};

/// Initialize the global monitoring system
pub fn init() -> Result<()> {
    tracing::info!("Initializing monitoring system");

    // Initialize Prometheus registry
    registry::init()?;

    tracing::info!("Monitoring system initialized successfully");
    Ok(())
}

/// Export Prometheus metrics in text format
pub fn export_metrics() -> Result<String> {
    let registry = registry::get_registry();
    let registry_guard = registry.read();
    let metric_families = registry_guard.gather();

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let result = init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_metrics() {
        init().unwrap();
        let result = export_metrics();
        assert!(result.is_ok());

        let metrics_text = result.unwrap();
        assert!(!metrics_text.is_empty());
    }

    #[test]
    fn test_init_monitoring() {
        let result = init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_metrics_format() {
        init().unwrap();
        let result = export_metrics();
        assert!(result.is_ok());

        let metrics_text = result.unwrap();
        // Check that it contains some expected Prometheus format elements
        assert!(metrics_text.contains("# HELP"));
        assert!(metrics_text.contains("# TYPE"));
    }

    #[test]
    fn test_metrics_initialization() {
        // Test that we can initialize metrics multiple times
        let result1 = init();
        assert!(result1.is_ok());

        let result2 = init();
        assert!(result2.is_ok());
    }
}
