//! Unit tests for AuthConfig + AuthManager — extracted from `src/auth/mod.rs` via the
//! `#[path]` attribute (phase3 monolith test-extraction).

use super::*;

/// Build a valid `AuthConfig` for unit tests: injects a deterministic
/// 64-char secret that satisfies `validate()` without leaking into the
/// release build. Do NOT use this value in any production config.
fn test_config() -> AuthConfig {
    AuthConfig {
        jwt_secret: Secret::new("t".repeat(64)),
        ..AuthConfig::default()
    }
}

#[tokio::test]
async fn test_auth_manager_creation() {
    let auth_manager = AuthManager::new(test_config()).unwrap();

    assert!(auth_manager.config().enabled);
    assert_eq!(auth_manager.config().jwt_expiration, 3600);
}

#[tokio::test]
async fn test_api_key_creation_and_validation() {
    let auth_manager = AuthManager::new(test_config()).unwrap();

    let (api_key, key_info) = auth_manager
        .create_api_key("user123", "test_key", vec![Permission::Read], None)
        .await
        .unwrap();

    assert_eq!(key_info.user_id, "user123");
    assert_eq!(key_info.name, "test_key");
    assert!(key_info.active);

    let user_claims = auth_manager.validate_api_key(&api_key).await.unwrap();
    assert_eq!(user_claims.user_id, "user123");
}

#[tokio::test]
async fn test_jwt_generation_and_validation() {
    let auth_manager = AuthManager::new(test_config()).unwrap();

    let token = auth_manager
        .generate_jwt("user123", "testuser", vec![Role::Admin])
        .unwrap();

    let claims = auth_manager.validate_jwt(&token).unwrap();
    assert_eq!(claims.user_id, "user123");
    assert_eq!(claims.username, "testuser");
    assert!(claims.roles.contains(&Role::Admin));
}

#[tokio::test]
async fn test_rate_limiting() {
    let mut config = test_config();
    config.rate_limit_per_minute = 2; // Very low limit for testing

    let auth_manager = AuthManager::new(config).unwrap();

    let (api_key, _) = auth_manager
        .create_api_key("user123", "test_key", vec![Permission::Read], None)
        .await
        .unwrap();

    // First two requests should succeed
    auth_manager.validate_api_key(&api_key).await.unwrap();
    auth_manager.validate_api_key(&api_key).await.unwrap();

    // Third request should fail
    let result = auth_manager.validate_api_key(&api_key).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        VectorizerError::RateLimitExceeded { .. }
    ));
}

#[test]
fn validate_rejects_empty_secret() {
    let config = AuthConfig::default(); // empty secret
    let err = config.validate().unwrap_err();
    assert!(
        matches!(err, VectorizerError::InvalidConfiguration { ref message } if message.contains("empty")),
        "expected InvalidConfiguration about empty secret, got {err:?}"
    );
}

#[test]
fn validate_rejects_legacy_default_secret() {
    let config = AuthConfig {
        jwt_secret: Secret::new(LEGACY_INSECURE_DEFAULT_SECRET.to_string()),
        ..AuthConfig::default()
    };
    let err = config.validate().unwrap_err();
    assert!(
        matches!(err, VectorizerError::InvalidConfiguration { ref message } if message.contains("legacy")),
        "expected InvalidConfiguration about legacy default, got {err:?}"
    );
}

#[test]
fn validate_rejects_short_secret() {
    let config = AuthConfig {
        jwt_secret: Secret::new("short-but-nonempty".to_string()),
        ..AuthConfig::default()
    };
    let err = config.validate().unwrap_err();
    assert!(
        matches!(err, VectorizerError::InvalidConfiguration { ref message } if message.contains("chars")),
        "expected InvalidConfiguration about length, got {err:?}"
    );
}

#[test]
fn validate_accepts_valid_secret() {
    let config = AuthConfig {
        jwt_secret: Secret::new("a".repeat(64)),
        ..AuthConfig::default()
    };
    config.validate().unwrap();
}

#[test]
fn validate_skipped_when_disabled() {
    let config = AuthConfig {
        enabled: false,
        ..AuthConfig::default() // empty secret, but enabled=false
    };
    config.validate().unwrap();
}

#[test]
fn manager_new_refuses_legacy_default() {
    let config = AuthConfig {
        jwt_secret: Secret::new(LEGACY_INSECURE_DEFAULT_SECRET.to_string()),
        ..AuthConfig::default()
    };
    assert!(AuthManager::new(config).is_err());
}

/// Regression guard for the `Secret<String>` migration. If any future
/// refactor demotes `jwt_secret` back to a bare `String`, this test
/// fails because the debug output of the config will contain the raw key.
#[test]
fn auth_config_debug_redacts_jwt_secret() {
    let marker = "jwt-sentinel-must-not-leak-into-debug-output";
    let config = AuthConfig {
        jwt_secret: Secret::new(marker.repeat(2)), // > 64 chars for validate()
        ..AuthConfig::default()
    };

    let rendered = format!("{:?}", config);
    assert!(
        !rendered.contains(marker),
        "AuthConfig Debug leaked the jwt_secret: {rendered}"
    );
    assert!(
        rendered.contains("<redacted>"),
        "expected the Secret<String> redaction marker in {rendered}"
    );
}

/// YAML / JSON on-the-wire shape must stay identical to a plain
/// `String` — consumers of `config.yml` and the encrypted auth store
/// should not have to change.
#[test]
fn auth_config_serde_round_trip_preserves_wire_format() {
    let original = AuthConfig {
        jwt_secret: Secret::new("x".repeat(64)),
        ..AuthConfig::default()
    };
    let json = serde_json::to_string(&original).expect("serialize AuthConfig");
    assert!(
        json.contains(&"x".repeat(64)),
        "wire format must contain the plain string: {json}"
    );

    let back: AuthConfig = serde_json::from_str(&json).expect("deserialize AuthConfig");
    assert_eq!(back.jwt_secret.expose_secret(), &"x".repeat(64));
}
