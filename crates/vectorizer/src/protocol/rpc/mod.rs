//! VectorizerRPC wire types — re-exported from Thunder (`thunder-rpc`).
//!
//! The TCP listener + dispatch (`server.rs`, `dispatch.rs`) moved
//! into `vectorizer-server::protocol::rpc` under
//! phase4_split-vectorizer-workspace sub-phase 4 because they pull
//! `AuthHandlerState` from the now-extracted `server/` module. The
//! umbrella `vectorizer` crate keeps only the wire types re-export so
//! engine code that needs to construct a `Request` / `Response`
//! (without dispatching it) doesn't have to learn the full server-side
//! API.
//!
//! The shapes now come from [`thunder::wire`] rather than a bespoke
//! `vectorizer-protocol::rpc_wire`: Thunder is the packaged form of the
//! very wire this crate hand-copied from SynapRPC (4-byte little-endian
//! length prefix + MessagePack body, v1, frozen), so the on-wire bytes
//! are unchanged and there is one codec to maintain for the whole
//! HiveLLM family. `Value` supersedes the retired `VectorizerValue`
//! name — same eight variants, `Bytes` now carrying `Arc<[u8]>`.

/// Frame codec + protocol config: [`thunder::wire::encode_frame`],
/// [`thunder::wire::decode_frame`], `Config`. Replaces the retired
/// `rpc_wire::codec`.
pub use thunder::wire;
pub use thunder::{Request, Response, Value};
