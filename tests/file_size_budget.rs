//! Per-file LOC budget test for the `rest_handlers/` split.
//!
//! When `src/server/rest_handlers.rs` was split into
//! `src/server/rest_handlers/*.rs` (phase3_split-rest-handlers-monolith)
//! each new file was sized to stay under ~500 LOC so it fits one
//! reviewer's attention span. This test pins those budgets so that a
//! future drive-by edit cannot silently regrow a file past its limit.
//!
//! If this test fails:
//! 1. Prefer splitting the file further along the existing concern
//!    axes (e.g. split `search` batch helpers out of `search.rs`).
//! 2. Only bump the limit here if the file legitimately gained a
//!    single tightly-coupled handler that cannot be extracted.

use std::fs;
use std::path::PathBuf;

fn count_lines(path: &str) -> usize {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(path);
    let contents =
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("failed to read {}: {}", p.display(), e));
    contents.lines().count()
}

/// The budget for each file in `src/server/rest_handlers/`. A file is
/// allowed to exceed 500 LOC only when listed below with a concrete
/// reason.
const BUDGETS: &[(&str, usize, &str)] = &[
    (
        "src/server/rest_handlers/mod.rs",
        150,
        "module declarations + pub use",
    ),
    ("src/server/rest_handlers/common.rs", 100, "3 small helpers"),
    ("src/server/rest_handlers/meta.rs", 300, "6 small handlers"),
    (
        "src/server/rest_handlers/collections.rs",
        550,
        "7 handlers incl. list/create",
    ),
    ("src/server/rest_handlers/vectors.rs", 350, "8 handlers"),
    (
        "src/server/rest_handlers/insert.rs",
        550,
        "single large insert_text handler",
    ),
    (
        "src/server/rest_handlers/search.rs",
        500,
        "7 search-family handlers",
    ),
    (
        "src/server/rest_handlers/intelligent_search.rs",
        400,
        "4 orchestrator handlers",
    ),
    (
        "src/server/rest_handlers/discovery.rs",
        700,
        "10 pipeline-stage handlers; see design note in tasks.md",
    ),
    (
        "src/server/rest_handlers/files.rs",
        450,
        "7 file-navigation handlers",
    ),
    (
        "src/server/rest_handlers/admin.rs",
        400,
        "8 admin/workspace/config handlers",
    ),
    (
        "src/server/rest_handlers/backups.rs",
        400,
        "4 backup handlers incl. restore",
    ),
];

#[test]
fn rest_handlers_files_stay_within_budget() {
    let mut violations = Vec::new();

    for (path, budget, note) in BUDGETS {
        let lines = count_lines(path);
        if lines > *budget {
            violations.push(format!(
                "  {path} — {lines} LOC (budget {budget}, {note}). \
                 Either split the file further along an existing concern axis, \
                 or update the BUDGETS table with a concrete reason."
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "rest_handlers LOC budgets exceeded:\n{}",
        violations.join("\n")
    );
}
