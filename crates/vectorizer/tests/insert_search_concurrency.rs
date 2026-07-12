//! Search-during-batch-insert concurrency (phase38 hot-path spec).
//!
//! Before phase38, `Collection::insert_batch` held the HNSW index
//! WRITE lock for the entire batch — payload indexing, quantization,
//! and graph discovery included — so every concurrent search blocked
//! until the whole batch finished. The spec requires a search to
//! acquire the index read path without waiting for the full batch.
//!
//! The test starts a large batch on a writer thread and asserts that
//! searches complete while the writer is still running. On a machine
//! fast enough to finish the whole batch before a single search gets
//! scheduled the assertion is skipped as unprovable (and the batch
//! size is chosen so that does not happen on realistic hardware).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, Vector};

const DIM: usize = 256;
const SEED_VECTORS: usize = 200;
// Debug builds insert ~10x slower; a smaller batch keeps CI time sane
// while still running multi-second (the interleaving window we need).
const BATCH_VECTORS: usize = if cfg!(debug_assertions) {
    6_000
} else {
    30_000
};

fn pseudo_vector(i: usize) -> Vec<f32> {
    // Deterministic, non-degenerate directions without a rand dep.
    (0..DIM)
        .map(|d| ((i * 31 + d * 17) % 97) as f32 / 97.0 + 0.01)
        .collect()
}

fn make(id: usize) -> Vector {
    Vector {
        id: format!("v{id}"),
        data: pseudo_vector(id),
        payload: None,
        sparse: None,
        document_id: None,
    }
}

#[test]
fn searches_complete_while_batch_insert_is_running() {
    let store = Arc::new(VectorStore::new_cpu_only());
    let config = CollectionConfig {
        graph: None,
        dimension: DIM,
        metric: DistanceMetric::Cosine,
        quantization: vectorizer::models::QuantizationConfig::None,
        hnsw_config: HnswConfig::default(),
        compression: Default::default(),
        embedding_provider: "bm25".to_string(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
        encryption: None,
    };
    store.create_collection("mixed_load", config).unwrap();

    // Seed so searches have something to return from the start.
    store
        .insert("mixed_load", (0..SEED_VECTORS).map(make).collect())
        .unwrap();

    let searches_during_batch = Arc::new(AtomicUsize::new(0));

    let writer = {
        let store = Arc::clone(&store);
        std::thread::spawn(move || {
            let batch: Vec<Vector> = (SEED_VECTORS..SEED_VECTORS + BATCH_VECTORS)
                .map(make)
                .collect();
            let started = Instant::now();
            store.insert("mixed_load", batch).unwrap();
            started.elapsed()
        })
    };

    // Reader loop: run searches until the writer finishes, counting the
    // ones that complete while the batch is still in flight.
    let query = pseudo_vector(3);
    let mut slowest = Duration::ZERO;
    while !writer.is_finished() {
        let t = Instant::now();
        let results = store
            .get_collection("mixed_load")
            .unwrap()
            .search(&query, 5)
            .unwrap();
        let took = t.elapsed();
        slowest = slowest.max(took);
        assert!(!results.is_empty(), "seeded collection must return hits");
        if !writer.is_finished() {
            searches_during_batch.fetch_add(1, Ordering::Relaxed);
        }
    }

    let batch_took = writer.join().unwrap();
    let completed = searches_during_batch.load(Ordering::Relaxed);

    // Only meaningful when the batch actually ran long enough for the
    // reader loop to get scheduled at least once.
    if batch_took > Duration::from_millis(500) {
        assert!(
            completed > 0,
            "no search completed during a {batch_took:?} batch — the \
             index write lock is being held for the whole batch again \
             (slowest observed search: {slowest:?})"
        );
    } else {
        eprintln!(
            "batch finished in {batch_took:?} — too fast to prove \
             interleaving on this machine; {completed} searches overlapped"
        );
    }
}
