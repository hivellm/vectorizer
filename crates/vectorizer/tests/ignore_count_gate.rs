//! `#[ignore]` regression gate (phase39 test-harness spec).
//!
//! The 2026-07-11 analysis found 152 ignored tests while the testing
//! doc claimed ~40 — ignore-count creep is invisible without a gate.
//! This test fails when the repository-wide count EXCEEDS the recorded
//! baseline, forcing the author of a new `#[ignore]` to either migrate
//! the test onto the in-process harness or consciously bump the
//! baseline here (a reviewable, greppable diff). Reducing the count
//! below the baseline is always fine — update the baseline downward
//! when you do.
//!
//! Bare `#[ignore]` (without a reason string) is a hard error
//! regardless of count.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};

/// Total `#[ignore...]` attributes allowed under `crates/`.
///
/// History: 152 at the 2026-07-11 analysis; 154 after phase38 added
/// reason-annotated ignores for slow PQ-training and perf tests;
/// 150 after phase39 un-ignored the 4 gRPC update tests (the "fails
/// in CI" bug no longer reproduces — they pass in-process).
const IGNORE_BASELINE: usize = 150;

fn repo_crates_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p
}

fn scan(dir: &Path, total: &mut usize, bare: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            // Skip build artifacts if any ever land here.
            if name != "target" {
                scan(&path, total, bare);
            }
        } else if name.ends_with(".rs") {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            for (i, line) in content.lines().enumerate() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("#[ignore") {
                    *total += 1;
                    // `#[ignore]` or `#[ignore] // ...` — no reason string.
                    if !trimmed.starts_with("#[ignore =") {
                        bare.push(format!("{}:{}", path.display(), i + 1));
                    }
                }
            }
        }
    }
}

#[test]
fn ignore_count_stays_at_or_below_baseline() {
    let mut total = 0;
    let mut bare = Vec::new();
    scan(&repo_crates_dir(), &mut total, &mut bare);

    assert!(
        bare.is_empty(),
        "bare #[ignore] without a reason string is forbidden \
         (phase39 spec) — annotate with #[ignore = \"why + how to run\"]:\n{}",
        bare.join("\n")
    );

    assert!(
        total <= IGNORE_BASELINE,
        "#[ignore] count grew: {total} > baseline {IGNORE_BASELINE}. \
         Migrate the new test onto the in-process harness \
         (crates/vectorizer-server/tests/common) instead of ignoring it, \
         or consciously bump IGNORE_BASELINE in this file with a reason \
         in the commit message."
    );

    // Keep the baseline honest: if the real count drops far below the
    // recorded baseline, tighten it so regressions can't hide in the gap.
    assert!(
        total + 20 > IGNORE_BASELINE,
        "ignore count ({total}) is way below the baseline \
         ({IGNORE_BASELINE}) — lower IGNORE_BASELINE to match reality"
    );
}
