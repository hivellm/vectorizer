//! Qdrant API compatibility models
//!
//! This module provides data structures and types that are compatible with Qdrant's API,
//! enabling seamless integration and migration from Qdrant to Vectorizer.

pub mod batch;
pub mod collection;
pub mod error;
pub mod filter;
pub mod filter_processor;
pub mod point;
pub mod search;

pub use batch::*;
pub use collection::*;
pub use error::*;
pub use filter::*;
pub use filter_processor::FilterProcessor;
// Re-export specific types to avoid ambiguity
pub use point::QdrantOperationStatus as PointOperationStatus;
pub use point::{
    QdrantCountResult as PointCountResult, QdrantOperationStatus as SearchOperationStatus,
    QdrantScrollResult as PointScrollResult, *,
};
pub use search::{
    QdrantCountResult as SearchCountResult, QdrantScrollResult as SearchScrollResult, *,
};
