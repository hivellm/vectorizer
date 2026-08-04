//! VectorizerRPC wire types — `Request`, `Response`, `VectorizerValue`.
//!
//! Wire spec § 2 + § 3: `docs/specs/VECTORIZER_RPC.md`.
//!
//! These types are Thunder's (`thunder-rpc`, lib `thunder`) — the family's
//! shared binary RPC crate that `vectorizer-server` also runs, so the SDK and
//! the server cannot disagree on the wire: they are literally the same Rust
//! types compiled from the same source. `VectorizerValue` stays as an alias
//! for [`thunder::Value`] so existing call sites keep reading the same way;
//! the eight variants are unchanged, with `Bytes` now carrying an
//! `Arc<[u8]>` instead of a `Vec<u8>`.

pub use thunder::Value as VectorizerValue;
pub use thunder::{Request, Response};
