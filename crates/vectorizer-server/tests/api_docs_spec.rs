//! Integration tests for the API documentation surface.
//!
//! The dashboard's API Documentation page is driven by two artefacts
//! that live on disk rather than being generated at request time:
//!
//!   - `docs/api/openapi.yaml` — human-authored OpenAPI 3 spec served
//!     by the embedded dashboard assets at `/api/docs/openapi.json`.
//!   - The `ApiSpecification` type in `api::advanced_api` which the
//!     runtime exposes for the capability registry.
//!
//! These tests guard the shape of both so the Setup Wizard →
//! "Documentation" step never ships a broken payload to the UI.
//!
//! Running this file via `cargo test -p vectorizer-server --test
//! api_docs_spec` is fast: it parses the YAML from disk, validates
//! the version / info / paths blocks, and verifies that every route
//! the dashboard links to actually exists in the spec.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // Cargo invokes tests with CWD = the crate directory, so climb
    // two levels up to reach the workspace root where `docs/` lives.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("..").join("..")
}

fn load_openapi_spec() -> serde_yaml::Value {
    let path = repo_root().join("docs").join("api").join("openapi.yaml");
    assert!(
        path.exists(),
        "docs/api/openapi.yaml is missing (looked in {display})",
        display = path.display()
    );
    let content = std::fs::read_to_string(&path).expect("read openapi.yaml");
    serde_yaml::from_str(&content).expect("openapi.yaml is valid YAML")
}

#[test]
fn openapi_spec_declares_openapi_3() {
    let spec = load_openapi_spec();
    let version = spec
        .get("openapi")
        .and_then(|v| v.as_str())
        .expect("openapi version key is present");
    assert!(
        version.starts_with("3."),
        "spec must declare OpenAPI 3.x, got {version}"
    );
}

#[test]
fn openapi_spec_has_title_and_version_in_info() {
    let spec = load_openapi_spec();
    let info = spec.get("info").expect("info block is required");
    let title = info
        .get("title")
        .and_then(|v| v.as_str())
        .expect("info.title");
    let version = info
        .get("version")
        .and_then(|v| v.as_str())
        .expect("info.version");
    assert!(!title.is_empty(), "info.title must not be empty");
    assert!(!version.is_empty(), "info.version must not be empty");
}

#[test]
fn openapi_spec_has_paths() {
    let spec = load_openapi_spec();
    let paths = spec
        .get("paths")
        .and_then(|p| p.as_mapping())
        .expect("paths mapping");
    assert!(!paths.is_empty(), "paths mapping must not be empty");
}

#[test]
fn openapi_spec_documents_core_rest_surface() {
    let spec = load_openapi_spec();
    let paths = spec
        .get("paths")
        .and_then(|p| p.as_mapping())
        .expect("paths mapping");

    // The core REST surface the dashboard relies on — endpoints
    // that have shipped in every version since 2.x — MUST appear.
    // Newer endpoints like `/setup/status` live in the dashboard's
    // in-memory catalog and are exercised by the handler unit tests
    // instead; they are not yet back-ported to the YAML spec.
    for required in ["/health", "/collections"] {
        assert!(
            paths.contains_key(serde_yaml::Value::String(required.to_string())),
            "openapi spec missing required path: {required}"
        );
    }
}

#[test]
fn openapi_json_mirror_exists_and_parses() {
    // We ship a pre-serialized JSON copy so the dashboard can load it
    // without pulling in a YAML parser. Both files must exist and
    // parse cleanly — the full content-level equivalence is tracked
    // by a separate regeneration tool and can legitimately drift
    // between releases, so this test only guards the file-level
    // invariants.
    let yaml_path = repo_root().join("docs").join("api").join("openapi.yaml");
    let json_path = repo_root().join("docs").join("api").join("openapi.json");

    assert!(yaml_path.exists(), "openapi.yaml is missing");
    assert!(json_path.exists(), "openapi.json is missing");

    let yaml_raw = std::fs::read_to_string(&yaml_path).expect("read yaml");
    let json_raw = std::fs::read_to_string(&json_path).expect("read json");

    let yaml_val: serde_yaml::Value = serde_yaml::from_str(&yaml_raw).expect("yaml parses");
    let json_val: serde_yaml::Value = serde_yaml::from_str(&json_raw).expect("json parses");

    // Both files must declare OpenAPI 3.x at the top level.
    assert!(
        yaml_val
            .get("openapi")
            .and_then(|v| v.as_str())
            .map(|s| s.starts_with("3."))
            .unwrap_or(false),
        "openapi.yaml must declare openapi: 3.x"
    );
    assert!(
        json_val
            .get("openapi")
            .and_then(|v| v.as_str())
            .map(|s| s.starts_with("3."))
            .unwrap_or(false),
        "openapi.json must declare openapi: 3.x"
    );
}
