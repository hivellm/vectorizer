//! Integration tests for [`vectorizer::db::UpsertQueue`]
//! (issue #263, phase9 §3).
//!
//! Covers per-collection isolation, hard-limit refusal, the CAS-based
//! admission race, high-water status, RAII drop, and disabled-mode
//! pass-through.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use vectorizer::config::BackpressureConfig;
use vectorizer::db::{AdmissionError, AdmissionStatus, UpsertQueue};

fn cfg(high_water: usize, hard_limit: usize) -> BackpressureConfig {
    BackpressureConfig {
        upsert_queue_high_water: high_water,
        upsert_queue_hard_limit: hard_limit,
        ..BackpressureConfig::default()
    }
}

#[test]
fn admit_then_drop_balances_to_zero() {
    let q = UpsertQueue::from_config(&cfg(2, 4));
    {
        let (_t, status) = q.try_admit("alpha").unwrap();
        assert_eq!(status, AdmissionStatus::AdmittedNormal);
        assert_eq!(q.depth("alpha"), 1);
    }
    assert_eq!(q.depth("alpha"), 0);
}

#[test]
fn high_water_status_above_threshold() {
    let q = UpsertQueue::from_config(&cfg(2, 4));
    let (_t1, s1) = q.try_admit("alpha").unwrap();
    assert_eq!(s1, AdmissionStatus::AdmittedNormal);

    let (_t2, s2) = q.try_admit("alpha").unwrap();
    // Depth = 2, high_water = 2 → first admission AT high_water.
    assert_eq!(s2, AdmissionStatus::AdmittedHighWater);

    let (_t3, s3) = q.try_admit("alpha").unwrap();
    assert_eq!(s3, AdmissionStatus::AdmittedHighWater);
}

#[test]
fn hard_limit_refuses_with_retry_after() {
    let bp = cfg(2, 3);
    let q = UpsertQueue::from_config(&bp);

    // Fill to hard limit.
    let _t1 = q.try_admit("alpha").unwrap().0;
    let _t2 = q.try_admit("alpha").unwrap().0;
    let _t3 = q.try_admit("alpha").unwrap().0;
    assert_eq!(q.depth("alpha"), 3);

    let err = q.try_admit("alpha").expect_err("4th must be refused");
    match err {
        AdmissionError::QueueFull {
            depth,
            hard_limit,
            retry_after_seconds,
        } => {
            assert_eq!(depth, 3);
            assert_eq!(hard_limit, 3);
            assert_eq!(retry_after_seconds, bp.retry_after_seconds);
        }
    }
}

#[test]
fn collections_are_isolated() {
    let q = UpsertQueue::from_config(&cfg(2, 3));

    let _t = q.try_admit("alpha").unwrap().0;
    let _u = q.try_admit("alpha").unwrap().0;
    let _v = q.try_admit("alpha").unwrap().0;

    // alpha is full; beta starts fresh.
    assert!(q.try_admit("alpha").is_err());
    let (_beta_ticket, _) = q
        .try_admit("beta")
        .expect("beta has its own counter and starts fresh");
    assert_eq!(q.depth("alpha"), 3);
    assert_eq!(q.depth("beta"), 1);
}

#[test]
fn manual_release_decrements_immediately() {
    let q = UpsertQueue::from_config(&cfg(2, 4));
    let (t, _) = q.try_admit("alpha").unwrap();
    assert_eq!(q.depth("alpha"), 1);
    t.release();
    assert_eq!(q.depth("alpha"), 0);
}

#[test]
fn drop_releases_on_unwind() {
    let q = UpsertQueue::from_config(&cfg(2, 4));

    let q_for_panic = q.clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _t = q_for_panic.try_admit("alpha").unwrap();
        panic!("simulated failure with ticket held");
    }));
    assert!(result.is_err());

    assert_eq!(
        q.depth("alpha"),
        0,
        "ticket must be released even on panic unwind",
    );
}

#[test]
fn disabled_mode_admits_above_hard_limit() {
    let bp = BackpressureConfig {
        enabled: false,
        upsert_queue_high_water: 1,
        upsert_queue_hard_limit: 2,
        ..BackpressureConfig::default()
    };
    let q = UpsertQueue::from_config(&bp);

    // Admit 5 — would refuse the 3rd in enabled mode.
    let mut tickets = Vec::new();
    for _ in 0..5 {
        let (t, _) = q.try_admit("alpha").expect("disabled queue admits all");
        tickets.push(t);
    }
    assert_eq!(q.depth("alpha"), 5);
}

#[test]
fn cas_admission_does_not_overshoot_hard_limit_under_contention() {
    // Multiple threads race to admit the same collection. The
    // post-admission depth must never exceed `hard_limit`. Validates
    // the CAS loop in `try_admit` against the naive
    // increment-then-check-then-decrement race.
    use std::thread;

    let bp = cfg(8, 16);
    let q = UpsertQueue::from_config(&bp);
    let attempts = 256;
    let admitted = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for _ in 0..16 {
        let q = q.clone();
        let admitted = Arc::clone(&admitted);
        handles.push(thread::spawn(move || {
            let mut local: Vec<vectorizer::db::UpsertTicket> = Vec::new();
            for _ in 0..attempts {
                if let Ok((t, _)) = q.try_admit("alpha") {
                    admitted.fetch_add(1, Ordering::SeqCst);
                    local.push(t);
                }
            }
            // Drop tickets only at thread exit so we observe the peak.
            local
        }));
    }
    let mut all_tickets: Vec<vectorizer::db::UpsertTicket> = Vec::new();
    for h in handles {
        all_tickets.extend(h.join().unwrap());
    }

    assert!(
        q.depth("alpha") <= bp.upsert_queue_hard_limit,
        "depth {} must never exceed hard_limit {}",
        q.depth("alpha"),
        bp.upsert_queue_hard_limit,
    );
    drop(all_tickets);
    assert_eq!(q.depth("alpha"), 0);
}
