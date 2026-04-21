//! File-operation REST handlers.
//!
//! - [`operations`] — project-file introspection endpoints (content,
//!   list, summary, chunks, outline, related, search-by-type)
//! - [`upload`] — `/files/upload` multipart handler + `/files/config`
//! - [`validation`] — shared validators used by upload (size, MIME,
//!   extension, path safety)

pub mod operations;
pub mod upload;
pub mod validation;
