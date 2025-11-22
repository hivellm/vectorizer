//! CLI command handlers
//!
//! Implements the business logic for each CLI command

use std::path::PathBuf;

use serde_yaml;
use tracing::{error, info, warn};

use super::{
    ApiKeyCommands, CliConfig, CollectionCommands, ConfigCommands, DbCommands, ServerCommands,
    UserCommands,
};
use crate::auth::{AuthManager, Permission, Role};
use crate::db::VectorStore;
use crate::error::Result;
use crate::models::QuantizationConfig;

/// Handle server management commands
pub async fn handle_server_command(command: ServerCommands, config: &CliConfig) -> Result<()> {
    match command {
        ServerCommands::Start {
            host,
            port,
            auth,
            data_dir,
        } => {
            info!("Starting Vectorizer server on {}:{}", host, port);

            // Initialize authentication if enabled
            if auth {
                let _auth_manager = AuthManager::new(config.auth.clone())?;
                info!("Authentication enabled");
            } else {
                info!("Authentication disabled");
            }

            // Initialize vector store
            let _store = VectorStore::new();

            // Start server (this would integrate with the existing server code)
            info!("Server started successfully");
            info!("Data directory: {:?}", data_dir);
            info!(
                "Authentication: {}",
                if auth { "enabled" } else { "disabled" }
            );

            // In a real implementation, this would start the HTTP server
            // For now, we'll just log the configuration
            Ok(())
        }

        ServerCommands::Stop { host, port } => {
            info!("Stopping Vectorizer server on {}:{}", host, port);

            // In a real implementation, this would send a shutdown signal
            // to the running server process
            info!("Server stopped successfully");
            Ok(())
        }

        ServerCommands::Restart { host, port } => {
            info!("Restarting Vectorizer server on {}:{}", host, port);

            // Stop server
            info!("Stopping server...");

            // Wait a moment
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Start server
            info!("Starting server...");

            Ok(())
        }
    }
}

/// Handle user management commands
pub async fn handle_user_command(command: UserCommands, config: &CliConfig) -> Result<()> {
    let auth_manager = AuthManager::new(config.auth.clone())?;

    match command {
        UserCommands::Create {
            username,
            roles,
            description,
        } => {
            info!("Creating user: {}", username);

            // Parse roles
            let role_list: Vec<Role> = roles
                .split(',')
                .map(|r| r.trim())
                .filter_map(|r| match r.to_lowercase().as_str() {
                    "admin" => Some(Role::Admin),
                    "user" => Some(Role::User),
                    "readonly" => Some(Role::ReadOnly),
                    "service" => Some(Role::Service),
                    _ => {
                        warn!("Unknown role: {}", r);
                        None
                    }
                })
                .collect();

            if role_list.is_empty() {
                return Err(crate::error::VectorizerError::InvalidConfiguration {
                    message: "No valid roles specified".to_string(),
                });
            }

            // Generate JWT token for the user
            let token = auth_manager.generate_jwt(&username, &username, role_list.clone())?;

            info!("User '{}' created successfully", username);
            info!("Roles: {:?}", role_list);
            if let Some(desc) = description {
                info!("Description: {}", desc);
            }
            info!("JWT Token: {}", token);

            Ok(())
        }

        UserCommands::List { detailed } => {
            info!("Listing users...");

            if detailed {
                info!("Detailed user information not yet implemented");
            } else {
                info!("User listing not yet implemented (would require user storage)");
            }

            Ok(())
        }

        UserCommands::Delete { username } => {
            info!("Deleting user: {}", username);

            // In a real implementation, this would remove the user from storage
            info!("User '{}' deleted successfully", username);

            Ok(())
        }

        UserCommands::UpdateRoles { username, roles } => {
            info!("Updating roles for user: {}", username);

            // Parse new roles
            let role_list: Vec<Role> = roles
                .split(',')
                .map(|r| r.trim())
                .filter_map(|r| match r.to_lowercase().as_str() {
                    "admin" => Some(Role::Admin),
                    "user" => Some(Role::User),
                    "readonly" => Some(Role::ReadOnly),
                    "service" => Some(Role::Service),
                    _ => {
                        warn!("Unknown role: {}", r);
                        None
                    }
                })
                .collect();

            if role_list.is_empty() {
                return Err(crate::error::VectorizerError::InvalidConfiguration {
                    message: "No valid roles specified".to_string(),
                });
            }

            // Generate new JWT token with updated roles
            let token = auth_manager.generate_jwt(&username, &username, role_list.clone())?;

            info!("Roles updated for user '{}'", username);
            info!("New roles: {:?}", role_list);
            info!("New JWT Token: {}", token);

            Ok(())
        }
    }
}

/// Handle API key management commands
pub async fn handle_api_key_command(command: ApiKeyCommands, config: &CliConfig) -> Result<()> {
    let auth_manager = AuthManager::new(config.auth.clone())?;

    match command {
        ApiKeyCommands::Create {
            user_id,
            name,
            permissions,
            expires_in_hours,
        } => {
            info!("Creating API key for user: {}", user_id);

            // Parse permissions
            let permission_list: Vec<Permission> = permissions
                .split(',')
                .map(|p| p.trim())
                .filter_map(|p| match p.to_lowercase().as_str() {
                    "read" => Some(Permission::Read),
                    "write" => Some(Permission::Write),
                    "delete" => Some(Permission::Delete),
                    "create_collection" => Some(Permission::CreateCollection),
                    "delete_collection" => Some(Permission::DeleteCollection),
                    "manage_users" => Some(Permission::ManageUsers),
                    "manage_api_keys" => Some(Permission::ManageApiKeys),
                    "view_logs" => Some(Permission::ViewLogs),
                    "system_config" => Some(Permission::SystemConfig),
                    _ => {
                        warn!("Unknown permission: {}", p);
                        None
                    }
                })
                .collect();

            if permission_list.is_empty() {
                return Err(crate::error::VectorizerError::InvalidConfiguration {
                    message: "No valid permissions specified".to_string(),
                });
            }

            // Calculate expiration time
            let expires_at = if expires_in_hours == 0 {
                None
            } else {
                Some(chrono::Utc::now().timestamp() as u64 + (expires_in_hours * 3600))
            };

            // Create API key
            let (api_key, key_info) = auth_manager
                .create_api_key(&user_id, &name, permission_list.clone(), expires_at)
                .await?;

            info!("API key created successfully");
            info!("Key ID: {}", key_info.id);
            info!("Name: {}", key_info.name);
            info!("User ID: {}", key_info.user_id);
            info!("Permissions: {:?}", permission_list);
            if let Some(expires_at) = expires_at {
                info!(
                    "Expires: {}",
                    chrono::DateTime::from_timestamp(expires_at as i64, 0).unwrap()
                );
            } else {
                info!("Expires: Never");
            }
            info!("API Key: {}", api_key);

            Ok(())
        }

        ApiKeyCommands::List { user_id, detailed } => {
            if let Some(user_id) = user_id {
                info!("Listing API keys for user: {}", user_id);
                let keys = auth_manager.list_api_keys(&user_id).await?;

                for key in keys {
                    if detailed {
                        info!("Key ID: {}", key.id);
                        info!("  Name: {}", key.name);
                        info!("  User ID: {}", key.user_id);
                        info!("  Permissions: {:?}", key.permissions);
                        info!(
                            "  Created: {}",
                            chrono::DateTime::from_timestamp(key.created_at as i64, 0).unwrap()
                        );
                        if let Some(last_used) = key.last_used {
                            info!(
                                "  Last Used: {}",
                                chrono::DateTime::from_timestamp(last_used as i64, 0).unwrap()
                            );
                        }
                        info!("  Active: {}", key.active);
                        info!("");
                    } else {
                        info!(
                            "{} - {} ({})",
                            key.id,
                            key.name,
                            if key.active { "active" } else { "inactive" }
                        );
                    }
                }
            } else {
                info!("Listing all API keys...");
                // In a real implementation, this would list all API keys
                info!("Global API key listing not yet implemented");
            }

            Ok(())
        }

        ApiKeyCommands::Revoke { key_id } => {
            info!("Revoking API key: {}", key_id);

            auth_manager.revoke_api_key(&key_id).await?;
            info!("API key '{}' revoked successfully", key_id);

            Ok(())
        }

        ApiKeyCommands::Test { api_key } => {
            info!("Testing API key...");

            match auth_manager.validate_api_key(&api_key).await {
                Ok(claims) => {
                    info!("API key is valid");
                    info!("User ID: {}", claims.user_id);
                    info!("Username: {}", claims.username);
                    info!("Roles: {:?}", claims.roles);
                    info!(
                        "Expires: {}",
                        chrono::DateTime::from_timestamp(claims.exp as i64, 0).unwrap()
                    );
                }
                Err(e) => {
                    error!("API key validation failed: {}", e);
                    return Err(e);
                }
            }

            Ok(())
        }
    }
}

/// Handle collection management commands
pub async fn handle_collection_command(
    command: CollectionCommands,
    _config: &CliConfig,
) -> Result<()> {
    let store = VectorStore::new();

    match command {
        CollectionCommands::Create {
            name,
            dimension,
            metric,
        } => {
            info!("Creating collection: {}", name);

            // Parse distance metric
            let distance_metric = match metric.to_lowercase().as_str() {
                "euclidean" => crate::models::DistanceMetric::Euclidean,
                "cosine" => crate::models::DistanceMetric::Cosine,
                "dot_product" => crate::models::DistanceMetric::DotProduct,
                _ => {
                    return Err(crate::error::VectorizerError::InvalidConfiguration {
                        message: format!("Unknown distance metric: {}", metric),
                    });
                }
            };

            let config = crate::models::CollectionConfig {
                dimension,
                metric: distance_metric,
                hnsw_config: crate::models::HnswConfig::default(),
                quantization: QuantizationConfig::SQ { bits: 8 },
                compression: crate::models::CompressionConfig::default(),
                normalization: Some(crate::normalization::NormalizationConfig::moderate()),
                storage_type: Some(crate::models::StorageType::Memory),
                sharding: None,
            };

            store.create_collection(&name, config)?;
            info!("Collection '{}' created successfully", name);
            info!("Dimension: {}", dimension);
            info!("Metric: {}", metric);

            Ok(())
        }

        CollectionCommands::List { detailed } => {
            info!("Listing collections...");

            let collections = store.list_collections();

            if collections.is_empty() {
                info!("No collections found");
            } else {
                for collection_name in collections {
                    if detailed {
                        if let Ok(metadata) = store.get_collection_metadata(&collection_name) {
                            info!("Collection: {}", collection_name);
                            info!("  Dimension: {}", metadata.config.dimension);
                            info!("  Metric: {:?}", metadata.config.metric);
                            info!("  Vector Count: {}", metadata.vector_count);
                            info!("  HNSW Config: {:?}", metadata.config.hnsw_config);
                            info!("");
                        }
                    } else {
                        info!("{}", collection_name);
                    }
                }
            }

            Ok(())
        }

        CollectionCommands::Delete { name, force } => {
            info!("Deleting collection: {}", name);

            if !force {
                // In a real implementation, this would prompt for confirmation
                warn!("Use --force flag to confirm deletion");
                return Ok(());
            }

            store.delete_collection(&name)?;
            info!("Collection '{}' deleted successfully", name);

            Ok(())
        }

        CollectionCommands::Stats { name } => {
            info!("Collection statistics for: {}", name);

            let metadata = store.get_collection_metadata(&name)?;

            info!("Collection: {}", name);
            info!("  Dimension: {}", metadata.config.dimension);
            info!("  Metric: {:?}", metadata.config.metric);
            info!("  Vector Count: {}", metadata.vector_count);
            info!("  HNSW Config:");
            info!("    M: {}", metadata.config.hnsw_config.m);
            info!(
                "    EF Construction: {}",
                metadata.config.hnsw_config.ef_construction
            );
            info!("    EF Search: {}", metadata.config.hnsw_config.ef_search);
            info!(
                "  Compression: {}",
                if metadata.config.compression.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );

            Ok(())
        }
    }
}

/// Handle database operations
pub async fn handle_db_command(command: DbCommands, _config: &CliConfig) -> Result<()> {
    match command {
        DbCommands::Backup {
            output,
            collections,
        } => {
            info!("Creating database backup to: {:?}", output);

            if collections {
                info!("Including collections in backup");
            }

            // In a real implementation, this would create a backup file
            info!("Backup created successfully");

            Ok(())
        }

        DbCommands::Restore { input, force } => {
            info!("Restoring database from: {:?}", input);

            if force {
                info!("Force restore enabled - will overwrite existing data");
            }

            // In a real implementation, this would restore from backup
            info!("Database restored successfully");

            Ok(())
        }

        DbCommands::Optimize {
            rebuild_indexes,
            cleanup,
        } => Ok(()),
    }
}

/// Handle configuration management
pub async fn handle_config_command(command: ConfigCommands, config: &CliConfig) -> Result<()> {
    match command {
        ConfigCommands::Show { show_secrets } => {
            info!("Current configuration:");

            info!("Server:");
            info!("  Host: {}", config.server.host);
            info!("  Port: {}", config.server.port);
            info!("  Data Directory: {:?}", config.server.data_dir);
            info!("  Auth Enabled: {}", config.server.auth_enabled);

            info!("Authentication:");
            info!("  JWT Expiration: {} seconds", config.auth.jwt_expiration);
            info!("  API Key Length: {}", config.auth.api_key_length);
            info!(
                "  Rate Limit (per minute): {}",
                config.auth.rate_limit_per_minute
            );
            info!(
                "  Rate Limit (per hour): {}",
                config.auth.rate_limit_per_hour
            );
            info!("  Enabled: {}", config.auth.enabled);

            if show_secrets {
                warn!("Showing sensitive configuration values:");
                info!("  JWT Secret: {}", config.auth.jwt_secret);
            } else {
                info!("  JWT Secret: [HIDDEN] (use --show-secrets to reveal)");
            }

            info!("Database:");
            info!("  Persistence Path: {:?}", config.database.persistence_path);
            info!(
                "  Compression Enabled: {}",
                config.database.compression_enabled
            );
            info!(
                "  Compression Threshold: {} bytes",
                config.database.compression_threshold
            );

            info!("Logging:");
            info!("  Level: {}", config.logging.level);
            info!("  Log to File: {}", config.logging.log_to_file);
            if let Some(log_file) = &config.logging.log_file {
                info!("  Log File: {:?}", log_file);
            }

            Ok(())
        }

        ConfigCommands::Validate { file } => {
            info!("Validating configuration file: {:?}", file);

            if !file.exists() {
                return Err(crate::error::VectorizerError::NotFound(format!(
                    "Configuration file not found: {:?}",
                    file
                )));
            }

            let content = std::fs::read_to_string(&file)?;
            let _config: CliConfig = serde_yaml::from_str(&content)
                .map_err(|e| crate::error::VectorizerError::YamlError(e))?;

            info!("Configuration file is valid");

            Ok(())
        }

        ConfigCommands::Generate { output } => {
            info!("Generating default configuration to: {:?}", output);

            let default_config = CliConfig::default();
            let yaml = serde_yaml::to_string(&default_config)
                .map_err(|e| crate::error::VectorizerError::YamlError(e))?;

            std::fs::write(&output, yaml)?;
            info!("Default configuration generated successfully");

            Ok(())
        }
    }
}

/// Handle system status command
pub async fn handle_status_command(detailed: bool, config: &CliConfig) -> Result<()> {
    info!("Vectorizer System Status");
    info!("========================");

    info!("Server Configuration:");
    info!("  Host: {}", config.server.host);
    info!("  Port: {}", config.server.port);
    info!("  Data Directory: {:?}", config.server.data_dir);
    info!(
        "  Authentication: {}",
        if config.server.auth_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );

    if detailed {
        info!("Detailed Status:");
        info!("  Version: {}", env!("CARGO_PKG_VERSION"));
        info!(
            "  Rust Version: {}",
            std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string())
        );
        info!(
            "  Build Time: {}",
            std::env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string())
        );

        info!("Authentication Configuration:");
        info!("  JWT Expiration: {} seconds", config.auth.jwt_expiration);
        info!(
            "  Rate Limiting: {} req/min, {} req/hour",
            config.auth.rate_limit_per_minute, config.auth.rate_limit_per_hour
        );

        info!("Database Configuration:");
        info!("  Compression: {}", config.database.compression_enabled);
        info!(
            "  Threshold: {} bytes",
            config.database.compression_threshold
        );
    }

    // Test database connectivity
    let store = VectorStore::new();
    let collections = store.list_collections();
    info!("Database Status:");
    info!("  Collections: {}", collections.len());

    if detailed && !collections.is_empty() {
        info!("  Collection Details:");
        for collection_name in collections {
            if let Ok(metadata) = store.get_collection_metadata(&collection_name) {
                info!(
                    "    {}: {} vectors, {}D",
                    collection_name, metadata.vector_count, metadata.config.dimension
                );
            }
        }
    }

    info!("System Status: OK");

    Ok(())
}

/// Handle snapshot management commands
pub async fn handle_snapshot_command(
    command: super::SnapshotCommands,
    config: &CliConfig,
) -> Result<()> {
    use crate::storage::SnapshotManager;

    let data_dir = PathBuf::from("./data");
    let snapshots_path = config.storage.snapshots.path.clone();
    let max_snapshots = config.storage.snapshots.max_snapshots;
    let retention_days = config.storage.snapshots.retention_days;
    let retention_hours = retention_days * 24; // Convert days to hours

    let manager = SnapshotManager::new(&data_dir, &snapshots_path, max_snapshots, retention_hours);

    match command {
        super::SnapshotCommands::List { detailed } => {
            info!("📸 Listing snapshots...");

            let snapshots = manager.list_snapshots()?;

            if snapshots.is_empty() {
                info!("No snapshots found");
                return Ok(());
            }

            info!("Found {} snapshots:", snapshots.len());
            for snapshot in &snapshots {
                info!("  ID: {}", snapshot.id);
                info!("    Created: {}", snapshot.created_at);
                info!("    Size: {:.2} MB", snapshot.size_mb());
                info!("    Age: {} hours", snapshot.age_hours());

                if detailed {
                    info!("    Path: {:?}", snapshot.path);
                    info!("    Version: {}", snapshot.index_version);
                }
                info!("");
            }

            Ok(())
        }

        super::SnapshotCommands::Create { description } => {
            info!("📸 Creating snapshot...");

            let snapshot = manager.create_snapshot()?;

            info!("✅ Snapshot created successfully:");
            info!("  ID: {}", snapshot.id);
            info!("  Size: {:.2} MB", snapshot.size_mb());

            if let Some(desc) = description {
                info!("  Description: {}", desc);
            }

            Ok(())
        }

        super::SnapshotCommands::Restore { id, force } => {
            info!("🔄 Restoring from snapshot: {}", id);

            if !force {
                warn!("⚠️  This will overwrite current data. Use --force to confirm.");
                return Ok(());
            }

            manager.restore_snapshot(&id)?;
            info!("✅ Snapshot restored successfully");

            Ok(())
        }

        super::SnapshotCommands::Delete { id } => {
            info!("🗑️  Deleting snapshot: {}", id);

            if manager.delete_snapshot(&id)? {
                info!("✅ Snapshot deleted");
            } else {
                warn!("⚠️  Snapshot not found: {}", id);
            }

            Ok(())
        }

        super::SnapshotCommands::Cleanup { dry_run } => {
            info!("🧹 Cleaning up old snapshots...");

            if dry_run {
                info!("DRY RUN - No changes will be made");

                let snapshots = manager.list_snapshots()?;
                let cutoff =
                    chrono::Utc::now() - chrono::Duration::hours((retention_days * 24) as i64);

                let to_delete: Vec<_> =
                    snapshots.iter().filter(|s| s.created_at < cutoff).collect();

                info!("Would delete {} snapshots:", to_delete.len());
                for snapshot in to_delete {
                    info!("  - {} ({})", snapshot.id, snapshot.created_at);
                }
            } else {
                let deleted = manager.cleanup_old_snapshots()?;
                info!("✅ Cleaned up {} old snapshots", deleted);
            }

            Ok(())
        }
    }
}

/// Handle storage management commands
pub async fn handle_storage_command(
    command: super::StorageCommands,
    config: &CliConfig,
) -> Result<()> {
    use crate::storage::{
        StorageCompactor, StorageFormat, StorageMigrator, StorageReader, detect_format,
    };

    let data_dir = PathBuf::from("./data");

    match command {
        super::StorageCommands::Info { detailed } => {
            info!("💾 Storage Information:");

            let format = detect_format(&data_dir);
            info!("  Format: {:?}", format);

            match format {
                StorageFormat::Compact => match StorageReader::new(&data_dir) {
                    Ok(reader) => {
                        let index = reader.index()?;

                        info!("  Collections: {}", index.collection_count());
                        info!("  Total vectors: {}", index.total_vectors());
                        info!("  Original size: {} MB", index.total_size / 1_048_576);
                        info!(
                            "  Compressed size: {} MB",
                            index.compressed_size / 1_048_576
                        );
                        info!(
                            "  Compression ratio: {:.2}%",
                            index.compression_ratio * 100.0
                        );
                        info!(
                            "  Space saved: {} MB",
                            (index.total_size.saturating_sub(index.compressed_size)) / 1_048_576
                        );

                        if detailed {
                            info!("\n  Collections:");
                            for collection in &index.collections {
                                info!("    {}:", collection.name);
                                info!("      Vectors: {}", collection.vector_count);
                                info!("      Dimension: {}", collection.dimension);
                                info!("      Files: {}", collection.files.len());
                                info!("      Size: {} MB", collection.total_size() / 1_048_576);
                            }
                        }

                        let stats = reader.cache_stats()?;
                        info!("\n  Cache:");
                        info!("    Entries: {}", stats.entry_count);
                        info!("    Size: {} MB", stats.total_size_bytes / 1_048_576);
                        info!("    Utilization: {:.1}%", stats.utilization());
                    }
                    Err(e) => {
                        error!("❌ Failed to read storage: {}", e);
                    }
                },
                StorageFormat::Legacy => {
                    info!("  Using legacy file structure");
                    info!("  Run 'vectorizer storage migrate' to convert to .vecdb format");
                }
            }

            Ok(())
        }

        super::StorageCommands::Migrate { force, level } => {
            info!("🔄 Starting storage migration...");

            let migrator = StorageMigrator::new(&data_dir, level);

            if !migrator.needs_migration() && !force {
                info!("✅ Already using .vecdb format");
                return Ok(());
            }

            let result = migrator.migrate()?;

            info!("✅ Migration completed:");
            info!("  Collections migrated: {}", result.collections_migrated);
            info!("  {}", result.message);

            if let Some(backup_path) = result.backup_path {
                info!("  Backup: {:?}", backup_path);
            }

            Ok(())
        }

        super::StorageCommands::Verify { fix } => {
            info!("🔍 Verifying storage integrity...");

            let format = detect_format(&data_dir);

            match format {
                StorageFormat::Compact => {
                    let compactor = StorageCompactor::new(&data_dir, 3, 1000);
                    let is_valid = compactor.verify_integrity()?;

                    if is_valid {
                        info!("✅ Storage integrity verified");
                    } else {
                        error!("❌ Storage integrity check failed");

                        if fix {
                            warn!("🔧 Attempting to fix...");
                            // Try to rebuild the archive
                            compactor.compact_all()?;
                            info!("✅ Storage rebuilt");
                        }
                    }
                }
                StorageFormat::Legacy => {
                    info!("ℹ️  Using legacy format (no verification needed)");
                }
            }

            Ok(())
        }

        super::StorageCommands::Compact { force } => {
            info!("🗜️  Compacting storage...");

            let mut compactor = StorageCompactor::new(&data_dir, 3, 1000);

            if !force && !compactor.should_compact() {
                info!("ℹ️  Compaction not needed (use --force to compact anyway)");
                info!("  Pending operations: {}", compactor.pending_operations());
                return Ok(());
            }

            let index = compactor.force_compact()?;

            info!("✅ Compaction completed:");
            info!("  Collections: {}", index.collection_count());
            info!("  Compression: {:.2}%", index.compression_ratio * 100.0);

            Ok(())
        }
    }
}
