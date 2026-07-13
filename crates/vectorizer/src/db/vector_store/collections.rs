//! Collection lifecycle — create, delete, lookup (with lazy loading
//! from disk), ownership, graph enablement, and empty-collection
//! cleanup.
//!
//! Split by concern (phase41 §4.2) so each file stays reviewable:
//!
//! - [`lifecycle`]  — create / rename / delete, graph enablement,
//!   empty-collection cleanup
//! - [`disk_load`]  — `get_collection` / `get_collection_mut`, the two
//!   workhorses that resolve alias chains, try the in-memory `DashMap`
//!   first, and fall back to `.vecdb` (compact) / legacy `.bin` (raw)
//!   on disk
//! - [`tenancy`]    — ownership / multi-tenancy queries built on top of
//!   the two accessors above

mod disk_load;
mod lifecycle;
mod tenancy;
