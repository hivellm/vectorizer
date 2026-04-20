//! HiveHub REST handlers (backups, tenants, usage).
//!
//! These are the server-facing endpoints that front the `vectorizer::hub`
//! managers. The directory is named `hub_handlers/` rather than `hub/`
//! to avoid shadowing the crate-level [`vectorizer::hub`] module inside
//! `src/server/`.
//!
//! - [`backup`] — per-user backup list / create / restore / upload /
//!   download / delete
//! - [`tenant`] — tenant statistics / migration / cleanup (currently
//!   unwired because of an axum/tonic version conflict; kept as a
//!   module so the code isn't lost)
//! - [`usage`] — usage statistics / quota / API-key validation

pub mod backup;
// pub mod tenant; // Disabled due to axum version conflicts with tonic.
// The module stays on disk (hub_handlers/tenant.rs) so the code isn't
// lost; wire it back up once that conflict is resolved.
pub mod usage;
