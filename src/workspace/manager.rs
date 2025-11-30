//! Workspace manager
//!
//! Provides high-level functionality for managing workspace configurations

use std::path::{Path, PathBuf};

use tracing::{debug, error, info, warn};

use crate::error::{Result, VectorizerError};
use crate::file_watcher::normalize_wsl_path;
use crate::workspace::config::*;
use crate::workspace::parser::*;
use crate::workspace::simplified_config::*;
use crate::workspace::validator::*;

/// Workspace manager
#[derive(Debug)]
pub struct WorkspaceManager {
    /// Workspace configuration
    config: WorkspaceConfig,

    /// Workspace root directory
    workspace_root: PathBuf,

    /// Configuration file path
    config_path: PathBuf,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new(config: WorkspaceConfig, workspace_root: PathBuf, config_path: PathBuf) -> Self {
        Self {
            config,
            workspace_root,
            config_path,
        }
    }

    /// Load workspace from configuration file
    pub fn load_from_file<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref();
        let workspace_root = config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        debug!("Loading workspace from: {}", config_path.display());

        // Try to parse as simplified config first, then fall back to full config
        let config = match Self::try_load_simplified_config(config_path) {
            Ok(simplified_config) => {
                info!("✅ Loaded simplified workspace configuration with intelligent defaults");
                simplified_config.to_full_workspace_config()
            }
            Err(_) => {
                debug!("Not a simplified config, trying full configuration format");
                parse_workspace_config(config_path)?
            }
        };

        debug!(
            "Parsed workspace config with {} projects from YAML",
            config.projects.len()
        );

        // Validate configuration
        let validation_result = validate_workspace_config(&config, &workspace_root);
        if !validation_result.is_valid() {
            error!("Workspace configuration validation failed:");
            for error in &validation_result.errors {
                error!("  - {}", error);
            }
            return Err(VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Workspace configuration validation failed",
            )));
        }

        // Log warnings
        for warning in &validation_result.warnings {
            warn!("Workspace configuration warning: {}", warning);
        }

        Ok(Self::new(config, workspace_root, config_path.to_path_buf()))
    }

    /// Try to load simplified workspace configuration
    fn try_load_simplified_config<P: AsRef<Path>>(
        config_path: P,
    ) -> std::result::Result<SimplifiedWorkspaceConfig, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(config_path)?;

        debug!("Trying to parse as simplified workspace configuration...");
        debug!("Content preview: {}", &content[..content.len().min(200)]);

        // Try to parse as simplified config - it will work if it has the simplified structure
        // (projects with collections that only have name, description, include_patterns, exclude_patterns)
        match crate::workspace::parser::parse_simplified_workspace_config_from_str(&content) {
            Ok(config) => {
                debug!("✅ Successfully parsed as simplified workspace configuration");
                Ok(config)
            }
            Err(e) => {
                debug!(
                    "❌ Failed to parse as simplified workspace configuration: {}",
                    e
                );
                Err("Not a simplified workspace configuration".into())
            }
        }
    }

    /// Find and load workspace configuration
    pub fn find_and_load<P: AsRef<Path>>(start_path: P) -> Result<Self> {
        let start_path = start_path.as_ref();

        info!(
            "Searching for workspace configuration starting from: {}",
            start_path.display()
        );

        // Find configuration file
        let config_path = find_workspace_config(start_path)?.ok_or_else(|| {
            VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No workspace configuration found",
            ))
        })?;

        // Load workspace
        Self::load_from_file(config_path)
    }

    /// Create a new workspace with default configuration
    pub fn create_default<P: AsRef<Path>>(workspace_root: P) -> Result<Self> {
        let workspace_root = workspace_root.as_ref().to_path_buf();
        let config_path = workspace_root.join("workspace.yml");

        info!(
            "Creating default workspace at: {}",
            workspace_root.display()
        );

        // Create default configuration
        let mut config = create_default_workspace_config();

        // Update workspace metadata
        config.workspace.name = workspace_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Default Workspace")
            .to_string();

        // Save configuration
        save_workspace_config(&config, &config_path)?;

        Ok(Self::new(config, workspace_root, config_path))
    }

    /// Get workspace configuration
    pub fn config(&self) -> &WorkspaceConfig {
        &self.config
    }

    /// Get workspace root directory
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Get configuration file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Get enabled projects
    pub fn enabled_projects(&self) -> Vec<&ProjectConfig> {
        let total_projects = self.config.projects.len();
        let enabled = self
            .config
            .projects
            .iter()
            .filter(|p| p.enabled)
            .collect::<Vec<_>>();
        debug!(
            "Workspace has {} total projects, {} enabled",
            total_projects,
            enabled.len()
        );
        enabled
    }

    /// Get project by name
    pub fn get_project(&self, name: &str) -> Option<&ProjectConfig> {
        self.config.projects.iter().find(|p| p.name == name)
    }

    /// Get project path
    pub fn get_project_path(&self, project_name: &str) -> Result<PathBuf> {
        let project = self.get_project(project_name).ok_or_else(|| {
            VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Project '{}' not found", project_name),
            ))
        })?;

        // Normalize WSL paths before joining
        let project_path_str = project.path.to_string_lossy();
        let normalized_project_path = normalize_wsl_path(&project_path_str);

        let workspace_root_str = self.workspace_root.to_string_lossy();
        let normalized_workspace_root = normalize_wsl_path(&workspace_root_str);

        Ok(if normalized_project_path.is_absolute() {
            normalized_project_path
        } else {
            normalized_workspace_root.join(&normalized_project_path)
        })
    }

    /// Get collection configuration
    pub fn get_collection_config(
        &self,
        project_name: &str,
        collection_name: &str,
    ) -> Result<&CollectionConfig> {
        let project = self.get_project(project_name).ok_or_else(|| {
            VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Project '{}' not found", project_name),
            ))
        })?;

        project
            .collections
            .iter()
            .find(|c| c.name == collection_name)
            .ok_or_else(|| {
                VectorizerError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Collection '{}' not found in project '{}'",
                        collection_name, project_name
                    ),
                ))
            })
    }

    /// Add a new project
    pub fn add_project(&mut self, project: ProjectConfig) -> Result<()> {
        debug!("Adding project: {}", project.name);

        // Check for duplicate names
        if self.get_project(&project.name).is_some() {
            return Err(VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Project '{}' already exists", project.name),
            )));
        }

        // Validate project path (normalize WSL paths first)
        let project_path_str = project.path.to_string_lossy();
        let normalized_project_path = normalize_wsl_path(&project_path_str);

        let workspace_root_str = self.workspace_root.to_string_lossy();
        let normalized_workspace_root = normalize_wsl_path(&workspace_root_str);

        let project_path = if normalized_project_path.is_absolute() {
            normalized_project_path
        } else {
            normalized_workspace_root.join(&normalized_project_path)
        };

        if !project_path.exists() {
            return Err(VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Project path does not exist: {}", project_path.display()),
            )));
        }

        // Add project
        self.config.projects.push(project);

        // Update last_updated timestamp
        self.config.workspace.last_updated = chrono::Utc::now().to_rfc3339();

        info!(
            "Added project: {}",
            self.config.projects.last().unwrap().name
        );
        Ok(())
    }

    /// Remove a project
    pub fn remove_project(&mut self, project_name: &str) -> Result<()> {
        debug!("Removing project: {}", project_name);

        let initial_len = self.config.projects.len();
        self.config.projects.retain(|p| p.name != project_name);

        if self.config.projects.len() == initial_len {
            return Err(VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Project '{}' not found", project_name),
            )));
        }

        // Update last_updated timestamp
        self.config.workspace.last_updated = chrono::Utc::now().to_rfc3339();

        info!("Removed project: {}", project_name);
        Ok(())
    }

    /// Enable/disable a project
    pub fn set_project_enabled(&mut self, project_name: &str, enabled: bool) -> Result<()> {
        debug!("Setting project '{}' enabled: {}", project_name, enabled);

        let project = self
            .config
            .projects
            .iter_mut()
            .find(|p| p.name == project_name)
            .ok_or_else(|| {
                VectorizerError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Project '{}' not found", project_name),
                ))
            })?;

        project.enabled = enabled;

        // Update last_updated timestamp
        self.config.workspace.last_updated = chrono::Utc::now().to_rfc3339();

        info!("Set project '{}' enabled: {}", project_name, enabled);
        Ok(())
    }

    /// Add a collection to a project
    pub fn add_collection(
        &mut self,
        project_name: &str,
        collection: CollectionConfig,
    ) -> Result<()> {
        debug!(
            "Adding collection '{}' to project '{}'",
            collection.name, project_name
        );

        let project = self
            .config
            .projects
            .iter_mut()
            .find(|p| p.name == project_name)
            .ok_or_else(|| {
                VectorizerError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Project '{}' not found", project_name),
                ))
            })?;

        // Check for duplicate collection names
        if project
            .collections
            .iter()
            .any(|c| c.name == collection.name)
        {
            return Err(VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "Collection '{}' already exists in project '{}'",
                    collection.name, project_name
                ),
            )));
        }

        // Add collection
        project.collections.push(collection);

        // Update last_updated timestamp
        self.config.workspace.last_updated = chrono::Utc::now().to_rfc3339();

        info!(
            "Added collection '{}' to project '{}'",
            project.collections.last().unwrap().name,
            project_name
        );
        Ok(())
    }

    /// Remove a collection from a project
    pub fn remove_collection(&mut self, project_name: &str, collection_name: &str) -> Result<()> {
        debug!(
            "Removing collection '{}' from project '{}'",
            collection_name, project_name
        );

        let project = self
            .config
            .projects
            .iter_mut()
            .find(|p| p.name == project_name)
            .ok_or_else(|| {
                VectorizerError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Project '{}' not found", project_name),
                ))
            })?;

        let initial_len = project.collections.len();
        project.collections.retain(|c| c.name != collection_name);

        if project.collections.len() == initial_len {
            return Err(VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Collection '{}' not found in project '{}'",
                    collection_name, project_name
                ),
            )));
        }

        // Update last_updated timestamp
        self.config.workspace.last_updated = chrono::Utc::now().to_rfc3339();

        info!(
            "Removed collection '{}' from project '{}'",
            collection_name, project_name
        );
        Ok(())
    }

    /// Save workspace configuration
    pub fn save(&self) -> Result<()> {
        debug!(
            "Saving workspace configuration to: {}",
            self.config_path.display()
        );

        // Validate before saving
        let validation_result = validate_workspace_config(&self.config, &self.workspace_root);
        if !validation_result.is_valid() {
            error!("Cannot save invalid workspace configuration:");
            for error in &validation_result.errors {
                error!("  - {}", error);
            }
            return Err(VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Cannot save invalid workspace configuration",
            )));
        }

        // Save configuration
        save_workspace_config(&self.config, &self.config_path)?;

        info!("Workspace configuration saved successfully");
        Ok(())
    }

    /// Reload workspace configuration from file
    pub fn reload(&mut self) -> Result<()> {
        info!(
            "Reloading workspace configuration from: {}",
            self.config_path.display()
        );

        // Parse configuration
        let config = parse_workspace_config(&self.config_path)?;

        // Validate configuration
        let validation_result = validate_workspace_config(&config, &self.workspace_root);
        if !validation_result.is_valid() {
            error!("Reloaded workspace configuration validation failed:");
            for error in &validation_result.errors {
                error!("  - {}", error);
            }
            return Err(VectorizerError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Reloaded workspace configuration validation failed",
            )));
        }

        // Update configuration
        self.config = config;

        info!("Workspace configuration reloaded successfully");
        Ok(())
    }

    /// Get workspace status
    pub fn get_status(&self) -> WorkspaceStatus {
        let enabled_projects = self.enabled_projects();
        let total_collections: usize = enabled_projects.iter().map(|p| p.collections.len()).sum();

        WorkspaceStatus {
            workspace_name: self.config.workspace.name.clone(),
            workspace_version: self.config.workspace.version.clone(),
            total_projects: self.config.projects.len(),
            enabled_projects: enabled_projects.len(),
            total_collections,
            last_updated: self.config.workspace.last_updated.clone(),
        }
    }

    /// Validate workspace
    pub fn validate(&self) -> ValidationResult {
        validate_workspace_config(&self.config, &self.workspace_root)
    }
}

/// Workspace status information
#[derive(Debug, Clone)]
pub struct WorkspaceStatus {
    /// Workspace name
    pub workspace_name: String,

    /// Workspace version
    pub workspace_version: String,

    /// Total number of projects
    pub total_projects: usize,

    /// Number of enabled projects
    pub enabled_projects: usize,

    /// Total number of collections across all enabled projects
    pub total_collections: usize,

    /// Last update timestamp
    pub last_updated: String,
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_workspace_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let config = WorkspaceConfig::default();
        let config_path = temp_dir.path().join("workspace.yml");

        let manager = WorkspaceManager::new(config, temp_dir.path().to_path_buf(), config_path);

        assert_eq!(manager.workspace_root(), temp_dir.path());
    }

    #[test]
    fn test_create_default_workspace() {
        let temp_dir = tempdir().unwrap();

        let manager = WorkspaceManager::create_default(temp_dir.path()).unwrap();

        assert_eq!(manager.workspace_root(), temp_dir.path());
        assert!(manager.config_path().exists());
    }

    #[test]
    fn test_add_and_remove_project() {
        let temp_dir = tempdir().unwrap();
        let mut manager = WorkspaceManager::create_default(temp_dir.path()).unwrap();

        // Create a test project directory
        let project_dir = temp_dir.path().join("test-project");
        std::fs::create_dir(&project_dir).unwrap();

        let project = ProjectConfig {
            name: "test-project".to_string(),
            path: PathBuf::from("test-project"),
            description: "Test project".to_string(),
            enabled: true,
            embedding: None,
            collections: vec![],
        };

        // Add project
        manager.add_project(project).unwrap();
        assert!(manager.get_project("test-project").is_some());

        // Remove project
        manager.remove_project("test-project").unwrap();
        assert!(manager.get_project("test-project").is_none());
    }

    #[test]
    fn test_workspace_status() {
        let temp_dir = tempdir().unwrap();
        let manager = WorkspaceManager::create_default(temp_dir.path()).unwrap();

        let status = manager.get_status();
        assert_eq!(status.total_projects, 0);
        assert_eq!(status.enabled_projects, 0);
        assert_eq!(status.total_collections, 0);
    }
}
