// File-level operations module for MCP
// Provides file-centric abstractions over chunk-based vector storage

pub mod types;
pub mod errors;
pub mod cache;
pub mod operations;
pub mod mcp_integration;

pub use types::*;
pub use errors::*;
pub use operations::FileOperations;
pub use mcp_integration::FileMcpHandlers;

#[cfg(test)]
mod tests;

