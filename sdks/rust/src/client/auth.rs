//! Authentication surface.
//!
//! Covers the `/auth/*` REST endpoints: current-user info, logout,
//! token refresh, password validation, API key management (CRUD), user
//! management (admin), and the phase15 admin endpoints: key rotation,
//! scoped key creation, token introspection, and audit-log query.
//!
//! All write operations that require `Role::Admin` are marked in
//! their doc comments. Calling them without the right role will
//! return a [`VectorizerError`] wrapping the server's 403 response.

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::{
    ApiKey, ApiKeyUsageReport, ApiKeyView, AuditEntry, AuditQuery, CreateApiKeyRequest,
    CreateScopedApiKeyRequest, CreateUserRequest, JwtToken, PasswordPolicyReport, RotatedKey,
    TokenIntrospection, UpdateApiKeyPermissionsRequest, User,
};

impl VectorizerClient {
    /// Return the authenticated user's claims.
    ///
    /// Calls `GET /auth/me`. Requires a valid JWT / API key on the
    /// configured transport.
    pub async fn me(&self) -> Result<User> {
        let response = self.make_request("GET", "/auth/me", None).await?;
        serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse me response: {e}")))
    }

    /// Invalidate the current session token.
    ///
    /// Calls `POST /auth/logout`. The token is blacklisted until its
    /// natural expiry.
    pub async fn logout(&self) -> Result<()> {
        self.make_request("POST", "/auth/logout", None).await?;
        Ok(())
    }

    /// Exchange the current token for a fresh one with an extended TTL.
    ///
    /// Calls `POST /auth/refresh`.
    pub async fn refresh_token(&self) -> Result<JwtToken> {
        let response = self
            .make_request("POST", "/auth/refresh", Some(serde_json::json!({})))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse refresh_token response: {e}"))
        })
    }

    /// Validate a password against the server's password policy without
    /// creating an account.
    ///
    /// Calls `POST /auth/validate-password` with `{password}`.
    pub async fn validate_password(&self, password: &str) -> Result<PasswordPolicyReport> {
        let payload = serde_json::json!({ "password": password });
        let response = self
            .make_request("POST", "/auth/validate-password", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse validate_password response: {e}"))
        })
    }

    /// Create a new API key for the calling user.
    ///
    /// Calls `POST /auth/keys`. The `api_key` field in the returned
    /// [`ApiKey`] is only present at creation time — store it securely.
    pub async fn create_api_key(&self, request: CreateApiKeyRequest) -> Result<ApiKey> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!("Failed to serialize create_api_key request: {e}"))
        })?;
        let response = self
            .make_request("POST", "/auth/keys", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse create_api_key response: {e}"))
        })
    }

    /// List the API keys belonging to the calling user.
    ///
    /// Calls `GET /auth/keys`. The `api_key` field is omitted in
    /// list responses for security.
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        let response = self.make_request("GET", "/auth/keys", None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_api_keys response: {e}"))
        })?;
        let arr = val
            .get("keys")
            .and_then(|k| k.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse api key entry: {e}"))
                })
            })
            .collect()
    }

    /// Revoke an API key by id.
    ///
    /// Calls `DELETE /auth/keys/{id}`. Admin can revoke any key;
    /// regular users can only revoke their own.
    pub async fn revoke_api_key(&self, id: &str) -> Result<()> {
        self.make_request("DELETE", &format!("/auth/keys/{id}"), None)
            .await?;
        Ok(())
    }

    /// Create a new user (admin only).
    ///
    /// Calls `POST /auth/users`. Requires `Role::Admin`.
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!("Failed to serialize create_user request: {e}"))
        })?;
        let response = self
            .make_request("POST", "/auth/users", Some(payload))
            .await?;
        // Server returns {user_id, username, roles, message}
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse create_user response: {e}"))
        })
    }

    /// List all users (admin only).
    ///
    /// Calls `GET /auth/users`. Requires `Role::Admin`.
    pub async fn list_users(&self) -> Result<Vec<User>> {
        let response = self.make_request("GET", "/auth/users", None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_users response: {e}"))
        })?;
        let arr = val
            .get("users")
            .and_then(|u| u.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse user entry: {e}"))
                })
            })
            .collect()
    }

    /// Delete a user (admin only).
    ///
    /// Calls `DELETE /auth/users/{username}`. Requires `Role::Admin`.
    /// The server refuses to delete self or the last admin.
    pub async fn delete_user(&self, username: &str) -> Result<()> {
        self.make_request("DELETE", &format!("/auth/users/{username}"), None)
            .await?;
        Ok(())
    }

    /// Change a user's password.
    ///
    /// Calls `PUT /auth/users/{username}/password` with
    /// `{new_password}`. Admins can change any password; non-admins
    /// must also supply `current_password`.
    pub async fn change_password(&self, username: &str, new_password: &str) -> Result<()> {
        let payload = serde_json::json!({ "new_password": new_password });
        self.make_request(
            "PUT",
            &format!("/auth/users/{username}/password"),
            Some(payload),
        )
        .await?;
        Ok(())
    }

    // ── phase15 admin endpoints ───────────────────────────────────────────────

    /// Atomically rotate an API key (admin only).
    ///
    /// Calls `POST /auth/keys/{id}/rotate` with an empty body.
    /// Returns both the old and new tokens plus a grace window during which
    /// the old token remains valid.
    pub async fn rotate_api_key(&self, id: &str) -> Result<RotatedKey> {
        let response = self
            .make_request(
                "POST",
                &format!("/auth/keys/{id}/rotate"),
                Some(serde_json::json!({})),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse rotate_api_key response: {e}"))
        })
    }

    /// Create an API key with optional per-collection scopes.
    ///
    /// Calls `POST /auth/keys`. When `scopes` is non-empty the key is
    /// restricted to the listed collections. When empty the key is
    /// default-deny on scope-enforced routes.
    pub async fn create_scoped_api_key(
        &self,
        request: CreateScopedApiKeyRequest,
    ) -> Result<ApiKey> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to serialize create_scoped_api_key request: {e}"
            ))
        })?;
        let response = self
            .make_request("POST", "/auth/keys", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse create_scoped_api_key response: {e}"
            ))
        })
    }

    /// Replace `permissions` (and optionally `scopes`) on an existing
    /// API key without rotating the credential. Admin-only.
    ///
    /// Calls `PUT /auth/keys/{id}/permissions`. Returns the updated
    /// key view with the live `usage_count` stamped in.
    ///
    /// `key_hash`, `id`, `user_id`, and `created_at` are immutable —
    /// rotate the key with `rotate_api_key` if those need to change.
    pub async fn update_api_key_permissions(
        &self,
        id: &str,
        request: UpdateApiKeyPermissionsRequest,
    ) -> Result<ApiKeyView> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to serialize update_api_key_permissions request: {e}"
            ))
        })?;
        let response = self
            .make_request(
                "PUT",
                &format!("/auth/keys/{id}/permissions"),
                Some(payload),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse update_api_key_permissions response: {e}"
            ))
        })
    }

    /// Fetch the per-day usage time-series for an API key. Admin-only.
    ///
    /// Calls `GET /auth/keys/{id}/usage?window={days}`. `days` is
    /// clamped server-side to 1..=30; `None` defaults to 7. The
    /// response carries the live key view, the bucket array (oldest
    /// first, including zero-count days), and the window total.
    pub async fn get_api_key_usage(
        &self,
        id: &str,
        window_days: Option<usize>,
    ) -> Result<ApiKeyUsageReport> {
        let path = match window_days {
            Some(n) => format!("/auth/keys/{id}/usage?window={n}"),
            None => format!("/auth/keys/{id}/usage"),
        };
        let response = self.make_request("GET", &path, None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get_api_key_usage response: {e}"))
        })
    }

    /// Introspect a token — RFC 7662.
    ///
    /// Calls `POST /auth/introspect` with `{token}`. Requires authentication
    /// but NOT admin. Returns `active: false` for any unrecognized token.
    pub async fn introspect_token(&self, token: &str) -> Result<TokenIntrospection> {
        let payload = serde_json::json!({ "token": token });
        let response = self
            .make_request("POST", "/auth/introspect", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse introspect_token response: {e}"))
        })
    }

    /// Query the admin audit log (admin only).
    ///
    /// Calls `GET /auth/audit` with optional query parameters.
    /// Returns entries newest-first, bounded by `query.limit` (server default 200).
    pub async fn list_audit_log(&self, query: AuditQuery) -> Result<Vec<AuditEntry>> {
        // Build query-string from non-None fields (values are ASCII identifiers
        // or RFC-3339 timestamps — no special encoding needed beyond standard).
        let mut parts: Vec<String> = Vec::new();
        if let Some(actor) = &query.actor {
            parts.push(format!("actor={actor}"));
        }
        if let Some(action) = &query.action {
            parts.push(format!("action={action}"));
        }
        if let Some(since) = &query.since {
            parts.push(format!("since={since}"));
        }
        if let Some(until) = &query.until {
            parts.push(format!("until={until}"));
        }
        if let Some(limit) = query.limit {
            parts.push(format!("limit={limit}"));
        }
        let path = if parts.is_empty() {
            "/auth/audit".to_string()
        } else {
            format!("/auth/audit?{}", parts.join("&"))
        };
        let response = self.make_request("GET", &path, None).await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_audit_log response: {e}"))
        })?;
        let arr = val
            .get("entries")
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse audit entry: {e}"))
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use crate::models::{
        ApiKey, CreateApiKeyRequest, CreateUserRequest, JwtToken, PasswordPolicyReport, User,
    };

    #[test]
    fn user_deserializes() {
        let raw = json!({
            "user_id": "u-1",
            "username": "alice",
            "roles": ["Admin", "User"]
        });
        let u: User = serde_json::from_value(raw).unwrap();
        assert_eq!(u.username, "alice");
        assert_eq!(u.roles.len(), 2);
    }

    #[test]
    fn user_round_trip() {
        let u = User {
            user_id: "u-42".into(),
            username: "bob".into(),
            roles: vec!["User".into()],
        };
        let serialized = serde_json::to_value(&u).unwrap();
        let parsed: User = serde_json::from_value(serialized).unwrap();
        assert_eq!(parsed.user_id, "u-42");
    }

    #[test]
    fn jwt_token_deserializes() {
        let raw = json!({
            "access_token": "eyJ...",
            "token_type": "Bearer",
            "expires_in": 3600
        });
        let t: JwtToken = serde_json::from_value(raw).unwrap();
        assert_eq!(t.token_type, "Bearer");
        assert_eq!(t.expires_in, 3600);
    }

    #[test]
    fn password_policy_report_valid() {
        let raw = json!({
            "valid": true,
            "errors": [],
            "strength": 80,
            "strength_label": "Strong"
        });
        let r: PasswordPolicyReport = serde_json::from_value(raw).unwrap();
        assert!(r.valid);
        assert_eq!(r.strength, 80);
        assert_eq!(r.strength_label, "Strong");
    }

    #[test]
    fn password_policy_report_invalid() {
        let raw = json!({
            "valid": false,
            "errors": ["too short", "needs uppercase"],
            "strength": 10,
            "strength_label": "Very Weak"
        });
        let r: PasswordPolicyReport = serde_json::from_value(raw).unwrap();
        assert!(!r.valid);
        assert_eq!(r.errors.len(), 2);
    }

    #[test]
    fn create_api_key_request_serializes() {
        let req = CreateApiKeyRequest {
            name: "ci-bot".into(),
            permissions: vec!["Read".into(), "Write".into()],
            expires_in: Some(86400),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["name"], "ci-bot");
        assert_eq!(v["expires_in"], 86400);
    }

    #[test]
    fn api_key_deserializes_creation_response() {
        let raw = json!({
            "id": "key-1",
            "name": "ci-bot",
            "permissions": ["Read"],
            "api_key": "sk-abc123",
            "created_at": 1714608000u64,
            "active": true,
            "warning": "Store this key securely"
        });
        let k: ApiKey = serde_json::from_value(raw).unwrap();
        assert_eq!(k.id, "key-1");
        assert_eq!(k.api_key.as_deref(), Some("sk-abc123"));
        assert!(k.active);
    }

    #[test]
    fn api_key_deserializes_list_response() {
        // api_key is omitted in list responses
        let raw = json!({
            "id": "key-2",
            "name": "deploy",
            "permissions": ["Write"],
            "created_at": 1714608000u64,
            "active": false
        });
        let k: ApiKey = serde_json::from_value(raw).unwrap();
        assert!(k.api_key.is_none());
        assert!(!k.active);
    }

    #[test]
    fn create_user_request_serializes() {
        let req = CreateUserRequest {
            username: "charlie".into(),
            password: "P@ssw0rd!".into(),
            roles: vec!["User".into()],
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["username"], "charlie");
    }

    // ── phase15 auth admin ────────────────────────────────────────────────────

    use crate::models::{
        AuditEntry, AuditQuery, CreateScopedApiKeyRequest, RotatedKey, TokenIntrospection,
        TokenScope,
    };

    #[test]
    fn rotated_key_deserializes() {
        let raw = json!({
            "old_key_id": "key-old",
            "new_key_id": "key-new",
            "new_token": "sk-new-token",
            "grace_until": 1714694400u64
        });
        let r: RotatedKey = serde_json::from_value(raw).unwrap();
        assert_eq!(r.old_key_id, "key-old");
        assert_eq!(r.new_key_id, "key-new");
        assert_eq!(r.grace_until, 1714694400);
    }

    #[test]
    fn create_scoped_api_key_request_serializes() {
        let req = CreateScopedApiKeyRequest {
            name: "scoped-key".into(),
            permissions: vec!["Read".into()],
            expires_in: Some(3600),
            scopes: vec![TokenScope {
                collection: "my-col".into(),
                permissions: vec!["read".into(), "write".into()],
            }],
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["name"], "scoped-key");
        assert_eq!(v["scopes"][0]["collection"], "my-col");
        assert_eq!(v["scopes"][0]["permissions"][1], "write");
    }

    #[test]
    fn token_introspection_active_deserializes() {
        let raw = json!({
            "active": true,
            "sub": "user-1",
            "exp": 1714694400u64,
            "username": "alice"
        });
        let t: TokenIntrospection = serde_json::from_value(raw).unwrap();
        assert!(t.active);
        assert_eq!(t.sub.as_deref(), Some("user-1"));
        assert_eq!(t.username.as_deref(), Some("alice"));
        assert!(t.scope.is_none());
    }

    #[test]
    fn token_introspection_inactive_deserializes() {
        let raw = json!({ "active": false });
        let t: TokenIntrospection = serde_json::from_value(raw).unwrap();
        assert!(!t.active);
        assert!(t.sub.is_none());
        assert!(t.exp.is_none());
    }

    #[test]
    fn audit_entry_deserializes() {
        let raw = json!({
            "actor": "admin",
            "action": "rotate_api_key",
            "target": "key-1",
            "at": "2026-05-02T12:00:00Z",
            "correlation_id": "corr-abc"
        });
        let e: AuditEntry = serde_json::from_value(raw).unwrap();
        assert_eq!(e.actor, "admin");
        assert_eq!(e.action, "rotate_api_key");
        assert_eq!(e.correlation_id.as_deref(), Some("corr-abc"));
    }

    #[test]
    fn audit_entry_without_correlation_id_deserializes() {
        let raw = json!({
            "actor": "admin",
            "action": "create_api_key",
            "target": "key-2",
            "at": "2026-05-02T13:00:00Z"
        });
        let e: AuditEntry = serde_json::from_value(raw).unwrap();
        assert!(e.correlation_id.is_none());
    }

    #[test]
    fn audit_query_serializes_with_defaults() {
        let q = AuditQuery::default();
        let v = serde_json::to_value(&q).unwrap();
        // All fields should be absent (skip_serializing_if = None).
        assert_eq!(v, json!({}));
    }

    #[test]
    fn audit_query_serializes_partial() {
        let q = AuditQuery {
            actor: Some("admin".into()),
            limit: Some(50),
            ..Default::default()
        };
        let v = serde_json::to_value(&q).unwrap();
        assert_eq!(v["actor"], "admin");
        assert_eq!(v["limit"], 50);
        assert!(v.get("action").is_none());
    }
}
