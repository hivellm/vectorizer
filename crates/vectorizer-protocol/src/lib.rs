//! Wire protocol types for Vectorizer — shared between the server
//! (umbrella `vectorizer` crate, soon `vectorizer-server`) and the
//! Rust SDK (`sdks/rust`). Carries the on-the-wire shapes only; the
//! dispatch / handler layer lives in `vectorizer::protocol::rpc::server`
//! and `vectorizer::grpc::server` because those types depend on the
//! storage engine, auth, and the capability registry.
//!
//! - [`rpc_wire`]  — length-prefixed MessagePack frames (Request /
//!   Response / VectorizerValue) and the codec helpers that read /
//!   write them. Wire spec: `docs/specs/VECTORIZER_RPC.md`.
//! - [`grpc_gen`]  — `tonic-prost`-generated modules for the three
//!   gRPC schemas: `vectorizer`, `cluster`, `qdrant_proto`. Built by
//!   this crate's `build.rs` against the `proto/` source tree.

#![deny(missing_docs)]

pub mod rpc_wire;

/// `tonic-prost`-generated gRPC modules. Mirrors the layout of the
/// underlying `proto/` source tree.
///
/// Generated code can't carry per-item lint annotations, so the
/// wrapping module silences the workspace lints that the proto
/// generator routinely trips (large enum variants from one-of
/// fields, `#[non_exhaustive]` boilerplate, missing docs).
#[allow(
    missing_docs,
    clippy::large_enum_variant,
    clippy::doc_markdown,
    clippy::module_inception
)]
pub mod grpc_gen {
    /// First-party Vectorizer gRPC schema (`proto/vectorizer.proto`).
    pub mod vectorizer {
        include!("grpc_gen/vectorizer.rs");
    }

    /// Cluster RPC schema (`proto/cluster.proto`).
    pub mod cluster {
        include!("grpc_gen/vectorizer.cluster.rs");
    }

    /// Qdrant compatibility schema (`proto/qdrant/*.proto`).
    pub mod qdrant_proto {
        include!("grpc_gen/qdrant/qdrant.rs");
    }
}
