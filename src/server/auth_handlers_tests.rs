//! Unit tests for auth handlers — extracted from `src/server/auth_handlers.rs` via the
//! `#[path]` attribute (phase3 monolith test-extraction).

use super::*;

#[test]
fn test_login_request_deserialization() {
    let json = r#"{"username": "test", "password": "pass123"}"#;
    let request: LoginRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.username, "test");
    assert_eq!(request.password, "pass123");
}

#[test]
fn test_create_api_key_request_deserialization() {
    let json = r#"{"name": "my-key", "permissions": ["read", "write"]}"#;
    let request: CreateApiKeyRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.name, "my-key");
    assert_eq!(request.permissions.len(), 2);
}

#[test]
fn test_login_response_serialization() {
    let response = LoginResponse {
        access_token: "token123".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user: UserInfo {
            user_id: "user1".to_string(),
            username: "testuser".to_string(),
            roles: vec!["User".to_string()],
        },
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("token123"));
    assert!(json.contains("Bearer"));
}

#[test]
fn persist_first_run_credentials_writes_contents() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path =
        persist_first_run_credentials(tmp.path(), "root", "correct-horse-battery-staple", true)
            .expect("persist succeeded");

    let body = std::fs::read_to_string(&path).expect("read credentials");
    assert!(body.contains("username=root"));
    assert!(body.contains("password=correct-horse-battery-staple"));
    assert!(body.contains("Generated: true"));
    assert!(
        path.ends_with(".root_credentials"),
        "expected file named .root_credentials, got {:?}",
        path
    );
}

#[cfg(unix)]
#[test]
fn persist_first_run_credentials_sets_0600_on_unix() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = persist_first_run_credentials(tmp.path(), "root", "pw", false).expect("persist");
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "expected 0o600, got {:o}", mode);
}

#[test]
fn persist_first_run_credentials_creates_parent_dir_when_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Nested path that does not yet exist — helper must mkdir -p.
    let nested = tmp.path().join("a").join("b");
    let path = persist_first_run_credentials(&nested, "root", "pw", false)
        .expect("persist created parents");
    assert!(path.exists());
}

/// Build an AuthHandlerState with a real JwtManager seeded with the given
/// roles so we can mint tokens for the admin-gate tests below.
async fn test_state_with_user_roles(roles: Vec<Role>) -> (AuthHandlerState, String) {
    use crate::auth::{AuthConfig, AuthManager, Secret};

    let config = AuthConfig {
        jwt_secret: Secret::new("z".repeat(64)),
        enabled: true,
        ..AuthConfig::default()
    };
    let manager = Arc::new(AuthManager::new(config).expect("valid auth config"));
    let token = manager
        .generate_jwt("user-under-test", "tester", roles)
        .expect("generate_jwt");
    (AuthHandlerState::new(manager), token)
}

#[tokio::test]
async fn require_admin_for_rest_returns_ok_when_auth_globally_disabled() {
    let headers = axum::http::HeaderMap::new();
    // Backward compat path: auth disabled = every request is effectively admin.
    require_admin_for_rest(&None, &headers)
        .await
        .expect("no-auth mode must allow the call through");
}

#[tokio::test]
async fn require_admin_for_rest_returns_401_without_any_auth_header() {
    let (state, _token) = test_state_with_user_roles(vec![Role::Admin]).await;
    let headers = axum::http::HeaderMap::new();
    let err = require_admin_for_rest(&Some(state), &headers)
        .await
        .expect_err("missing header must 401");
    assert_eq!(
        err.status_code,
        axum::http::StatusCode::UNAUTHORIZED.as_u16()
    );
}

#[tokio::test]
async fn require_admin_for_rest_returns_403_for_non_admin_token() {
    let (state, token) = test_state_with_user_roles(vec![Role::User]).await;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    let err = require_admin_for_rest(&Some(state), &headers)
        .await
        .expect_err("non-admin must 403");
    assert_eq!(err.status_code, axum::http::StatusCode::FORBIDDEN.as_u16());
}

#[tokio::test]
async fn require_admin_for_rest_returns_ok_for_admin_token() {
    let (state, token) = test_state_with_user_roles(vec![Role::Admin]).await;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    require_admin_for_rest(&Some(state), &headers)
        .await
        .expect("admin token must pass");
}
