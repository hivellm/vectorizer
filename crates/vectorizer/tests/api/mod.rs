//! API Tests
//!
//! Tests for different API interfaces:
//! - REST API
//! - GraphQL API
//! - gRPC API (see ../grpc/)
//! - MCP API

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod graphql;
pub mod mcp;
pub mod parity;
pub mod rest;

// gRPC tests are in tests/grpc/ directory
