//! First-boot JWT secret management.
//!
//! When the operator opts in (`--auto-generate-jwt-secret` or
//! `VECTORIZER_AUTO_GEN_JWT_SECRET=1`) and no explicit `auth.jwt_secret`
//! is set, the server persists a random 512-bit key to disk on first
//! boot and reuses it on every subsequent start.
//!
//! The persisted file is hex-encoded (128 ASCII chars) and written
//! atomically — a `.tmp` sibling is created, permissions are set, and
//! the final rename is the commit point. On POSIX we set mode `0o600`
//! (owner read/write only). Windows has no direct equivalent; the file
//! lives under the server's data directory whose ACLs are the operator's
//! responsibility — see `docs/security.md#jwt-secret`.

use std::fs;
use std::io::Write;
use std::path::Path;

use rand::TryRngCore;
use rand::rngs::OsRng;

use crate::error::{Result, VectorizerError};

/// Size of the generated secret in raw bytes (64 → 512 bits).
///
/// Hex-encoded this is 128 chars, comfortably above the 32-char minimum
/// that `AuthConfig::validate` enforces.
const SECRET_BYTES: usize = 64;

/// Expected length of the hex-encoded secret on disk.
const SECRET_HEX_LEN: usize = SECRET_BYTES * 2;

/// Load a persisted JWT secret from `path`, or generate+persist a fresh
/// one if the file is missing. Returns the secret as a 128-char hex string.
///
/// Atomicity: a fresh secret is written to `<path>.tmp` and then renamed
/// onto the final path. This prevents readers from observing a
/// partially-written file and prevents a crash mid-write from leaving
/// the canonical path in a corrupt state.
///
/// On POSIX, the temp file is opened with mode `0o600` before writing.
/// On Windows, permissions fall back to whatever the parent directory
/// ACL implies.
pub fn load_or_generate(path: &Path) -> Result<String> {
    if path.exists() {
        return load_existing(path);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(VectorizerError::IoError)?;
    }

    let secret = generate_hex_secret();
    write_atomic(path, &secret)?;
    Ok(secret)
}

fn load_existing(path: &Path) -> Result<String> {
    let raw = fs::read_to_string(path).map_err(VectorizerError::IoError)?;
    let trimmed = raw.trim();
    if trimmed.len() != SECRET_HEX_LEN {
        return Err(VectorizerError::InvalidConfiguration {
            message: format!(
                "Persisted JWT secret at {:?} has {} chars; expected {}. Delete the \
                 file and restart to regenerate, or replace with a valid hex-encoded \
                 {}-byte key.",
                path,
                trimmed.len(),
                SECRET_HEX_LEN,
                SECRET_BYTES
            ),
        });
    }
    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(VectorizerError::InvalidConfiguration {
            message: format!(
                "Persisted JWT secret at {:?} contains non-hex characters. Delete the \
                 file and restart to regenerate.",
                path
            ),
        });
    }
    Ok(trimmed.to_string())
}

// SAFE: `OsRng` is infallible in practice on every supported platform; if
// the OS RNG truly fails the server has bigger problems than auth. Panic
// here surfaces the failure loudly during boot rather than silently
// returning a weak key.
#[allow(clippy::expect_used)]
fn generate_hex_secret() -> String {
    let mut buf = [0u8; SECRET_BYTES];
    OsRng
        .try_fill_bytes(&mut buf)
        .expect("OsRng must provide entropy for JWT secret generation");
    hex::encode(buf)
}

fn write_atomic(final_path: &Path, contents: &str) -> Result<()> {
    let tmp_path = {
        let mut s = final_path.as_os_str().to_os_string();
        s.push(".tmp");
        std::path::PathBuf::from(s)
    };

    let mut opts = fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    let mut f = opts.open(&tmp_path).map_err(VectorizerError::IoError)?;
    f.write_all(contents.as_bytes())
        .map_err(VectorizerError::IoError)?;
    f.flush().map_err(VectorizerError::IoError)?;
    // Ensure the bytes hit disk before the rename commits them.
    f.sync_all().map_err(VectorizerError::IoError)?;
    drop(f);

    fs::rename(&tmp_path, final_path).map_err(|e| {
        // Best-effort cleanup — if rename fails, don't leave the .tmp lying around.
        let _ = fs::remove_file(&tmp_path);
        VectorizerError::IoError(e)
    })?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn generates_hex_secret_of_expected_length() {
        let s = generate_hex_secret();
        assert_eq!(s.len(), SECRET_HEX_LEN);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn load_or_generate_creates_file_on_first_call() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("jwt_secret.key");
        assert!(!path.exists());

        let secret = load_or_generate(&path).expect("first call generates");
        assert!(path.exists());
        assert_eq!(secret.len(), SECRET_HEX_LEN);
        assert!(secret.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn load_or_generate_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("jwt_secret.key");

        let first = load_or_generate(&path).unwrap();
        let second = load_or_generate(&path).unwrap();
        assert_eq!(
            first, second,
            "second call must reuse the persisted secret, not regenerate"
        );
    }

    #[test]
    fn load_or_generate_produces_different_secrets_across_fresh_dirs() {
        let tmp_a = TempDir::new().unwrap();
        let tmp_b = TempDir::new().unwrap();
        let a = load_or_generate(&tmp_a.path().join("jwt_secret.key")).unwrap();
        let b = load_or_generate(&tmp_b.path().join("jwt_secret.key")).unwrap();
        assert_ne!(
            a, b,
            "two independent first-boots must yield different secrets"
        );
    }

    #[test]
    fn deleting_file_causes_regeneration_with_fresh_value() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("jwt_secret.key");

        let original = load_or_generate(&path).unwrap();
        fs::remove_file(&path).unwrap();
        let regenerated = load_or_generate(&path).unwrap();

        assert_ne!(
            original, regenerated,
            "deleting the file and re-running must produce a new secret"
        );
    }

    #[test]
    fn corrupt_short_file_returns_clean_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("jwt_secret.key");
        fs::write(&path, "deadbeef").unwrap(); // 8 chars, way under 128

        let err = load_or_generate(&path).unwrap_err();
        let message = match err {
            VectorizerError::InvalidConfiguration { message } => message,
            other => panic!("expected InvalidConfiguration, got {other:?}"),
        };
        assert!(message.contains("8 chars"));
        assert!(message.contains("expected 128"));
    }

    #[test]
    fn corrupt_non_hex_file_returns_clean_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("jwt_secret.key");
        // Correct length but contains a `z`, which is not a hex digit.
        let bad = "z".repeat(SECRET_HEX_LEN);
        fs::write(&path, &bad).unwrap();

        let err = load_or_generate(&path).unwrap_err();
        let message = match err {
            VectorizerError::InvalidConfiguration { message } => message,
            other => panic!("expected InvalidConfiguration, got {other:?}"),
        };
        assert!(message.contains("non-hex"));
    }

    #[test]
    fn trailing_newline_is_tolerated() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("jwt_secret.key");
        let clean = "a".repeat(SECRET_HEX_LEN);
        fs::write(&path, format!("{clean}\n")).unwrap();

        let loaded = load_or_generate(&path).unwrap();
        assert_eq!(loaded, clean);
    }

    #[cfg(unix)]
    #[test]
    fn persisted_file_is_mode_0o600_on_posix() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("jwt_secret.key");
        load_or_generate(&path).unwrap();

        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "JWT secret file must be 0o600 on POSIX");
    }
}
