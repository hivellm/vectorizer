//! End-to-end integration tests for the VectorizerRPC binary
//! transport. These boot a real `TcpListener` on an ephemeral port,
//! reuse the production `spawn_rpc_listener` + dispatch path, and
//! drive it from a synthetic client built on `vectorizer::protocol::
//! rpc::codec`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod handshake;
