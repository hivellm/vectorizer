//! VectorizerRPC frame codec — `[u32 LE len][MessagePack body]`.
//!
//! Wire spec § 1: `docs/specs/VECTORIZER_RPC.md`.
//!
//! This module used to be a byte-for-byte hand-port of the server's
//! `vectorizer::protocol::rpc::codec`. Under
//! `phase4_split-vectorizer-workspace` sub-phase 6 the server's
//! codec moved into the standalone `vectorizer-protocol` crate;
//! this module now re-exports it so the SDK and the server cannot
//! disagree on framing — they're literally the same Rust functions
//! compiled from the same source.

pub use vectorizer_protocol::rpc_wire::codec::{
    MAX_BODY_SIZE, decode_frame, encode_frame, read_request, read_response, write_request,
    write_response,
};
