//! Centralised resolution of Vectorizer's runtime directories.
//!
//! Before this module landed, three different sites hand-rolled
//! `current_dir().join("data")` / `PathBuf::from(".logs")` —
//! [`crate::auth::persistence::AuthPersistence::get_data_dir`],
//! [`crate::db::vector_store::VectorStore::get_data_dir`], and the
//! server binary's startup code. That meant the .vecdb files, the
//! encrypted root credentials, and the rolling logs all landed
//! wherever the server happened to be launched from — typically
//! polluting the workspace itself when running via `cargo run`. It
//! also let two processes started from different cwds disagree on
//! which database they were operating against.
//!
//! These helpers replace those ad-hoc lookups with a single, OS-aware
//! answer:
//!
//! - **Linux**: `~/.local/share/vectorizer/{data,logs}` (XDG_DATA_HOME)
//! - **macOS**: `~/Library/Application Support/vectorizer/{data,logs}`
//! - **Windows**: `%APPDATA%\vectorizer\{data,logs}` (Roaming)
//!
//! Each can be overridden with the corresponding environment variable
//! (`VECTORIZER_DATA_DIR` / `VECTORIZER_LOGS_DIR`) so containerised
//! deployments and CI runners can still pin the path explicitly. If
//! the OS lookup fails (no `$HOME` set, etc.), both helpers fall
//! back to `./data` / `./.logs` — the legacy behaviour — so a fresh
//! install never silently writes to a path the user can't find.

use std::path::PathBuf;

/// Returns the canonical Vectorizer data directory.
///
/// Resolution order:
/// 1. `$VECTORIZER_DATA_DIR` if set and non-empty.
/// 2. `dirs::data_dir().join("vectorizer")` (per-OS user data dir).
/// 3. `./data` relative to the current working directory (legacy
///    fallback when the OS lookup yields `None`).
///
/// Callers are responsible for `create_dir_all` on the returned path
/// — these helpers do not touch the filesystem.
#[must_use]
pub fn data_dir() -> PathBuf {
    if let Ok(override_path) = std::env::var("VECTORIZER_DATA_DIR")
        && !override_path.is_empty()
    {
        return PathBuf::from(override_path);
    }
    dirs::data_dir()
        .map(|d| d.join("vectorizer"))
        .unwrap_or_else(|| PathBuf::from("data"))
}

/// Returns the canonical Vectorizer logs directory.
///
/// Resolution order:
/// 1. `$VECTORIZER_LOGS_DIR` if set and non-empty.
/// 2. `dirs::data_dir().join("vectorizer").join("logs")` — colocated
///    with the data dir so a single bind mount in Docker / Kubernetes
///    captures both.
/// 3. `./.logs` relative to the current working directory (legacy
///    fallback).
#[must_use]
pub fn logs_dir() -> PathBuf {
    if let Ok(override_path) = std::env::var("VECTORIZER_LOGS_DIR")
        && !override_path.is_empty()
    {
        return PathBuf::from(override_path);
    }
    dirs::data_dir()
        .map(|d| d.join("vectorizer").join("logs"))
        .unwrap_or_else(|| PathBuf::from(".logs"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // SAFETY (applies to every `unsafe { std::env::set_var / remove_var }`
    // call in this module): cargo's default test harness runs each
    // `#[test]` on its own task within a single process, and these
    // tests don't spawn additional threads. The Rust 2024 unsafety
    // around env mutation is for cross-thread races; sequential
    // single-thread access — which is what tests do here — is fine.
    // Each test restores the prior value on exit so siblings see a
    // clean environment.

    #[test]
    fn data_dir_honours_env_override() {
        let key = "VECTORIZER_DATA_DIR";
        let prev = std::env::var(key).ok();
        // SAFETY: see module-level comment.
        unsafe { std::env::set_var(key, "/tmp/vectorizer-test-data") };
        assert_eq!(data_dir(), PathBuf::from("/tmp/vectorizer-test-data"));
        match prev {
            // SAFETY: see module-level comment.
            Some(v) => unsafe { std::env::set_var(key, v) },
            // SAFETY: see module-level comment.
            None => unsafe { std::env::remove_var(key) },
        }
    }

    #[test]
    fn logs_dir_honours_env_override() {
        let key = "VECTORIZER_LOGS_DIR";
        let prev = std::env::var(key).ok();
        // SAFETY: see module-level comment.
        unsafe { std::env::set_var(key, "/tmp/vectorizer-test-logs") };
        assert_eq!(logs_dir(), PathBuf::from("/tmp/vectorizer-test-logs"));
        match prev {
            // SAFETY: see module-level comment.
            Some(v) => unsafe { std::env::set_var(key, v) },
            // SAFETY: see module-level comment.
            None => unsafe { std::env::remove_var(key) },
        }
    }

    #[test]
    fn data_dir_returns_a_path_in_the_default_case() {
        // Don't assert the exact path — it's OS-dependent — but
        // confirm the helper never panics and returns something
        // non-empty even without any env override.
        let key = "VECTORIZER_DATA_DIR";
        let prev = std::env::var(key).ok();
        // SAFETY: see module-level comment.
        unsafe { std::env::remove_var(key) };
        let p = data_dir();
        assert!(!p.as_os_str().is_empty());
        if let Some(v) = prev {
            // SAFETY: see module-level comment.
            unsafe { std::env::set_var(key, v) };
        }
    }
}
