//! MCP (Model Context Protocol) subsystem.
//!
//! - [`handlers`] — dispatches inbound `CallTool` requests to the right
//!   vector-store or cluster operation (handle_mcp_tool)
//! - [`tools`] — the catalog of MCP tools exposed to clients
//!   (get_mcp_tools)
//! - [`connection_manager`] / [`performance`] — carried over from the
//!   previous flat layout; kept behind `#[allow(dead_code)]` until the
//!   next consumer wires them back in
//!
//! The rmcp `ServerHandler` implementation (`VectorizerMcpService`)
//! lives next to its routing entry in [`crate::server::core::routing`].

#[allow(dead_code)]
pub mod connection_manager;
pub mod handlers;
#[allow(dead_code)]
pub mod performance;
pub mod tools;
