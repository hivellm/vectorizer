//! Qdrant Compatibility Tests

// Each file is a separate module to avoid import conflicts
#[path = "qdrant_api.rs"]
mod qdrant_api;

#[path = "qdrant_migration.rs"]
mod qdrant_migration;
