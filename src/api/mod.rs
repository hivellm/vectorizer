//! REST API module for Vectorizer
//!
//! This module provides HTTP endpoints for interacting with the vector database.

pub mod handlers;
pub mod routes;
pub mod server;
pub mod types;

#[cfg(test)]
mod tests;

pub use server::VectorizerServer;
