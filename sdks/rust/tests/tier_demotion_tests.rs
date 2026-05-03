#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Unit tests for the SDK tier-demotion surface (issue #265):
//! `delete_vector`, `delete_vectors`, `move_to_collection` and the
//! associated [`DeleteReport`] / [`MoveReport`] / [`VectorOpResult`]
//! types.
//!
//! Wire-level integration tests live alongside the server in
//! `crates/vectorizer/tests/api/rest/move_vectors_real.rs`.

use serde_json::json;
use vectorizer_sdk::{DeleteReport, MoveReport, VectorOpResult};

#[test]
fn delete_report_deserializes_server_contract() {
    // Mirror of `POST /batch_delete` response shape.
    let raw = json!({
        "collection": "cortex.consolidation.fp32",
        "count": 3,
        "deleted": 2,
        "failed": 1,
        "results": [
            {"index": 0, "id": "vec-1", "status": "ok"},
            {"index": 1, "id": "vec-2", "status": "ok"},
            {"index": 2, "id": "missing", "status": "error", "error": "not found"},
        ],
    });

    let report: DeleteReport = serde_json::from_value(raw).expect("decode DeleteReport");
    assert_eq!(report.collection, "cortex.consolidation.fp32");
    assert_eq!(report.count, 3);
    assert_eq!(report.deleted, 2);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results.len(), 3);
    assert_eq!(report.results[0].status, "ok");
    assert_eq!(report.results[2].status, "error");
    assert_eq!(report.results[2].error.as_deref(), Some("not found"));
    assert_eq!(report.results[0].index, Some(0));
}

#[test]
fn move_report_deserializes_server_contract() {
    // Mirror of `POST /collections/{src}/vectors/move` response.
    let raw = json!({
        "src": "cortex.consolidation.fp32",
        "dst": "cortex.consolidation.pq",
        "requested": 3,
        "moved": 1,
        "failed": 2,
        "results": [
            {"id": "vec-1", "status": "ok"},
            {"id": "vec-missing", "status": "missing_in_src", "error": "not found"},
            {"id": "vec-bad-dim", "status": "dst_insert_failed", "error": "dimension mismatch"},
        ],
    });

    let report: MoveReport = serde_json::from_value(raw).expect("decode MoveReport");
    assert_eq!(report.src, "cortex.consolidation.fp32");
    assert_eq!(report.dst, "cortex.consolidation.pq");
    assert_eq!(report.requested, 3);
    assert_eq!(report.moved, 1);
    assert_eq!(report.failed, 2);

    let statuses: Vec<&str> = report.results.iter().map(|r| r.status.as_str()).collect();
    assert_eq!(statuses, vec!["ok", "missing_in_src", "dst_insert_failed"]);
}

#[test]
fn vector_op_result_handles_missing_optional_fields() {
    // A bare `{id, status}` row (no `error`, no `index`) must decode.
    let row: VectorOpResult =
        serde_json::from_value(json!({"id": "vec-1", "status": "ok"})).unwrap();
    assert_eq!(row.id.as_deref(), Some("vec-1"));
    assert_eq!(row.status, "ok");
    assert!(row.error.is_none());
    assert!(row.index.is_none());
}

#[test]
fn vector_op_result_handles_null_id() {
    // The server emits `id: null` when the request contained a
    // non-string entry; must decode to `Option::None`.
    let row: VectorOpResult = serde_json::from_value(json!({
        "id": null,
        "status": "missing_in_src",
        "error": "id must be a string",
    }))
    .unwrap();
    assert!(row.id.is_none());
    assert_eq!(row.status, "missing_in_src");
    assert_eq!(row.error.as_deref(), Some("id must be a string"));
}

#[test]
fn move_report_round_trips_through_serde() {
    // Sanity check: building a MoveReport in code and round-tripping
    // through serde produces the same value the server would emit.
    let report = MoveReport {
        src: "src".into(),
        dst: "dst".into(),
        requested: 1,
        moved: 1,
        failed: 0,
        results: vec![VectorOpResult {
            id: Some("vec-1".into()),
            status: "ok".into(),
            error: None,
            index: None,
        }],
    };
    let serialized = serde_json::to_value(&report).unwrap();
    let parsed: MoveReport = serde_json::from_value(serialized).unwrap();
    assert_eq!(parsed, report);
}
