//! Qdrant Compatibility Tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

// Each file is a separate module to avoid import conflicts
#[path = "qdrant_api.rs"]
mod qdrant_api;

#[path = "qdrant_migration.rs"]
mod qdrant_migration;
