//! Binary codec helpers — thin wrappers around `bincode` v3.
//!
//! Provides `serialize` / `deserialize` functions that match the old `bincode`
//! v1 API signatures while using the v3 engine underneath with
//! [`bincode::config::legacy()`] for wire-format compatibility.
//!
//! All existing call sites can switch from `bincode::serialize` to
//! `crate::codec::serialize` (and likewise for `deserialize`) with no other
//! code changes.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use serde::Serialize;
use serde::de::DeserializeOwned;

// Re-export the error types so callers can keep `codec::Error` in their
// signatures without importing bincode directly.

/// Unified error type covering both encode and decode failures.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("encode error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("decode error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
}

/// Bincode v1 used `Box<bincode::ErrorKind>` as its error type.
/// Many call sites propagate via `?` into `From<bincode::Error>`.
/// This type alias keeps those `From` impls working after migration.
pub type Result<T> = std::result::Result<T, Error>;

/// Wire-format-compatible configuration (matches bincode v1 defaults).
const fn legacy_config() -> bincode::config::Configuration<
    bincode::config::LittleEndian,
    bincode::config::Fixint,
    bincode::config::NoLimit,
> {
    bincode::config::legacy()
}

/// Serialize `value` into a `Vec<u8>` using the bincode v1 wire format.
///
/// Drop-in replacement for `bincode::serialize`.
pub fn serialize<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>> {
    Ok(bincode::serde::encode_to_vec(value, legacy_config())?)
}

/// Deserialize a `T` from `bytes` using the bincode v1 wire format.
///
/// Drop-in replacement for `bincode::deserialize`.
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let (val, _len) = bincode::serde::decode_from_slice(bytes, legacy_config())?;
    Ok(val)
}

/// Return the serialized size of `value` in the bincode v1 wire format.
///
/// Drop-in replacement for `bincode::serialized_size`.
/// Implemented by performing a full serialize — slightly less efficient than
/// the old dedicated function, but correct and rarely called.
pub fn serialized_size<T: Serialize + ?Sized>(value: &T) -> Result<u64> {
    let bytes = serialize(value)?;
    Ok(bytes.len() as u64)
}
