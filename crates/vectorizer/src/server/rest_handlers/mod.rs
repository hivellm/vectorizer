//! REST API handlers.
//!
//! The original `rest_handlers.rs` grew to ~3800 lines covering 50+
//! endpoints across 10 concerns. It is now split into sibling files
//! so that each REST concern is reviewable in isolation:
//!
//! - [`common`]             — shared helpers (tenant extraction, metrics UUID)
//! - [`meta`]               — /health, /stats, /indexing/progress, /status,
//!                            /logs, /metrics (Prometheus)
//! - [`collections`]        — collection CRUD + /collections/empty cleanup
//! - [`vectors`]            — vector CRUD + embed + batch insert
//! - [`insert`]             — /insert_text (the big chunk-and-embed endpoint)
//! - [`search`]             — text / hybrid / file search + batch ops
//! - [`intelligent_search`] — high-level orchestrator: intelligent / multi /
//!                            semantic / contextual
//! - [`discovery`]          — the /discover pipeline stages (filter, score,
//!                            expand, broad, focus, promote, compress,
//!                            plan, render)
//! - [`files`]              — file-navigation endpoints (content, summary,
//!                            chunks, outline, related, by-type search)
//! - [`admin`]              — workspace CRUD + /config + /admin/restart
//! - [`backups`]            — /backups list / create / restore / dir
//!
//! The public surface is preserved verbatim via `pub use`: every name
//! that `src/server/mod.rs` used to reach as `rest_handlers::X` is still
//! available at exactly that path. Route wiring in `src/server/mod.rs`
//! is unchanged.

mod admin;
mod backups;
mod collections;
mod common;
mod discovery;
mod files;
mod insert;
mod intelligent_search;
mod meta;
mod search;
mod vectors;

pub use admin::{
    add_workspace, get_config, get_workspace_config, list_workspaces, remove_workspace,
    restart_server, update_config, update_workspace_config,
};
pub use backups::{create_backup, get_backup_directory, list_backups, restore_backup};
pub use collections::{
    cleanup_empty_collections, create_collection, delete_collection, force_save_collection,
    get_collection, list_collections, list_empty_collections,
};
pub(crate) use common::collection_metrics_uuid;
pub use discovery::{
    broad_discovery, build_answer_plan, compress_evidence, discover, expand_queries,
    filter_collections, promote_readme, render_llm_prompt, score_collections, semantic_focus,
};
pub use files::{
    get_file_chunks_ordered, get_file_content, get_file_summary, get_project_outline,
    get_related_files, list_files_in_collection, search_by_file_type,
};
pub use insert::insert_text;
pub use intelligent_search::{
    contextual_search, intelligent_search, multi_collection_search, semantic_search,
};
pub use meta::{
    get_indexing_progress, get_logs, get_prometheus_metrics, get_stats, get_status, health_check,
};
pub use search::{
    batch_delete_vectors, batch_search_vectors, batch_update_vectors, hybrid_search_vectors,
    search_by_file, search_vectors, search_vectors_by_text,
};
pub use vectors::{
    batch_insert_texts, delete_vector, delete_vector_generic, embed_text, get_vector, insert_texts,
    list_vectors, update_vector,
};

#[cfg(test)]
#[path = "../rest_handlers_tests.rs"]
mod tests;
