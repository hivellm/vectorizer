//! Hive Vectorizer Rust SDK
//!
//! High-performance Rust client for the Hive Vectorizer vector database.
//! Provides async/await support for vector operations, semantic search, and collection management.

pub mod client;
pub mod error;
pub mod models;
pub mod utils;

// Re-export main types for convenience
pub use client::VectorizerClient;
pub use error::{VectorizerError, Result};
pub use models::*;

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default API base URL
pub const DEFAULT_BASE_URL: &str = "http://localhost:15001";

/// Default WebSocket URL
pub const DEFAULT_WS_URL: &str = "ws://localhost:15001/ws";

/// Default MCP server URL
pub const DEFAULT_MCP_URL: &str = "http://localhost:15002/sse";

/// Default request timeout in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default maximum retries
pub const DEFAULT_MAX_RETRIES: usize = 3;

/// Default retry delay in seconds
pub const DEFAULT_RETRY_DELAY_SECS: u64 = 1;
