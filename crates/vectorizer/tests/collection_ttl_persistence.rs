//! Collection-TTL rule survives a restart (phase1_persist-collection-ttl-config).
//!
//! `POST /collections/{name}/ttl` stamps `__expires_at` on every vector that
//! arrives afterwards, but the rule itself used to live only in the process:
//! after a restart new inserts silently stopped expiring, while
//! `GET …/ttl` honestly reported `null`. The rule now travels with the
//! collection as `PersistedCollection::ttl_secs`.
//!
//! This test pins the whole loop against the live storage path:
//!
//!   configure TTL → insert → `.vecdb` compaction → fresh store loads the
//!   archive → the rule is back AND still applies to new writes, while the
//!   restored vector keeps the expiry it was saved with.
//!
//! It then does the same for a native snapshot, which is a second archive
//! format carrying the same field.
//!
//! Standalone test binary with a single `#[test]`: it mutates
//! `VECTORIZER_DATA_DIR`, which must not race other tests in the same
//! process (same rationale as `bm25_vocab_persistence.rs`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, Payload, Vector};

const COLLECTION: &str = "ttl_persistence";
const TTL_SECS: u64 = 600;

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

/// Stored expiry read through the raw accessor — the filtered read path hides
/// expired vectors and so cannot answer "what was stamped?".
fn stored_expiry(store: &VectorStore, collection: &str, id: &str) -> Option<i64> {
    store
        .get_collection(collection)
        .unwrap()
        .get_all_vectors()
        .into_iter()
        .find(|v| v.id == id)
        .expect("vector stored")
        .payload
        .and_then(|p| p.expires_at())
}

#[test]
fn collection_ttl_survives_a_restart_and_a_snapshot_restore() {
    let data_dir = tempfile::tempdir().unwrap();
    // SAFETY: single #[test] in this binary — no concurrent env access.
    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.path()) };

    // ── "first boot": configure the TTL and write a vector ──────────────
    let store = VectorStore::new_cpu_only();
    store.create_collection(COLLECTION, config()).unwrap();
    store.set_collection_ttl(COLLECTION, Some(TTL_SECS));
    store.insert(COLLECTION, vec![vector("v1")]).unwrap();

    let expiry_before = stored_expiry(&store, COLLECTION, "v1")
        .expect("sanity: the configured TTL must stamp an expiry");

    // ── compaction writes the live .vecdb archive ───────────────────────
    let compactor = vectorizer::storage::StorageCompactor::new(data_dir.path(), 6, 1000);
    compactor
        .compact_from_memory(&store)
        .expect("compaction must write the archive");

    // ── "restart": a fresh store loads the archive ──────────────────────
    let restarted = VectorStore::new_cpu_only();
    let loaded = restarted.load_all_persisted_collections().unwrap();
    assert!(loaded >= 1, "the archive must yield the collection back");

    assert_eq!(
        restarted.collection_ttl(COLLECTION),
        Some(TTL_SECS),
        "the TTL rule must come back with the collection, or new inserts \
         silently stop expiring after a deploy"
    );

    // The rule must be live, not merely readable: a vector written after the
    // restart has to be stamped.
    restarted.insert(COLLECTION, vec![vector("v2")]).unwrap();
    assert!(
        stored_expiry(&restarted, COLLECTION, "v2").is_some(),
        "a restored TTL that does not reach new writes is the bug this test \
         exists to catch"
    );

    // And the restored vector keeps the expiry it was saved with — loading
    // must not refresh it from the load time.
    assert_eq!(
        stored_expiry(&restarted, COLLECTION, "v1"),
        Some(expiry_before),
        "restoring a vector must not extend its life"
    );

    // ── native snapshot carries the rule too ────────────────────────────
    let snapshot = restarted.snapshot_collection_native(COLLECTION).unwrap();
    restarted.set_collection_ttl(COLLECTION, None);
    assert_eq!(restarted.collection_ttl(COLLECTION), None);

    restarted
        .restore_native_snapshot(COLLECTION, &snapshot.id)
        .unwrap();
    assert_eq!(
        restarted.collection_ttl(COLLECTION),
        Some(TTL_SECS),
        "a snapshot restores the collection as it was, TTL included"
    );

    // ── an archive written before the field existed reads as "no TTL" ───
    let legacy_json = serde_json::json!({
        "version": 1,
        "collections": [{
            "name": "legacy_collection",
            "config": null,
            "vectors": [],
            "hnsw_dump_basename": null,
        }],
    })
    .to_string();
    let legacy: vectorizer::persistence::PersistedVectorStore =
        serde_json::from_str(&legacy_json).expect("old archives must still deserialise");
    assert_eq!(
        legacy.collections[0].ttl_secs, None,
        "a missing ttl_secs means no TTL — the behaviour that archive was \
         written with"
    );
}
