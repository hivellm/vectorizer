//! VectorizerRPC wire types — shared between the server
//! (`vectorizer-server`) and the Rust SDK (`sdks/rust`). Carries the
//! on-the-wire shapes only; the dispatch / handler layer lives in
//! `vectorizer-server::protocol::rpc` because those types depend on the
//! storage engine, auth, and the capability registry.
//!
//! - [`rpc_wire`]  — length-prefixed MessagePack frames (Request /
//!   Response / VectorizerValue) and the codec helpers that read /
//!   write them. Wire spec: `docs/specs/VECTORIZER_RPC.md`.
//!
//! The tonic/prost gRPC schemas were relocated to the `vectorizer-grpc`
//! crate so this crate's RPC-wire layer can migrate to `thunder-rpc`.

#![deny(missing_docs)]

pub mod rpc_wire;
