//! Phase17 — dashboard session + CSRF cookie helpers.
//!
//! Three guarantees this module enforces for every cookie the server emits
//! to the dashboard:
//!
//! 1. `HttpOnly` on the JWT session cookie — JS injection cannot read it via
//!    `document.cookie`.
//! 2. `Secure` — the cookie never travels over plain HTTP, unless the
//!    operator explicitly opted into `auth.cookies.insecure_dev=true` AND
//!    is bound to a loopback address (the boot guard in
//!    `routing.rs::start` rejects the combination with `0.0.0.0`).
//! 3. `SameSite=Strict` — the cookie is never attached to a cross-site
//!    request. Combined with the CSRF token check on mutating routes, this
//!    closes the cross-origin write surface.
//!
//! The CSRF token is a 32-byte cryptographically random value, hex-encoded
//! into a 64-character string. It is emitted in a NON-`HttpOnly` cookie
//! (`XSRF-TOKEN`) so the SPA can read it from `document.cookie` and echo
//! it back in the `X-CSRF-Token` header — the server-side CSRF middleware
//! validates the header against the token bound to the request's JWT
//! session.

use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use rand::TryRngCore;
use vectorizer::auth::CookieConfig;

/// Cookie name for the HttpOnly JWT session cookie issued at login.
pub const SESSION_COOKIE_NAME: &str = "vectorizer_session";

/// Cookie name for the readable CSRF token cookie issued alongside the
/// session cookie. Matches the `XSRF-TOKEN` / `X-CSRF-Token` convention
/// shared by Angular, Axios, and Spring.
pub const CSRF_COOKIE_NAME: &str = "XSRF-TOKEN";

/// Header the SPA uses to echo the CSRF token on every mutating request.
pub const CSRF_HEADER_NAME: &str = "X-CSRF-Token";

/// Build the value of a `Set-Cookie` header for a session-scoped cookie.
///
/// `http_only=true` is used for the JWT cookie (JS must not read it).
/// `http_only=false` is used for the CSRF cookie (SPA reads it via
/// `document.cookie` to echo on subsequent requests).
///
/// `Secure` is included unconditionally except when `config.insecure_dev`
/// is `true` — the operator has explicitly opted into plain-HTTP local dev
/// and the boot guard in `routing.rs::start` has already verified the
/// host is not `0.0.0.0`.
pub fn build_session_cookie(
    name: &str,
    value: &str,
    max_age_secs: u64,
    http_only: bool,
    config: &CookieConfig,
) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(6);
    parts.push(format!("{name}={value}"));
    parts.push("Path=/".to_string());
    parts.push(format!("Max-Age={max_age_secs}"));
    parts.push("SameSite=Strict".to_string());
    if http_only {
        parts.push("HttpOnly".to_string());
    }
    if !config.insecure_dev {
        parts.push("Secure".to_string());
    }
    parts.join("; ")
}

/// Build a `Set-Cookie` header value that expires the named cookie
/// immediately. Used by `/auth/logout` to scrub the session + CSRF
/// cookies from the browser when blacklisting the JWT.
///
/// `Max-Age=0` plus `Expires=Thu, 01 Jan 1970 00:00:00 GMT` ensures
/// every browser drops the cookie regardless of which attribute it
/// honors first.
pub fn build_session_cookie_clear(name: &str, http_only: bool, config: &CookieConfig) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(7);
    parts.push(format!("{name}="));
    parts.push("Path=/".to_string());
    parts.push("Max-Age=0".to_string());
    parts.push("Expires=Thu, 01 Jan 1970 00:00:00 GMT".to_string());
    parts.push("SameSite=Strict".to_string());
    if http_only {
        parts.push("HttpOnly".to_string());
    }
    if !config.insecure_dev {
        parts.push("Secure".to_string());
    }
    parts.join("; ")
}

/// Append a `Set-Cookie` header. `axum::http::HeaderMap::insert` would
/// overwrite earlier session cookies — we need `append` so the JWT cookie
/// and the CSRF cookie are both emitted on the same response.
pub fn append_set_cookie(headers: &mut HeaderMap, value: String) {
    if let Ok(hv) = axum::http::HeaderValue::from_str(&value) {
        headers.append(SET_COOKIE, hv);
    }
}

/// Generate a new 32-byte random CSRF token, hex-encoded (64 ASCII chars).
///
/// `rand::rngs::OsRng` pulls from the OS CSPRNG. The hex encoding keeps
/// the token cookie-safe (no need for percent-encoding) and lets the SPA
/// pass it through `document.cookie` and `X-CSRF-Token` without any
/// escaping logic.
pub fn generate_csrf_token() -> String {
    let mut bytes = [0u8; 32];
    // SAFETY: `OsRng` reads from the OS CSPRNG; failure here would
    // indicate kernel entropy starvation, in which case minting a
    // deterministic CSRF token would silently weaken security. Panic
    // is the correct response — abort the request handler instead of
    // continuing with a predictable token.
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .expect("OS CSPRNG must succeed when minting a CSRF token");
    hex::encode(bytes)
}

/// Phase17 boot guard. Returns `Err` if the operator left
/// `auth.cookies.insecure_dev=true` while binding to a non-loopback
/// host — most importantly `0.0.0.0`, which would expose dashboard
/// sessions to plain-HTTP harvesting on every interface. The boot path
/// in `routing.rs::start` calls this before any listener is opened.
///
/// `127.0.0.1` and `::1` are the only allowed hosts when `insecure_dev`
/// is set. Anything else (including any LAN IP) is rejected.
pub fn validate_dev_mode_against_host(host: &str, config: &CookieConfig) -> Result<(), String> {
    if !config.insecure_dev {
        return Ok(());
    }
    let normalized = host.trim().to_ascii_lowercase();
    let is_loopback = matches!(normalized.as_str(), "127.0.0.1" | "::1" | "localhost");
    if is_loopback {
        Ok(())
    } else {
        Err(format!(
            "auth.cookies.insecure_dev=true is only permitted on a loopback bind \
             (127.0.0.1, ::1, localhost); refusing to start with host={host}"
        ))
    }
}

/// Read a cookie value from the inbound `Cookie:` header. Returns the
/// first matching value, or `None` if absent. Whitespace around `=` is
/// tolerated; quoted values are returned verbatim (the server only emits
/// unquoted values, but a permissive read keeps drift from breaking the
/// CSRF middleware).
pub fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some((k, v)) = pair.split_once('=') {
            if k.trim() == name {
                return Some(v.trim());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(insecure_dev: bool) -> CookieConfig {
        CookieConfig { insecure_dev }
    }

    #[test]
    fn session_cookie_carries_all_four_attributes_in_production() {
        let value = build_session_cookie(
            "vectorizer_session",
            "jwt.value.here",
            3600,
            true,
            &cfg(false),
        );
        assert!(value.starts_with("vectorizer_session=jwt.value.here"));
        assert!(value.contains("Path=/"));
        assert!(value.contains("Max-Age=3600"));
        assert!(value.contains("SameSite=Strict"));
        assert!(value.contains("HttpOnly"));
        assert!(value.contains("Secure"));
    }

    #[test]
    fn session_cookie_omits_secure_in_insecure_dev() {
        let value = build_session_cookie("vectorizer_session", "jwt", 3600, true, &cfg(true));
        assert!(value.contains("HttpOnly"));
        assert!(value.contains("SameSite=Strict"));
        assert!(!value.contains("Secure"));
    }

    #[test]
    fn csrf_cookie_omits_http_only_so_spa_can_read_it() {
        let value = build_session_cookie("XSRF-TOKEN", "csrf", 3600, false, &cfg(false));
        assert!(!value.contains("HttpOnly"));
        assert!(value.contains("SameSite=Strict"));
        assert!(value.contains("Secure"));
    }

    #[test]
    fn clear_cookie_carries_max_age_zero_and_expires_epoch() {
        let value = build_session_cookie_clear("vectorizer_session", true, &cfg(false));
        assert!(value.contains("Max-Age=0"));
        assert!(value.contains("Expires=Thu, 01 Jan 1970 00:00:00 GMT"));
        assert!(value.contains("HttpOnly"));
        assert!(value.contains("Secure"));
    }

    #[test]
    fn generated_csrf_token_is_64_hex_chars() {
        let t = generate_csrf_token();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
        // Two consecutive calls produce different values (probabilistic
        // but with negligible collision probability for 256-bit tokens).
        assert_ne!(t, generate_csrf_token());
    }

    #[test]
    fn dev_mode_passes_when_flag_off() {
        assert!(validate_dev_mode_against_host("0.0.0.0", &cfg(false)).is_ok());
    }

    #[test]
    fn dev_mode_rejects_when_flag_on_and_host_is_0_0_0_0() {
        let err = validate_dev_mode_against_host("0.0.0.0", &cfg(true)).expect_err("must reject");
        assert!(err.contains("0.0.0.0"));
        assert!(err.contains("insecure_dev"));
    }

    #[test]
    fn dev_mode_allows_loopback_with_flag_on() {
        assert!(validate_dev_mode_against_host("127.0.0.1", &cfg(true)).is_ok());
        assert!(validate_dev_mode_against_host("::1", &cfg(true)).is_ok());
        assert!(validate_dev_mode_against_host("localhost", &cfg(true)).is_ok());
    }

    #[test]
    fn dev_mode_rejects_lan_ip_with_flag_on() {
        assert!(validate_dev_mode_against_host("192.168.1.10", &cfg(true)).is_err());
    }

    #[test]
    fn read_cookie_extracts_named_value() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            "foo=bar; XSRF-TOKEN=abc123; baz=qux".parse().unwrap(),
        );
        assert_eq!(read_cookie(&headers, "XSRF-TOKEN"), Some("abc123"));
        assert_eq!(read_cookie(&headers, "foo"), Some("bar"));
        assert_eq!(read_cookie(&headers, "missing"), None);
    }
}
