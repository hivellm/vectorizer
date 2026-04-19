//! Validates that the real checked-in `config.example.yml` parses,
//! that each shipped mode override under `config/modes/` merges
//! cleanly, and that the merged result deserializes into a strict
//! `VectorizerConfig`.
//!
//! These tests run against the live YAML files (not test fixtures) so
//! they break the day someone hand-edits the base or a mode override
//! into a shape the typed config can't deserialize. That's the whole
//! point — the layered loader is only useful if the YAML it loads
//! survives the strict serde schema.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use vectorizer::config::layered::{LayeredOptions, load_layered};

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is the crate root for `cargo test`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn base_path() -> PathBuf {
    repo_root().join("config.example.yml")
}

fn modes_dir() -> PathBuf {
    repo_root().join("config").join("modes")
}

#[test]
fn base_config_example_yml_parses_into_strict_schema() {
    let cfg = load_layered(
        &base_path(),
        LayeredOptions {
            mode: None,
            modes_dir: Some(modes_dir()),
        },
    )
    .expect("config.example.yml must parse and deserialize");

    // Spot-check the v3.x defaults that other tasks document as
    // canonical. If any of these drift the matching task's
    // documentation has gone stale.
    assert!(
        cfg.rpc.enabled,
        "v3.x cutover (phase6_make-rpc-default-transport) requires rpc.enabled = true"
    );
    assert_eq!(cfg.rpc.port, 15503);
    assert_eq!(cfg.server.port, 15002);
}

#[test]
fn production_mode_override_merges_into_strict_schema() {
    let cfg = load_layered(
        &base_path(),
        LayeredOptions {
            mode: Some("production".into()),
            modes_dir: Some(modes_dir()),
        },
    )
    .expect("config.example.yml + config/modes/production.yml must merge");

    // Production overrides — these are the assertions that tell the
    // operator the mode override actually applied.
    assert_eq!(
        cfg.server.host, "0.0.0.0",
        "production mode binds on all interfaces"
    );
    assert_eq!(
        cfg.logging.level, "warn",
        "production hardens logging to warn-and-above"
    );

    // Unmodified-by-mode keys come from the base. If this drifts,
    // the merge accidentally clobbered a base key.
    assert_eq!(
        cfg.server.port, 15002,
        "port stays at base value (production override does not touch it)"
    );
    assert!(
        cfg.rpc.enabled,
        "rpc.enabled inherits the base value (true in v3.x)"
    );
}

#[test]
fn dev_mode_override_merges_into_strict_schema() {
    let cfg = load_layered(
        &base_path(),
        LayeredOptions {
            mode: Some("dev".into()),
            modes_dir: Some(modes_dir()),
        },
    )
    .expect("config.example.yml + config/modes/dev.yml must merge");

    assert_eq!(cfg.logging.level, "debug", "dev mode is verbose");
    assert_eq!(cfg.server.host, "127.0.0.1", "dev stays loopback-only");
    assert!(
        cfg.file_watcher.enabled,
        "dev mode keeps the file watcher on for the developer loop"
    );
}

#[test]
fn requesting_an_unknown_mode_returns_a_clear_error() {
    let result = load_layered(
        &base_path(),
        LayeredOptions {
            mode: Some("does-not-exist".into()),
            modes_dir: Some(modes_dir()),
        },
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("does-not-exist"),
        "error message must name the missing mode (for operator diagnostics); got: {msg}"
    );
}
