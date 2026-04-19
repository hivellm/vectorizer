//! Hive Vectorizer Rust SDK
//!
//! High-performance Rust client for the Hive Vectorizer vector
//! database. v3.x ships with **VectorizerRPC** as the default transport
//! (binary MessagePack over raw TCP, see [`rpc`]); HTTP stays available
//! as the legacy fallback under the `http` Cargo feature.
//!
//! # Quick start (RPC, default)
//!
//! ```no_run
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! use vectorizer_sdk::rpc::{RpcClient, HelloPayload};
//!
//! let client = RpcClient::connect("127.0.0.1:15503").await?;
//! client.hello(HelloPayload::new("vectorizer-sdk-rust/3.0.0")).await?;
//! let collections = client.list_collections().await?;
//! println!("collections: {collections:?}");
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod models;
pub mod rpc;
pub mod transport;
pub mod utils;

#[cfg(feature = "http")]
pub mod client;
#[cfg(feature = "http")]
pub mod http_transport;

#[cfg(feature = "umicp")]
pub mod umicp_transport;

// Re-export main types for convenience
#[cfg(all(feature = "http", feature = "umicp"))]
pub use client::UmicpConfig;
#[cfg(feature = "http")]
pub use client::{ClientConfig, VectorizerClient};
pub use error::{Result, VectorizerError};
#[cfg(feature = "http")]
pub use http_transport::HttpTransport;
pub use models::*;
pub use rpc::{HelloPayload, HelloResponse, RpcClient, RpcClientError, RpcPool};
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
