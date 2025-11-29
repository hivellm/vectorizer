// File-level operations module for MCP
// Provides file-centric abstractions over chunk-based vector storage

pub mod cache;
pub mod errors;
pub mod mcp_integration;
pub mod operations;
pub mod types;

pub use errors::*;
pub use mcp_integration::FileMcpHandlers;
pub use operations::FileOperations;
pub use types::*;

#[cfg(test)]
mod tests;
