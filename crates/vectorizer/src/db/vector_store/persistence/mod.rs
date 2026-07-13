//! Persistence — loading collections from disk (compact `.vecdb`
//! archive or legacy per-collection `.bin` files), cache-path helpers
//! for the HNSW dump-assisted fast load, and native per-collection
//! snapshots.
//!
//! The save side lives in [`super::autosave`]. Split into submodules
//! along the concern seams in phase41 (§4.3): the old 884-line
//! single file mixed snapshot management with the three loading
//! paths.
//!
//! - [`snapshots`] — native per-collection snapshots + reindex
//! - [`loading`]   — `.vecdb` / legacy / dynamic collection loading

mod loading;
mod snapshots;

pub use snapshots::NativeSnapshotInfo;
