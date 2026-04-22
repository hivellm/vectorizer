//! Server-status + session surface: `health_check`, `login`.
//!
//! Lives in its own module because it doesn't fit any of the
//! domain-specific surfaces (collections / vectors / search / ...)
//! and likely grows to include `/metrics`, `/stats`, and similar
//! observability endpoints in future releases.

use serde::Deserialize;

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::*;

/// Subset of `POST /auth/login`'s response shape — we only need the
/// JWT access token for subsequent requests, the full user record +
/// expiry are ignored here and can be recovered by the caller via
/// the standard `/auth/me` endpoint if needed.
#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_token: String,
}

/// JWT minted by `/auth/login`. Returned by [`VectorizerClient::login`]
/// so callers can hand it back into a new [`VectorizerClient`] via
/// `ClientConfig::api_key` — `HttpTransport::new` sniffs the shape and
/// sends it as `Authorization: Bearer <token>`.
#[derive(Debug, Clone)]
pub struct JwtToken {
    /// Raw encoded JWT (three base64url segments separated by `.`).
    pub access_token: String,
}

impl VectorizerClient {
    /// Check server health.
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.make_request("GET", "/health", None).await?;
        let health: HealthStatus = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse health check response: {e}"))
        })?;
        Ok(health)
    }

    /// Exchange a `(username, password)` pair for a JWT via
    /// `POST /auth/login`. The returned token is **not** retained by
    /// `self` — the transport was built with whatever credential
    /// (or none) was passed at construction. To use the JWT for
    /// subsequent requests, construct a new client with
    /// `ClientConfig::api_key = Some(jwt.access_token)`; the
    /// HTTP transport recognises the three-segment JWT shape and
    /// sends it as `Authorization: Bearer …` rather than
    /// `X-API-Key`.
    ///
    /// When the server runs with `auth.enabled: false` this endpoint
    /// may 404 — callers running against a no-auth dev server don't
    /// need to call `login` at all.
    pub async fn login(&self, username: &str, password: &str) -> Result<JwtToken> {
        let payload = serde_json::json!({
            "username": username,
            "password": password,
        });
        let response = self
            .make_request("POST", "/auth/login", Some(payload))
            .await?;
        let parsed: LoginResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse login response: {e}")))?;
        Ok(JwtToken {
            access_token: parsed.access_token,
        })
    }
}
