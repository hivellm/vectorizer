//! Workspace configuration and management
//!
//! This module provides functionality for managing multiple projects
//! through a centralized workspace configuration file.

pub mod config;
pub mod parser;
pub mod validator;
pub mod manager;

pub use config::*;
pub use parser::*;
pub use validator::*;
pub use manager::*;
