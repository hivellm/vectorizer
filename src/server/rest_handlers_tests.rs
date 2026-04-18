//! Unit tests for REST handlers — extracted from
//! `src/server/rest_handlers.rs` under
//! `phase3_split-rest-handlers-monolith` via the `#[path]` attribute.

use super::*;

#[test]
fn collection_metrics_uuid_is_deterministic() {
    let a = collection_metrics_uuid("docs");
    let b = collection_metrics_uuid("docs");
    assert_eq!(a, b, "same name must yield same UUID across calls");
}

#[test]
fn collection_metrics_uuid_differs_between_names() {
    let docs = collection_metrics_uuid("docs");
    let products = collection_metrics_uuid("products");
    assert_ne!(docs, products, "different names must yield different UUIDs");
}

#[test]
fn collection_metrics_uuid_is_v5() {
    let id = collection_metrics_uuid("any-collection");
    assert_eq!(
        id.get_version_num(),
        5,
        "expected UUIDv5, got v{}",
        id.get_version_num()
    );
}

#[test]
fn collection_metrics_uuid_handles_empty_and_unicode() {
    // Edge cases that previously went through `new_v4` without a problem
    // and must keep round-tripping to themselves under v5.
    assert_eq!(
        collection_metrics_uuid(""),
        collection_metrics_uuid(""),
        "empty name must be stable"
    );
    assert_eq!(
        collection_metrics_uuid("coleção"),
        collection_metrics_uuid("coleção"),
        "unicode name must be stable"
    );
    assert_ne!(
        collection_metrics_uuid(""),
        collection_metrics_uuid("coleção"),
        "empty and unicode must collide only by accident, which v5 avoids"
    );
}
