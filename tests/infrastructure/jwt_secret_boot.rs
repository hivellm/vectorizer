//! End-to-end test for the auto-generated JWT secret feature.
//!
//! Exercises the same code path the server's boot logic uses: first
//! `load_or_generate` call creates the file, a subsequent call reuses it,
//! and the resulting hex string is accepted by `AuthManager::new` with
//! `enabled = true` and can round-trip a JWT token.

use tempfile::TempDir;
use vectorizer::auth::jwt_secret::load_or_generate;
use vectorizer::auth::roles::Role;
use vectorizer::auth::{AuthConfig, AuthManager, Secret};

#[test]
fn first_boot_generates_key_and_second_boot_reuses_it() {
    let tmp = TempDir::new().expect("tempdir");
    let key_path = tmp.path().join("jwt_secret.key");

    // First boot: file does not exist yet.
    assert!(!key_path.exists());
    let first = load_or_generate(&key_path).expect("first boot persists a fresh key");
    assert!(key_path.exists(), "file must exist after first boot");
    assert_eq!(first.len(), 128, "hex-encoded 64-byte key is 128 chars");

    // Second boot: same path, same server instance would read the persisted key.
    let second = load_or_generate(&key_path).expect("second boot reuses the persisted key");
    assert_eq!(first, second, "restart must not rotate the JWT secret");
}

#[test]
fn generated_secret_is_accepted_by_auth_manager_and_signs_a_valid_token() {
    let tmp = TempDir::new().expect("tempdir");
    let key_path = tmp.path().join("jwt_secret.key");

    let secret = load_or_generate(&key_path).expect("key generation");

    let config = AuthConfig {
        jwt_secret: Secret::new(secret),
        enabled: true,
        ..AuthConfig::default()
    };
    let manager =
        AuthManager::new(config).expect("AuthConfig::validate passes with a 128-char hex key");

    // Round-trip a token to prove the signing key is coherent end-to-end.
    let token = manager
        .generate_jwt("user-e2e", "integration", vec![Role::User])
        .expect("generate_jwt");
    let claims = manager.validate_jwt(&token).expect("validate_jwt");
    assert_eq!(claims.user_id, "user-e2e");
    assert_eq!(claims.username, "integration");
}
