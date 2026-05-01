//! Bulk-upsert backpressure primitives. Tracks issue
//! [#263](https://github.com/hivellm/vectorizer/issues/263).
//!
//! [`BackpressureGuard`] wraps a [`tokio::sync::Semaphore`] sized by
//! [`crate::config::BackpressureConfig::resolved_max_concurrent_vocab_builds`]
//! and exposes RAII-style permits via [`BackpressurePermit`]. Holding a
//! permit serializes the CPU-heavy section of a vocabulary build /
//! batch embed; permits release on drop, including unwind.
//!
//! The wiring of this guard into the actual hot path lives in later
//! phases of `phase9_bulk-upsert-backpressure`. This file only
//! introduces the primitive + its tests so the unit-level invariant
//! (at most N concurrent permit holders) is verified in isolation.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::config::BackpressureConfig;

/// Bounded gate around CPU-heavy vocabulary-build / batch-embed work.
/// Cheap to clone — it's an `Arc` internally.
#[derive(Debug, Clone)]
pub struct BackpressureGuard {
    sem: Arc<Semaphore>,
    capacity: usize,
    in_flight: Arc<AtomicUsize>,
    enabled: bool,
}

impl BackpressureGuard {
    /// Build a guard from a [`BackpressureConfig`]. When `enabled` is
    /// false, [`Self::acquire`] still returns a permit but no real
    /// semaphore work is done — used so callers don't have to branch.
    pub fn from_config(cfg: &BackpressureConfig) -> Self {
        let capacity = cfg.resolved_max_concurrent_vocab_builds();
        Self::with_capacity(capacity, cfg.enabled)
    }

    /// Build a guard with explicit capacity. Capacity is clamped to
    /// `>= 1` so the semaphore is never empty.
    pub fn with_capacity(capacity: usize, enabled: bool) -> Self {
        let capacity = capacity.max(1);
        Self {
            sem: Arc::new(Semaphore::new(capacity)),
            capacity,
            in_flight: Arc::new(AtomicUsize::new(0)),
            enabled,
        }
    }

    /// Total permit capacity (post-clamp).
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Permits currently held. Sourced from a counter rather than the
    /// semaphore so it can be sampled cheaply for metrics.
    pub fn in_flight(&self) -> usize {
        self.in_flight.load(Ordering::Relaxed)
    }

    /// Permits currently available for `acquire` to take without
    /// waiting. Reports `capacity` while disabled.
    pub fn available_permits(&self) -> usize {
        if !self.enabled {
            return self.capacity;
        }
        self.sem.available_permits()
    }

    /// Acquire a permit. While [`BackpressureConfig::enabled`] is
    /// false, returns a no-op permit immediately so callers don't have
    /// to branch on whether enforcement is active.
    pub async fn acquire(&self) -> BackpressurePermit {
        if !self.enabled {
            self.in_flight.fetch_add(1, Ordering::Relaxed);
            return BackpressurePermit {
                _permit: None,
                in_flight: Arc::clone(&self.in_flight),
            };
        }

        // INVARIANT: `self.sem` is private and the guard never calls
        // `Semaphore::close`, so `acquire_owned` cannot return `Err`.
        // Treating an err as a no-op permit is preferable to a panic
        // if that invariant is ever violated by a future refactor.
        let permit = Arc::clone(&self.sem).acquire_owned().await.ok();
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        BackpressurePermit {
            _permit: permit,
            in_flight: Arc::clone(&self.in_flight),
        }
    }

    /// Try to acquire a permit without waiting. Returns `None` when
    /// every permit is in use. Always returns `Some` while disabled.
    pub fn try_acquire(&self) -> Option<BackpressurePermit> {
        if !self.enabled {
            self.in_flight.fetch_add(1, Ordering::Relaxed);
            return Some(BackpressurePermit {
                _permit: None,
                in_flight: Arc::clone(&self.in_flight),
            });
        }

        let permit = Arc::clone(&self.sem).try_acquire_owned().ok()?;
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        Some(BackpressurePermit {
            _permit: Some(permit),
            in_flight: Arc::clone(&self.in_flight),
        })
    }
}

/// RAII handle held while a CPU-heavy section runs. Drop releases the
/// underlying semaphore permit *and* decrements the in-flight counter,
/// even on panic unwind.
#[derive(Debug)]
#[must_use = "drop the permit at the end of the gated section, not before"]
pub struct BackpressurePermit {
    _permit: Option<OwnedSemaphorePermit>,
    in_flight: Arc<AtomicUsize>,
}

impl Drop for BackpressurePermit {
    fn drop(&mut self) {
        self.in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}
