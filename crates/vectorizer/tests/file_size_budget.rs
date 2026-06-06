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
//!
//! Location note: `rest_handlers/` was relocated from `crates/vectorizer`
//! to `crates/vectorizer-server` during phase4_split-vectorizer-workspace.
//! This test lives here for historical continuity; the paths below are
//! resolved relative to this crate's manifest dir and cross into the
//! sibling server crate.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

fn count_lines(path: &str) -> usize {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("vectorizer-server");
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
    (
        "src/server/rest_handlers/common.rs",
        160,
        "3 small helpers + admit_upsert (issue #263 phase9 §3 — \
         centralizes the per-collection upsert admission + 429 \
         translation reused by insert_text, insert_vectors, and \
         do_batch_insert_texts)",
    ),
    (
        "src/server/rest_handlers/meta.rs",
        430,
        "6 small handlers + phase25 §5 quantization aggregation \
         helpers (quantization_label / compression_ratio + 4 unit \
         tests pinning their behaviour) + phase33 §4 provider \
         discovery block on GET /stats (issue #306 — lists every \
         registered embedding provider with its dimension + default \
         flag so callers can avoid the silent BM25 coercion trap).",
    ),
    (
        "src/server/rest_handlers/collections.rs",
        1010,
        "7 handlers incl. list/create + phase13 reencode_collection / \
         set_collection_ttl + phase14 rename / reindex / native snapshot \
         CRUD (snapshot_native, list_collection_snapshots_native, \
         restore_collection_snapshot_native) + phase33 §2 \
         embedding_provider validation on create_collection (issue \
         #306 — resolves the requested provider against the registry \
         and rejects with 400 unsupported_provider / \
         provider_dimension_mismatch instead of silent BM25 coercion). \
         Re-tighten when the schema-evolution endpoints split out \
         (follow-up task).",
    ),
    (
        "src/server/rest_handlers/vectors.rs",
        1060,
        "8 handlers + batch_insert_texts / insert_texts REST aliases + \
         do_batch_insert_texts engine (phase6 + phase8) + phase13 \
         delete_by_filter / bulk_update_metadata / copy_vectors / \
         set_vector_expiry tier-control primitives + phase33 §3 \
         model param resolution on /embed (issue #306 — honours the \
         requested model or returns 400 unsupported_model instead of \
         silently routing every embed through the default provider). \
         Re-tighten when the tier-control handlers split out \
         (follow-up task).",
    ),
    (
        "src/server/rest_handlers/insert.rs",
        700,
        "insert_text handler + the shared insert_one_text engine + the \
         pure helpers `validate_client_id`, `build_chunk_payload`, \
         `parse_metadata` reused by `insert_vectors.rs` (client-id \
         contract + flat chunk payload landed in phase9)",
    ),
    (
        "src/server/rest_handlers/insert_vectors.rs",
        300,
        "insert_vectors handler + insert_one_vector + build_vector_payload \
         (pre-vectorized bulk-insert path, phase9)",
    ),
    (
        "src/server/rest_handlers/search.rs",
        1040,
        "7 search-family handlers + hybrid search (dense + sparse + \
         rank-fusion + per-axis weights) + batch_search_vectors + \
         search_by_file + search_by_collection variants + Qdrant-shape \
         adapters + phase14 explain_search HNSW execution-trace handler. \
         Split across concern axes is blocked until the hybrid-search \
         task lands (phase7_hybrid-search-extraction); re-tighten this \
         budget there.",
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
