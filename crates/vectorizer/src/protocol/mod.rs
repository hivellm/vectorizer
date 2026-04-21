//! Wire protocols other than HTTP.
//!
//! Today this means [`rpc`] — a length-prefixed MessagePack transport
//! over raw TCP.
//!
//! All protocols dispatch through the same capability registry at
//! [`crate::server::capabilities`], so a new operation lands on every
//! transport at once.

pub mod rpc;
