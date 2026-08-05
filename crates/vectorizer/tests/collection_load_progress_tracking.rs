//! The startup catalog load publishes its progress
//! (phase1_collections-list-hides-lazy-load-progress, issue #391).
//!
//! `load_all_persisted_collections` inserts collections into the store one at
//! a time, so anything reading the store while it runs sees a partial catalog.
//! On a 181-collection store that surfaced as `GET /collections` answering 11
//! collections with a `total_collections` that agreed with them — a partial
//! answer wearing a complete answer's clothes.
//!
//! This pins the reporting half of the fix against the real storage path:
//! seed collections → compact to `.vecdb` → a fresh store loads the archive
//! through the tracked entry point → the counters describe what actually
//! happened.
//!
//! Standalone test binary with a single `#[test]`: it mutates
//! `VECTORIZER_DATA_DIR`, which must not race other tests in the same process
//! (same rationale as `collection_ttl_persistence.rs`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::db::{CollectionLoadProgress, CollectionLoadStatus, VectorStore};
use vectorizer::models::{CollectionConfig, DistanceMetric, Payload, Vector};

const COLLECTIONS: usize = 5;

fn config() -> CollectionConfig {
    CollectionConfig {
        dimension: 4,
        metric: DistanceMetric::Cosine,
        ..Default::default()
    }
}

fn vector(id: &str) -> Vector {
    Vector {
        id: id.to_string(),
        data: vec![0.1, 0.2, 0.3, 0.4],
        sparse: None,
        payload: Some(Payload::new(serde_json::json!({ "id": id }))),
        document_id: None,
    }
}

#[test]
fn the_tracked_loader_reports_what_it_loaded() {
    let data_dir = tempfile::tempdir().unwrap();
    // SAFETY: single #[test] in this binary — no concurrent env access.
    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.path()) };

    // ── seed a catalog worth loading ────────────────────────────────────
    let store = VectorStore::new_cpu_only();
    for i in 0..COLLECTIONS {
        let name = format!("progress_c{i}");
        store.create_collection(&name, config()).unwrap();
        store.insert(&name, vec![vector(&format!("v{i}"))]).unwrap();
    }

    let compactor = vectorizer::storage::StorageCompactor::new(data_dir.path(), 6, 1000);
    compactor
        .compact_from_memory(&store)
        .expect("compaction must write the archive");

    // ── a fresh store loads it, reporting as it goes ────────────────────
    let progress = CollectionLoadProgress::new();

    // Before the loader runs, the handle admits it knows nothing. This is the
    // state a reader hitting the server one millisecond after boot sees, and
    // it must not be mistaken for "the store is empty".
    let before = progress.snapshot();
    assert_eq!(before.status, CollectionLoadStatus::Pending);
    assert!(before.is_loading());
    assert!(!before.is_complete());

    let restarted = VectorStore::new_cpu_only();
    let loaded = restarted
        .load_all_persisted_collections_tracked(&progress)
        .unwrap();
    assert_eq!(
        loaded, COLLECTIONS,
        "the archive must yield every seeded collection back"
    );

    let after = progress.snapshot();
    assert_eq!(
        after.expected, COLLECTIONS,
        "the denominator must come from the catalog, not from what landed"
    );
    assert_eq!(
        after.loaded, COLLECTIONS,
        "every loaded collection must be counted"
    );
    assert_eq!(
        after.loaded,
        restarted.list_collections().len(),
        "the count must agree with the store it describes"
    );

    // The loader deliberately does NOT settle the state: the caller owns that,
    // because it usually has more startup work to do. Pinning it here so a
    // future refactor that moves `finish()` into the loader has to justify
    // itself — the server relies on this to avoid reporting ready early.
    assert_eq!(
        after.status,
        CollectionLoadStatus::Loading,
        "the loader must leave settling to its caller"
    );
    assert!(!after.is_complete());

    progress.finish();
    let settled = progress.snapshot();
    assert!(settled.is_complete());
    assert!(!settled.is_loading());
    assert_eq!(
        settled.loaded, COLLECTIONS,
        "settling must not disturb the counts"
    );

    // ── the untracked entry point still works ───────────────────────────
    // Every existing caller goes through this one; it must keep loading the
    // same catalog without a progress handle.
    let untracked = VectorStore::new_cpu_only();
    assert_eq!(
        untracked.load_all_persisted_collections().unwrap(),
        COLLECTIONS,
        "the untracked delegate must load exactly what the tracked one does"
    );
}
