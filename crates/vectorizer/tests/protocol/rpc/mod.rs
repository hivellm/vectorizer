//! End-to-end integration tests for the VectorizerRPC binary
//! transport. These boot a real listener on an ephemeral port, reuse the
//! production `spawn_rpc_listener` + dispatch path, and drive it from
//! Thunder's own `Client` — the transport the SDKs use.

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod handshake;
