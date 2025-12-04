//! User-scoped backup functionality for HiveHub cluster mode
//!
//! Provides backup and restore operations isolated per user/tenant,
//! enabling HiveHub to manage backups for individual users.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::db::VectorStore;
use crate::error::{Result, VectorizerError};
use crate::models::Vector;

/// Backup metadata for a user's backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBackupInfo {
    /// Unique backup ID
    pub id: Uuid,
    /// User/tenant ID who owns this backup
    pub user_id: Uuid,
    /// Human-readable backup name
    pub name: String,
    /// Backup description (optional)
    #[serde(default)]
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Collections included in this backup
    pub collections: Vec<String>,
    /// Total number of vectors
    pub vector_count: u64,
    /// Backup size in bytes (compressed)
    pub size_bytes: u64,
    /// Backup format version
    pub format_version: u32,
    /// Checksum of the backup data (SHA-256)
    #[serde(default)]
    pub checksum: Option<String>,
    /// Whether backup is compressed
    #[serde(default = "default_compressed")]
    pub compressed: bool,
}

fn default_compressed() -> bool {
    true
}

/// Collection data within a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCollectionData {
    /// Collection name (without user prefix)
    pub name: String,
    /// Original full collection name
    pub full_name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Distance metric
    pub metric: String,
    /// Vectors in this collection
    pub vectors: Vec<BackupVector>,
    /// Collection metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Vector data in backup format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupVector {
    /// Vector ID
    pub id: String,
    /// Vector data
    pub data: Vec<f32>,
    /// Sparse vector data (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sparse: Option<crate::models::SparseVector>,
    /// Payload/metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// Full backup data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBackupData {
    /// Backup metadata
    pub info: UserBackupInfo,
    /// Collection data
    pub collections: Vec<BackupCollectionData>,
}

/// Configuration for the backup manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Base directory for storing backups
    #[serde(default = "default_backup_dir")]
    pub backup_dir: PathBuf,
    /// Maximum backups per user (0 = unlimited)
    #[serde(default = "default_max_backups_per_user")]
    pub max_backups_per_user: usize,
    /// Maximum backup age in hours (0 = unlimited)
    #[serde(default)]
    pub max_backup_age_hours: u64,
    /// Enable compression
    #[serde(default = "default_compression_enabled")]
    pub compression_enabled: bool,
    /// Compression level (1-9, higher = better compression)
    #[serde(default = "default_compression_level")]
    pub compression_level: u32,
}

fn default_backup_dir() -> PathBuf {
    PathBuf::from("./data/hub_backups")
}

fn default_max_backups_per_user() -> usize {
    10
}

fn default_compression_enabled() -> bool {
    true
}

fn default_compression_level() -> u32 {
    6
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: default_backup_dir(),
            max_backups_per_user: default_max_backups_per_user(),
            max_backup_age_hours: 0,
            compression_enabled: default_compression_enabled(),
            compression_level: default_compression_level(),
        }
    }
}

/// User-scoped backup manager for HiveHub cluster mode
pub struct UserBackupManager {
    /// Configuration
    config: BackupConfig,
    /// Reference to the vector store
    store: Arc<VectorStore>,
    /// Backup metadata cache (user_id -> Vec<BackupInfo>)
    metadata_cache: RwLock<HashMap<Uuid, Vec<UserBackupInfo>>>,
}

impl UserBackupManager {
    /// Create a new backup manager
    pub fn new(config: BackupConfig, store: Arc<VectorStore>) -> Result<Self> {
        // Ensure backup directory exists
        fs::create_dir_all(&config.backup_dir).map_err(VectorizerError::IoError)?;

        Ok(Self {
            config,
            store,
            metadata_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Estimate the backup size for a user's collections
    ///
    /// Returns estimated size in bytes before compression
    pub async fn estimate_backup_size(
        &self,
        user_id: &Uuid,
        collection_names: Option<&[String]>,
    ) -> Result<u64> {
        // Get user's collections
        let user_collections = self.store.list_collections_for_owner(user_id);

        // Filter collections if specific ones requested
        let collections_to_check: Vec<String> = match collection_names {
            Some(names) => names
                .iter()
                .filter(|name| user_collections.contains(name))
                .cloned()
                .collect(),
            None => user_collections,
        };

        let mut estimated_bytes: u64 = 0;

        for collection_name in collections_to_check {
            if let Ok(collection) = self.store.get_collection(&collection_name) {
                let vector_count = collection.vector_count() as u64;
                let dimension = collection.config().dimension as u64;

                // Estimate: vectors (f32 = 4 bytes each) + metadata overhead (~100 bytes per vector)
                let vector_bytes = vector_count * dimension * 4;
                let metadata_bytes = vector_count * 100;
                estimated_bytes += vector_bytes + metadata_bytes;
            }
        }

        // Add JSON serialization overhead (~20%)
        estimated_bytes = (estimated_bytes as f64 * 1.2) as u64;

        Ok(estimated_bytes)
    }

    /// Get the backup directory for a specific user
    fn user_backup_dir(&self, user_id: &Uuid) -> PathBuf {
        self.config.backup_dir.join(user_id.to_string())
    }

    /// Get the backup file path
    fn backup_file_path(&self, user_id: &Uuid, backup_id: &Uuid) -> PathBuf {
        self.user_backup_dir(user_id)
            .join(format!("{}.backup.gz", backup_id))
    }

    /// Get the metadata file path
    fn metadata_file_path(&self, user_id: &Uuid, backup_id: &Uuid) -> PathBuf {
        self.user_backup_dir(user_id)
            .join(format!("{}.meta.json", backup_id))
    }

    /// Create a backup for a user's collections
    ///
    /// # Arguments
    /// * `user_id` - The user/tenant ID
    /// * `name` - Human-readable backup name
    /// * `description` - Optional description
    /// * `collection_names` - Optional list of collection names to backup (None = all user's collections)
    pub async fn create_backup(
        &self,
        user_id: Uuid,
        name: String,
        description: Option<String>,
        collection_names: Option<Vec<String>>,
    ) -> Result<UserBackupInfo> {
        info!(
            "Creating backup '{}' for user {} with collections: {:?}",
            name, user_id, collection_names
        );

        // Ensure user backup directory exists
        let user_dir = self.user_backup_dir(&user_id);
        fs::create_dir_all(&user_dir).map_err(|e| VectorizerError::IoError(e))?;

        // Get user's collections
        let user_collections = self.store.list_collections_for_owner(&user_id);

        // Filter collections if specific ones requested
        let collections_to_backup: Vec<String> = match collection_names {
            Some(names) => {
                // Validate that requested collections belong to user
                let mut validated = Vec::new();
                for name in names {
                    if user_collections.contains(&name) {
                        validated.push(name);
                    } else {
                        warn!(
                            "Collection '{}' not found or not owned by user {}",
                            name, user_id
                        );
                    }
                }
                validated
            }
            None => user_collections,
        };

        if collections_to_backup.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "No collections to backup".to_string(),
            });
        }

        // Generate backup ID
        let backup_id = Uuid::new_v4();

        // Collect data from each collection
        let mut backup_collections = Vec::new();
        let mut total_vectors: u64 = 0;

        for collection_name in &collections_to_backup {
            match self.store.get_collection(collection_name) {
                Ok(collection) => {
                    let metadata = collection.metadata();
                    let config = collection.config();

                    // Get all vectors from collection
                    let vectors: Vec<BackupVector> = collection
                        .get_all_vectors()
                        .into_iter()
                        .map(|v| BackupVector {
                            id: v.id.clone(),
                            data: v.data.clone(),
                            sparse: v.sparse.clone(),
                            payload: v.payload.as_ref().map(|p| p.data.clone()),
                        })
                        .collect();

                    total_vectors += vectors.len() as u64;

                    // Extract simple collection name (remove user prefix if present)
                    let simple_name = collection_name
                        .split(':')
                        .last()
                        .unwrap_or(collection_name)
                        .to_string();

                    backup_collections.push(BackupCollectionData {
                        name: simple_name,
                        full_name: collection_name.clone(),
                        dimension: config.dimension,
                        metric: format!("{:?}", config.metric),
                        vectors,
                        metadata: HashMap::new(),
                    });

                    debug!(
                        "Added collection '{}' with {} vectors to backup",
                        collection_name, metadata.vector_count
                    );
                }
                Err(e) => {
                    warn!("Failed to backup collection '{}': {}", collection_name, e);
                }
            }
        }

        // Create backup data
        let backup_info = UserBackupInfo {
            id: backup_id,
            user_id,
            name: name.clone(),
            description,
            created_at: Utc::now(),
            collections: collections_to_backup.clone(),
            vector_count: total_vectors,
            size_bytes: 0, // Will be updated after serialization
            format_version: 1,
            checksum: None, // Will be updated after serialization
            compressed: self.config.compression_enabled,
        };

        let backup_data = UserBackupData {
            info: backup_info.clone(),
            collections: backup_collections,
        };

        // Serialize and compress
        let json_data = serde_json::to_vec(&backup_data)
            .map_err(|e| VectorizerError::Serialization(e.to_string()))?;

        let file_path = self.backup_file_path(&user_id, &backup_id);
        let final_size: u64;

        if self.config.compression_enabled {
            // Compress with gzip
            let file = fs::File::create(&file_path).map_err(|e| VectorizerError::IoError(e))?;
            let mut encoder = GzEncoder::new(file, Compression::new(self.config.compression_level));
            encoder
                .write_all(&json_data)
                .map_err(|e| VectorizerError::IoError(e))?;
            encoder.finish().map_err(|e| VectorizerError::IoError(e))?;

            final_size = fs::metadata(&file_path)
                .map_err(|e| VectorizerError::IoError(e))?
                .len();
        } else {
            fs::write(&file_path, &json_data).map_err(|e| VectorizerError::IoError(e))?;
            final_size = json_data.len() as u64;
        }

        // Calculate checksum
        let checksum = Self::calculate_checksum(&file_path)?;

        // Update backup info with final values
        let mut final_info = backup_info;
        final_info.size_bytes = final_size;
        final_info.checksum = Some(checksum);

        // Save metadata file
        let meta_path = self.metadata_file_path(&user_id, &backup_id);
        let meta_json = serde_json::to_string_pretty(&final_info)
            .map_err(|e| VectorizerError::Serialization(e.to_string()))?;
        fs::write(&meta_path, meta_json).map_err(|e| VectorizerError::IoError(e))?;

        // Update cache
        {
            let mut cache = self.metadata_cache.write();
            cache
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(final_info.clone());
        }

        // Cleanup old backups if needed
        self.cleanup_old_backups(&user_id).await?;

        info!(
            "Backup '{}' created successfully: {} collections, {} vectors, {} bytes",
            name,
            collections_to_backup.len(),
            total_vectors,
            final_size
        );

        Ok(final_info)
    }

    /// List all backups for a user
    pub async fn list_backups(&self, user_id: &Uuid) -> Result<Vec<UserBackupInfo>> {
        let user_dir = self.user_backup_dir(user_id);

        if !user_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        let entries = fs::read_dir(&user_dir).map_err(|e| VectorizerError::IoError(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| VectorizerError::IoError(e))?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false)
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".meta.json"))
                    .unwrap_or(false)
            {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<UserBackupInfo>(&content) {
                        Ok(info) => backups.push(info),
                        Err(e) => warn!("Failed to parse backup metadata {:?}: {}", path, e),
                    },
                    Err(e) => warn!("Failed to read backup metadata {:?}: {}", path, e),
                }
            }
        }

        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Get backup info by ID
    pub async fn get_backup(&self, user_id: &Uuid, backup_id: &Uuid) -> Result<UserBackupInfo> {
        let meta_path = self.metadata_file_path(user_id, backup_id);

        if !meta_path.exists() {
            return Err(VectorizerError::NotFound(format!(
                "Backup {} not found for user {}",
                backup_id, user_id
            )));
        }

        let content = fs::read_to_string(&meta_path).map_err(|e| VectorizerError::IoError(e))?;
        let info: UserBackupInfo = serde_json::from_str(&content)
            .map_err(|e| VectorizerError::Deserialization(e.to_string()))?;

        Ok(info)
    }

    /// Download backup data (returns compressed bytes)
    pub async fn download_backup(&self, user_id: &Uuid, backup_id: &Uuid) -> Result<Vec<u8>> {
        let file_path = self.backup_file_path(user_id, backup_id);

        if !file_path.exists() {
            return Err(VectorizerError::NotFound(format!(
                "Backup {} not found for user {}",
                backup_id, user_id
            )));
        }

        let data = fs::read(&file_path).map_err(|e| VectorizerError::IoError(e))?;

        info!(
            "Downloaded backup {} for user {} ({} bytes)",
            backup_id,
            user_id,
            data.len()
        );

        Ok(data)
    }

    /// Restore a backup for a user
    ///
    /// # Arguments
    /// * `user_id` - The user/tenant ID
    /// * `backup_id` - The backup to restore
    /// * `overwrite` - Whether to overwrite existing collections
    pub async fn restore_backup(
        &self,
        user_id: &Uuid,
        backup_id: &Uuid,
        overwrite: bool,
    ) -> Result<RestoreResult> {
        info!(
            "Restoring backup {} for user {} (overwrite: {})",
            backup_id, user_id, overwrite
        );

        let file_path = self.backup_file_path(user_id, backup_id);

        if !file_path.exists() {
            return Err(VectorizerError::NotFound(format!(
                "Backup {} not found for user {}",
                backup_id, user_id
            )));
        }

        // Read and decompress backup
        let backup_data = self.load_backup_data(&file_path)?;

        // Verify user ID matches
        if backup_data.info.user_id != *user_id {
            return Err(VectorizerError::AuthorizationError(
                "Backup does not belong to this user".to_string(),
            ));
        }

        let mut result = RestoreResult {
            backup_id: *backup_id,
            collections_restored: Vec::new(),
            collections_skipped: Vec::new(),
            vectors_restored: 0,
            errors: Vec::new(),
        };

        // Restore each collection
        for collection_data in backup_data.collections {
            let collection_name = &collection_data.full_name;

            // Check if collection exists
            let exists = self.store.get_collection(collection_name).is_ok();

            if exists && !overwrite {
                result.collections_skipped.push(collection_name.clone());
                continue;
            }

            // Delete existing collection if overwriting
            if exists && overwrite {
                if let Err(e) = self.store.delete_collection(collection_name) {
                    result.errors.push(format!(
                        "Failed to delete existing collection '{}': {}",
                        collection_name, e
                    ));
                    continue;
                }
            }

            // Parse metric
            let metric = match collection_data.metric.to_lowercase().as_str() {
                "cosine" => crate::models::DistanceMetric::Cosine,
                "euclidean" => crate::models::DistanceMetric::Euclidean,
                "dotproduct" | "dot" => crate::models::DistanceMetric::DotProduct,
                _ => crate::models::DistanceMetric::Cosine,
            };

            // Create collection config
            let config = crate::models::CollectionConfig {
                dimension: collection_data.dimension,
                metric,
                hnsw_config: crate::models::HnswConfig::default(),
                quantization: crate::models::QuantizationConfig::None,
                compression: crate::models::CompressionConfig::default(),
                normalization: None,
                storage_type: Some(crate::models::StorageType::Memory),
                sharding: None,
                graph: None,
            };

            // Create collection
            if let Err(e) = self.store.create_collection(collection_name, config) {
                result.errors.push(format!(
                    "Failed to create collection '{}': {}",
                    collection_name, e
                ));
                continue;
            }

            // Insert vectors
            let vectors: Vec<Vector> = collection_data
                .vectors
                .into_iter()
                .map(|v| Vector {
                    id: v.id,
                    data: v.data,
                    sparse: v.sparse,
                    payload: v.payload.map(crate::models::Payload::new),
                })
                .collect();

            let vector_count = vectors.len();

            if let Err(e) = self.store.insert(collection_name, vectors) {
                result.errors.push(format!(
                    "Failed to insert vectors into '{}': {}",
                    collection_name, e
                ));
                continue;
            }

            result.collections_restored.push(collection_name.clone());
            result.vectors_restored += vector_count as u64;

            debug!(
                "Restored collection '{}' with {} vectors",
                collection_name, vector_count
            );
        }

        // Save restored collections to disk
        for collection_name in &result.collections_restored {
            if let Err(e) = self.store.save_collection_to_file(collection_name) {
                result.errors.push(format!(
                    "Failed to save collection '{}': {}",
                    collection_name, e
                ));
            }
        }

        info!(
            "Restore complete: {} collections, {} vectors, {} errors",
            result.collections_restored.len(),
            result.vectors_restored,
            result.errors.len()
        );

        Ok(result)
    }

    /// Delete a backup
    pub async fn delete_backup(&self, user_id: &Uuid, backup_id: &Uuid) -> Result<()> {
        let file_path = self.backup_file_path(user_id, backup_id);
        let meta_path = self.metadata_file_path(user_id, backup_id);

        if !file_path.exists() && !meta_path.exists() {
            return Err(VectorizerError::NotFound(format!(
                "Backup {} not found for user {}",
                backup_id, user_id
            )));
        }

        // Remove backup file
        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| VectorizerError::IoError(e))?;
        }

        // Remove metadata file
        if meta_path.exists() {
            fs::remove_file(&meta_path).map_err(|e| VectorizerError::IoError(e))?;
        }

        // Update cache
        {
            let mut cache = self.metadata_cache.write();
            if let Some(backups) = cache.get_mut(user_id) {
                backups.retain(|b| b.id != *backup_id);
            }
        }

        info!("Deleted backup {} for user {}", backup_id, user_id);

        Ok(())
    }

    /// Upload and import a backup from raw data
    pub async fn upload_backup(
        &self,
        user_id: Uuid,
        data: Vec<u8>,
        name: Option<String>,
    ) -> Result<UserBackupInfo> {
        info!(
            "Uploading backup for user {} ({} bytes)",
            user_id,
            data.len()
        );

        // Try to decompress and parse
        let backup_data = self.parse_backup_bytes(&data)?;

        // Verify or update user_id
        let mut final_data = backup_data;
        final_data.info.user_id = user_id;
        final_data.info.id = Uuid::new_v4(); // Generate new ID for uploaded backup

        if let Some(n) = name {
            final_data.info.name = n;
        }

        final_data.info.created_at = Utc::now();

        // Ensure user backup directory exists
        let user_dir = self.user_backup_dir(&user_id);
        fs::create_dir_all(&user_dir).map_err(|e| VectorizerError::IoError(e))?;

        // Save the backup
        let file_path = self.backup_file_path(&user_id, &final_data.info.id);

        let json_data = serde_json::to_vec(&final_data)
            .map_err(|e| VectorizerError::Serialization(e.to_string()))?;

        if self.config.compression_enabled {
            let file = fs::File::create(&file_path).map_err(|e| VectorizerError::IoError(e))?;
            let mut encoder = GzEncoder::new(file, Compression::new(self.config.compression_level));
            encoder
                .write_all(&json_data)
                .map_err(|e| VectorizerError::IoError(e))?;
            encoder.finish().map_err(|e| VectorizerError::IoError(e))?;
        } else {
            fs::write(&file_path, &json_data).map_err(|e| VectorizerError::IoError(e))?;
        }

        // Update size and checksum
        let final_size = fs::metadata(&file_path)
            .map_err(|e| VectorizerError::IoError(e))?
            .len();
        let checksum = Self::calculate_checksum(&file_path)?;

        final_data.info.size_bytes = final_size;
        final_data.info.checksum = Some(checksum);
        final_data.info.compressed = self.config.compression_enabled;

        // Save metadata
        let meta_path = self.metadata_file_path(&user_id, &final_data.info.id);
        let meta_json = serde_json::to_string_pretty(&final_data.info)
            .map_err(|e| VectorizerError::Serialization(e.to_string()))?;
        fs::write(&meta_path, meta_json).map_err(|e| VectorizerError::IoError(e))?;

        info!(
            "Uploaded backup '{}' for user {}: {} bytes",
            final_data.info.name, user_id, final_size
        );

        Ok(final_data.info)
    }

    /// Load backup data from file
    fn load_backup_data(&self, path: &Path) -> Result<UserBackupData> {
        let file = fs::File::open(path).map_err(|e| VectorizerError::IoError(e))?;

        // Try to decompress (gzip)
        let mut decoder = GzDecoder::new(file);
        let mut json_data = Vec::new();

        match decoder.read_to_end(&mut json_data) {
            Ok(_) => {}
            Err(_) => {
                // Not compressed, read directly
                json_data = fs::read(path).map_err(|e| VectorizerError::IoError(e))?;
            }
        }

        let backup: UserBackupData = serde_json::from_slice(&json_data)
            .map_err(|e| VectorizerError::Deserialization(e.to_string()))?;

        Ok(backup)
    }

    /// Parse backup from raw bytes (handles both compressed and uncompressed)
    fn parse_backup_bytes(&self, data: &[u8]) -> Result<UserBackupData> {
        // Try gzip decompression first
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();

        let json_data = match decoder.read_to_end(&mut decompressed) {
            Ok(_) => decompressed,
            Err(_) => data.to_vec(), // Not compressed
        };

        let backup: UserBackupData = serde_json::from_slice(&json_data)
            .map_err(|e| VectorizerError::Deserialization(e.to_string()))?;

        Ok(backup)
    }

    /// Calculate SHA-256 checksum of a file
    fn calculate_checksum(path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let data = fs::read(path).map_err(|e| VectorizerError::IoError(e))?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    /// Cleanup old backups for a user
    async fn cleanup_old_backups(&self, user_id: &Uuid) -> Result<()> {
        if self.config.max_backups_per_user == 0 && self.config.max_backup_age_hours == 0 {
            return Ok(());
        }

        let mut backups = self.list_backups(user_id).await?;

        // Sort by age (oldest first for removal)
        backups.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let now = Utc::now();
        let mut to_delete = Vec::new();

        // Check age limit
        if self.config.max_backup_age_hours > 0 {
            let age_limit = chrono::Duration::hours(self.config.max_backup_age_hours as i64);
            for backup in &backups {
                if now - backup.created_at > age_limit {
                    to_delete.push(backup.id);
                }
            }
        }

        // Check count limit
        if self.config.max_backups_per_user > 0 {
            let remaining: Vec<_> = backups
                .iter()
                .filter(|b| !to_delete.contains(&b.id))
                .collect();

            if remaining.len() > self.config.max_backups_per_user {
                let excess = remaining.len() - self.config.max_backups_per_user;
                for backup in remaining.iter().take(excess) {
                    if !to_delete.contains(&backup.id) {
                        to_delete.push(backup.id);
                    }
                }
            }
        }

        // Delete old backups
        for backup_id in to_delete {
            if let Err(e) = self.delete_backup(user_id, &backup_id).await {
                warn!("Failed to cleanup old backup {}: {}", backup_id, e);
            } else {
                debug!("Cleaned up old backup {} for user {}", backup_id, user_id);
            }
        }

        Ok(())
    }
}

/// Result of a restore operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    /// Backup ID that was restored
    pub backup_id: Uuid,
    /// Collections that were successfully restored
    pub collections_restored: Vec<String>,
    /// Collections that were skipped (already exist)
    pub collections_skipped: Vec<String>,
    /// Total vectors restored
    pub vectors_restored: u64,
    /// Errors encountered during restore
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_config_default() {
        let config = BackupConfig::default();
        assert_eq!(config.max_backups_per_user, 10);
        assert!(config.compression_enabled);
        assert_eq!(config.compression_level, 6);
    }

    #[test]
    fn test_backup_info_serialization() {
        let info = UserBackupInfo {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "test_backup".to_string(),
            description: Some("Test description".to_string()),
            created_at: Utc::now(),
            collections: vec!["col1".to_string(), "col2".to_string()],
            vector_count: 100,
            size_bytes: 1024,
            format_version: 1,
            checksum: Some("abc123".to_string()),
            compressed: true,
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: UserBackupInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.id, parsed.id);
        assert_eq!(info.name, parsed.name);
        assert_eq!(info.vector_count, parsed.vector_count);
    }
}
