//! VectorizerRPC wire layer — length-prefixed MessagePack over raw TCP.
//!
//! Wire spec: `docs/specs/VECTORIZER_RPC.md`. Ported from the
//! production-tested SynapRPC implementation at
//! `../Synap/synap-server/src/protocol/synap_rpc/`.
//!
//! Layout:
//!
//! - [`codec`] — frame encode/decode (`u32 LE len` + MessagePack body).
//! - [`types`] — `Request`, `Response`, `VectorizerValue` wire types.
//!
//! The TCP listener + dispatch layer that consumes these types lives
//! in `vectorizer::protocol::rpc::server` because it depends on the
//! storage engine and auth — they're not part of the wire contract.

pub mod codec;
pub mod types;

pub use types::{Request, Response, VectorizerValue};
