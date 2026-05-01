//! Integration test for the gated BM25 vocab-build path
//! ([`vectorizer::file_loader::Indexer::build_vocabulary_gated`],
//! issue #263).
//!
//! Verifies that when N indexers share a single [`BackpressureGuard`]
//! with capacity K, at most K vocabulary builds are ever in flight at
//! once. The peak is sampled from the guard's own `in_flight()`
//! counter, so the assertion holds end-to-end through the indexer
//! wrapper rather than testing the primitive in isolation.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use vectorizer::config::BackpressureConfig;
use vectorizer::db::BackpressureGuard;
use vectorizer::embedding::{Bm25Embedding, EmbeddingManager};
use vectorizer::file_loader::{Indexer, LoaderConfig};

fn make_indexer(collection: &str, guard: BackpressureGuard) -> Indexer {
    let mut manager = EmbeddingManager::new();
    manager.register_provider("bm25".to_string(), Box::new(Bm25Embedding::new(64)));
    manager.set_default_provider("bm25").unwrap();

    let cfg = LoaderConfig {
        max_chunk_size: 512,
        chunk_overlap: 64,
        include_patterns: vec!["**/*".to_string()],
        exclude_patterns: vec![],
        embedding_dimension: 64,
        embedding_type: "bm25".to_string(),
        collection_name: collection.to_string(),
        max_file_size: 1024 * 1024,
    };

    Indexer::with_embedding_manager(cfg, manager).with_backpressure(guard)
}

fn fake_corpus(tag: &str, n: usize) -> Vec<(PathBuf, String)> {
    // Enough docs that build_vocabulary has nontrivial CPU cost so
    // we can actually observe contention from a sampler task.
    (0..n)
        .map(|i| {
            (
                PathBuf::from(format!("{tag}/doc{i}.txt")),
                format!("{tag} document {i} alpha beta gamma delta epsilon"),
            )
        })
        .collect()
}

async fn run_with_capacity(capacity: usize, num_indexers: usize) -> usize {
    let cfg = BackpressureConfig {
        max_concurrent_vocab_builds: capacity,
        ..BackpressureConfig::default()
    };
    let guard = BackpressureGuard::from_config(&cfg);
    assert_eq!(guard.capacity(), capacity);

    let stop = Arc::new(AtomicBool::new(false));
    let peak = Arc::new(AtomicUsize::new(0));

    let sampler = {
        let guard = guard.clone();
        let peak = Arc::clone(&peak);
        let stop = Arc::clone(&stop);
        tokio::spawn(async move {
            while !stop.load(Ordering::Acquire) {
                let now = guard.in_flight();
                peak.fetch_max(now, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_micros(200)).await;
            }
            // One last sample after the stop signal so we don't miss
            // a peak that occurred between the last sleep and stop.
            peak.fetch_max(guard.in_flight(), Ordering::SeqCst);
        })
    };

    let mut handles = Vec::with_capacity(num_indexers);
    for n in 0..num_indexers {
        let mut indexer = make_indexer(&format!("coll_{n}"), guard.clone());
        let docs = fake_corpus(&format!("t{n}"), 800);
        handles.push(tokio::spawn(async move {
            indexer
                .build_vocabulary_gated(&docs)
                .await
                .expect("gated build must succeed");
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    stop.store(true, Ordering::Release);
    sampler.await.unwrap();
    assert_eq!(guard.in_flight(), 0, "all permits returned at end");

    peak.load(Ordering::SeqCst)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn capacity_one_serializes_six_concurrent_builds() {
    let observed_peak = run_with_capacity(1, 6).await;
    assert!(
        observed_peak <= 1,
        "capacity = 1: peak in_flight must be <= 1, observed {observed_peak}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn capacity_two_caps_eight_concurrent_builds() {
    let observed_peak = run_with_capacity(2, 8).await;
    assert!(
        observed_peak <= 2,
        "capacity = 2: peak in_flight must be <= 2, observed {observed_peak}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn no_guard_attached_runs_unbounded() {
    // Without `.with_backpressure(...)`, the gated method must still
    // succeed and behave like the legacy sync `build_vocabulary`.
    // Pins the backwards-compat contract: existing callers that don't
    // opt in see no behavior change.
    let mut manager = EmbeddingManager::new();
    manager.register_provider("bm25".to_string(), Box::new(Bm25Embedding::new(64)));
    manager.set_default_provider("bm25").unwrap();

    let cfg = LoaderConfig {
        max_chunk_size: 512,
        chunk_overlap: 64,
        include_patterns: vec!["**/*".to_string()],
        exclude_patterns: vec![],
        embedding_dimension: 64,
        embedding_type: "bm25".to_string(),
        collection_name: "no-guard".to_string(),
        max_file_size: 1024 * 1024,
    };
    let mut indexer = Indexer::with_embedding_manager(cfg, manager);

    indexer
        .build_vocabulary_gated(&fake_corpus("legacy", 100))
        .await
        .expect("ungated build must succeed");
}
