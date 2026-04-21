//! VectorizerRPC server-side glue: TCP listener + dispatch.
//!
//! Wire spec § 1, 4, 5: `docs/specs/VECTORIZER_RPC.md`. The wire
//! types + codec live in `vectorizer-protocol::rpc_wire` (extracted
//! under sub-phase 2); the dispatch table + accept loop live here
//! because they consume `vectorizer::db::VectorStore`,
//! `vectorizer::embedding::EmbeddingManager`, and the server's
//! `AuthHandlerState`.

pub mod dispatch;
pub mod server;

pub use server::spawn_rpc_listener;
