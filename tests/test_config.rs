//! Test Configuration and Utilities
//!
//! This module provides configuration and utilities for running tests
//! across different environments and scenarios.

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Test configuration for different test scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Test timeout duration
    pub timeout: Duration,
    /// Number of concurrent operations for load tests
    pub concurrent_operations: usize,
    /// Number of vectors to insert in batch tests
    pub batch_size: usize,
    /// Vector dimension for tests
    pub vector_dimension: usize,
    /// Enable performance assertions
    pub enable_performance_checks: bool,
    /// Maximum response time for performance tests
    pub max_response_time_ms: u64,
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
    /// Test data directory
    pub test_data_dir: String,
    /// MCP server configuration for tests
    pub mcp_config: McpTestConfig,
}

/// MCP-specific test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTestConfig {
    /// MCP server host for tests
    pub host: String,
    /// MCP server port for tests
    pub port: u16,
    /// Connection timeout for MCP tests
    pub connection_timeout: Duration,
    /// Maximum concurrent MCP connections
    pub max_connections: usize,
    /// Enable MCP authentication in tests
    pub enable_auth: bool,
    /// Test API key for MCP
    pub test_api_key: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            concurrent_operations: 10,
            batch_size: 100,
            vector_dimension: 384,
            enable_performance_checks: true,
            max_response_time_ms: 1000,
            enable_detailed_logging: false,
            test_data_dir: "test_data".to_string(),
            mcp_config: McpTestConfig::default(),
        }
    }
}

impl Default for McpTestConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 15003,
            connection_timeout: Duration::from_secs(10),
            max_connections: 5,
            enable_auth: false,
            test_api_key: "test_api_key_123".to_string(),
        }
    }
}

impl TestConfig {
    /// Create test configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(timeout_secs) = env::var("TEST_TIMEOUT_SECS") {
            if let Ok(secs) = timeout_secs.parse::<u64>() {
                config.timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(concurrent) = env::var("TEST_CONCURRENT_OPERATIONS") {
            if let Ok(ops) = concurrent.parse::<usize>() {
                config.concurrent_operations = ops;
            }
        }

        if let Ok(batch_size) = env::var("TEST_BATCH_SIZE") {
            if let Ok(size) = batch_size.parse::<usize>() {
                config.batch_size = size;
            }
        }

        if let Ok(dimension) = env::var("TEST_VECTOR_DIMENSION") {
            if let Ok(dim) = dimension.parse::<usize>() {
                config.vector_dimension = dim;
            }
        }

        if let Ok(enable_perf) = env::var("TEST_ENABLE_PERFORMANCE_CHECKS") {
            config.enable_performance_checks = enable_perf.parse().unwrap_or(true);
        }

        if let Ok(max_time) = env::var("TEST_MAX_RESPONSE_TIME_MS") {
            if let Ok(ms) = max_time.parse::<u64>() {
                config.max_response_time_ms = ms;
            }
        }

        if let Ok(enable_logging) = env::var("TEST_ENABLE_DETAILED_LOGGING") {
            config.enable_detailed_logging = enable_logging.parse().unwrap_or(false);
        }

        if let Ok(data_dir) = env::var("TEST_DATA_DIR") {
            config.test_data_dir = data_dir;
        }

        // MCP configuration
        if let Ok(mcp_host) = env::var("TEST_MCP_HOST") {
            config.mcp_config.host = mcp_host;
        }

        if let Ok(mcp_port) = env::var("TEST_MCP_PORT") {
            if let Ok(port) = mcp_port.parse::<u16>() {
                config.mcp_config.port = port;
            }
        }

        if let Ok(mcp_timeout) = env::var("TEST_MCP_TIMEOUT_SECS") {
            if let Ok(secs) = mcp_timeout.parse::<u64>() {
                config.mcp_config.connection_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(max_conn) = env::var("TEST_MCP_MAX_CONNECTIONS") {
            if let Ok(conn) = max_conn.parse::<usize>() {
                config.mcp_config.max_connections = conn;
            }
        }

        if let Ok(enable_auth) = env::var("TEST_MCP_ENABLE_AUTH") {
            config.mcp_config.enable_auth = enable_auth.parse().unwrap_or(false);
        }

        if let Ok(api_key) = env::var("TEST_MCP_API_KEY") {
            config.mcp_config.test_api_key = api_key;
        }

        config
    }

    /// Get test configuration for CI environment
    pub fn ci_config() -> Self {
        let mut config = Self::default();
        config.timeout = Duration::from_secs(60); // Longer timeout for CI
        config.concurrent_operations = 5; // Fewer concurrent operations for CI
        config.batch_size = 50; // Smaller batch size for CI
        config.enable_performance_checks = false; // Disable performance checks in CI
        config.enable_detailed_logging = true; // Enable logging in CI
        config.mcp_config.connection_timeout = Duration::from_secs(30);
        config.mcp_config.max_connections = 3;
        config
    }

    /// Get test configuration for performance testing
    pub fn performance_config() -> Self {
        let mut config = Self::default();
        config.timeout = Duration::from_secs(120); // Longer timeout for performance tests
        config.concurrent_operations = 50; // More concurrent operations
        config.batch_size = 1000; // Larger batch size
        config.enable_performance_checks = true;
        config.max_response_time_ms = 500; // Stricter performance requirements
        config.enable_detailed_logging = true;
        config.mcp_config.max_connections = 20;
        config
    }

    /// Get test configuration for integration testing
    pub fn integration_config() -> Self {
        let mut config = Self::default();
        config.timeout = Duration::from_secs(180); // Long timeout for integration tests
        config.concurrent_operations = 20;
        config.batch_size = 200;
        config.enable_performance_checks = true;
        config.max_response_time_ms = 2000; // More lenient for integration tests
        config.enable_detailed_logging = true;
        config.mcp_config.connection_timeout = Duration::from_secs(60);
        config.mcp_config.max_connections = 10;
        config
    }
}

/// Test environment detection
pub struct TestEnvironment;

impl TestEnvironment {
    /// Check if running in CI environment
    pub fn is_ci() -> bool {
        env::var("CI").is_ok()
            || env::var("GITHUB_ACTIONS").is_ok()
            || env::var("GITLAB_CI").is_ok()
            || env::var("JENKINS_URL").is_ok()
    }

    /// Check if running in development environment
    pub fn is_development() -> bool {
        env::var("RUST_ENV")
            .map(|v| v == "development")
            .unwrap_or(false)
            || env::var("ENVIRONMENT")
                .map(|v| v == "development")
                .unwrap_or(false)
    }

    /// Check if running in test environment
    pub fn is_test() -> bool {
        env::var("RUST_ENV").map(|v| v == "test").unwrap_or(false)
            || env::var("ENVIRONMENT")
                .map(|v| v == "test")
                .unwrap_or(false)
    }

    /// Get current test environment
    pub fn current() -> String {
        if Self::is_ci() {
            "ci".to_string()
        } else if Self::is_development() {
            "development".to_string()
        } else if Self::is_test() {
            "test".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

/// Test data generation utilities
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate random vector data
    pub fn generate_vector(dimension: usize) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect()
    }

    /// Generate test collection data
    pub fn generate_collection_data(name: &str, dimension: usize) -> serde_json::Value {
        serde_json::json!({
            "name": name,
            "dimension": dimension,
            "metric": "cosine"
        })
    }

    /// Generate test vector data
    pub fn generate_vector_data(
        id: &str,
        dimension: usize,
        payload: Option<serde_json::Value>,
    ) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "data": Self::generate_vector(dimension),
            "payload": payload.unwrap_or(serde_json::json!({"title": format!("Test Vector {}", id)}))
        })
    }

    /// Generate batch vector data
    pub fn generate_batch_vector_data(
        count: usize,
        dimension: usize,
        prefix: &str,
    ) -> serde_json::Value {
        let vectors: Vec<serde_json::Value> = (0..count)
            .map(|i| {
                Self::generate_vector_data(
                    &format!("{}_vec_{}", prefix, i),
                    dimension,
                    Some(serde_json::json!({
                        "title": format!("Test Vector {}", i),
                        "index": i,
                        "content": format!("This is test content for vector {}", i)
                    })),
                )
            })
            .collect();

        serde_json::json!({
            "vectors": vectors
        })
    }

    /// Generate search query data
    pub fn generate_search_data(query: &str, limit: usize) -> serde_json::Value {
        serde_json::json!({
            "query": query,
            "limit": limit
        })
    }
}

/// Test assertion utilities
pub struct TestAssertions;

impl TestAssertions {
    /// Assert response time is within acceptable range
    pub fn assert_response_time(duration: Duration, max_ms: u64) {
        let duration_ms = duration.as_millis() as u64;
        assert!(
            duration_ms <= max_ms,
            "Response time {}ms exceeds maximum {}ms",
            duration_ms,
            max_ms
        );
    }

    /// Assert API response is successful
    pub fn assert_success_status(status: axum::http::StatusCode) {
        assert!(
            status.is_success(),
            "Expected success status, got {}",
            status
        );
    }

    /// Assert API response is client error
    pub fn assert_client_error_status(status: axum::http::StatusCode) {
        assert!(
            status.is_client_error(),
            "Expected client error status, got {}",
            status
        );
    }

    /// Assert API response is server error
    pub fn assert_server_error_status(status: axum::http::StatusCode) {
        assert!(
            status.is_server_error(),
            "Expected server error status, got {}",
            status
        );
    }

    /// Assert JSON response structure
    pub fn assert_json_structure(response: &serde_json::Value, expected_fields: &[&str]) {
        for field in expected_fields {
            assert!(
                response.get(field).is_some(),
                "Expected field '{}' in response",
                field
            );
        }
    }

    /// Assert vector data is valid
    pub fn assert_vector_data(vector: &serde_json::Value, expected_dimension: usize) {
        assert!(vector.get("id").is_some(), "Vector missing 'id' field");
        assert!(vector.get("data").is_some(), "Vector missing 'data' field");

        let data = vector.get("data").unwrap().as_array().unwrap();
        assert_eq!(
            data.len(),
            expected_dimension,
            "Vector dimension mismatch: expected {}, got {}",
            expected_dimension,
            data.len()
        );

        // Verify all elements are numbers
        for (i, value) in data.iter().enumerate() {
            assert!(
                value.is_number(),
                "Vector data[{}] is not a number: {:?}",
                i,
                value
            );
        }
    }
}

/// Test logging utilities
pub struct TestLogger;

impl TestLogger {
    /// Initialize test logging
    pub fn init() {
        if env::var("RUST_LOG").is_err() {
            unsafe {
                env::set_var("RUST_LOG", "warn");
            }
        }

        let _ = tracing_subscriber::fmt::try_init();
    }

    /// Log test start
    pub fn log_test_start(test_name: &str) {
        println!("🧪 Starting test: {}", test_name);
    }

    /// Log test completion
    pub fn log_test_completion(test_name: &str, duration: Duration) {
        println!("✅ Completed test: {} ({:?})", test_name, duration);
    }

    /// Log test failure
    pub fn log_test_failure(test_name: &str, error: &str) {
        println!("❌ Failed test: {} - {}", test_name, error);
    }

    /// Log performance metrics
    pub fn log_performance_metrics(operation: &str, duration: Duration, count: usize) {
        let duration_ms = duration.as_millis() as f64;
        let throughput = if duration_ms > 0.0 {
            (count as f64 * 1000.0) / duration_ms
        } else {
            0.0
        };

        println!(
            "📊 {}: {} operations in {:.2}ms ({:.2} ops/sec)",
            operation, count, duration_ms, throughput
        );
    }
}
