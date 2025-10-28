//! Enhanced file watcher implementation

use std::path::PathBuf;
use std::sync::Arc;

use super::debouncer::Debouncer;
use super::hash_validator::HashValidator;
use super::{
    CollectionVectorMapping, FileChangeEvent, FileChangeEventWithMetadata, FileIndex, FileIndexArc,
    FileWatcherConfig, FileWatcherError, Result,
};

/// Enhanced file system event types
#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemEvent {
    Created {
        path: PathBuf,
    },
    Modified {
        path: PathBuf,
    },
    Deleted {
        path: PathBuf,
    },
    Renamed {
        old_path: PathBuf,
        new_path: PathBuf,
    },
}

/// Enhanced file watcher implementation
pub struct EnhancedFileWatcher {
    config: FileWatcherConfig,
    debouncer: Arc<Debouncer>,
    hash_validator: Arc<HashValidator>,
}

impl EnhancedFileWatcher {
    pub fn new(
        config: FileWatcherConfig,
        debouncer: Arc<Debouncer>,
        hash_validator: Arc<HashValidator>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            debouncer,
            hash_validator,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        // Simple implementation - just return Ok for now
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        // Simple implementation - just return Ok for now
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        false // Simple implementation
    }

    pub fn get_config(&self) -> &FileWatcherConfig {
        &self.config
    }

    pub fn get_watched_paths(&self) -> Vec<String> {
        self.config
            .watch_paths
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }

    pub fn get_event_count(&self) -> u64 {
        0 // Simple implementation
    }

    pub fn get_last_event_time(&self) -> Option<std::time::SystemTime> {
        None // Simple implementation
    }

    pub fn clear_events(&mut self) {
        // Simple implementation - no-op
    }

    pub fn get_recent_events(&self, _limit: usize) -> Vec<FileChangeEventWithMetadata> {
        Vec::new() // Simple implementation
    }
}

/// Workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub name: String,
    pub path: PathBuf,
    pub collections: Vec<CollectionConfig>,
}

/// Project configuration
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    pub collections: Vec<CollectionConfig>,
}

/// Collection configuration
#[derive(Debug, Clone)]
pub struct CollectionConfig {
    pub name: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_event_created() {
        let event = FileSystemEvent::Created {
            path: PathBuf::from("/test/file.txt"),
        };

        match event {
            FileSystemEvent::Created { path } => {
                assert_eq!(path, PathBuf::from("/test/file.txt"));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_filesystem_event_modified() {
        let event = FileSystemEvent::Modified {
            path: PathBuf::from("/test/file.txt"),
        };

        match event {
            FileSystemEvent::Modified { path } => {
                assert_eq!(path, PathBuf::from("/test/file.txt"));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_filesystem_event_deleted() {
        let event = FileSystemEvent::Deleted {
            path: PathBuf::from("/test/file.txt"),
        };

        match event {
            FileSystemEvent::Deleted { path } => {
                assert_eq!(path, PathBuf::from("/test/file.txt"));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_filesystem_event_renamed() {
        let event = FileSystemEvent::Renamed {
            old_path: PathBuf::from("/test/old.txt"),
            new_path: PathBuf::from("/test/new.txt"),
        };

        match event {
            FileSystemEvent::Renamed { old_path, new_path } => {
                assert_eq!(old_path, PathBuf::from("/test/old.txt"));
                assert_eq!(new_path, PathBuf::from("/test/new.txt"));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_filesystem_event_equality() {
        let event1 = FileSystemEvent::Created {
            path: PathBuf::from("/test/file.txt"),
        };
        let event2 = FileSystemEvent::Created {
            path: PathBuf::from("/test/file.txt"),
        };
        let event3 = FileSystemEvent::Modified {
            path: PathBuf::from("/test/file.txt"),
        };

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    #[test]
    fn test_enhanced_file_watcher_creation() {
        let config = FileWatcherConfig::default();
        let debouncer = Arc::new(Debouncer::new(100));
        let hash_validator = Arc::new(HashValidator::new());

        let watcher = EnhancedFileWatcher::new(config, debouncer, hash_validator);
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_enhanced_file_watcher_start_stop() {
        let config = FileWatcherConfig::default();
        let debouncer = Arc::new(Debouncer::new(100));
        let hash_validator = Arc::new(HashValidator::new());

        let mut watcher = EnhancedFileWatcher::new(config, debouncer, hash_validator).unwrap();

        assert!(!watcher.is_running());
        assert!(watcher.start().await.is_ok());
        assert!(watcher.stop().is_ok());
    }

    #[test]
    fn test_enhanced_file_watcher_get_config() {
        let config = FileWatcherConfig::default();
        let debouncer = Arc::new(Debouncer::new(100));
        let hash_validator = Arc::new(HashValidator::new());

        let watcher = EnhancedFileWatcher::new(config.clone(), debouncer, hash_validator).unwrap();
        let retrieved_config = watcher.get_config();

        assert_eq!(retrieved_config.debounce_delay_ms, config.debounce_delay_ms);
    }

    #[test]
    fn test_enhanced_file_watcher_get_watched_paths() {
        let mut config = FileWatcherConfig::default();
        config.watch_paths = Some(vec![PathBuf::from("/path1"), PathBuf::from("/path2")]);

        let debouncer = Arc::new(Debouncer::new(100));
        let hash_validator = Arc::new(HashValidator::new());

        let watcher = EnhancedFileWatcher::new(config, debouncer, hash_validator).unwrap();
        let paths = watcher.get_watched_paths();

        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"/path1".to_string()));
        assert!(paths.contains(&"/path2".to_string()));
    }

    #[test]
    fn test_enhanced_file_watcher_event_methods() {
        let config = FileWatcherConfig::default();
        let debouncer = Arc::new(Debouncer::new(100));
        let hash_validator = Arc::new(HashValidator::new());

        let mut watcher = EnhancedFileWatcher::new(config, debouncer, hash_validator).unwrap();

        assert_eq!(watcher.get_event_count(), 0);
        assert!(watcher.get_last_event_time().is_none());

        watcher.clear_events();

        let events = watcher.get_recent_events(10);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_workspace_config_creation() {
        let collections = vec![CollectionConfig {
            name: "test_collection".to_string(),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec!["target/**".to_string()],
        }];

        let workspace = WorkspaceConfig {
            name: "test_workspace".to_string(),
            path: PathBuf::from("/workspace"),
            collections,
        };

        assert_eq!(workspace.name, "test_workspace");
        assert_eq!(workspace.collections.len(), 1);
    }

    #[test]
    fn test_project_config_creation() {
        let project = ProjectConfig {
            name: "test_project".to_string(),
            path: PathBuf::from("/project"),
            collections: Vec::new(),
        };

        assert_eq!(project.name, "test_project");
        assert_eq!(project.collections.len(), 0);
    }

    #[test]
    fn test_collection_config_creation() {
        let collection = CollectionConfig {
            name: "src_files".to_string(),
            include_patterns: vec!["**/*.rs".to_string(), "**/*.toml".to_string()],
            exclude_patterns: vec!["**/target/**".to_string()],
        };

        assert_eq!(collection.name, "src_files");
        assert_eq!(collection.include_patterns.len(), 2);
        assert_eq!(collection.exclude_patterns.len(), 1);
    }
}
