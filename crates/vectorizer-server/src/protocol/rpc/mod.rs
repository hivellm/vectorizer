//! VectorizerRPC server-side glue: Thunder `Dispatch` binding + dispatch table.
//!
//! Wire spec § 1, 4, 5: `docs/specs/VECTORIZER_RPC.md`. The wire types +
//! transport are Thunder's (`thunder-rpc`); the `Dispatch` impl + listener
//! bootstrap (`server`) and the per-command dispatch table (`dispatch`) live
//! here because they consume `vectorizer::db::VectorStore`,
//! `vectorizer::embedding::EmbeddingManager`, and the server's
//! `AuthHandlerState`.

pub mod dispatch;
pub mod server;

pub use server::spawn_rpc_listener;
