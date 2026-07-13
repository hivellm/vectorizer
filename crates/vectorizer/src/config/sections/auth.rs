//! Authentication configuration data.
//!
//! Plain serde types only — validation and manager wiring live in
//! `crate::auth`, which re-exports [`AuthConfig`] and [`CookieConfig`]
//! from here (see `phase41_architecture-decoupling` §2).

use serde::{Deserialize, Serialize};

use crate::config::secret::Secret;

/// Cookie hardening configuration for dashboard session + CSRF cookies.
///
/// Every dashboard cookie is emitted with `HttpOnly; Secure;
/// SameSite=Strict; Path=/` by default. The `insecure_dev` escape
/// hatch drops only the `Secure` flag so a developer can run the
/// dashboard against plain-HTTP `127.0.0.1`. Boot rejects this flag
/// when binding to `0.0.0.0` — see `VectorizerServer::start`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CookieConfig {
    /// Drop the `Secure` cookie attribute. INTENDED FOR LOCAL DEVELOPMENT
    /// ONLY against plain-HTTP `127.0.0.1`. Boot fails if this is `true`
    /// while binding to `0.0.0.0`.
    #[serde(default)]
    pub insecure_dev: bool,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT secret key for token signing. Must be set explicitly (via config
    /// file or `VECTORIZER_JWT_SECRET` env var) to a value that is at least
    /// `MIN_JWT_SECRET_LEN` characters long and is NOT the legacy default.
    ///
    /// Stored in a redacting `Secret<String>` — `Debug`/`Display` of an
    /// `AuthConfig` will show `<redacted>` for this field.
    pub jwt_secret: Secret<String>,
    /// JWT token expiration time in seconds (default: 3600 = 1 hour)
    pub jwt_expiration: u64,
    /// API key length (default: 32)
    pub api_key_length: usize,
    /// Rate limiting: requests per minute per API key
    pub rate_limit_per_minute: u32,
    /// Rate limiting: requests per hour per API key
    pub rate_limit_per_hour: u32,
    /// Enable authentication (default: true)
    pub enabled: bool,
    /// Cookie hardening configuration for dashboard sessions.
    #[serde(default)]
    pub cookies: CookieConfig,
    /// Local-development convenience: when `true`, the auth middleware
    /// short-circuits with an implicit `local-dev-admin` principal and
    /// every response carries the `X-Vectorizer-Dev-Mode: true`
    /// header. The boot path refuses to start with this flag set on
    /// any non-loopback host (`0.0.0.0`, LAN IPs, etc.) so the only
    /// way to engage it is to bind explicitly to `127.0.0.1`, `::1`,
    /// or `localhost`. Defaults to `false`; absent payloads
    /// deserialize cleanly thanks to `#[serde(default)]`.
    #[serde(default)]
    pub dev_mode_skip_loopback: bool,
}

impl Default for AuthConfig {
    /// Returns a default config with an **empty** JWT secret. Callers must
    /// populate `jwt_secret` before instantiating an `AuthManager`, or boot
    /// will fail via `AuthConfig::validate` / `AuthManager::new`. This is
    /// intentional: shipping a real default value historically caused a known
    /// auth-bypass vulnerability.
    fn default() -> Self {
        Self {
            jwt_secret: Secret::new(String::new()),
            jwt_expiration: 3600, // 1 hour
            api_key_length: 32,
            rate_limit_per_minute: 100,
            rate_limit_per_hour: 1000,
            enabled: true,
            cookies: CookieConfig::default(),
            dev_mode_skip_loopback: false,
        }
    }
}
