//! Every published artifact declares the same version
//! (phase3_release-3-6-1).
//!
//! A release has to move twelve version carriers across five languages, and
//! nothing checked that they agreed. The 3.6.1 cut nearly shipped a Go SDK
//! still announcing 3.6.0: the release checklist listed six carriers and
//! claimed Go published by tag alone, but `sdks/go/version.go` holds a
//! `const Version` the client reports at runtime. It was caught by reading
//! the tree rather than the checklist — which is not a process that scales.
//!
//! This test is the check that would have caught it, and will catch the next
//! one. The workspace crate version is the reference; every other carrier
//! must match it exactly.
//!
//! When this fails after a bump, the fix is to bump the file it names — not
//! to relax the assertion. A carrier that legitimately versions independently
//! should be deleted from `CARRIERS` with a comment saying why.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("..");
    p
}

/// Does `.gitmodules` declare `dir` as a submodule?
///
/// `sdks/go` lives in its own repository and is vendored as a submodule, so a
/// checkout that does not fetch submodules leaves the directory empty —
/// `actions/checkout` skips them by default. Same escape hatch as
/// `dependabot_coverage.rs`: absent-because-not-materialised is not a
/// failure, absent-because-someone-moved-it is.
fn declares_submodule(dir: &str) -> bool {
    let expected = format!("path = {dir}");
    fs::read_to_string(repo_root().join(".gitmodules"))
        .map(|config| config.lines().any(|line| line.trim() == expected))
        .unwrap_or(false)
}

/// (path, the exact line that must appear, submodule dir if the file lives in one)
///
/// Matching whole lines rather than "does the version appear anywhere" keeps
/// the check honest: a file mentioning the number in prose would otherwise
/// pass while its actual declaration lagged.
fn carriers(version: &str) -> Vec<(&'static str, String, Option<&'static str>)> {
    vec![
        // Workspace crates. `crates/vectorizer` is the reference and is
        // checked too, so a hand-edit there cannot silently redefine it.
        (
            "crates/vectorizer/Cargo.toml",
            format!("version = \"{version}\""),
            None,
        ),
        (
            "crates/vectorizer-server/Cargo.toml",
            format!("version = \"{version}\""),
            None,
        ),
        (
            "crates/vectorizer-core/Cargo.toml",
            format!("version = \"{version}\""),
            None,
        ),
        (
            "crates/vectorizer-grpc/Cargo.toml",
            format!("version = \"{version}\""),
            None,
        ),
        (
            "crates/vectorizer-cli/Cargo.toml",
            format!("version = \"{version}\""),
            None,
        ),
        // SDKs.
        (
            "sdks/rust/Cargo.toml",
            format!("version = \"{version}\""),
            None,
        ),
        (
            "sdks/typescript/package.json",
            format!("\"version\": \"{version}\","),
            None,
        ),
        (
            "sdks/python/pyproject.toml",
            format!("version = \"{version}\""),
            None,
        ),
        (
            "sdks/python/__init__.py",
            format!("__version__ = \"{version}\""),
            None,
        ),
        (
            "sdks/go/version.go",
            format!("const Version = \"{version}\""),
            Some("sdks/go"),
        ),
        (
            "sdks/csharp/Vectorizer.csproj",
            format!("<Version>{version}</Version>"),
            None,
        ),
        (
            "sdks/csharp/src/Vectorizer.Rpc/Vectorizer.Rpc.csproj",
            format!("<Version>{version}</Version>"),
            None,
        ),
    ]
}

#[test]
fn every_version_carrier_matches_the_workspace_crate() {
    let version = env!("CARGO_PKG_VERSION");
    let mut missing = Vec::new();
    let mut skipped = Vec::new();

    for (path, expected_line, submodule) in carriers(version) {
        let full = repo_root().join(path);

        if !full.exists() {
            match submodule {
                Some(dir) if declares_submodule(dir) => {
                    skipped.push(format!("{path} (submodule {dir} not checked out)"));
                    continue;
                }
                _ => {
                    missing.push(format!("  {path} — file not found"));
                    continue;
                }
            }
        }

        let contents = fs::read_to_string(&full)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", full.display()));

        if !contents.lines().any(|line| line.trim() == expected_line) {
            missing.push(format!("  {path} — expected a line `{expected_line}`"));
        }
    }

    assert!(
        missing.is_empty(),
        "these artifacts do not declare version {version}:\n{}\n\n\
         A release moves all of them together. Bump the files listed above; \
         do not relax this assertion. Skipped this run: {}",
        missing.join("\n"),
        if skipped.is_empty() {
            "none".to_string()
        } else {
            skipped.join(", ")
        }
    );
}

#[test]
fn the_changelog_documents_the_current_version() {
    // The 3.6.0 release shipped to four registries with its entries still
    // under `[Unreleased]` — there was no `## [3.6.0]` heading at all, so the
    // published changelog told users nothing about what they had installed.
    let version = env!("CARGO_PKG_VERSION");
    let changelog = fs::read_to_string(repo_root().join("CHANGELOG.md"))
        .expect("CHANGELOG.md must exist at the repo root");

    let heading = format!("## [{version}]");
    assert!(
        changelog.lines().any(|line| line.starts_with(&heading)),
        "CHANGELOG.md has no `{heading}` section. Every version that gets \
         published needs one — 3.6.0 shipped without it and its notes sat \
         under [Unreleased] until the next release cut noticed."
    );
}
