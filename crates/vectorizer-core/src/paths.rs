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

/// Detect whether `path` is on the container's writable layer
/// (i.e. lacks a backing volume/bind-mount) and warn accordingly.
///
/// Returns the message a caller should `tracing::warn!` when the
/// data directory is ephemeral, or `None` when it appears to be on
/// a real mount. The detection is Linux-only (relies on
/// `/proc/self/mountinfo`) and a no-op on other platforms so
/// bare-metal Windows / macOS deployments don't see false-positive
/// warnings.
///
/// Heuristic: a containerised path like `/data` is "ephemeral" when
/// no entry in `/proc/self/mountinfo` covers it — that is, the
/// longest mount prefix is just `/` (the container's root overlay
/// filesystem). Any tighter prefix (`/data`, `/var/lib/...`, etc.)
/// indicates a bind / named volume that survives container recreate.
///
/// Phase32 / issue #300 — the canonical scenario this guards against
/// is `docker compose up -d --force-recreate vectorizer` wiping every
/// collection because the operator mounted `/data` but the binary
/// historically wrote to `/.local/share/vectorizer` on the writable
/// layer.
#[must_use]
pub fn ephemeral_data_dir_warning(path: &std::path::Path) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        ephemeral_linux(path)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        None
    }
}

#[cfg(target_os = "linux")]
fn ephemeral_linux(path: &std::path::Path) -> Option<String> {
    let mountinfo = std::fs::read_to_string("/proc/self/mountinfo").ok()?;
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    classify_against_mountinfo(&mountinfo, &abs)
}

/// Pure helper extracted for testability — given the raw text of
/// `/proc/self/mountinfo` and an absolute path, return the
/// ephemeral-data-dir warning when the path is only covered by `/`
/// (i.e. lives on the container's writable layer), or `None` when a
/// tighter mount prefix (bind / named volume) covers it.
///
/// Kept private to the crate; tests in this module synthesise
/// mountinfo strings to cover both branches without relying on the
/// host's actual mount table — CI runners (notably GitHub Actions'
/// Ubuntu images where `/tmp` is part of the rootfs) made a
/// `/tmp`-as-its-own-mount assumption flake on real `/proc/self/mountinfo`.
#[cfg(target_os = "linux")]
fn classify_against_mountinfo(mountinfo: &str, abs: &std::path::Path) -> Option<String> {
    let abs_str = abs.to_string_lossy();

    // Each line of mountinfo has the mount point as the 5th
    // whitespace-separated field. Collect every mount point, find the
    // longest one that is a prefix of `abs`.
    let mut best: &str = "";
    for line in mountinfo.lines() {
        let Some(mp) = line.split_whitespace().nth(4) else {
            continue;
        };
        if abs_str.starts_with(mp) && mp.len() > best.len() {
            best = mp;
        }
    }

    // Only `/` matched — no bind / named volume covers this path, so
    // it lives on the container's writable layer.
    if best == "/" || best.is_empty() {
        Some(format!(
            "data dir at {} is ephemeral; recommend mounting a volume \
             (issue #300). Without a persistent mount, every \
             container recreate wipes collections, auth keys, and the \
             JWT secret. Example: --volume vec-data:{}",
            abs.display(),
            abs.display()
        ))
    } else {
        None
    }
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

    #[cfg(target_os = "linux")]
    #[test]
    fn classify_synthetic_writable_layer_path_is_flagged() {
        // No tighter mount than `/` covers `/data` → ephemeral.
        // Two mountinfo fields (root + sysfs) — sysfs at `/sys` does
        // not prefix `/data`, so the longest match is `/` and the
        // detector must emit the warning.
        let mountinfo = "1 0 0:1 / / rw,relatime - overlay overlay rw\n\
                         2 1 0:2 / /sys rw,nosuid,nodev,noexec,relatime - sysfs sysfs rw\n";
        let msg = classify_against_mountinfo(mountinfo, std::path::Path::new("/data"))
            .expect("/data with only `/` mount must be flagged ephemeral");
        assert!(
            msg.contains("/data"),
            "warning must name the offending path: {msg}"
        );
        assert!(
            msg.contains("issue #300"),
            "warning must link the phase32 issue: {msg}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn classify_synthetic_bind_mounted_path_is_silent() {
        // A bind / named volume at `/data` covers the path → no
        // warning. This is the canonical "operator did the right
        // thing" case.
        let mountinfo = "1 0 0:1 / / rw,relatime - overlay overlay rw\n\
                         42 1 8:1 /vol /data rw,relatime - ext4 /dev/sda1 rw\n";
        assert!(
            classify_against_mountinfo(mountinfo, std::path::Path::new("/data")).is_none(),
            "/data with a tighter mount prefix must not be flagged"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn classify_handles_unknown_path_outside_any_mount() {
        // No mount matches → treat as ephemeral (best == empty branch).
        let mountinfo = "1 0 0:1 / /unrelated rw - tmpfs tmpfs rw\n";
        let msg = classify_against_mountinfo(mountinfo, std::path::Path::new("/data"));
        assert!(msg.is_some(), "path with no covering mount must warn");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn classify_picks_longest_prefix_when_several_mounts_match() {
        // Both `/` and `/data` cover `/data/sub`; the detector must
        // pick `/data` and stay silent.
        let mountinfo = "1 0 0:1 / / rw - overlay overlay rw\n\
                         2 1 8:1 / /data rw - ext4 /dev/sda1 rw\n";
        assert!(
            classify_against_mountinfo(mountinfo, std::path::Path::new("/data/sub")).is_none(),
            "longest matching prefix wins — /data should suppress /"
        );
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
