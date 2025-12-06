//! GraphQL API module for Vectorizer
//!
//! This module provides a full GraphQL API with:
//! - Query operations (collections, vectors, search, graph)
//! - Mutation operations (create, update, delete)
//! - GraphQL Playground for interactive exploration
//!
//! # Example
//!
//! ```graphql
//! query {
//!   collections {
//!     name
//!     vectorCount
//!     config {
//!       dimension
//!       metric
//!     }
//!   }
//! }
//! ```

mod schema;
mod types;

#[cfg(test)]
mod tests;

pub use schema::{
    VectorizerSchema, create_schema, create_schema_with_auto_save, create_schema_with_hub,
};
pub use types::*;
