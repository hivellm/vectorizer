//! Server-side glue for VectorizerRPC.
//!
//! The wire types + transport are Thunder's (`thunder-rpc`); this
//! module hosts the Thunder `Dispatch` binding (`spawn_rpc_listener`)
//! and the per-command dispatch table (`dispatch`) that depend on the
//! storage engine, embedding manager, and `AuthHandlerState`.

pub mod rpc;
