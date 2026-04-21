//! VectorizerRPC client — length-prefixed MessagePack over raw TCP.
//!
//! The wire spec at `docs/specs/VECTORIZER_RPC.md` (in the parent
//! Vectorizer repo) is the byte-level contract. This module ports the
//! server-side codec + types byte-for-byte so a v1 server can talk to
//! a v1 SDK client without translation.
//!
//! ## Quick start
//!
//! ```no_run
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! use vectorizer_sdk::rpc::{RpcClient, HelloPayload};
//!
//! let mut client = RpcClient::connect("127.0.0.1:15503").await?;
//! client.hello(HelloPayload::new("vectorizer-sdk-rust/3.0.0")).await?;
//! let pong = client.ping().await?;
//! assert_eq!(pong, "PONG");
//! # Ok(())
//! # }
//! ```
//!
//! ## Layout
//!
//! - [`codec`]    — frame encode/decode (`u32 LE len` + MessagePack body).
//! - [`types`]    — `Request`, `Response`, `VectorizerValue` wire types.
//! - [`client`]   — `RpcClient`: connect, hello, call, ping, close.
//! - [`pool`]     — minimal `RpcPool<T>` for reusing connections.
//! - [`endpoint`] — `parse_endpoint(url)` for the canonical
//!   `vectorizer://host[:port]` URL scheme.

pub mod client;
pub mod codec;
pub mod commands;
pub mod endpoint;
pub mod pool;
pub mod types;

pub use client::{HelloPayload, HelloResponse, RpcClient, RpcClientError};
pub use commands::{CollectionInfo, SearchHit};
pub use endpoint::{Endpoint, ParseError, parse_endpoint};
pub use pool::RpcPool;
pub use types::{Request, Response, VectorizerValue};
