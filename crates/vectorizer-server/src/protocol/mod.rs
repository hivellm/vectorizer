//! Server-side glue for VectorizerRPC.
//!
//! The wire types + codec live in `vectorizer-protocol`; this
//! module hosts the TCP listener (`spawn_rpc_listener`) and the
//! per-command dispatch table (`dispatch`) that depend on the
//! storage engine, embedding manager, and `AuthHandlerState`.

pub mod rpc;
