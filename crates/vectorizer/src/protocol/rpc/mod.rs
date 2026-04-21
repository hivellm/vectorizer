//! VectorizerRPC wire types — re-exported from `vectorizer-protocol`.
//!
//! The TCP listener + dispatch (`server.rs`, `dispatch.rs`) moved
//! into `vectorizer-server::protocol::rpc` under
//! phase4_split-vectorizer-workspace sub-phase 4 because they pull
//! `AuthHandlerState` from the now-extracted `server/` module. The
//! umbrella `vectorizer` crate keeps only the wire types + codec
//! re-export so engine code that needs to construct a `Request` /
//! `Response` (without dispatching it) doesn't have to learn the
//! full server-side API.

pub use vectorizer_protocol::rpc_wire::{Request, Response, VectorizerValue, codec, types};
