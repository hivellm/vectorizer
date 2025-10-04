//! Workspace configuration and management
//!
//! This module provides functionality for managing multiple projects
//! through a centralized workspace configuration file.

pub mod config;
pub mod manager;
pub mod parser;
pub mod validator;
pub mod simplified_config;

pub use config::*;
pub use manager::*;
pub use parser::*;
pub use validator::*;
pub use simplified_config::*;
