//! GRPC configuration structures

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// GRPC server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub keep_alive_timeout: u64,
    pub max_message_size: usize,
    pub enable_reflection: bool,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 15003,
            max_connections: 100,
            keep_alive_timeout: 30,
            max_message_size: 4 * 1024 * 1024, // 4MB
            enable_reflection: true,
        }
    }
}

/// GRPC client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcClientConfig {
    pub server_url: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub keep_alive_interval: u64,
    pub max_receive_message_length: usize,
    pub max_send_message_length: usize,
}

impl Default for GrpcClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:15003".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            retry_delay_ms: 1000,
            keep_alive_interval: 30,
            max_receive_message_length: 4 * 1024 * 1024, // 4MB
            max_send_message_length: 4 * 1024 * 1024,    // 4MB
        }
    }
}

/// GRPC service-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcServiceConfig {
    pub timeout_seconds: u64,
    pub max_results: Option<usize>,
}

impl Default for GrpcServiceConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 10,
            max_results: None,
        }
    }
}

/// GRPC services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcServicesConfig {
    pub search: GrpcServiceConfig,
    pub list_collections: GrpcServiceConfig,
    pub get_collection_info: GrpcServiceConfig,
    pub embed_text: GrpcServiceConfig,
    pub get_indexing_progress: GrpcServiceConfig,
    pub update_indexing_progress: GrpcServiceConfig,
}

impl Default for GrpcServicesConfig {
    fn default() -> Self {
        Self {
            search: GrpcServiceConfig {
                timeout_seconds: 10,
                max_results: Some(1000),
            },
            list_collections: GrpcServiceConfig {
                timeout_seconds: 5,
                max_results: None,
            },
            get_collection_info: GrpcServiceConfig {
                timeout_seconds: 5,
                max_results: None,
            },
            embed_text: GrpcServiceConfig {
                timeout_seconds: 15,
                max_results: None,
            },
            get_indexing_progress: GrpcServiceConfig {
                timeout_seconds: 5,
                max_results: None,
            },
            update_indexing_progress: GrpcServiceConfig {
                timeout_seconds: 5,
                max_results: None,
            },
        }
    }
}

/// Complete GRPC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcConfig {
    pub server: GrpcServerConfig,
    pub client: GrpcClientConfig,
    pub services: GrpcServicesConfig,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            server: GrpcServerConfig::default(),
            client: GrpcClientConfig::default(),
            services: GrpcServicesConfig::default(),
        }
    }
}

impl GrpcConfig {
    /// Load GRPC configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override with environment variables if present
        if let Ok(server_url) = std::env::var("VECTORIZER_GRPC_URL") {
            config.client.server_url = server_url;
        }

        if let Ok(host) = std::env::var("VECTORIZER_GRPC_HOST") {
            config.server.host = host;
        }

        if let Ok(port) = std::env::var("VECTORIZER_GRPC_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.server.port = port_num;
            }
        }

        config
    }

    /// Get GRPC server address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get client timeout duration
    pub fn client_timeout(&self) -> Duration {
        Duration::from_secs(self.client.timeout_seconds)
    }

    /// Get keep-alive interval duration
    pub fn keep_alive_interval(&self) -> Duration {
        Duration::from_secs(self.client.keep_alive_interval)
    }

    /// Get retry delay duration
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.client.retry_delay_ms)
    }
}
