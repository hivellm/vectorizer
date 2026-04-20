//! Integration tests for the query cache (`src/cache/query_cache.rs`).
//!
//! These tests exercise the cache as it lives at the integration tier
//! — invalidation flows, concurrent read/write consistency, and the
//! `cached_or_compute` helper's hit/miss accounting end-to-end. The
//! Prometheus-counter wiring inside `QueryCache::get` is also covered
//! here so we know the live `/prometheus/metrics` scrape will reflect
//! cache behaviour.

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod query_cache_behaviour;
