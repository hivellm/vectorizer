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

use std::path::Path;

use regex::Regex;
use walkdir::WalkDir;

const SRC_ROOT: &str = "src";

#[test]
fn src_tree_has_no_unqualified_tier1_markers() {
    let forbidden = Regex::new(r"\b(TODO|FIXME|HACK|XXX)\b").expect("forbidden pattern compiles");
    let allow_task = Regex::new(r"TASK\(phase[0-9]+_[a-z0-9-]+\)").expect("allow pattern compiles");
    let allow_sentinel =
        Regex::new(r"grep-ignore\(tier1-markers\)").expect("sentinel pattern compiles");

    let project_root = locate_project_root();
    let scan_root = project_root.join(SRC_ROOT);

    let mut violations: Vec<String> = Vec::new();

    for entry in WalkDir::new(&scan_root)
        .into_iter()
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

fn locate_project_root() -> std::path::PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo during tests");
    Path::new(&manifest_dir).to_path_buf()
}
