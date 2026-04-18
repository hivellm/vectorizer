//! Path-traversal guards for user- and config-derived paths.
//!
//! `std::path::PathBuf::from(user_input)` is the standard way to build a path
//! from an external source, but it has no awareness of a containing directory:
//! passing `"../../etc/passwd"` yields a `PathBuf` that will happily walk out
//! of any allegedly restricted base. This module centralizes the containment
//! check — you give it a base directory and a candidate, it returns the
//! candidate only if its canonical form remains rooted at the base.
//!
//! Used by `src/file_watcher/discovery.rs` and `src/discovery/` to keep
//! workspace indexers inside the workspace root and refuse symlinks that
//! escape it.

use std::path::{Path, PathBuf};

use crate::error::{Result, VectorizerError};

/// Reject a raw path string that contains obvious traversal markers before
/// it's ever turned into a `PathBuf`. Catches the easy case of
/// `"../../etc/passwd"` in user input.
///
/// Rules:
/// - Empty strings are rejected (callers must supply a real path).
/// - Paths containing a NUL byte are rejected (classic filesystem attack).
/// - Paths whose components include `..` are rejected.
/// - Absolute paths are rejected (callers that need absolute paths must
///   call this with `allow_absolute = true`).
pub fn reject_traversal(raw: &str, allow_absolute: bool) -> Result<&str> {
    if raw.is_empty() {
        return Err(VectorizerError::InvalidConfiguration {
            message: "path is empty".to_string(),
        });
    }
    if raw.contains('\0') {
        return Err(VectorizerError::InvalidConfiguration {
            message: "path contains a NUL byte".to_string(),
        });
    }
    let candidate = Path::new(raw);
    if candidate.is_absolute() && !allow_absolute {
        return Err(VectorizerError::InvalidConfiguration {
            message: format!("absolute paths are not allowed here: {raw}"),
        });
    }
    for component in candidate.components() {
        use std::path::Component;
        if matches!(component, Component::ParentDir) {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!("path traversal ('..') rejected: {raw}"),
            });
        }
    }
    Ok(raw)
}

/// Resolve `candidate` against `base` and confirm the canonical result stays
/// inside `base`. Follows symlinks, so a symlink pointing outside the base is
/// rejected too.
///
/// `base` must exist and be canonicalizable (typically `std::env::current_dir()`
/// or an operator-supplied workspace root). `candidate` can be absolute or
/// relative; if relative it's joined to `base` before canonicalization.
///
/// Returns the canonical `PathBuf` on success.
pub fn canonicalize_within(base: &Path, candidate: &Path) -> Result<PathBuf> {
    let base_canon = base
        .canonicalize()
        .map_err(|e| VectorizerError::InvalidConfiguration {
            message: format!(
                "base directory cannot be canonicalized ({}): {}",
                base.display(),
                e
            ),
        })?;

    let joined = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        base_canon.join(candidate)
    };

    let candidate_canon =
        joined
            .canonicalize()
            .map_err(|e| VectorizerError::InvalidConfiguration {
                message: format!(
                    "path does not exist or is unreadable ({}): {}",
                    joined.display(),
                    e
                ),
            })?;

    if !candidate_canon.starts_with(&base_canon) {
        return Err(VectorizerError::InvalidConfiguration {
            message: format!(
                "path {} resolves outside the base {} (potential traversal/symlink-escape)",
                candidate_canon.display(),
                base_canon.display()
            ),
        });
    }

    Ok(candidate_canon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_traversal_blocks_parent_components() {
        assert!(reject_traversal("../../etc/passwd", false).is_err());
        assert!(reject_traversal("foo/../bar", false).is_err());
    }

    #[test]
    fn reject_traversal_blocks_absolute_when_not_allowed() {
        #[cfg(unix)]
        assert!(reject_traversal("/etc/passwd", false).is_err());
        #[cfg(windows)]
        assert!(reject_traversal("C:/Windows", false).is_err());
    }

    #[test]
    fn reject_traversal_blocks_empty_and_null() {
        assert!(reject_traversal("", false).is_err());
        assert!(reject_traversal("foo\0bar", false).is_err());
    }

    #[test]
    fn reject_traversal_accepts_safe_relative() {
        assert!(reject_traversal("src/foo.rs", false).is_ok());
        assert!(reject_traversal("docs/README.md", false).is_ok());
        assert!(reject_traversal("./a/b", false).is_ok());
    }

    #[test]
    fn canonicalize_within_rejects_escape() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base = tmp.path();
        let nested = base.join("inner");
        std::fs::create_dir(&nested).expect("mkdir inner");

        // Candidate that tries to escape via `..` after join.
        let escape = Path::new("inner/../..");
        let err = canonicalize_within(base, escape).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("outside the base") || msg.contains("traversal"),
            "expected escape-rejection message, got: {msg}"
        );
    }

    #[test]
    fn canonicalize_within_accepts_valid_subpath() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base = tmp.path();
        let file = base.join("child.txt");
        std::fs::write(&file, b"hi").expect("write");

        let resolved = canonicalize_within(base, Path::new("child.txt")).expect("accepted");
        assert!(resolved.starts_with(base.canonicalize().unwrap()));
    }

    #[test]
    fn canonicalize_within_rejects_nonexistent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base = tmp.path();
        let err = canonicalize_within(base, Path::new("no-such-thing")).unwrap_err();
        assert!(
            err.to_string().contains("does not exist"),
            "expected missing-path message, got: {err}"
        );
    }
}
