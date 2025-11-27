//! Qdrant API compatibility models
//!
//! This module provides data structures and types that are compatible with Qdrant's API,
//! enabling seamless integration and migration from Qdrant to Vectorizer.

pub mod alias;
pub mod batch;
pub mod cluster;
pub mod collection;
pub mod error;
pub mod filter;
pub mod filter_processor;
pub mod point;
pub mod search;
pub mod sharding;
pub mod snapshot;

pub use alias::*;
pub use batch::*;
pub use cluster::*;
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
pub use sharding::*;
pub use snapshot::*;
