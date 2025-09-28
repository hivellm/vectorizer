//! Cache manager implementation

use super::*;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::info;

/// Cache manager for handling cache operations
pub struct CacheManager {
    /// Cache metadata
    metadata: Arc<AsyncRwLock<CacheMetadata>>,

    /// Cache configuration
    config: CacheConfig,

    /// Cache directory path
    cache_path: std::path::PathBuf,

    /// Metadata file path
    metadata_path: std::path::PathBuf,

    /// Background tasks handles
    background_tasks: Arc<AsyncMutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl CacheManager {
    /// Create new cache manager
    pub async fn new(config: CacheConfig) -> CacheResult<Self> {
        let cache_path = config.cache_path.clone();
        let metadata_path = cache_path.join("metadata.json");

        // Ensure cache directory exists
        fs::create_dir_all(&cache_path).await?;

        // Load or create metadata
        let metadata = if metadata_path.exists() {
            Self::load_metadata(&metadata_path).await?
        } else {
            CacheMetadata::new("1.0.0".to_string())
        };

        Ok(Self {
            metadata: Arc::new(AsyncRwLock::new(metadata)),
            config,
            cache_path,
            metadata_path,
            background_tasks: Arc::new(AsyncMutex::new(Vec::new())),
        })
    }

    /// Load metadata from file
    async fn load_metadata(path: &Path) -> CacheResult<CacheMetadata> {
        let content = fs::read_to_string(path).await?;
        let metadata: CacheMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    /// Save metadata to file
    async fn save_metadata(&self) -> CacheResult<()> {
        let metadata = self.metadata.read().await;
        let content = serde_json::to_string_pretty(&*metadata)?;
        fs::write(&self.metadata_path, content).await?;
        Ok(())
    }

    /// Get cache metadata
    pub async fn get_metadata(&self) -> CacheMetadata {
        self.metadata.read().await.clone()
    }

    /// Update cache metadata
    pub async fn update_metadata<F>(&self, updater: F) -> CacheResult<()>
    where
        F: FnOnce(&mut CacheMetadata),
    {
        {
            let mut metadata = self.metadata.write().await;
            updater(&mut *metadata);
        }
        self.save_metadata().await?;
        Ok(())
    }

    /// Get collection cache info
    pub async fn get_collection_info(&self, name: &str) -> Option<CollectionCacheInfo> {
        let metadata = self.metadata.read().await;
        metadata.get_collection(name).cloned()
    }

    /// Update collection cache info
    pub async fn update_collection_info(&self, info: CollectionCacheInfo) -> CacheResult<()> {
        self.update_metadata(|metadata| {
            metadata.update_collection(info);
        })
        .await?;
        Ok(())
    }

    /// Remove collection cache info
    pub async fn remove_collection_info(
        &self,
        name: &str,
    ) -> CacheResult<Option<CollectionCacheInfo>> {
        let result = {
            let mut metadata = self.metadata.write().await;
            metadata.remove_collection(name)
        };

        if result.is_some() {
            self.save_metadata().await?;
        }

        Ok(result)
    }

    /// Check if collection exists in cache
    pub async fn has_collection(&self, name: &str) -> bool {
        info!("🔍 CacheManager::has_collection called for '{}'", name);
        let metadata = self.metadata.read().await;
        let result = metadata.has_collection(name);
        info!(
            "🔍 CacheManager::has_collection result for '{}': {}",
            name, result
        );
        result
    }

    /// Get all collection names
    pub async fn get_collection_names(&self) -> Vec<String> {
        let metadata = self.metadata.read().await;
        metadata.collection_names()
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let metadata = self.metadata.read().await;
        metadata.stats.clone()
    }

    /// Update cache statistics
    pub async fn update_stats(&self, stats: CacheStats) -> CacheResult<()> {
        self.update_metadata(|metadata| {
            metadata.update_stats(stats);
        })
        .await?;
        Ok(())
    }

    /// Record cache hit
    pub async fn record_hit(&self) -> CacheResult<()> {
        self.update_metadata(|metadata| {
            metadata.stats.hits += 1;
            metadata.update_access();

            // Recalculate hit rate
            let total = metadata.stats.hits + metadata.stats.misses;
            if total > 0 {
                metadata.stats.hit_rate = metadata.stats.hits as f32 / total as f32;
            }
        })
        .await?;
        Ok(())
    }

    /// Record cache miss
    pub async fn record_miss(&self) -> CacheResult<()> {
        self.update_metadata(|metadata| {
            metadata.stats.misses += 1;
            metadata.update_access();

            // Recalculate hit rate
            let total = metadata.stats.hits + metadata.stats.misses;
            if total > 0 {
                metadata.stats.hit_rate = metadata.stats.hits as f32 / total as f32;
            }
        })
        .await?;
        Ok(())
    }

    /// Get cache configuration
    pub fn get_config(&self) -> &CacheConfig {
        &self.config
    }

    /// Update cache configuration
    pub async fn update_config(&mut self, config: CacheConfig) -> CacheResult<()> {
        self.config = config.clone();

        // Update cache path if changed
        if self.config.cache_path != self.cache_path {
            self.cache_path = self.config.cache_path.clone();
            self.metadata_path = self.cache_path.join("metadata.json");

            // Ensure new cache directory exists
            fs::create_dir_all(&self.cache_path).await?;
        }

        Ok(())
    }

    /// Get cache directory path
    pub fn get_cache_path(&self) -> &Path {
        &self.cache_path
    }

    /// Get metadata file path
    pub fn get_metadata_path(&self) -> &Path {
        &self.metadata_path
    }

    /// Calculate total cache size
    pub async fn calculate_total_size(&self) -> u64 {
        let metadata = self.metadata.read().await;
        metadata.calculate_total_size()
    }

    /// Check if cache is stale
    pub async fn is_stale(&self) -> bool {
        let metadata = self.metadata.read().await;
        metadata.is_stale(self.config.ttl_seconds)
    }

    /// Clean up old cache entries
    pub async fn cleanup(&self) -> CacheResult<CleanupResult> {
        if !self.config.cleanup.enabled {
            return Ok(CleanupResult::new());
        }

        let mut result = CleanupResult::new();
        let now = Utc::now();
        let max_age = Duration::from_secs(self.config.cleanup.max_age_seconds);

        self.update_metadata(|metadata| {
            let mut collections_to_remove = Vec::new();

            for (name, collection_info) in &metadata.collections {
                if collection_info.is_stale(self.config.cleanup.max_age_seconds) {
                    collections_to_remove.push(name.clone());
                    result.removed_collections += 1;
                }
            }

            for name in collections_to_remove {
                if let Some(removed) = metadata.remove_collection(&name) {
                    result.removed_size_bytes += removed.calculate_size();
                }
            }

            metadata.stats.last_cleanup = Some(now);
        })
        .await?;

        self.save_metadata().await?;
        Ok(result)
    }

    /// Validate cache integrity
    pub async fn validate(&self) -> CacheResult<ValidationResult> {
        let mut result = ValidationResult::new();

        match self.config.validation_level {
            ValidationLevel::None => {
                result.status = ValidationStatus::Skipped;
            }
            ValidationLevel::Basic => {
                result = self.validate_basic().await?;
            }
            ValidationLevel::Full => {
                result = self.validate_full().await?;
            }
        }

        Ok(result)
    }

    /// Basic cache validation
    async fn validate_basic(&self) -> CacheResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Check if metadata file exists and is readable
        if !self.metadata_path.exists() {
            result.status = ValidationStatus::Invalid;
            result
                .errors
                .push("Metadata file does not exist".to_string());
            return Ok(result);
        }

        // Check if cache directory exists
        if !self.cache_path.exists() {
            result.status = ValidationStatus::Invalid;
            result
                .errors
                .push("Cache directory does not exist".to_string());
            return Ok(result);
        }

        // Validate metadata structure
        let metadata = self.metadata.read().await;
        for (name, collection_info) in &metadata.collections {
            if collection_info.name != *name {
                result.errors.push(format!(
                    "Collection name mismatch: {} != {}",
                    name, collection_info.name
                ));
            }
        }

        if result.errors.is_empty() {
            result.status = ValidationStatus::Valid;
        } else {
            result.status = ValidationStatus::Invalid;
        }

        Ok(result)
    }

    /// Full cache validation
    async fn validate_full(&self) -> CacheResult<ValidationResult> {
        let mut result = self.validate_basic().await?;

        if result.status == ValidationStatus::Invalid {
            return Ok(result);
        }

        // Validate file hashes and existence
        let metadata = self.metadata.read().await;
        for (collection_name, collection_info) in &metadata.collections {
            for (file_path, file_info) in &collection_info.file_hashes {
                if !file_path.exists() {
                    result.errors.push(format!(
                        "File {} in collection {} does not exist",
                        file_path.display(),
                        collection_name
                    ));
                } else {
                    // Check file size
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        if metadata.len() != file_info.size {
                            result.errors.push(format!(
                                "File {} size mismatch: expected {}, found {}",
                                file_path.display(),
                                file_info.size,
                                metadata.len()
                            ));
                        }
                    }
                }
            }
        }

        if !result.errors.is_empty() {
            result.status = ValidationStatus::Invalid;
        }

        Ok(result)
    }

    /// Start background cleanup task
    pub async fn start_background_cleanup(&self) -> CacheResult<()> {
        if !self.config.cleanup.enabled {
            return Ok(());
        }

        let cleanup_interval = Duration::from_secs(self.config.cleanup.interval_seconds);
        let manager = Arc::new(self.clone());

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;

                if let Err(e) = manager.cleanup().await {
                    eprintln!("Cache cleanup error: {}", e);
                }
            }
        });

        let mut tasks = self.background_tasks.lock().await;
        tasks.push(task);

        Ok(())
    }

    /// Stop all background tasks
    pub async fn stop_background_tasks(&self) {
        let mut tasks = self.background_tasks.lock().await;
        for task in tasks.drain(..) {
            task.abort();
        }
    }
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        Self {
            metadata: Arc::clone(&self.metadata),
            config: self.config.clone(),
            cache_path: self.cache_path.clone(),
            metadata_path: self.metadata_path.clone(),
            background_tasks: Arc::clone(&self.background_tasks),
        }
    }
}

/// Cache cleanup result
#[derive(Debug, Clone)]
pub struct CleanupResult {
    /// Number of removed collections
    pub removed_collections: usize,

    /// Total size of removed data in bytes
    pub removed_size_bytes: u64,

    /// Number of removed files
    pub removed_files: usize,
}

impl CleanupResult {
    pub fn new() -> Self {
        Self {
            removed_collections: 0,
            removed_size_bytes: 0,
            removed_files: 0,
        }
    }
}

/// Cache validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Validation status
    pub status: ValidationStatus,

    /// Validation errors
    pub errors: Vec<String>,

    /// Validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            status: ValidationStatus::Unknown,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        matches!(
            self.status,
            ValidationStatus::Valid | ValidationStatus::ValidWithWarnings
        )
    }

    /// Get summary message
    pub fn summary(&self) -> String {
        match self.status {
            ValidationStatus::Valid => "Cache validation passed".to_string(),
            ValidationStatus::ValidWithWarnings => {
                format!(
                    "Cache validation passed with {} warnings",
                    self.warnings.len()
                )
            }
            ValidationStatus::Invalid => {
                format!("Cache validation failed with {} errors", self.errors.len())
            }
            ValidationStatus::Skipped => "Cache validation skipped".to_string(),
            ValidationStatus::Unknown => "Cache validation status unknown".to_string(),
        }
    }
}

/// Cache validation status
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    /// Validation skipped
    Skipped,
    /// Cache is valid
    Valid,
    /// Cache is valid with warnings
    ValidWithWarnings,
    /// Cache is invalid
    Invalid,
    /// Validation status unknown
    Unknown,
}
