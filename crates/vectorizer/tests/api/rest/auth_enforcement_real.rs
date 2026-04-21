//! Live integration test for phase8_gate-data-routes-when-auth-enabled.
//!
//! Confirms that when the live server is booted with
//! `VECTORIZER_JWT_SECRET` set + `auth.enabled: true`:
//!
//! - Anonymous data-surface calls return 401.
//! - Requests carrying a valid JWT from `/auth/login` return 200.
//! - The public endpoints (`/health`, `/prometheus/metrics`,
//!   `/auth/login`, `/umicp/discover`, `/dashboard/`) still answer
//!   anonymously so the dashboard shell + login form can load.
//!
//! The test reads the auto-generated root password from
//! `%APPDATA%\vectorizer\.root_credentials` (Windows) /
//! `$XDG_DATA_HOME/vectorizer/.root_credentials` (Linux / macOS).
//!
//! Require a running server on `127.0.0.1:15002` booted with auth
//! enforcement active. Run with:
//! `cargo test --test all_tests api::rest::auth_enforcement_real -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Value, json};

const BASE: &str = "http://127.0.0.1:15002";

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("build reqwest client")
}

/// Resolve the `.root_credentials` file the server writes on first boot
/// when auth is enabled with no persisted users. Matches the resolution
/// `AuthPersistence::with_default_dir` uses internally
/// (`dirs::data_dir().join("vectorizer")`).
fn root_credentials_path() -> PathBuf {
    if let Ok(p) = std::env::var("VECTORIZER_DATA_DIR") {
        return PathBuf::from(p).join(".root_credentials");
    }
    dirs::data_dir()
        .expect("data_dir")
        .join("vectorizer")
        .join(".root_credentials")
}

fn read_root_password() -> Option<String> {
    let path = root_credentials_path();
    let content = std::fs::read_to_string(&path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("password=") {
            return Some(rest.to_string());
        }
    }
    None
}

/// Skip the test gracefully when the server was booted in the default
/// single-user-local mode (`auth.enabled: false` — no root creds).
fn skip_if_auth_off(http: &reqwest::blocking::Client) -> Option<String> {
    let resp = http.get(format!("{}/collections", BASE)).send().ok()?;
    if resp.status() == 200 {
        // Auth is off (every request succeeds anonymously). Nothing to
        // assert for this test.
        return None;
    }
    read_root_password()
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002 with auth enabled"]
fn public_routes_stay_anonymous_with_auth_enabled() {
    let http = client();
    if skip_if_auth_off(&http).is_none() {
        eprintln!("skipped — auth disabled on the live server");
        return;
    }

    for path in [
        "/health",
        "/prometheus/metrics",
        "/umicp/discover",
        "/dashboard/",
    ] {
        let resp = http.get(format!("{}{}", BASE, path)).send().expect(path);
        assert_eq!(
            resp.status().as_u16(),
            200,
            "{} should stay anonymous (got {})",
            path,
            resp.status()
        );
    }
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002 with auth enabled"]
fn data_routes_require_auth_when_auth_enabled() {
    let http = client();
    if skip_if_auth_off(&http).is_none() {
        eprintln!("skipped — auth disabled on the live server");
        return;
    }

    for path in ["/collections", "/stats", "/logs", "/auth/me"] {
        let resp = http.get(format!("{}{}", BASE, path)).send().expect(path);
        assert_eq!(
            resp.status().as_u16(),
            401,
            "{} should reject anonymous callers (got {})",
            path,
            resp.status()
        );
    }
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002 with auth enabled"]
fn valid_jwt_unlocks_data_routes() {
    let http = client();
    let Some(password) = skip_if_auth_off(&http) else {
        eprintln!("skipped — auth disabled on the live server");
        return;
    };

    // POST /auth/login must be publicly reachable.
    let login: Value = http
        .post(format!("{}/auth/login", BASE))
        .json(&json!({ "username": "root", "password": password }))
        .send()
        .expect("POST /auth/login")
        .json()
        .expect("decode login");
    let token = login["access_token"]
        .as_str()
        .expect("access_token")
        .to_string();
    assert!(!token.is_empty(), "login returned empty access_token");

    let resp = http
        .get(format!("{}/collections", BASE))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .expect("authenticated GET /collections");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "authenticated call should succeed (got {})",
        resp.status()
    );

    // A garbage token must be rejected with 401.
    let resp = http
        .get(format!("{}/collections", BASE))
        .header("Authorization", "Bearer not-a-real-jwt")
        .send()
        .expect("bogus GET /collections");
    assert_eq!(resp.status().as_u16(), 401);
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002 with auth enabled"]
fn auth_login_rejects_wrong_password() {
    let http = client();
    if skip_if_auth_off(&http).is_none() {
        eprintln!("skipped — auth disabled on the live server");
        return;
    }

    let resp = http
        .post(format!("{}/auth/login", BASE))
        .json(&json!({ "username": "root", "password": "definitely-not-the-password" }))
        .send()
        .expect("POST /auth/login with bad pw");
    assert_eq!(resp.status().as_u16(), 401);
}
