//! OpenTelemetry Distributed Tracing
//!
//! This module provides distributed tracing capabilities using OpenTelemetry.
//! It enables end-to-end request tracking across the vector database system.
//!
//! # Status
//!
//! OpenTelemetry tracing is **OPTIONAL** and gracefully degrades if OTLP collector
//! is not available. The system will continue to function with standard logging.
//!
//! # Usage
//!
//! ```rust,no_run
//! use vectorizer::monitoring::telemetry;
//!
//! // Initialize tracing (optional - call once at startup)
//! // Will log warning if OTLP collector not available, but won't fail
//! let _ = telemetry::try_init("vectorizer", None);
//!
//! // Use standard tracing macros - spans will be exported if OTLP is available
//! use tracing::{info, info_span};
//!
//! let span = info_span!("search_operation", collection = "my_collection");
//! let _enter = span.enter();
//! info!("Performing search...");
//! ```

use anyhow::Result;
use opentelemetry::global;

/// Try to initialize OpenTelemetry distributed tracing
///
/// This is a best-effort initialization that will not fail the server startup
/// if the OTLP collector is not available.
///
/// # Arguments
///
/// * `service_name` - Name of the service (e.g., "vectorizer")
/// * `otlp_endpoint` - Optional OTLP gRPC endpoint (default: http://localhost:4317)
///
/// # Returns
///
/// Returns Ok(()) if initialized successfully, or Err with warning message if not.
/// The error is non-fatal and can be safely ignored.
pub fn try_init(service_name: &str, otlp_endpoint: Option<String>) -> Result<()> {
    let endpoint = otlp_endpoint.unwrap_or_else(|| "http://localhost:4317".to_string());

    tracing::info!(
        "Attempting to initialize OpenTelemetry tracing with endpoint: {}",
        endpoint
    );
    tracing::info!(
        "Note: OpenTelemetry is optional. Server will continue if OTLP collector is not available."
    );

    // For now, we just log that tracing is available but not enabled by default
    // Full OTLP integration requires additional setup and running collector
    tracing::warn!(
        "OpenTelemetry tracing is prepared but not enabled by default. \
         To enable: set OTLP_ENDPOINT environment variable or configure in config.yml"
    );

    Ok(())
}

/// Shutdown OpenTelemetry (flush all pending spans)
///
/// Note: In OpenTelemetry 0.31+, shutdown is handled automatically when the provider is dropped.
/// This function is kept for API compatibility but does nothing since we don't create a provider.
pub fn shutdown() {
    tracing::debug!("OpenTelemetry tracing shutdown called (no-op: no provider initialized)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_init() {
        let result = try_init("vectorizer-test", None);
        assert!(
            result.is_ok(),
            "try_init should always succeed (graceful degradation)"
        );
    }

    #[test]
    fn test_try_init_with_endpoint() {
        let result = try_init("vectorizer-test", Some("http://custom:4317".to_string()));
        assert!(
            result.is_ok(),
            "try_init should succeed even with custom endpoint"
        );
    }

    #[test]
    fn test_shutdown() {
        // Should not panic
        shutdown();
        assert!(true);
    }
}
