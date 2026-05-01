//! `Retry-After` header parser tests for the Rust SDK
//! (issue #263, phase9 §7).
//!
//! The full retry loop is exercised end-to-end at the server level
//! by `crates/vectorizer-server/tests/backpressure_429.rs`; here we
//! only lock in the value-parsing edges (default, cap, zero, junk)
//! that determine how aggressively the SDK backs off.
//!
//! These constants are kept in sync with `http_transport.rs`:
//!   - missing/unparseable header → 1 s default
//!   - `Retry-After: 0` → 1 s default (never busy-loop)
//!   - cap at 30 s so a misconfigured server can't pin the client

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use vectorizer_sdk::http_transport::parse_retry_after_secs;

#[test]
fn missing_header_returns_default() {
    assert_eq!(parse_retry_after_secs(None), Duration::from_secs(1));
}

#[test]
fn empty_or_whitespace_returns_default() {
    assert_eq!(parse_retry_after_secs(Some("")), Duration::from_secs(1));
    assert_eq!(parse_retry_after_secs(Some("   ")), Duration::from_secs(1));
}

#[test]
fn zero_returns_default_to_avoid_busy_loop() {
    assert_eq!(parse_retry_after_secs(Some("0")), Duration::from_secs(1));
}

#[test]
fn unparseable_string_returns_default() {
    assert_eq!(
        parse_retry_after_secs(Some("not-a-number")),
        Duration::from_secs(1),
    );
}

#[test]
fn small_values_pass_through_verbatim() {
    assert_eq!(parse_retry_after_secs(Some("3")), Duration::from_secs(3));
    assert_eq!(parse_retry_after_secs(Some("7")), Duration::from_secs(7));
    assert_eq!(parse_retry_after_secs(Some(" 5 ")), Duration::from_secs(5));
}

#[test]
fn large_values_are_capped_at_30s() {
    // The 30 s cap keeps a misconfigured server from pinning the
    // client into a half-hour sleep. If this test ever flips, audit
    // RETRY_AFTER_MAX_SECS in http_transport.rs first.
    assert_eq!(
        parse_retry_after_secs(Some("3600")),
        Duration::from_secs(30),
    );
    assert_eq!(parse_retry_after_secs(Some("31")), Duration::from_secs(30));
}
