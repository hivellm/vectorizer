//! Re-export shim for the redacting `Secret<T>` newtype.
//!
//! The type itself now lives in `crate::config::secret` so that
//! `config`-owned types (e.g. `AuthConfig::jwt_secret`) can reference
//! it without creating a `config -> auth` dependency edge (see
//! `phase41_architecture-decoupling` §2). This module re-exports it
//! under the historical `crate::auth::secret::Secret` path.

pub use crate::config::secret::Secret;
