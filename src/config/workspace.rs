//! Workspace management for file watching and indexing
//!
//! A workspace is a directory that maps to a specific collection for file indexing.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

/// Workspace configuration file path
const WORKSPACE_CONFIG_FILE: &str = "./workspace.yml";

/// Represents a single workspace directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique workspace ID
    pub id: String,
    /// File system path to the workspace
    pub path: String,
    /// Collection name to index files into
    pub collection_name: String,
    /// Whether the workspace is currently active
    #[serde(default = "default_active")]
    pub active: bool,
    /// Include patterns (glob)
    #[serde(default)]
    pub include_patterns: Vec<String>,
    /// Exclude patterns (glob)
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    /// When the workspace was created
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    /// When the workspace was last modified
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
    /// When the workspace was last indexed
    pub last_indexed: Option<DateTime<Utc>>,
    /// Number of files indexed
    #[serde(default)]
    pub file_count: usize,
}

fn default_active() -> bool {
    true
}

impl Workspace {
    /// Create a new workspace
    pub fn new(path: &str, collection_name: &str) -> Self {
        let id = format!(
            "ws-{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        Self {
            id,
            path: path.to_string(),
            collection_name: collection_name.to_string(),
            active: true,
            include_patterns: vec![
                "*.md".to_string(),
                "*.txt".to_string(),
                "*.rs".to_string(),
                "*.py".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
            ],
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_indexed: None,
            file_count: 0,
        }
    }

    /// Check if path exists
    pub fn exists(&self) -> bool {
        Path::new(&self.path).exists()
    }

    /// Get absolute path
    pub fn absolute_path(&self) -> PathBuf {
        let path = Path::new(&self.path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(path)
        }
    }
}

/// Workspace configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceConfig {
    /// List of workspaces
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
}

/// Workspace manager for managing directory-to-collection mappings
#[derive(Debug, Clone)]
pub struct WorkspaceManager {
    /// Workspaces indexed by ID
    workspaces: Arc<RwLock<HashMap<String, Workspace>>>,
    /// Path to workspaces indexed by path
    path_index: Arc<RwLock<HashMap<String, String>>>,
    /// Config file path
    config_path: PathBuf,
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new() -> Self {
        let manager = Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            path_index: Arc::new(RwLock::new(HashMap::new())),
            config_path: PathBuf::from(WORKSPACE_CONFIG_FILE),
        };

        // Load existing workspaces from config
        if let Err(e) = manager.load_from_file() {
            warn!("Could not load workspace config: {}", e);
        }

        manager
    }

    /// Create workspace manager with custom config path
    pub fn with_config_path(path: PathBuf) -> Self {
        let manager = Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            path_index: Arc::new(RwLock::new(HashMap::new())),
            config_path: path,
        };

        if let Err(e) = manager.load_from_file() {
            warn!("Could not load workspace config: {}", e);
        }

        manager
    }

    /// Load workspaces from config file
    fn load_from_file(&self) -> Result<(), String> {
        if !self.config_path.exists() {
            info!("Workspace config file not found, starting with empty config");
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read workspace config: {}", e))?;

        let config: WorkspaceConfig = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse workspace config: {}", e))?;

        let mut workspaces = self.workspaces.write();
        let mut path_index = self.path_index.write();

        for workspace in config.workspaces {
            path_index.insert(workspace.path.clone(), workspace.id.clone());
            workspaces.insert(workspace.id.clone(), workspace);
        }

        info!("Loaded {} workspaces from config", workspaces.len());
        Ok(())
    }

    /// Save workspaces to config file
    fn save_to_file(&self) -> Result<(), String> {
        let workspaces = self.workspaces.read();
        let config = WorkspaceConfig {
            workspaces: workspaces.values().cloned().collect(),
        };

        let yaml = serde_yaml::to_string(&config)
            .map_err(|e| format!("Failed to serialize workspace config: {}", e))?;

        fs::write(&self.config_path, yaml)
            .map_err(|e| format!("Failed to write workspace config: {}", e))?;

        info!("Saved {} workspaces to config", workspaces.len());
        Ok(())
    }

    /// Add a new workspace
    pub fn add_workspace(&self, path: &str, collection_name: &str) -> Result<Workspace, String> {
        // Normalize path
        let normalized_path = self.normalize_path(path)?;

        // Check if workspace already exists for this path
        {
            let path_index = self.path_index.read();
            if path_index.contains_key(&normalized_path) {
                return Err(format!("Workspace already exists for path: {}", path));
            }
        }

        // Create new workspace
        let workspace = Workspace::new(&normalized_path, collection_name);

        // Add to indexes
        {
            let mut workspaces = self.workspaces.write();
            let mut path_index = self.path_index.write();

            path_index.insert(normalized_path.clone(), workspace.id.clone());
            workspaces.insert(workspace.id.clone(), workspace.clone());
        }

        // Persist to file
        if let Err(e) = self.save_to_file() {
            error!("Failed to save workspace config: {}", e);
        }

        info!(
            "Added workspace: {} -> {}",
            workspace.path, workspace.collection_name
        );

        Ok(workspace)
    }

    /// Remove a workspace by path
    pub fn remove_workspace(&self, path: &str) -> Result<Workspace, String> {
        let normalized_path = self.normalize_path(path)?;

        let workspace = {
            let mut workspaces = self.workspaces.write();
            let mut path_index = self.path_index.write();

            let workspace_id = path_index
                .remove(&normalized_path)
                .ok_or_else(|| format!("Workspace not found for path: {}", path))?;

            workspaces
                .remove(&workspace_id)
                .ok_or_else(|| "Workspace not found".to_string())?
        };

        // Persist to file
        if let Err(e) = self.save_to_file() {
            error!("Failed to save workspace config: {}", e);
        }

        info!("Removed workspace: {}", workspace.path);

        Ok(workspace)
    }

    /// Remove a workspace by ID
    pub fn remove_workspace_by_id(&self, workspace_id: &str) -> Result<Workspace, String> {
        let workspace = {
            let mut workspaces = self.workspaces.write();
            let mut path_index = self.path_index.write();

            let workspace = workspaces
                .remove(workspace_id)
                .ok_or_else(|| format!("Workspace not found: {}", workspace_id))?;

            path_index.remove(&workspace.path);

            workspace
        };

        if let Err(e) = self.save_to_file() {
            error!("Failed to save workspace config: {}", e);
        }

        info!("Removed workspace: {}", workspace.path);

        Ok(workspace)
    }

    /// Get a workspace by path
    pub fn get_workspace(&self, path: &str) -> Option<Workspace> {
        let normalized_path = self.normalize_path(path).ok()?;
        let path_index = self.path_index.read();
        let workspace_id = path_index.get(&normalized_path)?;
        let workspaces = self.workspaces.read();
        workspaces.get(workspace_id).cloned()
    }

    /// Get a workspace by ID
    pub fn get_workspace_by_id(&self, workspace_id: &str) -> Option<Workspace> {
        let workspaces = self.workspaces.read();
        workspaces.get(workspace_id).cloned()
    }

    /// List all workspaces
    pub fn list_workspaces(&self) -> Vec<Workspace> {
        let workspaces = self.workspaces.read();
        workspaces.values().cloned().collect()
    }

    /// List active workspaces only
    pub fn list_active_workspaces(&self) -> Vec<Workspace> {
        let workspaces = self.workspaces.read();
        workspaces.values().filter(|w| w.active).cloned().collect()
    }

    /// Update workspace's last indexed time and file count
    pub fn update_index_stats(&self, workspace_id: &str, file_count: usize) -> Result<(), String> {
        let mut workspaces = self.workspaces.write();
        let workspace = workspaces
            .get_mut(workspace_id)
            .ok_or_else(|| format!("Workspace not found: {}", workspace_id))?;

        workspace.last_indexed = Some(Utc::now());
        workspace.file_count = file_count;
        workspace.updated_at = Utc::now();

        drop(workspaces);

        self.save_to_file()
    }

    /// Set workspace active status
    pub fn set_active(&self, workspace_id: &str, active: bool) -> Result<(), String> {
        let mut workspaces = self.workspaces.write();
        let workspace = workspaces
            .get_mut(workspace_id)
            .ok_or_else(|| format!("Workspace not found: {}", workspace_id))?;

        workspace.active = active;
        workspace.updated_at = Utc::now();

        drop(workspaces);

        self.save_to_file()
    }

    /// Get collection name for a file path
    pub fn get_collection_for_path(&self, file_path: &str) -> Option<String> {
        let workspaces = self.workspaces.read();

        // Find workspace that contains this file path
        for workspace in workspaces.values() {
            if file_path.starts_with(&workspace.path) {
                return Some(workspace.collection_name.clone());
            }
        }

        None
    }

    /// Normalize a path to absolute form
    fn normalize_path(&self, path: &str) -> Result<String, String> {
        let path = Path::new(path);
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?
                .join(path)
        };

        // Canonicalize if the path exists
        if absolute.exists() {
            absolute
                .canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .map_err(|e| format!("Failed to canonicalize path: {}", e))
        } else {
            Ok(absolute.to_string_lossy().to_string())
        }
    }

    /// Get watch paths for file watcher
    pub fn get_watch_paths(&self) -> Vec<PathBuf> {
        let workspaces = self.workspaces.read();
        workspaces
            .values()
            .filter(|w| w.active && w.exists())
            .map(|w| w.absolute_path())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_workspace_creation() {
        let workspace = Workspace::new("/test/path", "test_collection");
        assert!(workspace.id.starts_with("ws-"));
        assert_eq!(workspace.path, "/test/path");
        assert_eq!(workspace.collection_name, "test_collection");
        assert!(workspace.active);
    }

    #[test]
    fn test_workspace_manager() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("workspace.yml");

        let manager = WorkspaceManager::with_config_path(config_path.clone());

        // Add workspace
        let workspace = manager
            .add_workspace(temp_dir.path().to_str().unwrap(), "test_collection")
            .unwrap();

        assert!(workspace.id.starts_with("ws-"));

        // List workspaces
        let workspaces = manager.list_workspaces();
        assert_eq!(workspaces.len(), 1);

        // Remove workspace
        let removed = manager
            .remove_workspace(temp_dir.path().to_str().unwrap())
            .unwrap();
        assert_eq!(removed.id, workspace.id);

        // List should be empty
        let workspaces = manager.list_workspaces();
        assert!(workspaces.is_empty());
    }
}
