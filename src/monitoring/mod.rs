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
//! ```rust,no_run
//! use vectorizer::monitoring::Metrics;
//!
//! // Initialize monitoring
//! let metrics = Metrics::new();
//!
//! // Record a search operation
//! metrics.search_requests_total.inc();
//! let timer = metrics.search_latency_seconds.start_timer();
//! // ... perform search ...
//! timer.observe_duration();
//! ```

pub mod metrics;
pub mod registry;

pub use metrics::Metrics;

use anyhow::Result;
use prometheus::{Encoder, TextEncoder};

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
}

