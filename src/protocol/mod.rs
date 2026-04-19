//! Wire protocols other than HTTP.
//!
//! Today this means [`rpc`] — a length-prefixed MessagePack transport
//! over raw TCP. RESP3 (`phase6_add-resp3-protocol-server`) will live
//! alongside it as `protocol::resp3` once that task lands.
//!
//! All protocols dispatch through the same capability registry at
//! [`crate::server::capabilities`], so a new operation lands on every
//! transport at once.

pub mod rpc;
