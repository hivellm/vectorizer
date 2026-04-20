//! VectorizerRPC — length-prefixed MessagePack transport over raw TCP.
//!
//! Wire spec: `docs/specs/VECTORIZER_RPC.md`. This module ports the
//! production-tested SynapRPC layer at
//! `../Synap/synap-server/src/protocol/synap_rpc/` and adapts it to
//! Vectorizer's state types and capability registry.
//!
//! Layout:
//!
//! - [`codec`]    — frame encode/decode (`u32 LE len` + MessagePack body)
//!   — re-exported from `vectorizer-protocol` so the SDK consumes the
//!   same code (phase4_split-vectorizer-workspace, sub-phase 2).
//! - [`types`]    — `Request`, `Response`, `VectorizerValue` wire types
//!   — same re-export.
//! - [`server`]   — TCP listener + per-connection accept loop (depends
//!   on `VectorStore` + auth, stays in this crate).
//! - [`dispatch`] — command name → handler dispatch table; one arm per
//!   capability registry entry.

pub mod dispatch;
pub mod server;

pub use vectorizer_protocol::rpc_wire::codec;
pub use vectorizer_protocol::rpc_wire::types;
pub use vectorizer_protocol::rpc_wire::{Request, Response, VectorizerValue};

pub use server::spawn_rpc_listener;
