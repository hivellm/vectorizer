//! Qdrant-compatible REST handlers.
//!
//! Each submodule below implements one slice of the Qdrant HTTP API so
//! that existing Qdrant clients can point at Vectorizer unchanged:
//!
//! - [`handlers`]         — collections CRUD
//! - [`vector_handlers`]  — points upsert/retrieve/delete/scroll/count
//! - [`search_handlers`]  — search + batch / recommend / matrix / groups
//! - [`query_handlers`]   — Query API (Qdrant 1.7+)
//! - [`alias_handlers`]   — collection aliases
//! - [`snapshot_handlers`] — per-collection and full snapshots
//! - [`sharding_handlers`] — shard key management
//! - [`cluster_handlers`]  — cluster status / metadata / peer
//!
//! Route wiring (the `.route()` chain) lives in
//! [`crate::server::core::routing`].

pub mod alias_handlers;
pub mod cluster_handlers;
pub mod handlers;
pub mod query_handlers;
pub mod search_handlers;
pub mod sharding_handlers;
pub mod snapshot_handlers;
pub mod vector_handlers;
