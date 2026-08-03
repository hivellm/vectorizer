//! VectorizerRPC client — length-prefixed MessagePack over raw TCP.
//!
//! The wire spec at `docs/specs/VECTORIZER_RPC.md` (in the parent
//! Vectorizer repo) is the byte-level contract. Framing, multiplexing and
//! the value model come from `thunder-rpc` — the same crate the server runs
//! — so a v1 server and a v1 SDK client cannot drift apart.
//!
//! ## Quick start
//!
//! ```no_run
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! use vectorizer_sdk::rpc::{RpcClient, HelloPayload};
//!
//! let client = RpcClient::connect("127.0.0.1:15503").await?;
//! client.hello(HelloPayload::new("vectorizer-sdk-rust/3.6.0")).await?;
//! let pong = client.ping().await?;
//! assert_eq!(pong, "PONG");
//! # Ok(())
//! # }
//! ```
//!
//! ## Layout
//!
//! - [`types`]    — `Request`, `Response`, `VectorizerValue` (Thunder's).
//! - [`client`]   — `RpcClient`: connect, hello, call, ping, close, plus
//!   `protocol_config()`, the client half of the server's wire config.
//! - [`commands`] — typed wrappers for the v1 command catalog.
//! - [`pool`]     — minimal `RpcPool` for reusing connections.
//! - [`endpoint`] — `parse_endpoint(url)` for the canonical
//!   `vectorizer://host[:port]` URL scheme.

pub mod client;
pub mod commands;
pub mod endpoint;
pub mod pool;
pub mod types;

pub use client::{
    DEFAULT_RPC_PORT, HelloPayload, HelloResponse, RpcClient, RpcClientError, protocol_config,
};
pub use commands::{
    AdminStats, AdminStatus, AnswerPlanResult, AnswerPlanSection, ApiKeyCreated, AuthMeResult,
    BatchDeleteResult, BatchInsertResult, BatchItemResult, BatchSearchResult, BatchUpdateResult,
    BulkUpdateMetadataRpcResult, CleanupEmptyResult, CollectionInfo, CompressBullet, CopyRpcResult,
    CreateCollectionResult, DeleteByFilterRpcResult, DiscoverEdgesForNodeResult,
    DiscoverEdgesResult, DiscoverResult, DiscoveryChunk, EmbedResult, ExpandQueriesResult,
    GraphDiscoveryStatus, MoveRpcResult, RebalanceStatus, RefreshTokenResult, RenderPromptResult,
    ReplicationConfigureResult, RotatedApiKey, SearchExplainResult, SearchHit, SearchTrace,
    SetExpiryResult, SlowQueryConfigResult, ValidatePasswordResult, VectorListResult,
    VectorWriteResult,
};
pub use endpoint::{Endpoint, ParseError, parse_endpoint};
pub use pool::RpcPool;
pub use types::{Request, Response, VectorizerValue};
