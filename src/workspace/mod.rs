//! Workspace configuration and management
//!
//! This module provides functionality for managing multiple projects
//! through a centralized workspace configuration file.

pub mod config;
pub mod manager;
pub mod parser;
pub mod project_analyzer;
pub mod simplified_config;
pub mod templates;
pub mod validator;
pub mod setup_config;

pub use config::*;
pub use manager::*;
pub use parser::*;
pub use project_analyzer::*;
pub use simplified_config::*;
pub use templates::*;
pub use validator::*;
pub use setup_config::*;

