//! BM25 vocabulary save → restart → search round-trip (phase37).
//!
//! Before phase37, auto-save wrote a minimal tokenizer file
//! (`vocab_size: 0`) and nothing ever called the vocabulary loaders, so
//! a restarted server embedded text queries in the hash-fallback space
//! — disjoint from the stored vectors — and search returned nothing
//! until a full re-index (reproduced during the v3.4.0 Docker
//! validation). This test pins the fixed pipeline end to end:
//!
//!   build vocab → insert vectors → auto-save (real tokenizer via the
//!   injected saver) → fresh manager restores from disk → the same
//!   query returns the same top hit.
//!
//! Standalone test binary: it mutates `VECTORIZER_DATA_DIR`, which
//! must not race other test crates in the same process. The single
//! `#[test]` keeps env mutation serialized.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use vectorizer::db::VectorStore;
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector};

const COLLECTION: &str = "phase37_roundtrip";
const DIM: usize = 128;

fn corpus() -> Vec<String> {
    vec![
        "rust is a systems programming language focused on safety".to_string(),
        "the write ahead log guarantees durability across crashes".to_string(),
        "vector databases index embeddings for similarity search".to_string(),
        "bm25 ranks documents by term frequency and rarity".to_string(),
    ]
}

fn manager_with_bm25(fit: bool) -> EmbeddingManager {
    let mut bm25 = Bm25Embedding::new(DIM);
    if fit {
        bm25.build_vocabulary(&corpus());
    }
    let mut manager = EmbeddingManager::new();
    manager.register_provider("bm25".to_string(), Box::new(bm25));
    manager.set_default_provider("bm25").unwrap();
    manager
}

#[test]
fn bm25_vocabulary_survives_restart_and_search_still_matches() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .try_init();
    let data_dir = tempfile::tempdir().unwrap();
    // SAFETY: single #[test] in this binary — no concurrent env access.
    unsafe { std::env::set_var("VECTORIZER_DATA_DIR", data_dir.path()) };

    // ---- "first boot": index the corpus ----------------------------
    let manager = Arc::new(manager_with_bm25(true));

    let store = VectorStore::new_cpu_only();
    let config = CollectionConfig {
        graph: None,
        dimension: DIM,
        metric: DistanceMetric::Cosine,
        quantization: Default::default(),
        hnsw_config: HnswConfig::default(),
        compression: Default::default(),
        embedding_provider: "bm25".to_string(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
        encryption: None,
    };
    store.create_collection(COLLECTION, config).unwrap();

    let vectors: Vec<Vector> = corpus()
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let embedding = manager.embed(text).unwrap();
            Vector {
                id: format!("doc{i}"),
                data: embedding,
                payload: Some(Payload::new(serde_json::json!({ "text": text }))),
                sparse: None,
                document_id: None,
            }
        })
        .collect();
    store.insert(COLLECTION, vectors).unwrap();

    let query = "durability of the write ahead log";
    let pre_restart_top = {
        let emb = manager.embed(query).unwrap();
        let results = store
            .get_collection(COLLECTION)
            .unwrap()
            .search(&emb, 1)
            .unwrap();
        results[0].id.clone()
    };
    assert_eq!(
        pre_restart_top, "doc1",
        "sanity: BM25 must match the WAL doc"
    );

    // ---- auto-save with the injected tokenizer saver ---------------
    {
        let saver_manager = Arc::clone(&manager);
        store.set_tokenizer_saver(Arc::new(move |_collection, path| {
            saver_manager.save_vocabulary_json("bm25", path)
        }));
    }
    store.save_collection_to_file(COLLECTION).unwrap();

    let tokenizer_path = data_dir.path().join(format!("{COLLECTION}_tokenizer.json"));
    let tokenizer: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&tokenizer_path).unwrap()).unwrap();
    assert_eq!(
        tokenizer["type"], "bm25",
        "real snapshot, not the minimal tokenizer"
    );
    assert!(
        !tokenizer["vocabulary"].as_object().unwrap().is_empty(),
        "vocabulary must be persisted"
    );

    // ---- "restart": fresh provider, restore from disk --------------
    let mut restarted_manager = manager_with_bm25(false);
    let report = restarted_manager
        .restore_vocabulary_from_disk("bm25", data_dir.path())
        .unwrap();
    assert_eq!(
        report.restored_from.as_deref(),
        Some(COLLECTION),
        "restore must pick up the persisted snapshot"
    );

    // Reload the persisted collection into a fresh store and search
    // with the restored provider: the same query must find the same doc.
    let restarted_store = VectorStore::new_cpu_only();
    let loaded = restarted_store.load_all_persisted_collections().unwrap();
    let dir_listing: Vec<String> = std::fs::read_dir(data_dir.path())
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        loaded >= 1,
        "persisted collection must reload; data_dir contains: {dir_listing:?}"
    );

    let emb = restarted_manager.embed(query).unwrap();
    let results = restarted_store
        .get_collection(COLLECTION)
        .unwrap()
        .search(&emb, 1)
        .unwrap();
    assert_eq!(
        results[0].id, pre_restart_top,
        "post-restart search must return the same top hit — if this \
         fails the query embedded in a different vector space than the \
         stored vectors (the exact v3.4.0 regression)"
    );

    // ---- degraded surfacing: a minimal tokenizer is NOT usable -----
    std::fs::write(
        data_dir.path().join("empty_tokenizer.json"),
        serde_json::json!({
            "collection_name": "empty",
            "tokenizer_type": "dynamic",
            "vocab_size": 0,
            "special_tokens": {},
        })
        .to_string(),
    )
    .unwrap();
    let mut fresh = manager_with_bm25(false);
    let report = fresh
        .restore_vocabulary_from_disk("bm25", data_dir.path())
        .unwrap();
    assert!(
        report.degraded_collections.iter().any(|c| c == "empty"),
        "collections with an empty snapshot must be reported degraded"
    );
    // The real snapshot still wins for the provider itself.
    assert_eq!(report.restored_from.as_deref(), Some(COLLECTION));
}
