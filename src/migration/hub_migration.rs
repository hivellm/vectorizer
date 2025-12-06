//! HiveHub Cloud Migration Module
//!
//! Provides tools for migrating existing standalone Vectorizer data to
//! HiveHub Cloud multi-tenant mode, including:
//! - Scanning collections without owner_id
//! - Mapping collections to users
//! - Renaming collections with tenant prefix
//! - Backup and rollback capabilities

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::db::VectorStore;
use crate::error::{Result, VectorizerError};

/// Migration status for a collection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    /// Not yet migrated
    Pending,
    /// Migration in progress
    InProgress,
    /// Successfully migrated
    Completed,
    /// Migration failed
    Failed,
    /// Rolled back after failure
    RolledBack,
    /// Skipped (already has owner)
    Skipped,
}

/// Collection migration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMigrationRecord {
    /// Original collection name
    pub original_name: String,
    /// New collection name (with tenant prefix)
    pub new_name: Option<String>,
    /// Assigned owner ID
    pub owner_id: Option<Uuid>,
    /// Migration status
    pub status: MigrationStatus,
    /// Error message if failed
    pub error: Option<String>,
    /// Vector count at migration time
    pub vector_count: usize,
    /// Migration timestamp
    pub migrated_at: Option<DateTime<Utc>>,
}

/// Migration plan for HiveHub transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Plan ID
    pub id: Uuid,
    /// Plan creation time
    pub created_at: DateTime<Utc>,
    /// Collections to migrate
    pub collections: Vec<CollectionMigrationRecord>,
    /// Default owner for unmapped collections
    pub default_owner: Option<Uuid>,
    /// Backup ID (if backup was created)
    pub backup_id: Option<String>,
    /// Whether to dry-run (preview only)
    pub dry_run: bool,
}

/// Migration result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Plan ID
    pub plan_id: Uuid,
    /// Total collections processed
    pub total: usize,
    /// Successfully migrated
    pub succeeded: usize,
    /// Failed migrations
    pub failed: usize,
    /// Skipped (already owned)
    pub skipped: usize,
    /// Detailed results
    pub details: Vec<CollectionMigrationRecord>,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// End time
    pub completed_at: DateTime<Utc>,
}

/// HiveHub migration manager
pub struct HubMigrationManager {
    /// Vector store reference
    store: Arc<VectorStore>,
    /// Backup directory
    backup_dir: PathBuf,
    /// Current migration plan (if any)
    current_plan: Option<MigrationPlan>,
}

impl HubMigrationManager {
    /// Create a new migration manager
    pub fn new(store: Arc<VectorStore>) -> Self {
        let data_dir = VectorStore::get_data_dir();
        let backup_dir = data_dir.join("hub_migration_backups");

        Self {
            store,
            backup_dir,
            current_plan: None,
        }
    }

    /// Create with custom backup directory
    pub fn with_backup_dir(store: Arc<VectorStore>, backup_dir: PathBuf) -> Self {
        Self {
            store,
            backup_dir,
            current_plan: None,
        }
    }

    /// Scan for collections that need migration (no owner_id)
    pub fn scan_collections(&self) -> Result<Vec<CollectionMigrationRecord>> {
        info!("Scanning collections for HiveHub migration...");

        let all_collections = self.store.list_collections();
        let mut records = Vec::new();

        for name in all_collections {
            // Check if collection already has owner (prefixed with user_)
            let has_owner = name.starts_with("user_") || name.contains(':');

            let vector_count = self
                .store
                .get_collection(&name)
                .map(|c| c.vector_count())
                .unwrap_or(0);

            let record = CollectionMigrationRecord {
                original_name: name.clone(),
                new_name: None,
                owner_id: None,
                status: if has_owner {
                    MigrationStatus::Skipped
                } else {
                    MigrationStatus::Pending
                },
                error: None,
                vector_count,
                migrated_at: None,
            };

            records.push(record);
        }

        info!(
            "Found {} collections: {} need migration, {} already owned",
            records.len(),
            records
                .iter()
                .filter(|r| r.status == MigrationStatus::Pending)
                .count(),
            records
                .iter()
                .filter(|r| r.status == MigrationStatus::Skipped)
                .count()
        );

        Ok(records)
    }

    /// Create a migration plan
    pub fn create_plan(
        &mut self,
        collection_mappings: HashMap<String, Uuid>,
        default_owner: Option<Uuid>,
        dry_run: bool,
    ) -> Result<MigrationPlan> {
        let records = self.scan_collections()?;

        let mut plan_collections = Vec::new();

        for mut record in records {
            if record.status == MigrationStatus::Skipped {
                plan_collections.push(record);
                continue;
            }

            // Find owner from mappings or use default
            let owner_id = collection_mappings
                .get(&record.original_name)
                .copied()
                .or(default_owner);

            if let Some(owner) = owner_id {
                record.owner_id = Some(owner);
                record.new_name = Some(format!("user_{}:{}", owner, record.original_name));
            } else {
                record.status = MigrationStatus::Failed;
                record.error = Some("No owner assigned and no default owner".to_string());
            }

            plan_collections.push(record);
        }

        let plan = MigrationPlan {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            collections: plan_collections,
            default_owner,
            backup_id: None,
            dry_run,
        };

        self.current_plan = Some(plan.clone());

        info!(
            "Created migration plan {}: {} collections",
            plan.id,
            plan.collections.len()
        );

        Ok(plan)
    }

    /// Create a backup before migration
    pub async fn create_backup(&mut self) -> Result<String> {
        info!("Creating backup before migration...");

        // Ensure backup directory exists
        std::fs::create_dir_all(&self.backup_dir).map_err(|e| VectorizerError::IoError(e))?;

        let backup_id = format!("hub_migration_{}", Utc::now().format("%Y%m%d_%H%M%S"));
        let backup_path = self.backup_dir.join(&backup_id);

        std::fs::create_dir_all(&backup_path).map_err(|e| VectorizerError::IoError(e))?;

        // Save current plan if exists
        if let Some(ref plan) = self.current_plan {
            let plan_path = backup_path.join("migration_plan.json");
            let plan_json = serde_json::to_string_pretty(plan)?;
            std::fs::write(&plan_path, plan_json)?;
        }

        // Save collection list
        let collections = self.store.list_collections();
        let collections_path = backup_path.join("collections.json");
        let collections_json = serde_json::to_string_pretty(&collections)?;
        std::fs::write(&collections_path, collections_json)?;

        // Note: Full data backup would use the existing backup system
        // This just saves metadata for rollback

        info!("Backup created: {}", backup_id);

        if let Some(ref mut plan) = self.current_plan {
            plan.backup_id = Some(backup_id.clone());
        }

        Ok(backup_id)
    }

    /// Execute the migration plan
    pub async fn execute(&mut self) -> Result<MigrationResult> {
        let plan = self.current_plan.take().ok_or_else(|| {
            VectorizerError::ConfigurationError("No migration plan created".to_string())
        })?;

        let started_at = Utc::now();
        let mut results = plan.collections.clone();
        let mut succeeded = 0;
        let mut failed = 0;
        let mut skipped = 0;

        info!(
            "Executing migration plan {} (dry_run: {})",
            plan.id, plan.dry_run
        );

        let total_count = results.len();
        for i in 0..total_count {
            let record = &mut results[i];

            if record.status == MigrationStatus::Skipped {
                skipped += 1;
                continue;
            }

            if record.owner_id.is_none() || record.new_name.is_none() {
                record.status = MigrationStatus::Failed;
                record.error = Some("Missing owner_id or new_name".to_string());
                failed += 1;
                continue;
            }

            record.status = MigrationStatus::InProgress;

            let new_name = record.new_name.clone().unwrap();
            let owner_id = record.owner_id.unwrap();
            let original_name = record.original_name.clone();

            info!(
                "[{}/{}] Migrating '{}' -> '{}' (owner: {})",
                i + 1,
                total_count,
                original_name,
                new_name,
                owner_id
            );

            if plan.dry_run {
                info!("  [DRY RUN] Would migrate collection");
                record.status = MigrationStatus::Completed;
                record.migrated_at = Some(Utc::now());
                succeeded += 1;
                continue;
            }

            // Execute actual migration
            match self
                .migrate_collection(&original_name, &new_name, owner_id)
                .await
            {
                Ok(()) => {
                    results[i].status = MigrationStatus::Completed;
                    results[i].migrated_at = Some(Utc::now());
                    succeeded += 1;
                    info!("  Successfully migrated");
                }
                Err(e) => {
                    results[i].status = MigrationStatus::Failed;
                    results[i].error = Some(e.to_string());
                    failed += 1;
                    error!("  Failed to migrate: {}", e);
                }
            }
        }

        let completed_at = Utc::now();

        let result = MigrationResult {
            plan_id: plan.id,
            total: results.len(),
            succeeded,
            failed,
            skipped,
            details: results,
            started_at,
            completed_at,
        };

        info!(
            "Migration completed: {} succeeded, {} failed, {} skipped",
            succeeded, failed, skipped
        );

        Ok(result)
    }

    /// Migrate a single collection
    async fn migrate_collection(
        &self,
        original_name: &str,
        new_name: &str,
        owner_id: Uuid,
    ) -> Result<()> {
        // Get original collection
        let collection = self.store.get_collection(original_name)?;

        // Get collection config
        let config = collection.config().clone();

        // Create new collection with owner
        self.store
            .create_collection_with_owner(new_name, config, owner_id)?;

        // Copy vectors from old to new collection
        let vectors = collection.get_all_vectors();
        if !vectors.is_empty() {
            self.store.insert(new_name, vectors)?;
        }

        // Delete original collection
        self.store.delete_collection(original_name)?;

        debug!(
            "Migrated collection '{}' to '{}' with owner {}",
            original_name, new_name, owner_id
        );

        Ok(())
    }

    /// Rollback a failed migration using backup
    pub async fn rollback(&self, backup_id: &str) -> Result<()> {
        info!("Rolling back migration using backup: {}", backup_id);

        let backup_path = self.backup_dir.join(backup_id);

        if !backup_path.exists() {
            return Err(VectorizerError::NotFound(format!(
                "Backup not found: {}",
                backup_id
            )));
        }

        // Load original collections list
        let collections_path = backup_path.join("collections.json");
        let collections_json = std::fs::read_to_string(&collections_path)?;
        let original_collections: Vec<String> = serde_json::from_str(&collections_json)?;

        // Load migration plan
        let plan_path = backup_path.join("migration_plan.json");
        if plan_path.exists() {
            let plan_json = std::fs::read_to_string(&plan_path)?;
            let plan: MigrationPlan = serde_json::from_str(&plan_json)?;

            // Reverse each completed migration
            for record in plan
                .collections
                .iter()
                .filter(|r| r.status == MigrationStatus::Completed)
            {
                if let (Some(new_name), Some(_owner_id)) = (&record.new_name, record.owner_id) {
                    info!("Rolling back: '{}' -> '{}'", new_name, record.original_name);

                    // Get migrated collection
                    if let Ok(collection) = self.store.get_collection(new_name) {
                        let config = collection.config().clone();
                        let vectors = collection.get_all_vectors();

                        // Recreate original collection
                        self.store
                            .create_collection(&record.original_name, config)?;
                        if !vectors.is_empty() {
                            self.store.insert(&record.original_name, vectors)?;
                        }

                        // Delete migrated collection
                        self.store.delete_collection(new_name)?;
                    }
                }
            }
        }

        info!("Rollback completed");
        Ok(())
    }

    /// List available backups
    pub fn list_backups(&self) -> Result<Vec<String>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in std::fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("hub_migration_") {
                        backups.push(name.to_string());
                    }
                }
            }
        }

        backups.sort();
        backups.reverse(); // Newest first

        Ok(backups)
    }

    /// Delete a backup
    pub fn delete_backup(&self, backup_id: &str) -> Result<()> {
        let backup_path = self.backup_dir.join(backup_id);

        if !backup_path.exists() {
            return Err(VectorizerError::NotFound(format!(
                "Backup not found: {}",
                backup_id
            )));
        }

        std::fs::remove_dir_all(&backup_path)?;
        info!("Deleted backup: {}", backup_id);

        Ok(())
    }

    /// Get current plan (if any)
    pub fn current_plan(&self) -> Option<&MigrationPlan> {
        self.current_plan.as_ref()
    }

    /// Clear current plan
    pub fn clear_plan(&mut self) {
        self.current_plan = None;
    }
}

/// Interactive collection mapping helper
pub struct CollectionMapper {
    /// Mappings from collection name to owner ID
    mappings: HashMap<String, Uuid>,
}

impl CollectionMapper {
    /// Create new mapper
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Add a mapping
    pub fn map(&mut self, collection: &str, owner_id: Uuid) {
        self.mappings.insert(collection.to_string(), owner_id);
    }

    /// Add mappings from a config file
    pub fn load_from_file(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let file_mappings: HashMap<String, String> = serde_json::from_str(&content)?;

        for (collection, owner_str) in file_mappings {
            let owner_id = Uuid::parse_str(&owner_str).map_err(|e| {
                VectorizerError::ConfigurationError(format!(
                    "Invalid UUID for collection '{}': {}",
                    collection, e
                ))
            })?;
            self.mappings.insert(collection, owner_id);
        }

        Ok(())
    }

    /// Get all mappings
    pub fn get_mappings(&self) -> HashMap<String, Uuid> {
        self.mappings.clone()
    }

    /// Check if collection is mapped
    pub fn is_mapped(&self, collection: &str) -> bool {
        self.mappings.contains_key(collection)
    }
}

impl Default for CollectionMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_status_serialize() {
        let status = MigrationStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"completed\"");
    }

    #[test]
    fn test_collection_mapper() {
        let mut mapper = CollectionMapper::new();
        let owner_id = Uuid::new_v4();

        mapper.map("test_collection", owner_id);

        assert!(mapper.is_mapped("test_collection"));
        assert!(!mapper.is_mapped("other_collection"));

        let mappings = mapper.get_mappings();
        assert_eq!(mappings.get("test_collection"), Some(&owner_id));
    }

    #[test]
    fn test_collection_migration_record() {
        let record = CollectionMigrationRecord {
            original_name: "my_collection".to_string(),
            new_name: Some("user_123:my_collection".to_string()),
            owner_id: Some(Uuid::new_v4()),
            status: MigrationStatus::Pending,
            error: None,
            vector_count: 100,
            migrated_at: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        let parsed: CollectionMigrationRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.original_name, "my_collection");
        assert_eq!(parsed.status, MigrationStatus::Pending);
    }
}
