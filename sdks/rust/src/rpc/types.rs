//! VectorizerRPC wire types — `Request`, `Response`, `VectorizerValue`.
//!
//! Wire spec § 2 + § 3: `docs/specs/VECTORIZER_RPC.md` in the parent
//! Vectorizer repo. Ported byte-for-byte from the server's
//! `vectorizer::protocol::rpc::types`. The on-wire representation is
//! identical to SynapRPC's `SynapValue`.

use serde::{Deserialize, Serialize};

/// A dynamically-typed value that crosses the VectorizerRPC wire.
///
/// Encoded with rmp-serde's default externally-tagged representation:
/// unit variants become a bare string (`"Null"`), newtype variants
/// become a single-key map (`{"Int": 42}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VectorizerValue {
    /// SQL NULL / absence of a value.
    Null,
    /// Boolean.
    Bool(bool),
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit IEEE 754 float.
    Float(f64),
    /// Raw bytes — stored without base64 (unlike the JSON transports).
    Bytes(Vec<u8>),
    /// UTF-8 string.
    Str(String),
    /// Heterogeneous array.
    Array(Vec<VectorizerValue>),
    /// Ordered map of `(key, value)` pairs. Vec preserves insertion
    /// order and allows non-string keys, matching MessagePack maps.
    Map(Vec<(VectorizerValue, VectorizerValue)>),
}

impl VectorizerValue {
    /// Borrow as a string slice if the variant is `Str`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Borrow as bytes if the variant is `Bytes` or `Str`.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b.as_slice()),
            Self::Str(s) => Some(s.as_bytes()),
            _ => None,
        }
    }

    /// Read as `i64` if the variant is `Int`.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Read as `f64` if the variant is `Float` (or coerce from `Int`).
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Read as `bool` if the variant is `Bool`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Borrow as an array if the variant is `Array`.
    pub fn as_array(&self) -> Option<&[VectorizerValue]> {
        match self {
            Self::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    /// Borrow as a map's `(k, v)` pairs if the variant is `Map`.
    pub fn as_map(&self) -> Option<&[(VectorizerValue, VectorizerValue)]> {
        match self {
            Self::Map(m) => Some(m.as_slice()),
            _ => None,
        }
    }

    /// Look up a string-keyed map entry. Returns `None` if `self` is
    /// not a `Map` or if the key is missing.
    pub fn map_get(&self, key: &str) -> Option<&VectorizerValue> {
        let pairs = self.as_map()?;
        pairs.iter().find_map(|(k, v)| match k.as_str() {
            Some(k_str) if k_str == key => Some(v),
            _ => None,
        })
    }
}

impl From<String> for VectorizerValue {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<&str> for VectorizerValue {
    fn from(s: &str) -> Self {
        Self::Str(s.to_owned())
    }
}

impl From<Vec<u8>> for VectorizerValue {
    fn from(b: Vec<u8>) -> Self {
        Self::Bytes(b)
    }
}

impl From<i64> for VectorizerValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<bool> for VectorizerValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

// ── Wire frames ──────────────────────────────────────────────────────────────

/// A request from client to server. Wire spec § 2.
///
/// `id` is chosen by the client and echoed back in the matching
/// [`Response`], enabling out-of-order (multiplexed) responses on a
/// single TCP connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Caller-chosen monotonic identifier; opaque to the server.
    pub id: u32,
    /// Command name from the capability registry, e.g. `"search.basic"`.
    pub command: String,
    /// Positional arguments — same order as the wire spec's command
    /// catalog at § 6.
    pub args: Vec<VectorizerValue>,
}

/// A response from server to client. Wire spec § 2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Echoes the `id` from the corresponding [`Request`].
    pub id: u32,
    /// `Ok(value)` on success, `Err(human-readable message)` on
    /// failure. v1 carries a string error per the spec; v2 will
    /// upgrade to a structured `Error { code, message, details }`.
    pub result: Result<VectorizerValue, String>,
}

impl Response {
    /// Build a successful response carrying `value`. Used by the
    /// in-test fake-server fixture under `tests/`.
    pub fn ok(id: u32, value: VectorizerValue) -> Self {
        Self {
            id,
            result: Ok(value),
        }
    }

    /// Build an error response carrying `msg`. Used by the in-test
    /// fake-server fixture under `tests/`.
    pub fn err(id: u32, msg: impl Into<String>) -> Self {
        Self {
            id,
            result: Err(msg.into()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn value_roundtrip_all_variants() {
        let variants: Vec<VectorizerValue> = vec![
            VectorizerValue::Null,
            VectorizerValue::Bool(true),
            VectorizerValue::Int(i64::MIN),
            VectorizerValue::Int(i64::MAX),
            VectorizerValue::Float(1.5),
            VectorizerValue::Bytes(vec![0, 1, 2, 255]),
            VectorizerValue::Str("hello".into()),
            VectorizerValue::Array(vec![
                VectorizerValue::Int(1),
                VectorizerValue::Str("two".into()),
            ]),
            VectorizerValue::Map(vec![(
                VectorizerValue::Str("k".into()),
                VectorizerValue::Int(99),
            )]),
        ];
        for v in variants {
            let encoded = rmp_serde::to_vec(&v).expect("encode");
            let decoded: VectorizerValue = rmp_serde::from_slice(&encoded).expect("decode");
            assert_eq!(v, decoded);
        }
    }

    #[test]
    fn map_get_finds_string_key() {
        let v = VectorizerValue::Map(vec![
            (
                VectorizerValue::Str("name".into()),
                VectorizerValue::Str("alpha".into()),
            ),
            (
                VectorizerValue::Str("count".into()),
                VectorizerValue::Int(42),
            ),
        ]);
        assert_eq!(v.map_get("name").and_then(|v| v.as_str()), Some("alpha"));
        assert_eq!(v.map_get("count").and_then(|v| v.as_int()), Some(42));
        assert!(v.map_get("missing").is_none());
    }
}
