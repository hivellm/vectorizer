//! Redacting `Secret<T>` newtype for credential material.
//!
//! Wrapping a value in `Secret<T>` gives three guarantees:
//!
//! 1. `Debug` and `Display` render `"<redacted>"` — the inner value never
//!    reaches a log macro, panic message, or error chain by accident.
//! 2. Serde round-trips the inner value, so on-disk and on-wire formats
//!    are identical to the unwrapped type.
//! 3. On `Drop`, the backing memory is zeroed via `zeroize`.
//!
//! The only way to read the inner value is `.expose_secret()`, which is
//! deliberately grep-able. Call sites that need plaintext (hashing,
//! signing, constant-time comparison) should be auditable by running
//! `grep -rn expose_secret src/`.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::Zeroize;

/// Wrapper that hides its inner value from logs and zeroizes on drop.
///
/// `T` must implement `Zeroize` so the backing memory can be scrubbed.
/// Common impls are provided by the `zeroize` crate for `String`,
/// `Vec<u8>`, and integer primitives.
#[derive(Clone)]
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Secret<T> {
    /// Wrap a value.
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    /// Borrow the inner value. Grep for `expose_secret` to audit plaintext access.
    pub fn expose_secret(&self) -> &T {
        &self.0
    }
}

impl<T: Zeroize> From<T> for Secret<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T: Zeroize + Default> Default for Secret<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

// Equality on the wrapped value. Note: this is NOT constant-time. Callers that
// compare credential material in security-sensitive paths should use a
// constant-time comparator (e.g. `subtle::ConstantTimeEq`) on the exposed value.
impl<T: Zeroize + PartialEq> PartialEq for Secret<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Zeroize + Eq> Eq for Secret<T> {}

impl<T: Zeroize + Serialize> Serialize for Secret<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Zeroize + Deserialize<'de>> Deserialize<'de> for Secret<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn debug_format_does_not_leak_inner_value() {
        let s: Secret<String> = Secret::new("super-secret-value".to_string());
        let rendered = format!("{:?}", s);
        assert_eq!(rendered, "Secret(<redacted>)");
        assert!(!rendered.contains("super-secret-value"));
    }

    #[test]
    fn display_format_does_not_leak_inner_value() {
        let s: Secret<String> = Secret::new("super-secret-value".to_string());
        let rendered = format!("{}", s);
        assert_eq!(rendered, "<redacted>");
        assert!(!rendered.contains("super-secret-value"));
    }

    #[test]
    fn expose_secret_returns_inner() {
        let s: Secret<String> = Secret::new("abc".to_string());
        assert_eq!(s.expose_secret(), "abc");
    }

    #[test]
    fn serde_round_trip_transparent() {
        let original: Secret<String> = Secret::new("my-token".to_string());
        let json = serde_json::to_string(&original).expect("serialize");
        assert_eq!(json, "\"my-token\"", "wire format must match plain String");

        let back: Secret<String> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.expose_secret(), "my-token");
    }

    #[test]
    fn equality_compares_inner() {
        let a: Secret<String> = Secret::new("same".to_string());
        let b: Secret<String> = Secret::new("same".to_string());
        let c: Secret<String> = Secret::new("different".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn from_inner_constructs_secret() {
        let s: Secret<String> = "hello".to_string().into();
        assert_eq!(s.expose_secret(), "hello");
    }

    #[test]
    fn default_produces_empty_inner() {
        let s: Secret<String> = Secret::default();
        assert!(s.expose_secret().is_empty());
    }

    #[test]
    fn error_formatting_in_debug_chain_stays_redacted() {
        // Simulate a struct containing a Secret being printed with `{:?}`.
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Wrapper {
            name: String,
            token: Secret<String>,
        }

        let w = Wrapper {
            name: "admin".into(),
            token: Secret::new("leaky".into()),
        };
        let rendered = format!("{:?}", w);
        assert!(rendered.contains("admin"));
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("leaky"));
    }
}
