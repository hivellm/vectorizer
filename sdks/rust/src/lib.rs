//! Hive Vectorizer Rust SDK
//!
//! High-performance Rust client for the Hive Vectorizer vector database.
//! Provides async/await support for vector operations, semantic search, and collection management.

pub mod client;
pub mod error;
pub mod http_transport;
pub mod models;
pub mod transport;
pub mod utils;

#[cfg(feature = "umicp")]
pub mod umicp_transport;

// Re-export main types for convenience
pub use client::{ClientConfig, VectorizerClient};

#[cfg(feature = "umicp")]
pub use client::UmicpConfig;

pub use error::{Result, VectorizerError};
pub use http_transport::HttpTransport;
pub use models::*;
pub use transport::{Protocol, Transport, parse_connection_string};

#[cfg(feature = "umicp")]
pub use umicp_transport::UmicpTransport;

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default API base URL
pub const DEFAULT_BASE_URL: &str = "http://localhost:15002";

/// Default MCP server URL
pub const DEFAULT_MCP_URL: &str = "http://localhost:15002/sse";

/// Default request timeout in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default maximum retries
pub const DEFAULT_MAX_RETRIES: usize = 3;

/// Default retry delay in seconds
pub const DEFAULT_RETRY_DELAY_SECS: u64 = 1;
