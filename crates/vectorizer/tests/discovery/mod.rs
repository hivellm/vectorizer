//! Integration tests for the discovery pipeline (`src/discovery/`).
//!
//! `src/discovery/` is the **search-result orchestration pipeline**
//! (filter → score → expand → broad → focus → compress → readme →
//! plan → render). Despite the name it does NOT walk the filesystem,
//! parse manifests, or touch path-traversal surfaces — those concerns
//! live in `src/file_loader/` and `src/file_watcher/` and have their
//! own test slots.
//!
//! These tests cover:
//!
//! - [`basics`] — happy-path `Discovery::discover` against an
//!   in-memory `VectorStore` with a known small collection; asserts
//!   pipeline returns + populates metrics.
//! - [`filter_score`] — `filter_collections` + `score_collections`
//!   against diverse name patterns and include/exclude lists.
//! - [`expand`] — `expand_queries_baseline` with every toggle
//!   combination + the `max_expansions` truncation edge case.
//! - [`compress`] — `compress_evidence` with empty / single /
//!   many-chunk inputs and per-doc cap behaviour.
//! - [`concurrent`] — multiple `Discovery::discover` calls running
//!   in parallel against the same store don't race or panic.

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod basics;
pub mod compress;
pub mod concurrent;
pub mod expand;
pub mod filter_score;
