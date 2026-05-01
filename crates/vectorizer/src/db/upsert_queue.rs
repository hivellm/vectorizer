//! Per-collection in-flight upsert tracker (issue #263, phase9 §3).
//!
//! [`UpsertQueue`] hands out RAII admission tickets that increment a
//! per-collection counter on `try_admit` and decrement on drop. When
//! the counter would cross the configured hard limit, admission is
//! rejected with [`AdmissionError::QueueFull`]; REST / gRPC / MCP
//! callers translate that into HTTP 429 / gRPC `RESOURCE_EXHAUSTED` /
//! a structured MCP error respectively.
//!
//! The high-water mark is informational — depths between high-water
//! and hard-limit emit a warn log + bump the
//! `vectorizer_upsert_rejected_total{reason="queue_high_water_warn"}`
//! counter (registered in phase 4) but admit the request.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dashmap::DashMap;

use crate::config::BackpressureConfig;

/// Reason an admission was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    /// In-flight depth is at or above `upsert_queue_hard_limit`.
    /// Caller should reply 429 with `Retry-After`.
    QueueFull {
        /// Current depth at the moment of refusal.
        depth: usize,
        /// Hard limit that was exceeded.
        hard_limit: usize,
        /// Suggested retry delay in seconds (mirrors
        /// [`BackpressureConfig::retry_after_seconds`]).
        retry_after_seconds: u32,
    },
}

/// Tracks per-collection in-flight upsert depth and hands out RAII
/// admission tickets. Cheap to clone (everything is `Arc` inside).
#[derive(Debug, Clone)]
pub struct UpsertQueue {
    /// Lock-free per-collection depth counter. Entries are created
    /// lazily on first admission and never removed — collection churn
    /// is expected to be low and removal would race with concurrent
    /// admissions.
    depth: Arc<DashMap<String, Arc<AtomicUsize>>>,
    /// Snapshot of the relevant config at construction time.
    /// Re-creating the queue is cheap if config is reloaded.
    cfg: Arc<UpsertQueueConfig>,
}

#[derive(Debug, Clone, Copy)]
struct UpsertQueueConfig {
    enabled: bool,
    high_water: usize,
    hard_limit: usize,
    retry_after_seconds: u32,
}

impl UpsertQueue {
    /// Build a queue with limits sourced from a [`BackpressureConfig`].
    /// When `enabled` is false, [`Self::try_admit`] always succeeds
    /// (depth is still tracked so metrics report something useful).
    pub fn from_config(cfg: &BackpressureConfig) -> Self {
        Self {
            depth: Arc::new(DashMap::new()),
            cfg: Arc::new(UpsertQueueConfig {
                enabled: cfg.enabled,
                high_water: cfg.upsert_queue_high_water,
                hard_limit: cfg.upsert_queue_hard_limit,
                retry_after_seconds: cfg.retry_after_seconds,
            }),
        }
    }

    /// Build a permissive queue that always admits — depth tracking
    /// still works, but no request is ever refused. Intended for
    /// tests and embedding contexts that don't enforce backpressure.
    pub fn permissive() -> Self {
        Self::from_config(&BackpressureConfig {
            enabled: false,
            ..BackpressureConfig::default()
        })
    }

    /// Current in-flight depth for a collection. Returns `0` for
    /// collections that haven't seen any admissions yet.
    pub fn depth(&self, collection: &str) -> usize {
        self.depth
            .get(collection)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Snapshot every known collection's current depth. Used by the
    /// `/prometheus/metrics` handler to refresh per-collection gauges
    /// at scrape time so reads of stale counters can't mask a depth
    /// that grew between admissions.
    pub fn snapshot_depths(&self) -> Vec<(String, usize)> {
        self.depth
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().load(Ordering::Relaxed)))
            .collect()
    }

    /// Configured hard limit. Useful for metrics labels and for
    /// reporting back in the `Retry-After` response body.
    pub fn hard_limit(&self) -> usize {
        self.cfg.hard_limit
    }

    /// Configured `Retry-After` value (seconds).
    pub fn retry_after_seconds(&self) -> u32 {
        self.cfg.retry_after_seconds
    }

    /// Attempt to admit one in-flight upsert against `collection`.
    /// On success returns an [`UpsertTicket`] that auto-decrements the
    /// counter on drop, even on panic unwind. On refusal returns
    /// [`AdmissionError::QueueFull`].
    ///
    /// Returns [`AdmissionStatus::AdmittedHighWater`] when the depth
    /// after this admission is at or above the configured high-water
    /// mark — callers may emit a warn log / bump a counter, but the
    /// upsert is still admitted.
    pub fn try_admit(
        &self,
        collection: &str,
    ) -> Result<(UpsertTicket, AdmissionStatus), AdmissionError> {
        // Look up or create the per-collection counter. We use
        // `entry().or_insert_with` because a `get` followed by an
        // `insert` would race two admissions for the same fresh
        // collection.
        let counter = self
            .depth
            .entry(collection.to_string())
            .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
            .clone();

        if !self.cfg.enabled {
            counter.fetch_add(1, Ordering::AcqRel);
            return Ok((
                UpsertTicket {
                    counter,
                    released: false,
                },
                AdmissionStatus::AdmittedNormal,
            ));
        }

        // CAS-style increment: only commit if we'd stay under the
        // hard limit. Avoids the "increment then check then decrement
        // on overflow" race that lets one extra request slip through
        // under contention.
        let mut prev = counter.load(Ordering::Acquire);
        loop {
            if prev >= self.cfg.hard_limit {
                return Err(AdmissionError::QueueFull {
                    depth: prev,
                    hard_limit: self.cfg.hard_limit,
                    retry_after_seconds: self.cfg.retry_after_seconds,
                });
            }
            match counter.compare_exchange_weak(prev, prev + 1, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(observed) => prev = observed,
            }
        }
        let new_depth = prev + 1;
        let status = if new_depth >= self.cfg.high_water {
            AdmissionStatus::AdmittedHighWater
        } else {
            AdmissionStatus::AdmittedNormal
        };

        Ok((
            UpsertTicket {
                counter,
                released: false,
            },
            status,
        ))
    }
}

/// Status returned alongside a successful admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionStatus {
    /// Depth is below the high-water mark.
    AdmittedNormal,
    /// Depth is at or above the high-water mark but still under the
    /// hard limit. Callers should warn-log and increment a counter
    /// but accept the upsert.
    AdmittedHighWater,
}

/// RAII handle held while an upsert is in flight. Drop decrements the
/// per-collection counter, including on panic unwind.
#[derive(Debug)]
#[must_use = "drop the ticket only when the upsert is fully complete"]
pub struct UpsertTicket {
    counter: Arc<AtomicUsize>,
    released: bool,
}

impl UpsertTicket {
    /// Manually release the ticket before drop. Useful when the
    /// caller wants to record the depth post-release for metrics
    /// before the ticket goes out of scope.
    pub fn release(mut self) {
        self.do_release();
    }

    fn do_release(&mut self) {
        if !self.released {
            self.counter.fetch_sub(1, Ordering::AcqRel);
            self.released = true;
        }
    }
}

impl Drop for UpsertTicket {
    fn drop(&mut self) {
        self.do_release();
    }
}
