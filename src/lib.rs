//! Vectorizer - High-performance, in-memory vector database written in Rust
//!
//! This crate provides a fast and efficient vector database for semantic search
//! and similarity queries, designed for AI-driven applications.

pub mod db;
pub mod error;
pub mod models;
pub mod persistence;

// Re-export commonly used types
pub use db::{Collection, VectorStore};
pub use error::{Result, VectorizerError};
pub use models::{CollectionConfig, Payload, SearchResult, Vector};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
