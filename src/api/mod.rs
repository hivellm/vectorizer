//! # API Module
//!
//! Advanced API and integration layer for Vectorizer.
//!
//! ## Overview
//!
//! This module provides comprehensive API capabilities for the Vectorizer service,
//! exposing vector database functionality through multiple interfaces including
//! REST, GraphQL, and Graph APIs.
//!
//! ## Key Components
//!
//! - **REST API**: Traditional HTTP endpoints for collection management, vector
//!   operations, and search (see `src/server/rest_handlers.rs`)
//! - **GraphQL API**: Flexible query interface with schema introspection and
//!   interactive playground
//! - **Graph API**: Relationship and graph traversal operations for connected
//!   data
//! - **Advanced API**: Extended capabilities including rate limiting, analytics,
//!   versioning, and SDK generation
//! - **Cluster API**: Distributed cluster operations and sharding
//!
//! ## API Interfaces
//!
//! ### REST API
//!
//! The REST API provides standard HTTP endpoints for all vector operations:
//!
//! - Collection management: `GET/POST/DELETE /collections`
//! - Vector operations: `POST /insert`, `POST /update`, `POST /delete`
//! - Search operations: `POST /collections/{name}/search`
//! - File operations: `POST /files/upload`
//! - Graph operations: `GET/POST /graph/*`
//!
//! ### GraphQL API
//!
//! The GraphQL API provides a flexible query interface:
//!
//! - Query operations for collections, vectors, and search
//! - Mutation operations for data modification
//! - GraphQL Playground at `/graphql` or `/graphiql`
//! - Schema introspection and type system
//!
//! ### Graph API
//!
//! The Graph API enables relationship and graph operations:
//!
//! - Node and edge management
//! - Graph traversal and path finding
//! - Relationship discovery
//! - Graph status and configuration
//!
//! ## Usage
//!
//! ```rust
//! use vectorizer::api;
//!
//! // Access REST handlers through server module
//! use vectorizer::server::rest_handlers;
//!
//! // Access GraphQL schema
//! use vectorizer::api::graphql::create_schema;
//!
//! // Access Graph API
//! use vectorizer::api::graph::create_graph_router;
//! ```
//!
//! ## Documentation
//!
//! For complete API documentation, see:
//! - REST API: `docs/api/README.md`
//! - GraphQL: `docs/api/graphql_schema.md`
//! - Graph API: `docs/api/graph_api.md`
//! - Module docs: `docs/modules/api.md`

pub mod advanced_api;
pub mod cluster;
pub mod graph;
pub mod graphql;

pub use advanced_api::*;
pub use cluster::*;
pub use graph::*;
pub use graphql::*;
