//! VectorizerRPC wire types — `Request`, `Response`, `VectorizerValue`.
//!
//! Wire spec § 2 + § 3: `docs/specs/VECTORIZER_RPC.md`.
//!
//! These types used to be hand-ported byte-for-byte from the server's
//! `vectorizer::protocol::rpc::types` and kept in sync by convention.
//! Under `phase4_split-vectorizer-workspace` sub-phase 6 the server's
//! wire types moved into the standalone `vectorizer-protocol` crate;
//! this module now re-exports them so the SDK and the server cannot
//! disagree on the wire format — they're literally the same Rust
//! types compiled from the same source.

pub use vectorizer_protocol::rpc_wire::types::{Request, Response, VectorizerValue};
