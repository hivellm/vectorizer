//! A provider-less collection stays provider-less across a restart
//! (phase6_raw-vector-collections).
//!
//! `embedding_provider` carries `#[serde(default = "default_embedding_provider")]`
//! so that `.vecdb` archives written before phase33 keep reading as `"bm25"`.
//! That default is correct for legacy archives and dangerous for this one: if
//! any step of the persistence path drops the field instead of writing it, the
//! sentinel comes back as `"bm25"` on reload — and the collection silently
//! regains a provider whose vectors it does not contain, which is precisely
//! the coercion phase33 (issue #306) removed.
//!
//! Nothing in the type system prevents that; only a written value does. This
//! test pins the whole loop against the live storage path:
//!
//!   create with the sentinel → `.vecdb` compaction → fresh store loads the
//!   archive → still provider-less, and still holding a width no registered
//!   provider has.
//!
//! It then does the same for a native snapshot, the second archive format
//! carrying the same field, and pins the legacy default it must not disturb.
//!
//! Standalone test binary with a single `#[test]`: it mutates
//! `VECTORIZER_DATA_DIR`, which must not race other tests in the same
//! process (same rationale as `collection_ttl_persistence.rs`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, Payload, RAW_VECTOR_PROVIDER, Vector};

const RAW: &str = "raw_persistence";
const ORDINARY: &str = "ordinary_persistence";

/// 384 is the point: it is a real model width and *not* BM25's 512, so a
/// collection that came back as `bm25` would also be internally inconsistent.
const RAW_DIMENSION: usize = 384;

fn raw_config() -> CollectionConfig {
    CollectionConfig {
        dimension: RAW_DIMENSION,
        metric: DistanceMetric::Cosine,
        embedding_provider: RAW_VECTOR_PROVIDER.to_string(),
        ..Default::default()
    }
}

fn vector(id: &str, dimension: usize) -> Vector {
    Vector {
        id: id.to_string(),
        data: vec![0.1; dimension],
        sparse: None,
        payload: Some(Payload::new(serde_json::json!({ "id": id }))),
        document_id: None,
    }
}

/// Read a collection's config out of the store as an owned copy.
///
/// Every read here goes through this helper on purpose. `get_collection`
/// returns a DashMap `Ref`, which holds a read lock on its shard until the
/// binding is dropped — and a binding lives to the end of its block, not to
/// its last use. Holding one across `restore_native_snapshot` deadlocks the
/// test: that call deletes the collection, which wants the shard's write lock.
/// It is the same re-entrancy that caused this repo's phase39 production
/// deadlock, and cloning out of a temporary is what keeps it from recurring.
fn config_of(store: &VectorStore, collection: &str) -> CollectionConfig {
    store.get_collection(collection).unwrap().config().clone()
}

#[test]
fn a_raw_vector_collection_survives_a_restart_without_regaining_a_provider() {
    let data_dir = tempfile::tempdir().unwrap();
    // SAFETY: single #[test] in this binary — no concurrent env access.
    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.path()) };

    // ── "first boot": one raw collection, one ordinary one ───────────────
    let store = VectorStore::new_cpu_only();
    store.create_collection(RAW, raw_config()).unwrap();
    store
        .insert(RAW, vec![vector("v1", RAW_DIMENSION)])
        .unwrap();

    // The ordinary collection is the control: it proves the assertion below
    // reads a real stored value rather than passing because the field always
    // comes back empty.
    store
        .create_collection(
            ORDINARY,
            CollectionConfig {
                dimension: 512,
                metric: DistanceMetric::Cosine,
                ..Default::default()
            },
        )
        .unwrap();
    store.insert(ORDINARY, vec![vector("v1", 512)]).unwrap();

    assert!(
        config_of(&store, RAW).is_raw_vector(),
        "sanity: the sentinel must be set before we test that it survives"
    );

    // ── compaction writes the live .vecdb archive ────────────────────────
    let compactor = vectorizer::storage::StorageCompactor::new(data_dir.path(), 6, 1000);
    compactor
        .compact_from_memory(&store)
        .expect("compaction must write the archive");

    // ── "restart": a fresh store loads the archive ───────────────────────
    let restarted = VectorStore::new_cpu_only();
    let loaded = restarted.load_all_persisted_collections().unwrap();
    assert!(loaded >= 2, "the archive must yield both collections back");

    let restored = config_of(&restarted, RAW);
    assert!(
        restored.is_raw_vector(),
        "the collection came back as `{}` — the serde default fired, so the \
         field was never written, and a collection with no provider now \
         claims one whose vectors it does not contain",
        restored.embedding_provider
    );
    assert_eq!(
        restored.dimension, RAW_DIMENSION,
        "the width the caller chose must survive too; a provider-less \
         collection that reloads at BM25's 512 is unusable"
    );

    assert_eq!(
        config_of(&restarted, ORDINARY).embedding_provider,
        "bm25",
        "control: an ordinary collection still reports its own provider, so \
         the assertion above is reading a stored value"
    );

    // ── native snapshot carries the field too ────────────────────────────
    let snapshot = restarted.snapshot_collection_native(RAW).unwrap();
    restarted
        .restore_native_snapshot(RAW, &snapshot.id)
        .unwrap();
    assert!(
        config_of(&restarted, RAW).is_raw_vector(),
        "a snapshot restores the collection as it was, sentinel included"
    );

    // ── the legacy default this must not disturb ─────────────────────────
    //
    // Archives written before phase33 have no `embedding_provider` at all,
    // and must keep reading as `bm25` — the behaviour they were written with.
    // Removing that default to protect the sentinel would silently strip the
    // provider from every pre-3.5 collection on the next boot.
    let legacy: CollectionConfig = serde_json::from_value(serde_json::json!({
        "dimension": 512,
        "metric": "cosine",
        "quantization": {"type": "none"},
        "compression": {"enabled": false, "threshold_bytes": 1024, "algorithm": "lz4"},
        "hnsw_config": {"m": 16, "ef_construction": 200, "ef_search": 100, "seed": null},
    }))
    .expect("archives that predate the field must still deserialise");
    assert_eq!(
        legacy.embedding_provider, "bm25",
        "a missing provider means the historical bm25, not the sentinel"
    );
    assert!(!legacy.is_raw_vector());
}
