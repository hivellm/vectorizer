use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Telemetry configuration from config.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub service_version: String,
    pub prometheus: PrometheusConfig,
    pub tracing: TracingConfig,
    pub otlp: OtlpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    pub enabled: bool,
    pub port: u16,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub protocol: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "vectorizer".to_string(),
            service_version: "0.21.0".to_string(),
            prometheus: PrometheusConfig::default(),
            tracing: TracingConfig::default(),
            otlp: OtlpConfig::default(),
        }
    }
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 9090,
            path: "/metrics".to_string(),
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
        }
    }
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: "http://localhost:4317".to_string(),
            protocol: "grpc".to_string(),
        }
    }
}

/// Convert from config file format to telemetry module format
impl From<TelemetryConfig> for crate::telemetry::TelemetryManagerConfig {
    fn from(config: TelemetryConfig) -> Self {
        Self {
            service_name: config.service_name,
            service_version: config.service_version,
            prometheus_port: config.prometheus.port,
            enable_tracing: config.tracing.enabled,
            enable_metrics: config.prometheus.enabled,
            otlp_endpoint: if config.otlp.enabled {
                Some(config.otlp.endpoint)
            } else {
                None
            },
        }
    }
}
