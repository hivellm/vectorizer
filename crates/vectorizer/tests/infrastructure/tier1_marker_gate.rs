//! Tier-1 marker gate — regression test for AGENTS.md Tier-1 rule #1.
//!
//! This test mirrors `scripts/check-no-tier1-markers.sh` in pure Rust so it
//! runs on every platform (Windows, macOS, Linux) without needing bash.
//!
//! Forbidden tokens: TODO, FIXME, HACK, XXX.
//! Allowed exceptions:
//!   * `TASK(phaseN_<slug>)` — tracked rulebook follow-up task.
//!   * `grep-ignore(tier1-markers)` on the same line — detection-feature
//!     literal strings that must keep the token.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use regex::Regex;
use walkdir::WalkDir;

#[test]
fn src_tree_has_no_unqualified_tier1_markers() {
    let forbidden = Regex::new(r"\b(TODO|FIXME|HACK|XXX)\b").expect("forbidden pattern compiles");
    let allow_task = Regex::new(r"TASK\(phase[0-9]+_[a-z0-9-]+\)").expect("allow pattern compiles");
    let allow_sentinel =
        Regex::new(r"grep-ignore\(tier1-markers\)").expect("sentinel pattern compiles");

    let project_root = locate_project_root();
    let scan_roots = crate_source_roots(&project_root);
    assert!(
        !scan_roots.is_empty(),
        "no crate source tree found under {} — this gate would pass without \
         reading anything",
        project_root.display()
    );

    let mut violations: Vec<String> = Vec::new();

    for entry in scan_roots
        .iter()
        .flat_map(|root| WalkDir::new(root).into_iter())
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(ext, "rs" | "md") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };

        for (line_number, line) in content.lines().enumerate() {
            if !forbidden.is_match(line) {
                continue;
            }
            if allow_task.is_match(line) || allow_sentinel.is_match(line) {
                continue;
            }
            let rel = path.strip_prefix(&project_root).unwrap_or(path);
            violations.push(format!(
                "{}:{}:{}",
                rel.display(),
                line_number + 1,
                line.trim_end()
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Tier-1 marker(s) found outside the TASK(phaseN_<slug>) allow-list:\n{}",
        violations.join("\n")
    );
}

/// Workspace root, derived from this crate's manifest dir
/// (`<root>/crates/vectorizer` → `<root>`). Falls back to the manifest dir
/// itself if the layout ever changes, so the test degrades to single-crate
/// coverage rather than scanning nothing.
fn locate_project_root() -> std::path::PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo during tests");
    let manifest_dir = Path::new(&manifest_dir);
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .filter(|root| root.join("crates").is_dir())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest_dir.to_path_buf())
}

/// Every `crates/*/src` in the workspace — the same scope the shell gate
/// (`scripts/ci/check-no-tier1-markers.sh`) defaults to, so the two cannot
/// disagree. Scanning only this crate's `src` was leaving the server crate,
/// where most of the request handling lives, unchecked by this mirror.
fn crate_source_roots(project_root: &Path) -> Vec<std::path::PathBuf> {
    let crates_dir = project_root.join("crates");
    let Ok(entries) = std::fs::read_dir(&crates_dir) else {
        // Not the workspace layout — fall back to `<root>/src` if present.
        let solo = project_root.join("src");
        return if solo.is_dir() {
            vec![solo]
        } else {
            Vec::new()
        };
    };

    let mut roots: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("src"))
        .filter(|src| src.is_dir())
        .collect();
    roots.sort();
    roots
}
