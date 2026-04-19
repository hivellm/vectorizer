//! bcrypt-crate compatibility regression guard.
//!
//! Pinned to verify the things our auth handlers depend on across the
//! 0.17 → 0.19 minor bump (and any future Dependabot bump): the
//! algorithm output format, `DEFAULT_COST`, the round-trip semantics
//! of `hash` / `verify`, and the ability to verify a hash produced by
//! any other RFC-compliant bcrypt implementation. If any of these
//! shifts, our existing password records (stored with the older crate
//! version) would fail to verify and lock real users out — that's
//! what this test catches.
//!
//! Source for the external test vector: openwall's `crypt_blowfish`
//! reference suite, also reproduced in the OpenBSD source tree
//! (`/usr/src/lib/libutil/bcrypt.c`).

#[test]
fn default_cost_is_unchanged() {
    // Locking this at 12. If upstream ever lowers the default we want
    // the test to fail loudly so we can reaffirm the choice rather
    // than silently weakening every new password we hash.
    assert_eq!(
        bcrypt::DEFAULT_COST,
        12,
        "bcrypt::DEFAULT_COST drifted — auth handlers expect cost=12",
    );
}

#[test]
fn hash_round_trips_with_default_cost() {
    let password = "phase5-review-candle-bcrypt-bumps";
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).expect("hash should succeed");

    // bcrypt 0.19 keeps the `$2b$<cost>$<22-char-salt><31-char-hash>`
    // format. The cost prefix encodes the work factor — a silent drop
    // here would weaken every new password.
    assert!(
        hash.starts_with("$2b$12$"),
        "hash format changed: expected $2b$12$ prefix, got {hash}",
    );
    assert_eq!(hash.len(), 60, "bcrypt hashes are 60 chars by spec");

    assert!(
        bcrypt::verify(password, &hash).expect("verify should succeed"),
        "round-trip verify failed",
    );
    assert!(
        !bcrypt::verify("wrong-password", &hash).expect("verify should succeed"),
        "wrong password verified true — hash is effectively a no-op",
    );
}

#[test]
fn verifies_external_reference_vector() {
    // Cross-implementation vector: the password "U*U" hashed with
    // cost=5 + a fixed salt yields exactly this string under any
    // RFC-compliant bcrypt implementation. Source: openwall
    // crypt_blowfish reference test suite.
    //
    // If our crate ever silently switches algorithms (e.g. from
    // bcrypt-2b to scrypt), this verify() returns false and the test
    // fails — the algorithm change cannot slip through.
    let password = "U*U";
    let known_hash = "$2b$05$CCCCCCCCCCCCCCCCCCCCC.E5YPO9kmyuRGyh0XouQYb4YMJKvyOeW";

    assert!(
        bcrypt::verify(password, known_hash).expect("verify should succeed"),
        "external bcrypt vector failed to verify — algorithm changed",
    );
}

#[test]
fn cost_factor_is_honoured() {
    // Hash with an explicit (low) cost and assert the prefix shows
    // it — protects against an upstream change that ignores the cost
    // arg and silently uses a different value.
    let hash = bcrypt::hash("low-cost", 4).expect("hash should succeed");
    assert!(
        hash.starts_with("$2b$04$"),
        "explicit cost=4 ignored: got {hash}",
    );
}
