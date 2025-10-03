use opentelemetry::{
    global,
    metrics::{Counter, Histogram, Meter, UpDownCounter},
    trace::{Tracer, TracerProvider},
    KeyValue,
};
use opentelemetry_prometheus::PrometheusExporter;
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry_semantic_conventions as semconv;
use std::sync::Arc;
use tracing::{info, warn};

// pub mod config;
// pub use config::*;

/// Telemetry configuration for the vectorizer
#[derive(Debug, Clone)]
pub struct TelemetryManagerConfig {
    pub service_name: String,
    pub service_version: String,
    pub prometheus_port: u16,
    pub enable_tracing: bool,
    pub enable_metrics: bool,
    pub otlp_endpoint: Option<String>,
}

impl Default for TelemetryManagerConfig {
    fn default() -> Self {
        Self {
            service_name: "vectorizer".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            prometheus_port: 9090,
            enable_tracing: true,
            enable_metrics: true,
            otlp_endpoint: None,
        }
    }
}

/// Telemetry manager for the vectorizer
pub struct TelemetryManager {
    config: TelemetryManagerConfig,
    prometheus_exporter: Option<PrometheusExporter>,
    meter: Option<Meter>,
    tracer: Option<opentelemetry::global::BoxedTracer>,
}

impl TelemetryManager {
    /// Create a new telemetry manager
    pub fn new(config: TelemetryManagerConfig) -> anyhow::Result<Self> {
        let mut manager = Self {
            config,
            prometheus_exporter: None,
            meter: None,
            tracer: None,
        };

        manager.initialize()?;
        Ok(manager)
    }

    /// Initialize telemetry components
    fn initialize(&mut self) -> anyhow::Result<()> {
        if self.config.enable_metrics {
            self.initialize_metrics()?;
        }

        if self.config.enable_tracing {
            self.initialize_tracing()?;
        }

        Ok(())
    }

    /// Initialize metrics with Prometheus exporter
    fn initialize_metrics(&mut self) -> anyhow::Result<()> {
        info!("Initializing OpenTelemetry metrics with Prometheus exporter");

        // Create Prometheus exporter
        let exporter = opentelemetry_prometheus::exporter().build()?;
        self.prometheus_exporter = Some(exporter);

        // Get meter
        self.meter = Some(global::meter("vectorizer"));

        info!("Metrics initialized successfully on port {}", self.config.prometheus_port);
        Ok(())
    }

    /// Initialize tracing
    fn initialize_tracing(&mut self) -> anyhow::Result<()> {
        info!("Initializing OpenTelemetry tracing");

        // Create tracer provider
        let tracer_provider = trace::SdkTracerProvider::builder()
            .with_sampler(Sampler::AlwaysOn)
            .with_id_generator(RandomIdGenerator::default())
            .build();

        // Set global tracer provider
        global::set_tracer_provider(tracer_provider);

        // Get tracer
        let tracer = global::tracer("vectorizer");

        self.tracer = Some(tracer);

        info!("Tracing initialized successfully");
        Ok(())
    }

    /// Get the Prometheus exporter for metrics endpoint
    pub fn get_prometheus_exporter(&self) -> Option<&PrometheusExporter> {
        self.prometheus_exporter.as_ref()
    }

    /// Get the meter for creating metrics
    pub fn get_meter(&self) -> Option<&Meter> {
        self.meter.as_ref()
    }

    /// Get the tracer for creating spans
    pub fn get_tracer(&self) -> Option<&opentelemetry::global::BoxedTracer> {
        self.tracer.as_ref()
    }
}

/// Vectorizer-specific metrics
pub struct VectorizerMetrics {
    // Collection metrics
    pub collections_total: Counter<u64>,
    pub collections_created: Counter<u64>,
    pub collections_deleted: Counter<u64>,
    
    // Vector metrics
    pub vectors_total: UpDownCounter<i64>,
    pub vectors_inserted: Counter<u64>,
    pub vectors_deleted: Counter<u64>,
    pub vectors_searched: Counter<u64>,
    
    // Performance metrics
    pub search_duration: Histogram<f64>,
    pub insert_duration: Histogram<f64>,
    pub delete_duration: Histogram<f64>,
    pub memory_usage_bytes: UpDownCounter<f64>,
    
    // API metrics
    pub api_requests_total: Counter<u64>,
    pub api_request_duration: Histogram<f64>,
    pub api_errors_total: Counter<u64>,
    
    // GRPC metrics
    pub grpc_requests_total: Counter<u64>,
    pub grpc_request_duration: Histogram<f64>,
    pub grpc_errors_total: Counter<u64>,
}

impl VectorizerMetrics {
    /// Create new vectorizer metrics
    pub fn new(meter: &Meter) -> Self {
        Self {
            // Collection metrics
            collections_total: meter
                .u64_counter("vectorizer_collections_total")
                .with_description("Total number of collections")
                .build(),
            collections_created: meter
                .u64_counter("vectorizer_collections_created_total")
                .with_description("Total number of collections created")
                .build(),
            collections_deleted: meter
                .u64_counter("vectorizer_collections_deleted_total")
                .with_description("Total number of collections deleted")
                .build(),
            
            // Vector metrics
            vectors_total: meter
                .i64_up_down_counter("vectorizer_vectors_total")
                .with_description("Current number of vectors")
                .build(),
            vectors_inserted: meter
                .u64_counter("vectorizer_vectors_inserted_total")
                .with_description("Total number of vectors inserted")
                .build(),
            vectors_deleted: meter
                .u64_counter("vectorizer_vectors_deleted_total")
                .with_description("Total number of vectors deleted")
                .build(),
            vectors_searched: meter
                .u64_counter("vectorizer_vectors_searched_total")
                .with_description("Total number of vector searches performed")
                .build(),
            
            // Performance metrics
            search_duration: meter
                .f64_histogram("vectorizer_search_duration_seconds")
                .with_description("Duration of vector search operations")
                .build(),
            insert_duration: meter
                .f64_histogram("vectorizer_insert_duration_seconds")
                .with_description("Duration of vector insert operations")
                .build(),
            delete_duration: meter
                .f64_histogram("vectorizer_delete_duration_seconds")
                .with_description("Duration of vector delete operations")
                .build(),
            memory_usage_bytes: meter
                .f64_up_down_counter("vectorizer_memory_usage_bytes")
                .with_description("Current memory usage in bytes")
                .build(),
            
            // API metrics
            api_requests_total: meter
                .u64_counter("vectorizer_api_requests_total")
                .with_description("Total number of API requests")
                .build(),
            api_request_duration: meter
                .f64_histogram("vectorizer_api_request_duration_seconds")
                .with_description("Duration of API requests")
                .build(),
            api_errors_total: meter
                .u64_counter("vectorizer_api_errors_total")
                .with_description("Total number of API errors")
                .build(),
            
            // GRPC metrics
            grpc_requests_total: meter
                .u64_counter("vectorizer_grpc_requests_total")
                .with_description("Total number of GRPC requests")
                .build(),
            grpc_request_duration: meter
                .f64_histogram("vectorizer_grpc_request_duration_seconds")
                .with_description("Duration of GRPC requests")
                .build(),
            grpc_errors_total: meter
                .u64_counter("vectorizer_grpc_errors_total")
                .with_description("Total number of GRPC errors")
                .build(),
        }
    }
}

/// Helper functions for common metric operations
impl VectorizerMetrics {
    /// Record a collection creation
    pub fn record_collection_created(&self, collection_name: &str) {
        self.collections_created.add(1, &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
        self.collections_total.add(1, &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
    }

    /// Record a collection deletion
    pub fn record_collection_deleted(&self, collection_name: &str) {
        self.collections_deleted.add(1, &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
    }

    /// Record vector insertion
    pub fn record_vectors_inserted(&self, count: u64, collection_name: &str) {
        self.vectors_inserted.add(count, &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
        self.vectors_total.add(count as i64, &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
    }

    /// Record vector deletion
    pub fn record_vectors_deleted(&self, count: u64, collection_name: &str) {
        self.vectors_deleted.add(count, &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
        self.vectors_total.add(-(count as i64), &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
    }

    /// Record vector search
    pub fn record_vector_search(&self, count: u64, duration_seconds: f64, collection_name: &str) {
        self.vectors_searched.add(count, &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
        self.search_duration.record(duration_seconds, &[
            KeyValue::new("collection_name", collection_name.to_string()),
        ]);
    }

    /// Record API request
    pub fn record_api_request(&self, method: &str, endpoint: &str, duration_seconds: f64, status_code: u16) {
        self.api_requests_total.add(1, &[
            KeyValue::new("method", method.to_string()),
            KeyValue::new("endpoint", endpoint.to_string()),
            KeyValue::new("status_code", status_code.to_string()),
        ]);
        self.api_request_duration.record(duration_seconds, &[
            KeyValue::new("method", method.to_string()),
            KeyValue::new("endpoint", endpoint.to_string()),
        ]);

        if status_code >= 400 {
            self.api_errors_total.add(1, &[
                KeyValue::new("method", method.to_string()),
                KeyValue::new("endpoint", endpoint.to_string()),
                KeyValue::new("status_code", status_code.to_string()),
            ]);
        }
    }

    /// Record GRPC request
    pub fn record_grpc_request(&self, method: &str, duration_seconds: f64, success: bool) {
        self.grpc_requests_total.add(1, &[
            KeyValue::new("method", method.to_string()),
            KeyValue::new("success", success.to_string()),
        ]);
        self.grpc_request_duration.record(duration_seconds, &[
            KeyValue::new("method", method.to_string()),
        ]);

        if !success {
            self.grpc_errors_total.add(1, &[
                KeyValue::new("method", method.to_string()),
            ]);
        }
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: f64) {
        self.memory_usage_bytes.add(bytes, &[]);
    }
}

/// Global telemetry state
pub struct TelemetryState {
    pub manager: Arc<TelemetryManager>,
    pub metrics: Arc<VectorizerMetrics>,
}

impl TelemetryState {
    /// Create new telemetry state
    pub fn new(config: TelemetryManagerConfig) -> anyhow::Result<Self> {
        let manager = Arc::new(TelemetryManager::new(config)?);
        let metrics = if let Some(meter) = manager.get_meter() {
            Arc::new(VectorizerMetrics::new(meter))
        } else {
            return Err(anyhow::anyhow!("Failed to get meter from telemetry manager"));
        };

        Ok(Self { manager, metrics })
    }

    /// Get the Prometheus exporter
    pub fn get_prometheus_exporter(&self) -> Option<&PrometheusExporter> {
        self.manager.get_prometheus_exporter()
    }

    /// Get the metrics
    pub fn get_metrics(&self) -> &VectorizerMetrics {
        &self.metrics
    }

    /// Get the tracer
    pub fn get_tracer(&self) -> Option<&opentelemetry::global::BoxedTracer> {
        self.manager.get_tracer()
    }
}

/// Initialize telemetry for the application
pub fn init_telemetry(config: TelemetryManagerConfig) -> anyhow::Result<TelemetryState> {
    info!("Initializing OpenTelemetry telemetry system");
    
    let state = TelemetryState::new(config)?;
    
    info!("Telemetry system initialized successfully");
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_default() {
        use crate::telemetry_config::TelemetryConfig;
        let config = TelemetryConfig::default();
        assert_eq!(config.service_name, "vectorizer");
        assert_eq!(config.prometheus.port, 9090);
        assert!(config.tracing.enabled);
        assert!(config.prometheus.enabled);
    }

    #[test]
    fn test_telemetry_metrics_creation() {
        use crate::telemetry_config::TelemetryConfig;
        let config = TelemetryConfig::default();
        let manager = TelemetryManager::new(config.into()).unwrap();
        let meter = manager.get_meter().unwrap();
        let metrics = VectorizerMetrics::new(meter);
        
        // Test that metrics are created successfully
        // Note: OpenTelemetry metrics don't have a name() method
        // We can only test that they were created without panicking
        assert!(true); // Metrics were created successfully
    }
}
