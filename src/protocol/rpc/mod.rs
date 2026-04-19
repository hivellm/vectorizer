//! VectorizerRPC — length-prefixed MessagePack transport over raw TCP.
//!
//! Wire spec: `docs/specs/VECTORIZER_RPC.md`. This module ports the
//! production-tested SynapRPC layer at
//! `../Synap/synap-server/src/protocol/synap_rpc/` and adapts it to
//! Vectorizer's state types and capability registry.
//!
//! Layout:
//!
//! - [`codec`]    — frame encode/decode (`u32 LE len` + MessagePack body).
//! - [`types`]    — `Request`, `Response`, `VectorizerValue` wire types.
//! - [`server`]   — TCP listener + per-connection accept loop.
//! - [`dispatch`] — command name → handler dispatch table; one arm per
//!   capability registry entry.

pub mod codec;
pub mod dispatch;
pub mod server;
pub mod types;

pub use server::spawn_rpc_listener;
pub use types::{Request, Response, VectorizerValue};
