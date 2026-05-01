//! Integration tests for [`vectorizer::db::BackpressureGuard`]
//! (issue #263).
//!
//! Validates the proposal's non-negotiable invariant — with N permits
//! and M > N concurrent acquirers, at most N permits are ever held at
//! the same time — plus the disabled-mode pass-through, RAII drop,
//! and unwind-safety of the permit handle.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::Barrier;
use vectorizer::config::BackpressureConfig;
use vectorizer::db::BackpressureGuard;

#[test]
fn capacity_clamped_to_at_least_one() {
    let g = BackpressureGuard::with_capacity(0, true);
    assert_eq!(g.capacity(), 1);
    assert_eq!(g.available_permits(), 1);
}

#[test]
fn from_config_uses_resolved_max() {
    let cfg = BackpressureConfig {
        max_concurrent_vocab_builds: 5,
        ..BackpressureConfig::default()
    };
    let g = BackpressureGuard::from_config(&cfg);
    assert_eq!(g.capacity(), 5);
}

#[test]
fn capacity_zero_in_config_resolves_to_num_cpus() {
    let cfg = BackpressureConfig::default(); // max = 0
    let g = BackpressureGuard::from_config(&cfg);
    assert_eq!(g.capacity(), num_cpus::get().max(1));
}

#[tokio::test]
async fn permit_drop_releases_back_to_pool() {
    let g = BackpressureGuard::with_capacity(2, true);
    assert_eq!(g.available_permits(), 2);

    {
        let _p = g.acquire().await;
        assert_eq!(g.available_permits(), 1);
        assert_eq!(g.in_flight(), 1);
    }

    assert_eq!(g.available_permits(), 2);
    assert_eq!(g.in_flight(), 0);
}

#[tokio::test]
async fn try_acquire_fails_when_full() {
    let g = BackpressureGuard::with_capacity(1, true);
    let _p = g.acquire().await;

    assert!(g.try_acquire().is_none());
}

#[tokio::test]
async fn try_acquire_succeeds_after_drop() {
    let g = BackpressureGuard::with_capacity(1, true);

    {
        let _p = g.acquire().await;
        assert!(g.try_acquire().is_none());
    }

    assert!(g.try_acquire().is_some());
}

#[tokio::test]
async fn disabled_skips_semaphore_but_still_tracks_in_flight() {
    let g = BackpressureGuard::with_capacity(1, false);
    let p1 = g.acquire().await;
    let p2 = g.acquire().await; // would block in enabled mode
    assert_eq!(g.in_flight(), 2);
    drop(p1);
    drop(p2);
    assert_eq!(g.in_flight(), 0);
}

#[tokio::test]
async fn at_most_n_concurrent_holders() {
    // The non-negotiable invariant from the proposal: with N permits
    // and M > N concurrent acquirers, at most N permits are ever held
    // at the same time. Verified via a peak-counter sampled inside the
    // gated section.
    const N: usize = 4;
    const M: usize = 32;

    let guard = BackpressureGuard::with_capacity(N, true);
    let active = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    // Synchronizes M acquirers so they really fight for the same N
    // permits rather than running serially because they were spawned
    // too far apart.
    let barrier = Arc::new(Barrier::new(M));

    let mut handles = Vec::with_capacity(M);
    for _ in 0..M {
        let guard = guard.clone();
        let active = Arc::clone(&active);
        let peak = Arc::clone(&peak);
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            let _permit = guard.acquire().await;
            let now = active.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(now, Ordering::SeqCst);
            // Hold the permit briefly so contention is real.
            tokio::time::sleep(Duration::from_millis(5)).await;
            active.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let observed_peak = peak.load(Ordering::SeqCst);
    assert!(
        observed_peak <= N,
        "at most {N} concurrent permit holders, observed {observed_peak}",
    );
    assert_eq!(guard.in_flight(), 0);
    assert_eq!(guard.available_permits(), N);
}

#[tokio::test]
async fn drop_releases_on_unwind() {
    // Even if the gated section panics and unwinds, the permit is
    // returned. Drop order on a panic is part of the standard library
    // contract — this test locks it in for the project so a future
    // refactor can't silently regress it.
    let guard = BackpressureGuard::with_capacity(1, true);

    let result = tokio::spawn({
        let guard = guard.clone();
        async move {
            let _permit = guard.acquire().await;
            panic!("simulated failure inside gated section");
        }
    })
    .await;

    assert!(result.is_err(), "task should have panicked");
    assert_eq!(guard.available_permits(), 1);
    assert_eq!(guard.in_flight(), 0);

    // And we can take a fresh permit afterwards.
    let _p = guard.acquire().await;
}
