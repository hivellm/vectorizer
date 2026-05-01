//! Read-path isolation under write saturation (issue #263, phase9 §5).
//!
//! The proposal specs that `/health`, `GET /collections`, and `/auth/*`
//! must keep responding (p99 < 500 ms) while the write side is saturated.
//! Phase 2 caps concurrent BM25 vocab builds to `num_cpus`, and phase 3
//! caps per-collection upserts at `upsert_queue_hard_limit`; together
//! those guarantees prevent the runaway-CPU failure mode from
//! issue #263 without needing a literal separate Tokio runtime.
//!
//! This test verifies the guarantee at the primitive level: with a
//! [`BackpressureGuard`] saturated by N+1 vocab-build holders, an
//! unrelated read-path acquire (modeled as a no-op clone) still
//! completes within a tight bound. Treats the guard as the closest
//! observable proxy for "writers ate the runtime"; the full HTTP
//! load test belongs in operator-side benchmarks (see
//! docs/operations/backpressure.md).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Barrier;
use vectorizer::config::BackpressureConfig;
use vectorizer::db::{BackpressureGuard, UpsertQueue};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn read_path_stays_responsive_while_writers_saturate_guard() {
    // 2 vocab-build permits, 8 contending writers — guard is wedged.
    let cfg = BackpressureConfig {
        max_concurrent_vocab_builds: 2,
        ..BackpressureConfig::default()
    };
    let guard = BackpressureGuard::from_config(&cfg);
    assert_eq!(guard.capacity(), 2);

    let writers = 8;
    let barrier = Arc::new(Barrier::new(writers + 1));

    // Spawn writers that hold their permits for ~250ms each so the
    // guard is fully saturated for the duration of the read probe.
    let mut handles = Vec::with_capacity(writers);
    for _ in 0..writers {
        let guard = guard.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            let _permit = guard.acquire().await;
            tokio::time::sleep(Duration::from_millis(250)).await;
        }));
    }

    // Sync everyone, then immediately exercise read-path operations.
    barrier.wait().await;
    tokio::time::sleep(Duration::from_millis(20)).await; // let writers grab their permits

    // Read-path proxy: cloning the guard, sampling depth, and
    // touching an UpsertQueue must NOT contend on the saturated
    // semaphore. Each operation should finish in well under 500 ms.
    let read_queue = UpsertQueue::from_config(&cfg);
    let started = Instant::now();
    for _ in 0..1000 {
        // Cheap operations that the read path actually performs:
        // - sample depth for /metrics
        // - inspect available_permits for /metrics
        // - check existence in DashMap (mirrors /collections lookups)
        let _ = guard.available_permits();
        let _ = guard.in_flight();
        let _ = read_queue.depth("nonexistent");
        let _ = read_queue.snapshot_depths();
    }
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_millis(500),
        "1000 read-path probes against a saturated guard took {elapsed:?} \
         (expected < 500ms total — read path must not contend on writer semaphore)",
    );

    for h in handles {
        h.await.unwrap();
    }
    assert_eq!(guard.in_flight(), 0);
}
