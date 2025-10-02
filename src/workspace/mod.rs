//! Workspace configuration and management
//!
//! This module provides functionality for managing multiple projects
//! through a centralized workspace configuration file.

pub mod config;
pub mod manager;
pub mod parser;
pub mod validator;

pub use config::*;
pub use manager::*;
pub use parser::*;
pub use validator::*;
