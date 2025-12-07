mod auth_handlers;
mod discovery_handlers;
mod error_middleware;
pub mod file_operations_handlers;
mod file_upload_handlers;
mod file_validation;
mod graph_handlers;
mod graphql_handlers;
mod hub_backup_handlers;
// mod hub_tenant_handlers; // TODO: Fix type errors before enabling
mod hub_usage_handlers;
pub mod mcp_handlers;
pub mod mcp_tools;
mod qdrant_alias_handlers;
mod qdrant_cluster_handlers;
mod qdrant_handlers;
mod qdrant_query_handlers;
mod qdrant_search_handlers;
mod qdrant_sharding_handlers;
mod qdrant_snapshot_handlers;
mod qdrant_vector_handlers;
pub mod replication_handlers;
pub mod rest_handlers;

// Re-export main server types from the original implementation
use std::sync::Arc;

pub use auth_handlers::{
    AuthHandlerState, UserRecord, auth_middleware, require_admin_middleware,
    require_auth_middleware,
};
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::{delete, get, post, put};
pub use mcp_handlers::handle_mcp_tool;
pub use mcp_tools::get_mcp_tools;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::{debug, error, info, warn};

use crate::file_watcher::{FileWatcherMetrics, FileWatcherSystem, MetricsCollector};

/// Global server state to share between endpoints
#[derive(Clone)]
pub struct ServerState {
    pub file_watcher_system: Arc<tokio::sync::Mutex<Option<FileWatcherSystem>>>,
}

use crate::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::workspace::{WorkspaceConfig, WorkspaceManager};

/// Vectorizer server state
#[derive(Clone)]
pub struct VectorizerServer {
    pub store: Arc<VectorStore>,
    pub embedding_manager: Arc<EmbeddingManager>,
    pub start_time: std::time::Instant,
    pub file_watcher_system:
        Arc<tokio::sync::Mutex<Option<crate::file_watcher::FileWatcherSystem>>>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub auto_save_manager: Option<Arc<crate::db::AutoSaveManager>>,
    pub master_node: Option<Arc<crate::replication::MasterNode>>,
    pub replica_node: Option<Arc<crate::replication::ReplicaNode>>,
    pub query_cache: Arc<crate::cache::query_cache::QueryCache<serde_json::Value>>,
    background_task: Arc<
        tokio::sync::Mutex<
            Option<(
                tokio::task::JoinHandle<()>,
                tokio::sync::watch::Sender<bool>,
            )>,
        >,
    >,
    system_collector_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    file_watcher_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    file_watcher_cancel: Arc<tokio::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>>,
    grpc_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    auto_save_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Cluster manager (optional, only if cluster is enabled)
    pub cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
    /// Cluster client pool (optional, only if cluster is enabled)
    pub cluster_client_pool: Option<Arc<crate::cluster::ClusterClientPool>>,
    /// Maximum request body size in MB (from config)
    pub max_request_size_mb: usize,
    /// Snapshot manager (optional, for Qdrant snapshot API)
    pub snapshot_manager: Option<Arc<crate::storage::SnapshotManager>>,
    /// Authentication handler state (optional, only if auth is enabled)
    pub auth_handler_state: Option<AuthHandlerState>,
    /// HiveHub manager (optional, only if hub integration is enabled)
    pub hub_manager: Option<Arc<crate::hub::HubManager>>,
    /// User backup manager (optional, only if hub integration is enabled)
    pub backup_manager: Option<Arc<crate::hub::UserBackupManager>>,
    /// MCP Hub Gateway for multi-tenant MCP operations
    pub mcp_hub_gateway: Option<Arc<crate::hub::McpHubGateway>>,
}

/// Configuration for root user credentials
#[derive(Debug, Clone, Default)]
pub struct RootUserConfig {
    /// Root username (defaults to "root" if not set)
    pub root_user: Option<String>,
    /// Root password (generates random if not set)
    pub root_password: Option<String>,
}

impl VectorizerServer {
    /// Create a new vectorizer server
    pub async fn new() -> anyhow::Result<Self> {
        Self::new_with_root_config(RootUserConfig::default()).await
    }

    /// Create a new vectorizer server with root user configuration
    pub async fn new_with_root_config(root_config: RootUserConfig) -> anyhow::Result<Self> {
        info!("🔧 Initializing Vectorizer Server...");

        // Initialize monitoring system
        if let Err(e) = crate::monitoring::init() {
            warn!("Failed to initialize monitoring system: {}", e);
        }

        // Try to initialize OpenTelemetry (optional, graceful degradation)
        if let Err(e) = crate::monitoring::telemetry::try_init("vectorizer", None) {
            warn!("OpenTelemetry not available: {}", e);
        }

        // Initialize VectorStore with auto-save enabled
        let vector_store = VectorStore::new_auto();
        let store_arc = Arc::new(vector_store);

        // Check if we should cleanup empty collections on startup
        let should_cleanup = std::fs::read_to_string("config.yml")
            .ok()
            .and_then(|content| {
                serde_yaml::from_str::<crate::config::VectorizerConfig>(&content).ok()
            })
            .map(|config| config.server.startup_cleanup_empty)
            .unwrap_or(false);

        if should_cleanup {
            info!("🧹 Running startup cleanup of empty collections...");
            match store_arc.cleanup_empty_collections(false) {
                Ok(count) => {
                    if count > 0 {
                        info!("✅ Cleaned up {} empty collections on startup", count);
                    } else {
                        info!("✨ No empty collections found to cleanup");
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to cleanup empty collections on startup: {}", e);
                }
            }
        }

        info!("🔍 PRE_INIT: Creating embedding manager...");
        let mut embedding_manager = EmbeddingManager::new();
        info!("🔍 PRE_INIT: Creating BM25 embedding...");
        let bm25 = crate::embedding::Bm25Embedding::new(512);
        info!("🔍 PRE_INIT: Registering BM25 provider...");
        embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
        info!("🔍 PRE_INIT: Setting default provider...");
        embedding_manager.set_default_provider("bm25")?;
        info!("✅ PRE_INIT: Embedding manager configured");

        info!(
            "✅ Vectorizer Server initialized successfully - starting background collection loading"
        );
        info!("🔍 STEP 1: Server initialization completed, proceeding to file watcher setup");
        info!("🔍 STEP 1.1: About to initialize file watcher embedding manager...");

        // Initialize file watcher if enabled
        info!("🔍 STEP 2: Initializing file watcher embedding manager...");
        let mut embedding_manager_for_watcher = EmbeddingManager::new();
        let bm25_for_watcher = crate::embedding::Bm25Embedding::new(512);
        embedding_manager_for_watcher
            .register_provider("bm25".to_string(), Box::new(bm25_for_watcher));
        embedding_manager_for_watcher.set_default_provider("bm25")?;
        info!("✅ STEP 2: File watcher embedding manager initialized");

        info!("🔍 STEP 3: Creating Arc wrappers for file watcher components...");
        let embedding_manager_for_watcher_arc =
            Arc::new(RwLock::new(embedding_manager_for_watcher));
        let file_watcher_arc = embedding_manager_for_watcher_arc.clone();
        let store_for_watcher = store_arc.clone();
        info!("✅ STEP 3: Arc wrappers created successfully");

        info!("🔍 STEP 4: Checking if file watcher is enabled...");

        // Load cluster config for file watcher check
        let cluster_config_for_watcher = std::fs::read_to_string("config.yml")
            .ok()
            .and_then(|content| {
                serde_yaml::from_str::<crate::config::VectorizerConfig>(&content)
                    .ok()
                    .map(|config| config.cluster)
            })
            .unwrap_or_default();

        // Check if file watcher is enabled in config before starting
        // Also check if cluster mode requires file watcher to be disabled
        let file_watcher_enabled = std::fs::read_to_string("config.yml")
            .ok()
            .and_then(|content| serde_yaml::from_str::<serde_yaml::Value>(&content).ok())
            .and_then(|config| {
                config
                    .get("file_watcher")
                    .and_then(|fw| fw.get("enabled"))
                    .and_then(|enabled| enabled.as_bool())
            })
            .unwrap_or(false); // Default to disabled if not found

        // Disable file watcher if cluster mode is enabled and requires it
        let file_watcher_enabled = if cluster_config_for_watcher.enabled
            && cluster_config_for_watcher.memory.disable_file_watcher
        {
            if file_watcher_enabled {
                warn!(
                    "⚠️  File watcher is DISABLED because cluster mode is enabled with disable_file_watcher=true"
                );
            }
            false
        } else {
            file_watcher_enabled
        };

        let watcher_system_arc = Arc::new(tokio::sync::Mutex::new(
            None::<crate::file_watcher::FileWatcherSystem>,
        ));
        let watcher_system_for_task = watcher_system_arc.clone();
        let watcher_system_for_server = watcher_system_arc.clone();

        // Create cancellation token for file watcher
        let (file_watcher_cancel_tx, mut file_watcher_cancel_rx) =
            tokio::sync::watch::channel(false);
        let file_watcher_task_handle = if file_watcher_enabled {
            info!("✅ File watcher is ENABLED in config - starting...");
            let handle = tokio::task::spawn(async move {
                info!("🔍 STEP 4: Inside file watcher task - starting file watcher system...");
                info!("🔍 STEP 5: Creating FileWatcherSystem instance...");

                // Load file watcher configuration from workspace
                let watcher_config = load_file_watcher_config().await.unwrap_or_else(|e| {
                    warn!("Failed to load file watcher config: {}, using defaults", e);
                    crate::file_watcher::FileWatcherConfig::default()
                });

                let mut watcher_system = crate::file_watcher::FileWatcherSystem::new(
                    watcher_config,
                    store_for_watcher,
                    file_watcher_arc,
                );
                info!("✅ STEP 5: FileWatcherSystem instance created");

                info!("🔍 STEP 5.1: Initializing file discovery system...");
                if let Err(e) = watcher_system.initialize_discovery() {
                    error!("Failed to initialize file discovery system: {}", e);
                } else {
                    info!("✅ STEP 5.1: File discovery system initialized");
                }

                info!("🔍 STEP 6: Starting FileWatcherSystem...");
                if let Err(e) = watcher_system.start().await {
                    error!("❌ STEP 6: Failed to start file watcher: {}", e);
                } else {
                    info!("✅ STEP 6: File watcher started successfully");
                }

                // Store the watcher system for later use AFTER starting it
                {
                    let mut watcher_guard = watcher_system_for_task.lock().await;
                    *watcher_guard = Some(watcher_system);
                }

                info!("🔍 STEP 7: File watcher system is now running in background...");

                // Keep the task alive but check for cancellation
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                            // Check if cancelled
                            if *file_watcher_cancel_rx.borrow() {
                                info!("🛑 File watcher task received cancellation signal");
                                break;
                            }
                            debug!("🔍 File watcher is still running...");
                        }
                        _ = file_watcher_cancel_rx.changed() => {
                            if *file_watcher_cancel_rx.borrow() {
                                info!("🛑 File watcher task received cancellation signal");
                                break;
                            }
                        }
                    }
                }

                info!("✅ File watcher task completed");
            });
            Some(handle)
        } else {
            info!("⏭️  File watcher is DISABLED in config - skipping initialization");
            None
        };

        // Create cancellation token for background task
        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);

        // Start background collection loading and workspace indexing
        let store_for_loading = store_arc.clone();
        let embedding_manager_for_loading = Arc::new(embedding_manager);
        let watcher_system_for_loading = watcher_system_arc.clone();
        let background_handle = tokio::task::spawn(async move {
            info!("📦 Background task started - loading collections and checking workspace...");

            // Check for cancellation before starting
            if *cancel_rx.borrow() {
                info!("Background task cancelled before start");
                return;
            }

            // Check if vectorizer.vecdb exists - if so, ALWAYS load it
            let data_dir = VectorStore::get_data_dir();
            let vecdb_path = data_dir.join("vectorizer.vecdb");
            let vecdb_exists = vecdb_path.exists();

            // Load all persisted collections if .vecdb exists (ALWAYS, regardless of config)
            // OR if auto_load is explicitly enabled for raw files
            let should_auto_load = if vecdb_exists {
                info!("📦 vectorizer.vecdb exists - will ALWAYS load collections from it");
                true
            } else {
                // No .vecdb - check config for raw file loading
                std::fs::read_to_string("config.yml")
                    .ok()
                    .and_then(|content| serde_yaml::from_str::<serde_yaml::Value>(&content).ok())
                    .and_then(|config| {
                        config
                            .get("workspace")
                            .and_then(|ws| ws.get("auto_load_collections"))
                            .and_then(|enabled| enabled.as_bool())
                    })
                    .unwrap_or(false)
            };

            // Load all persisted collections in background
            let persisted_count = if should_auto_load {
                info!(
                    "🔍 COLLECTION_LOAD_STEP_1: Auto-load ENABLED - loading all persisted collections..."
                );
                match store_for_loading.load_all_persisted_collections() {
                    Ok(count) => {
                        if count > 0 {
                            info!(
                                "✅ COLLECTION_LOAD_STEP_2: Background loading completed - {} collections loaded",
                                count
                            );

                            // Update file watcher with loaded collections
                            info!(
                                "🔍 COLLECTION_LOAD_STEP_3: Updating file watcher with loaded collections..."
                            );
                            if let Some(watcher_system) =
                                watcher_system_for_loading.lock().await.as_ref()
                            {
                                let collections = store_for_loading.list_collections();
                                for collection_name in collections {
                                    if let Err(e) = watcher_system
                                        .update_with_collection(&collection_name)
                                        .await
                                    {
                                        warn!(
                                            "⚠️ Failed to update file watcher with collection '{}': {}",
                                            collection_name, e
                                        );
                                    } else {
                                        info!(
                                            "✅ Updated file watcher with collection: {}",
                                            collection_name
                                        );
                                    }
                                }

                                // Discover and index existing files after collections are loaded
                                info!(
                                    "🔍 COLLECTION_LOAD_STEP_4: Starting file discovery for existing files..."
                                );
                                match watcher_system.discover_existing_files().await {
                                    Ok(result) => {
                                        info!(
                                            "✅ File discovery completed: {} files indexed, {} skipped, {} errors",
                                            result.stats.files_indexed,
                                            result.stats.files_skipped,
                                            result.stats.files_errors
                                        );
                                    }
                                    Err(e) => {
                                        warn!("⚠️ File discovery failed: {}", e);
                                    }
                                }

                                // Sync with collections to remove orphaned files
                                info!("🔍 COLLECTION_LOAD_STEP_5: Starting collection sync...");
                                match watcher_system.sync_with_collections().await {
                                    Ok(result) => {
                                        info!(
                                            "✅ Collection sync completed: {} orphaned files removed",
                                            result.stats.orphaned_files_removed
                                        );
                                    }
                                    Err(e) => {
                                        warn!("⚠️ Collection sync failed: {}", e);
                                    }
                                }
                            } else {
                                debug!("⚠️ File watcher not available for update");
                            }

                            count
                        } else {
                            info!(
                                "✅ COLLECTION_LOAD_STEP_2: Background loading completed - no persisted collections found"
                            );

                            // Even with no persisted collections, try to discover existing files
                            info!(
                                "🔍 COLLECTION_LOAD_STEP_3: No persisted collections, attempting conservative file discovery..."
                            );

                            // Wait for file watcher to be available (with timeout)
                            let mut attempts = 0;
                            let max_attempts = 10; // Conservative timeout

                            loop {
                                if let Some(watcher_system) =
                                    watcher_system_for_loading.lock().await.as_ref()
                                {
                                    info!(
                                        "🔍 COLLECTION_LOAD_STEP_4: Starting conservative file discovery..."
                                    );
                                    match watcher_system.discover_existing_files().await {
                                        Ok(result) => {
                                            info!(
                                                "✅ File discovery completed: {} files indexed, {} skipped, {} errors",
                                                result.stats.files_indexed,
                                                result.stats.files_skipped,
                                                result.stats.files_errors
                                            );
                                        }
                                        Err(e) => {
                                            warn!("⚠️ File discovery failed: {}", e);
                                        }
                                    }

                                    // Perform comprehensive synchronization
                                    info!(
                                        "🔍 COLLECTION_LOAD_STEP_5: Starting comprehensive synchronization..."
                                    );
                                    let sync_start = std::time::Instant::now();
                                    match watcher_system.comprehensive_sync().await {
                                        Ok((sync_result, unindexed_files)) => {
                                            let sync_time_ms =
                                                sync_start.elapsed().as_millis() as u64;

                                            // Record sync metrics
                                            watcher_system
                                                .record_sync(
                                                    sync_result.stats.orphaned_files_removed as u64,
                                                    unindexed_files.len() as u64,
                                                    sync_time_ms,
                                                )
                                                .await;

                                            info!(
                                                "✅ Comprehensive sync completed: {} orphaned files removed, {} unindexed files detected",
                                                sync_result.stats.orphaned_files_removed,
                                                unindexed_files.len()
                                            );

                                            if !unindexed_files.is_empty() {
                                                info!(
                                                    "📄 Unindexed files detected: {:?}",
                                                    unindexed_files
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            warn!("⚠️ Comprehensive sync failed: {}", e);
                                            watcher_system
                                                .record_error("sync_error", &e.to_string())
                                                .await;
                                        }
                                    }

                                    break;
                                } else {
                                    attempts += 1;
                                    if attempts >= max_attempts {
                                        debug!(
                                            "⚠️ File watcher not available after {} seconds, skipping discovery",
                                            max_attempts
                                        );
                                        break;
                                    }
                                    info!(
                                        "⏳ Waiting for file watcher to be available... (attempt {}/{})",
                                        attempts, max_attempts
                                    );
                                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                }
                            }

                            0
                        }
                    }
                    Err(e) => {
                        warn!(
                            "⚠️  Failed to load persisted collections in background: {}",
                            e
                        );
                        0
                    }
                }
            } else {
                info!(
                    "⏭️  Auto-load DISABLED - collections will be loaded on first access (lazy loading)"
                );
                0
            };

            // Check for cancellation before workspace loading
            if *cancel_rx.borrow() {
                info!("Background task cancelled before workspace loading");
                return;
            }

            // Check for workspace configuration and reindex if needed
            // (data_dir and vecdb_path already declared above)

            match load_workspace_collections(
                &store_for_loading,
                &embedding_manager_for_loading,
                cancel_rx.clone(),
            )
            .await
            {
                Ok(workspace_count) => {
                    if workspace_count > 0 {
                        info!(
                            "✅ Workspace loading completed - {} collections indexed/loaded",
                            workspace_count
                        );

                        // Check if there are .bin files created during indexing
                        use crate::storage::StorageCompactor;
                        let compactor = StorageCompactor::new(&data_dir, 6, 1000);

                        // Count .bin files to see if we need to compact
                        let bin_count = std::fs::read_dir(&data_dir)
                            .ok()
                            .map(|entries| {
                                entries
                                    .filter_map(|e| e.ok())
                                    .filter(|e| {
                                        e.path().extension().and_then(|s| s.to_str()) == Some("bin")
                                    })
                                    .count()
                            })
                            .unwrap_or(0);

                        if bin_count > 0 {
                            info!(
                                "📦 Found {} .bin files - compacting to vectorizer.vecdb from memory...",
                                bin_count
                            );
                            info!(
                                "🔍 DEBUG: bin_count = {}, workspace_count = {}",
                                bin_count, workspace_count
                            );
                            info!("🔍 DEBUG: data_dir = {}", data_dir.display());

                            info!("🔍 DEBUG: Starting compact_from_memory...");

                            // Compact directly FROM MEMORY (no raw files needed)
                            match compactor.compact_from_memory(&store_for_loading) {
                                Ok(index) => {
                                    info!(
                                        "✅ First compaction complete - created vectorizer.vecdb from memory"
                                    );
                                    info!(
                                        "   Collections: {}, Vectors: {}",
                                        index.collection_count(),
                                        index.total_vectors()
                                    );
                                    info!("   Only vectorizer.vecdb and vectorizer.vecidx exist");

                                    // Verify the file was created
                                    if vecdb_path.exists() {
                                        let metadata = std::fs::metadata(&vecdb_path).unwrap();
                                        info!(
                                            "   📊 vectorizer.vecdb size: {} bytes",
                                            metadata.len()
                                        );
                                    } else {
                                        error!("❌ CRITICAL: vectorizer.vecdb was NOT created!");
                                    }

                                    // Remove any temporary .bin files that might have been created during indexing
                                    match compactor.remove_raw_files() {
                                        Ok(count) if count > 0 => {
                                            info!("🗑️  Removed {} temporary raw files", count);
                                        }
                                        Ok(_) => {
                                            info!("   No temporary raw files to remove");
                                        }
                                        Err(e) => {
                                            warn!("⚠️  Failed to remove raw files: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("❌ Compaction from memory failed: {}", e);
                                    error!("   Error details: {:?}", e);
                                    error!("   Will retry on next startup");
                                }
                            }
                        } else {
                            // No .bin files - either loaded from .vecdb or nothing to compact
                            info!("ℹ️  No .bin files found - vectorizer.vecdb is up to date");
                        }
                    } else {
                        info!("ℹ️  All collections already exist - no indexing needed");
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to process workspace: {}", e);
                }
            }

            // NOW enable auto-save after all collections are loaded
            info!("🔄 Enabling auto-save after successful initialization");
            store_for_loading.enable_auto_save();
            info!("✅ Auto-save enabled - collections will be saved every 5 minutes when modified");
        });

        // Create final embedding manager for the server struct
        let mut final_embedding_manager = EmbeddingManager::new();
        let final_bm25 = crate::embedding::Bm25Embedding::new(512);
        final_embedding_manager.register_provider("bm25".to_string(), Box::new(final_bm25));
        final_embedding_manager.set_default_provider("bm25")?;

        // Initialize AutoSaveManager (5min save + 1h snapshot intervals)
        info!("🔄 Initializing AutoSaveManager...");
        let auto_save_manager = Arc::new(crate::db::AutoSaveManager::new(store_arc.clone(), 1));

        // Clean up old snapshots on server startup
        info!("🧹 Cleaning up old snapshots on server startup...");
        match auto_save_manager.cleanup_old_snapshots() {
            Ok(deleted) => {
                if deleted > 0 {
                    info!(
                        "✅ Cleaned up {} old snapshots (retention: 48 hours)",
                        deleted
                    );
                } else {
                    info!("✅ No old snapshots to clean up");
                }
            }
            Err(e) => {
                warn!("⚠️  Failed to clean up old snapshots on startup: {}", e);
            }
        }

        let auto_save_handle = auto_save_manager.start();
        info!("✅ AutoSaveManager started (5min save + 1h snapshot intervals)");

        // Start system metrics collector
        info!("📊 Starting system metrics collector...");
        let system_collector = crate::monitoring::SystemCollector::new(store_arc.clone());
        let system_collector_handle = system_collector.start();
        info!("✅ System metrics collector started");

        // Initialize query cache
        info!("💾 Initializing query cache...");
        let cache_config = crate::cache::query_cache::QueryCacheConfig::default();
        let max_size = cache_config.max_size;
        let ttl_seconds = cache_config.ttl_seconds;
        let query_cache = Arc::new(crate::cache::query_cache::QueryCache::new(cache_config));
        info!(
            "✅ Query cache initialized (max_size: {}, ttl: {}s)",
            max_size, ttl_seconds
        );

        // Initialize cluster manager if cluster is enabled
        let (cluster_manager, cluster_client_pool, cluster_config_ref) = {
            // Try to load cluster config from config.yml or use default
            let cluster_config = std::fs::read_to_string("config.yml")
                .ok()
                .and_then(|content| {
                    serde_yaml::from_str::<crate::config::VectorizerConfig>(&content)
                        .ok()
                        .map(|config| config.cluster)
                })
                .unwrap_or_default();

            if cluster_config.enabled {
                info!("🔗 Initializing cluster manager...");

                // Validate cluster configuration
                let validator = crate::cluster::ClusterConfigValidator::new();

                // Also load file watcher config for validation
                let file_watcher_config = std::fs::read_to_string("config.yml")
                    .ok()
                    .and_then(|content| {
                        serde_yaml::from_str::<crate::config::VectorizerConfig>(&content)
                            .ok()
                            .map(|config| config.file_watcher)
                    })
                    .unwrap_or_default();

                let validation_result =
                    validator.validate_with_file_watcher(&cluster_config, &file_watcher_config);

                // Log validation warnings
                if validation_result.has_warnings() {
                    warn!("{}", validation_result.warning_message());
                }

                // Check for validation errors
                if validation_result.has_errors() {
                    if cluster_config.memory.strict_validation {
                        error!("{}", validation_result.error_message());
                        panic!(
                            "Cluster configuration validation failed. Fix the errors or set cluster.memory.strict_validation = false to continue with warnings."
                        );
                    } else {
                        warn!(
                            "Cluster configuration has errors (strict_validation=false, continuing anyway):"
                        );
                        warn!("{}", validation_result.error_message());
                    }
                }

                // Log cluster memory configuration
                info!(
                    "📊 Cluster memory config: max_cache={} MB, enforce_mmap={}, disable_file_watcher={}",
                    cluster_config.memory.max_cache_memory_bytes / (1024 * 1024),
                    cluster_config.memory.enforce_mmap_storage,
                    cluster_config.memory.disable_file_watcher
                );

                match crate::cluster::ClusterManager::new(cluster_config.clone()) {
                    Ok(manager) => {
                        let manager_arc = Arc::new(manager);
                        let timeout = std::time::Duration::from_millis(cluster_config.timeout_ms);
                        let client_pool = Arc::new(crate::cluster::ClusterClientPool::new(timeout));

                        info!("✅ Cluster manager initialized");
                        (Some(manager_arc), Some(client_pool), Some(cluster_config))
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to initialize cluster manager: {}", e);
                        (None, None, None)
                    }
                }
            } else {
                info!("ℹ️  Cluster mode disabled");
                (None, None, None)
            }
        };

        // Store cluster config for later use (e.g., storage type enforcement)
        let _cluster_config = cluster_config_ref;

        // Load API config for max request size
        let max_request_size_mb = std::fs::read_to_string("config.yml")
            .ok()
            .and_then(|content| {
                serde_yaml::from_str::<serde_yaml::Value>(&content)
                    .ok()
                    .and_then(|config| {
                        config
                            .get("api")
                            .and_then(|api| api.get("rest"))
                            .and_then(|rest| rest.get("max_request_size_mb"))
                            .and_then(|size| size.as_u64())
                            .map(|size| size as usize)
                    })
            })
            .unwrap_or(100); // Default to 100MB if not configured

        info!("📦 API max request size: {}MB", max_request_size_mb);

        // Initialize auth handler state if auth is enabled
        let auth_handler_state = {
            // Try to load auth config from config.yml
            let auth_config = std::fs::read_to_string("config.yml")
                .ok()
                .and_then(|content| {
                    serde_yaml::from_str::<crate::config::VectorizerConfig>(&content)
                        .ok()
                        .map(|config| config.auth)
                })
                .unwrap_or_default();

            if auth_config.enabled {
                info!("🔐 Initializing authentication system...");
                match crate::auth::AuthManager::new(auth_config) {
                    Ok(auth_manager) => {
                        let auth_manager_arc = Arc::new(auth_manager);
                        // Create auth handler state with root user configuration
                        let handler_state = AuthHandlerState::new_with_root_user(
                            auth_manager_arc,
                            root_config.root_user.clone(),
                            root_config.root_password.clone(),
                        )
                        .await;
                        info!("✅ Authentication system initialized");
                        Some(handler_state)
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to initialize authentication: {}", e);
                        None
                    }
                }
            } else {
                info!("ℹ️  Authentication disabled");
                None
            }
        };

        // Initialize HiveHub manager if hub integration is enabled
        let hub_manager = {
            // Try to load hub config from config.yml
            let hub_config = match std::fs::read_to_string("config.yml") {
                Ok(content) => {
                    match serde_yaml::from_str::<crate::config::VectorizerConfig>(&content) {
                        Ok(config) => {
                            info!(
                                "✅ Loaded hub config from config.yml: enabled={}",
                                config.hub.enabled
                            );
                            config.hub
                        }
                        Err(e) => {
                            warn!("⚠️  Failed to parse config.yml for hub config: {}", e);
                            crate::hub::HubConfig::default()
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to read config.yml for hub config: {}", e);
                    crate::hub::HubConfig::default()
                }
            };

            if hub_config.enabled {
                info!("🌐 Initializing HiveHub integration...");
                match crate::hub::HubManager::new(hub_config).await {
                    Ok(manager) => {
                        let manager_arc = Arc::new(manager);
                        // Start the usage reporter background task
                        if let Err(e) = manager_arc.start().await {
                            warn!("⚠️  Failed to start HiveHub usage reporter: {}", e);
                        }
                        info!("✅ HiveHub integration initialized");
                        Some(manager_arc)
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to initialize HiveHub integration: {}", e);
                        None
                    }
                }
            } else {
                info!("ℹ️  HiveHub integration disabled");
                None
            }
        };

        // Initialize user backup manager if hub integration is enabled
        let backup_manager = if hub_manager.is_some() {
            info!("📦 Initializing HiveHub backup manager...");
            let backup_config = crate::hub::BackupConfig::default();
            match crate::hub::UserBackupManager::new(backup_config, store_arc.clone()) {
                Ok(manager) => {
                    info!("✅ HiveHub backup manager initialized");
                    Some(Arc::new(manager))
                }
                Err(e) => {
                    warn!("⚠️  Failed to initialize backup manager: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize MCP Hub Gateway if hub integration is enabled
        let mcp_hub_gateway = if let Some(ref hub_mgr) = hub_manager {
            info!("🔌 Initializing MCP Hub Gateway...");
            let gateway = crate::hub::McpHubGateway::new(hub_mgr.clone());
            info!("✅ MCP Hub Gateway initialized");
            Some(Arc::new(gateway))
        } else {
            None
        };

        Ok(Self {
            store: store_arc,
            embedding_manager: Arc::new(final_embedding_manager),
            start_time: std::time::Instant::now(),
            file_watcher_system: watcher_system_for_server,
            metrics_collector: Arc::new(MetricsCollector::new()),
            auto_save_manager: Some(auto_save_manager),
            master_node: None,
            replica_node: None,
            query_cache,
            background_task: Arc::new(tokio::sync::Mutex::new(Some((
                background_handle,
                cancel_tx,
            )))),
            system_collector_task: Arc::new(tokio::sync::Mutex::new(Some(system_collector_handle))),
            file_watcher_task: Arc::new(tokio::sync::Mutex::new(file_watcher_task_handle)),
            file_watcher_cancel: Arc::new(tokio::sync::Mutex::new(Some(file_watcher_cancel_tx))),
            grpc_task: Arc::new(tokio::sync::Mutex::new(None)),
            auto_save_task: Arc::new(tokio::sync::Mutex::new(Some(auto_save_handle))),
            cluster_manager,
            cluster_client_pool,
            max_request_size_mb,
            snapshot_manager: {
                let data_dir = VectorStore::get_data_dir();
                let snapshots_dir = data_dir.join("snapshots");
                Some(Arc::new(crate::storage::SnapshotManager::new(
                    &data_dir,
                    &snapshots_dir,
                    10,  // max_snapshots: keep up to 10 snapshots
                    168, // retention_hours: 7 days
                )))
            },
            auth_handler_state,
            hub_manager,
            backup_manager,
            mcp_hub_gateway,
        })
    }

    /// Check if authentication should be required based on host binding
    /// Returns true if host is 0.0.0.0 (production mode) and auth is not enabled
    fn should_require_auth(host: &str, auth_enabled: bool) -> bool {
        let is_production_bind = host == "0.0.0.0";
        is_production_bind && !auth_enabled
    }

    /// Start the server
    pub async fn start(&self, host: &str, port: u16) -> anyhow::Result<()> {
        info!("🚀 Starting Vectorizer Server on {}:{}", host, port);

        // SECURITY CHECK: When binding to 0.0.0.0 (production), require authentication
        // Either standard auth or HiveHub integration must be enabled
        let is_production_bind = host == "0.0.0.0";
        if is_production_bind {
            let has_auth = self.auth_handler_state.is_some();
            let has_hub = self.hub_manager.is_some();

            if !has_auth && !has_hub {
                error!("❌ SECURITY ERROR: Cannot bind to 0.0.0.0 without authentication enabled!");
                error!(
                    "   When exposing the server to all network interfaces, authentication is required."
                );
                error!("   Please enable authentication in config.yml:");
                error!("   auth:");
                error!("     enabled: true");
                error!("     jwt_secret: \"your-secure-secret-key\"");
                error!("");
                error!("   Or enable HiveHub integration:");
                error!("   hub:");
                error!("     enabled: true");
                error!("");
                error!("   Or use --host 127.0.0.1 for local development only.");
                return Err(anyhow::anyhow!(
                    "Security: Authentication required when binding to 0.0.0.0"
                ));
            }

            if has_hub {
                info!("🌐 HiveHub integration enabled - accepting internal service requests");
            }
            if has_auth {
                warn!(
                    "🔐 Production mode detected (0.0.0.0) - Authentication is REQUIRED for all API requests"
                );
            }
        }

        // Start gRPC server in background
        let grpc_port = port + 1; // gRPC on next port
        let grpc_host = host.to_string();
        let grpc_store = self.store.clone();
        let grpc_cluster_manager = self.cluster_manager.clone();
        let grpc_snapshot_manager = self.snapshot_manager.clone();
        let grpc_handle = tokio::spawn(async move {
            if let Err(e) = Self::start_grpc_server(
                &grpc_host,
                grpc_port,
                grpc_store,
                grpc_cluster_manager,
                grpc_snapshot_manager,
            )
            .await
            {
                error!("❌ gRPC server failed: {}", e);
            }
        });
        // Store gRPC handle for shutdown
        *self.grpc_task.lock().await = Some(grpc_handle);
        info!("✅ gRPC server task spawned");

        // Create server state for metrics endpoint
        let server_state = ServerState {
            file_watcher_system: self.file_watcher_system.clone(),
        };

        // Create MCP router (main server) using StreamableHTTP transport
        info!("🔧 Creating MCP router with StreamableHTTP transport (rmcp 0.8.1)...");
        let mcp_router = self
            .create_mcp_router(is_production_bind, self.auth_handler_state.clone())
            .await;
        info!("✅ MCP router created (StreamableHTTP)");

        // Create REST API router to add to MCP
        let metrics_collector_1 = self.metrics_collector.clone();
        let metrics_router = Router::new()
            .route("/metrics", get(get_file_watcher_metrics))
            .with_state(Arc::new(server_state))
            .layer(axum::middleware::from_fn(
                move |req: axum::extract::Request, next: axum::middleware::Next| {
                    let metrics = metrics_collector_1.clone();
                    async move {
                        // Record connection opened
                        metrics.record_connection_opened();

                        // Record API request
                        let start = std::time::Instant::now();
                        let response = next.run(req).await;
                        let duration = start.elapsed().as_millis() as f64;

                        // Record API request metrics
                        let is_success = response.status().is_success();
                        metrics.record_api_request(is_success, duration);

                        // Record connection closed
                        metrics.record_connection_closed();

                        response
                    }
                },
            ));

        // Public routes that don't require authentication (even in production)
        let public_routes = Router::new()
            .route("/health", get(rest_handlers::health_check))
            .route(
                "/prometheus/metrics",
                get(rest_handlers::get_prometheus_metrics),
            )
            .with_state(self.clone());

        let metrics_collector_2 = self.metrics_collector.clone();
        let rest_routes = Router::new()
            // Stats and monitoring (may require auth in production)
            .route("/stats", get(rest_handlers::get_stats))
            .route(
                "/indexing/progress",
                get(rest_handlers::get_indexing_progress),
            )
            // GUI-specific endpoints
            .route("/status", get(rest_handlers::get_status))
            .route("/logs", get(rest_handlers::get_logs))
            .route(
                "/collections/{name}/force-save",
                post(rest_handlers::force_save_collection),
            )
            .route("/workspace/add", post(rest_handlers::add_workspace))
            .route("/workspace/remove", post(rest_handlers::remove_workspace))
            .route("/workspace/list", get(rest_handlers::list_workspaces))
            .route(
                "/workspace/config",
                get(rest_handlers::get_workspace_config),
            )
            .route(
                "/workspace/config",
                post(rest_handlers::update_workspace_config),
            )
            .route("/config", get(rest_handlers::get_config))
            .route("/config", post(rest_handlers::update_config))
            .route("/admin/restart", post(rest_handlers::restart_server))
            .route("/backups", get(rest_handlers::list_backups))
            .route("/backups/create", post(rest_handlers::create_backup))
            .route("/backups/restore", post(rest_handlers::restore_backup))
            .route(
                "/backups/directory",
                get(rest_handlers::get_backup_directory),
            )
            // HiveHub user-scoped backup routes
            .route("/hub/backups", get(hub_backup_handlers::list_user_backups))
            .route(
                "/hub/backups",
                post(hub_backup_handlers::create_user_backup),
            )
            .route(
                "/hub/backups/restore",
                post(hub_backup_handlers::restore_user_backup),
            )
            .route(
                "/hub/backups/upload",
                post(hub_backup_handlers::upload_user_backup),
            )
            .route(
                "/hub/backups/{backup_id}",
                get(hub_backup_handlers::get_user_backup),
            )
            .route(
                "/hub/backups/{backup_id}",
                delete(hub_backup_handlers::delete_user_backup),
            )
            .route(
                "/hub/backups/{backup_id}/download",
                get(hub_backup_handlers::download_user_backup),
            )
            // HiveHub usage statistics routes
            .route(
                "/hub/usage/statistics",
                get(hub_usage_handlers::get_usage_statistics),
            )
            .route("/hub/usage/quota", get(hub_usage_handlers::get_quota_info))
            // HiveHub tenant management routes (TODO: Fix handler implementations)
            // .route(
            //     "/api/hub/tenant/cleanup",
            //     post(hub_tenant_handlers::cleanup_tenant_data),
            // )
            // .route(
            //     "/api/hub/tenant/{tenant_id}/stats",
            //     get(hub_tenant_handlers::get_tenant_statistics),
            // )
            // .route(
            //     "/api/hub/tenant/{tenant_id}/migrate",
            //     post(hub_tenant_handlers::migrate_tenant_data),
            // )
            // HiveHub API key validation
            .route(
                "/hub/validate-key",
                post(hub_usage_handlers::validate_api_key),
            )
            // Collection management
            .route("/collections", get(rest_handlers::list_collections))
            .route("/collections", post(rest_handlers::create_collection))
            .route("/collections/{name}", get(rest_handlers::get_collection))
            .route(
                "/collections/{name}",
                delete(rest_handlers::delete_collection),
            )
            // Collection cleanup (file watcher bug fix)
            .route(
                "/collections/empty",
                get(rest_handlers::list_empty_collections),
            )
            .route(
                "/collections/cleanup",
                delete(rest_handlers::cleanup_empty_collections),
            )
            // Vector operations - single
            .route("/search", post(rest_handlers::search_vectors))
            .route(
                "/collections/{name}/search",
                post(rest_handlers::search_vectors),
            )
            .route(
                "/collections/{name}/search/text",
                post(rest_handlers::search_vectors_by_text),
            )
            .route(
                "/collections/{name}/search/file",
                post(rest_handlers::search_by_file),
            )
            .route(
                "/collections/{name}/hybrid_search",
                post(rest_handlers::hybrid_search_vectors),
            )
            .route("/insert", post(rest_handlers::insert_text))
            .route("/update", post(rest_handlers::update_vector))
            .route("/delete", post(rest_handlers::delete_vector))
            .route("/embed", post(rest_handlers::embed_text))
            .route("/vector", post(rest_handlers::get_vector))
            .route(
                "/collections/{name}/vectors",
                get(rest_handlers::list_vectors),
            )
            .route(
                "/collections/{name}/vectors/{id}",
                get(rest_handlers::get_vector),
            )
            .route(
                "/collections/{name}/vectors/{id}",
                delete(rest_handlers::delete_vector),
            )
            // Vector operations - batch
            .route("/batch_insert", post(rest_handlers::batch_insert_texts))
            .route("/insert_texts", post(rest_handlers::insert_texts))
            .route("/batch_search", post(rest_handlers::batch_search_vectors))
            .route("/batch_update", post(rest_handlers::batch_update_vectors))
            .route("/batch_delete", post(rest_handlers::batch_delete_vectors))
            // Intelligent search routes
            .route(
                "/intelligent_search",
                post(rest_handlers::intelligent_search),
            )
            .route(
                "/multi_collection_search",
                post(rest_handlers::multi_collection_search),
            )
            .route("/semantic_search", post(rest_handlers::semantic_search))
            .route("/contextual_search", post(rest_handlers::contextual_search))
            // Discovery routes
            .route("/discover", post(rest_handlers::discover))
            .route(
                "/discovery/filter_collections",
                post(rest_handlers::filter_collections),
            )
            .route(
                "/discovery/score_collections",
                post(rest_handlers::score_collections),
            )
            .route(
                "/discovery/expand_queries",
                post(rest_handlers::expand_queries),
            )
            .route(
                "/discovery/broad_discovery",
                post(rest_handlers::broad_discovery),
            )
            .route(
                "/discovery/semantic_focus",
                post(rest_handlers::semantic_focus),
            )
            // Cluster management routes (if cluster is enabled)
            // Note: These will be conditionally added when cluster is enabled
            .route(
                "/discovery/promote_readme",
                post(rest_handlers::promote_readme),
            )
            .route(
                "/discovery/compress_evidence",
                post(rest_handlers::compress_evidence),
            )
            .route(
                "/discovery/build_answer_plan",
                post(rest_handlers::build_answer_plan),
            )
            .route(
                "/discovery/render_llm_prompt",
                post(rest_handlers::render_llm_prompt),
            )
            // File Operations routes
            .route("/file/content", post(rest_handlers::get_file_content))
            .route("/file/list", post(rest_handlers::list_files_in_collection))
            .route("/file/summary", post(rest_handlers::get_file_summary))
            .route("/file/chunks", post(rest_handlers::get_file_chunks_ordered))
            .route("/file/outline", post(rest_handlers::get_project_outline))
            .route("/file/related", post(rest_handlers::get_related_files))
            .route(
                "/file/search_by_type",
                post(rest_handlers::search_by_file_type),
            )
            // File Upload routes
            .route("/files/upload", post(file_upload_handlers::upload_file))
            .route(
                "/files/config",
                get(file_upload_handlers::get_upload_config),
            )
            // Replication routes
            .route(
                "/replication/status",
                get(replication_handlers::get_replication_status),
            )
            .route(
                "/replication/configure",
                post(replication_handlers::configure_replication),
            )
            .route(
                "/replication/stats",
                get(replication_handlers::get_replication_stats),
            )
            .route(
                "/replication/replicas",
                get(replication_handlers::list_replicas),
            )
            // Qdrant-compatible routes (under /qdrant prefix)
            .route("/qdrant/collections", get(qdrant_handlers::get_collections))
            .route(
                "/qdrant/collections/{name}",
                get(qdrant_handlers::get_collection),
            )
            .route(
                "/qdrant/collections/{name}",
                put(qdrant_handlers::create_collection),
            )
            .route(
                "/qdrant/collections/{name}",
                delete(qdrant_handlers::delete_collection),
            )
            .route(
                "/qdrant/collections/{name}",
                axum::routing::patch(qdrant_handlers::update_collection),
            )
            .route(
                "/qdrant/collections/{name}/points",
                post(qdrant_vector_handlers::retrieve_points),
            )
            .route(
                "/qdrant/collections/{name}/points",
                put(qdrant_vector_handlers::upsert_points),
            )
            .route(
                "/qdrant/collections/{name}/points/delete",
                post(qdrant_vector_handlers::delete_points),
            )
            .route(
                "/qdrant/collections/aliases",
                post(qdrant_alias_handlers::update_aliases),
            )
            .route(
                "/qdrant/collections/{name}/aliases",
                get(qdrant_alias_handlers::list_collection_aliases),
            )
            .route("/qdrant/aliases", get(qdrant_alias_handlers::list_aliases))
            .route(
                "/qdrant/collections/{name}/points/scroll",
                post(qdrant_vector_handlers::scroll_points),
            )
            .route(
                "/qdrant/collections/{name}/points/count",
                post(qdrant_vector_handlers::count_points),
            )
            .route(
                "/qdrant/collections/{name}/points/search",
                post(qdrant_search_handlers::search_points),
            )
            .route(
                "/qdrant/collections/{name}/points/search/batch",
                post(qdrant_search_handlers::batch_search_points),
            )
            .route(
                "/qdrant/collections/{name}/points/recommend",
                post(qdrant_search_handlers::recommend_points),
            )
            .route(
                "/qdrant/collections/{name}/points/recommend/batch",
                post(qdrant_search_handlers::batch_recommend_points),
            )
            // Query API endpoints (Qdrant 1.7+)
            .route(
                "/qdrant/collections/{name}/points/query",
                post(qdrant_query_handlers::query_points),
            )
            .route(
                "/qdrant/collections/{name}/points/query/batch",
                post(qdrant_query_handlers::batch_query_points),
            )
            .route(
                "/qdrant/collections/{name}/points/query/groups",
                post(qdrant_query_handlers::query_points_groups),
            )
            // Search Groups and Matrix API endpoints
            .route(
                "/qdrant/collections/{name}/points/search/groups",
                post(qdrant_search_handlers::search_points_groups),
            )
            .route(
                "/qdrant/collections/{name}/points/search/matrix/pairs",
                post(qdrant_search_handlers::search_matrix_pairs),
            )
            .route(
                "/qdrant/collections/{name}/points/search/matrix/offsets",
                post(qdrant_search_handlers::search_matrix_offsets),
            )
            // Snapshot API endpoints
            .route(
                "/qdrant/collections/{name}/snapshots",
                get(qdrant_snapshot_handlers::list_collection_snapshots),
            )
            .route(
                "/qdrant/collections/{name}/snapshots",
                post(qdrant_snapshot_handlers::create_collection_snapshot),
            )
            .route(
                "/qdrant/collections/{name}/snapshots/{snapshot_name}",
                delete(qdrant_snapshot_handlers::delete_collection_snapshot),
            )
            .route(
                "/qdrant/collections/{name}/snapshots/recover",
                post(qdrant_snapshot_handlers::recover_collection_snapshot),
            )
            .route(
                "/qdrant/collections/{name}/snapshots/upload",
                post(qdrant_snapshot_handlers::upload_collection_snapshot),
            )
            .route(
                "/qdrant/snapshots",
                get(qdrant_snapshot_handlers::list_all_snapshots),
            )
            .route(
                "/qdrant/snapshots",
                post(qdrant_snapshot_handlers::create_full_snapshot),
            )
            // Sharding API endpoints
            .route(
                "/qdrant/collections/{name}/shards",
                get(qdrant_sharding_handlers::list_shard_keys),
            )
            .route(
                "/qdrant/collections/{name}/shards",
                put(qdrant_sharding_handlers::create_shard_key),
            )
            .route(
                "/qdrant/collections/{name}/shards/delete",
                post(qdrant_sharding_handlers::delete_shard_key),
            )
            // Cluster API endpoints
            .route(
                "/qdrant/cluster",
                get(qdrant_cluster_handlers::get_cluster_status),
            )
            .route(
                "/qdrant/cluster/recover",
                post(qdrant_cluster_handlers::cluster_recover),
            )
            .route(
                "/qdrant/cluster/peer/{peer_id}",
                delete(qdrant_cluster_handlers::remove_peer),
            )
            .route(
                "/qdrant/cluster/metadata/keys",
                get(qdrant_cluster_handlers::list_metadata_keys),
            )
            .route(
                "/qdrant/cluster/metadata/keys/{key}",
                get(qdrant_cluster_handlers::get_metadata_key),
            )
            .route(
                "/qdrant/cluster/metadata/keys/{key}",
                put(qdrant_cluster_handlers::update_metadata_key),
            )
            // Dashboard - serve static files from dist directory (production build)
            // Use ServeDir with fallback to index.html for SPA routing support
            // This ensures that direct URL access (e.g., /dashboard/collections) works on refresh
            //
            // Route priority for /dashboard/*:
            // 1. Exact file match (assets/, favicon.ico, etc.) - served with cache headers
            // 2. SPA fallback - any other route returns index.html for React Router
            .nest_service(
                "/dashboard",
                ServeDir::new("dashboard/dist")
                    .fallback(ServeFile::new("dashboard/dist/index.html")),
            )
            // Add cache headers for dashboard assets
            // Assets in /dashboard/assets/* are fingerprinted, so max-age=1year is safe
            // index.html should not be cached to ensure updates are picked up
            .layer(axum::middleware::from_fn(
                |req: axum::extract::Request, next: axum::middleware::Next| async move {
                    let path = req.uri().path().to_string();
                    let mut response = next.run(req).await;

                    // Only apply cache headers to dashboard routes
                    if path.starts_with("/dashboard") {
                        // Log dashboard requests at debug level
                        tracing::debug!("📊 Dashboard request: {}", path);

                        let headers = response.headers_mut();
                        if path.starts_with("/dashboard/assets/") {
                            // Fingerprinted assets: cache for 1 year
                            headers.insert(
                                axum::http::header::CACHE_CONTROL,
                                "public, max-age=31536000, immutable".parse().unwrap(),
                            );
                        } else if path == "/dashboard/" || path == "/dashboard" {
                            // index.html: no cache to ensure updates
                            headers.insert(
                                axum::http::header::CACHE_CONTROL,
                                "no-cache, no-store, must-revalidate".parse().unwrap(),
                            );
                        } else {
                            // SPA routes (fallback to index.html): no cache
                            headers.insert(
                                axum::http::header::CACHE_CONTROL,
                                "no-cache, no-store, must-revalidate".parse().unwrap(),
                            );
                        }
                    }

                    response
                },
            ))
            .layer(axum::middleware::from_fn(
                crate::monitoring::correlation_middleware,
            ))
            .layer(axum::middleware::from_fn(
                move |req: axum::extract::Request, next: axum::middleware::Next| {
                    let metrics = metrics_collector_2.clone();
                    let method = req.method().clone();
                    let uri = req.uri().clone();
                    async move {
                        // Log all requests, especially PUT to /qdrant/collections/*/points
                        if uri.path().contains("/qdrant/collections")
                            && uri.path().contains("/points")
                        {
                            info!("🔵 [MIDDLEWARE] {} {}", method, uri);
                        }

                        // Record connection opened
                        metrics.record_connection_opened();

                        // Record API request
                        let start = std::time::Instant::now();
                        let response = next.run(req).await;
                        let duration = start.elapsed().as_millis() as f64;

                        // Record API request metrics
                        let is_success = response.status().is_success();
                        metrics.record_api_request(is_success, duration);

                        // Record connection closed
                        metrics.record_connection_closed();

                        response
                    }
                },
            ))
            .with_state(self.clone());

        // Add cluster routes if cluster is enabled
        let rest_routes = if let (Some(cluster_mgr), Some(_client_pool)) = (
            self.cluster_manager.as_ref(),
            self.cluster_client_pool.as_ref(),
        ) {
            let cluster_state = crate::api::cluster::ClusterApiState {
                cluster_manager: cluster_mgr.clone(),
                store: self.store.clone(),
            };
            let cluster_router =
                crate::api::cluster::create_cluster_router().with_state(cluster_state);
            rest_routes.merge(cluster_router)
        } else {
            rest_routes
        };

        // Add graph routes
        let graph_state = crate::api::graph::GraphApiState {
            store: self.store.clone(),
        };
        let graph_router = crate::api::graph::create_graph_router().with_state(graph_state);
        let rest_routes = rest_routes.merge(graph_router);

        // Add GraphQL routes
        let graphql_schema = if let Some(ref auto_save) = self.auto_save_manager {
            crate::api::graphql::create_schema_with_auto_save(
                self.store.clone(),
                self.embedding_manager.clone(),
                self.start_time,
                auto_save.clone(),
            )
        } else {
            crate::api::graphql::create_schema(
                self.store.clone(),
                self.embedding_manager.clone(),
                self.start_time,
            )
        };
        let graphql_state = graphql_handlers::GraphQLState {
            schema: graphql_schema,
        };
        let graphql_router = Router::new()
            .route("/graphql", post(graphql_handlers::graphql_handler))
            .route("/graphql", get(graphql_handlers::graphql_playground))
            .route("/graphiql", get(graphql_handlers::graphql_playground))
            .with_state(graphql_state);
        let rest_routes = rest_routes.merge(graphql_router);
        info!("📊 GraphQL API available at /graphql (playground at /graphiql)");

        // Add auth routes and apply auth middleware if auth is enabled
        let rest_routes = if let Some(auth_state) = self.auth_handler_state.clone() {
            info!("🔐 Adding authentication routes...");

            // Public routes (no auth required) - login, password validation, and health check
            let public_auth_router = Router::new()
                .route("/auth/login", post(auth_handlers::login))
                .route(
                    "/auth/validate-password",
                    post(auth_handlers::validate_password_endpoint),
                )
                .with_state(auth_state.clone());

            // Protected auth routes (require authentication via handlers)
            // Note: Each handler internally checks auth_state.authenticated
            let protected_auth_router = Router::new()
                .route("/auth/me", get(auth_handlers::get_me))
                .route("/auth/logout", post(auth_handlers::logout))
                .route("/auth/refresh", post(auth_handlers::refresh_token))
                .route("/auth/keys", post(auth_handlers::create_api_key))
                .route("/auth/keys", get(auth_handlers::list_api_keys))
                .route("/auth/keys/{id}", delete(auth_handlers::revoke_api_key))
                // User management routes (admin only)
                .route("/auth/users", post(auth_handlers::create_user))
                .route("/auth/users", get(auth_handlers::list_users))
                .route("/auth/users/{username}", delete(auth_handlers::delete_user))
                .route(
                    "/auth/users/{username}/password",
                    put(auth_handlers::change_password),
                )
                .with_state(auth_state.clone());

            // Merge auth routes first
            let rest_routes = rest_routes
                .merge(public_auth_router)
                .merge(protected_auth_router);

            rest_routes
        } else {
            rest_routes
        };

        // Apply HiveHub middleware if hub integration is enabled
        // This middleware extracts tenant context from headers for multi-tenant isolation
        let rest_routes = if let Some(ref hub_manager) = self.hub_manager {
            info!("🔐 Applying HiveHub tenant middleware to routes...");

            use axum::middleware::from_fn_with_state;

            use crate::hub::middleware::{HubAuthMiddleware, hub_auth_middleware};

            let hub_auth = hub_manager.auth().clone();
            let hub_quota = hub_manager.quota().clone();
            let hub_config = hub_manager.config().clone();

            let hub_middleware_state = HubAuthMiddleware::new(hub_auth, hub_quota, hub_config);

            rest_routes.layer(from_fn_with_state(
                hub_middleware_state,
                hub_auth_middleware,
            ))
        } else {
            rest_routes
        };

        // Create UMICP state
        let umicp_state = crate::umicp::UmicpState {
            store: self.store.clone(),
            embedding_manager: self.embedding_manager.clone(),
        };

        // Create UMICP routes (needs custom state)
        // Note: Auth is enforced via the require_production_auth helper for /umicp POST
        let umicp_routes = Router::new()
            .route("/umicp", post(crate::umicp::transport::umicp_handler))
            .route("/umicp/health", get(crate::umicp::health_check))
            .route(
                "/umicp/discover",
                get(crate::umicp::transport::umicp_discover_handler),
            )
            .with_state(umicp_state);

        // Merge all routes - order matters!
        // 1. Public routes first (health check, prometheus metrics) - no auth required
        // 2. UMICP routes (most specific)
        // 3. MCP routes
        // 4. REST API routes (including /api/*, dashboard with SPA fallback via ServeDir)
        // 5. Metrics routes
        // Note: Dashboard SPA routing is handled by ServeDir with not_found_service in rest_routes
        let app = Router::new()
            .merge(public_routes) // Health check and prometheus - always public
            .merge(umicp_routes)
            .merge(mcp_router)
            .merge(rest_routes)
            .merge(metrics_router);

        // In production mode, apply global auth middleware BEFORE CORS
        // This middleware handles both standard auth (JWT/API key) and HiveHub integration
        let hub_mgr = self.hub_manager.clone();
        let app = if is_production_bind && (self.auth_handler_state.is_some() || hub_mgr.is_some())
        {
            let auth_mgr = self
                .auth_handler_state
                .as_ref()
                .map(|state| state.auth_manager.clone());
            let hub_manager = hub_mgr.clone();
            app.layer(axum::middleware::from_fn(move |req: axum::extract::Request, next: axum::middleware::Next| {
                let auth_manager = auth_mgr.clone();
                let hub_manager = hub_manager.clone();
                async move {
                    let path = req.uri().path();

                    // Public routes - no auth required
                    if path == "/health"
                        || path == "/prometheus/metrics"
                        || path == "/auth/login"
                        || path == "/auth/validate-password"
                        || path == "/umicp/health"
                        || path == "/umicp/discover"
                        || path.starts_with("/dashboard")
                    {
                        return next.run(req).await;
                    }

                    // Check for HiveHub internal service header
                    // When HiveHub integration is enabled, internal service requests bypass auth
                    if hub_manager.is_some() {
                        if req.headers().contains_key("x-hivehub-service") {
                            tracing::debug!("HiveHub internal service request - bypassing auth for {}", path);
                            return next.run(req).await;
                        }
                    }

                    // Standard authentication (if auth is enabled)
                    if let Some(ref auth_manager) = auth_manager {
                        // Extract credentials from headers
                        let (jwt_token, api_key) = extract_auth_credentials(&req);

                        // Debug: Log what we found
                        tracing::debug!("Auth check for {}: jwt={:?}, api_key={:?}", path, jwt_token.is_some(), api_key.is_some());

                        // Validate credentials
                        if !check_mcp_auth_with_credentials(jwt_token, api_key, auth_manager).await {
                            return axum::response::Response::builder()
                                .status(axum::http::StatusCode::UNAUTHORIZED)
                                .header("Content-Type", "application/json")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(axum::body::Body::from(
                                    r#"{"error":"unauthorized","message":"Authentication required. Provide a valid JWT token or API key."}"#
                                ))
                                .unwrap();
                        }
                    } else if hub_manager.is_none() {
                        // No auth configured and no hub integration - reject
                        return axum::response::Response::builder()
                            .status(axum::http::StatusCode::UNAUTHORIZED)
                            .header("Content-Type", "application/json")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(axum::body::Body::from(
                                r#"{"error":"unauthorized","message":"Authentication not configured."}"#
                            ))
                            .unwrap();
                    }

                    next.run(req).await
                }
            }))
            // Apply CORS after auth middleware
            .layer(CorsLayer::permissive())
            // Apply security headers
            .layer(axum::middleware::from_fn(security_headers_middleware))
        } else {
            // Development mode: just apply CORS and security headers
            app.layer(CorsLayer::permissive())
                .layer(axum::middleware::from_fn(security_headers_middleware))
        };

        info!("🌐 Vectorizer Server available at:");
        info!("   📡 MCP StreamableHTTP: http://{}:{}/mcp", host, port);
        info!("   🔌 REST API: http://{}:{}", host, port);
        info!("   🔗 UMICP: http://{}:{}/umicp", host, port);
        info!(
            "   🔍 UMICP Discovery (v0.2.1): http://{}:{}/umicp/discover",
            host, port
        );
        info!("   🎯 Qdrant API: http://{}:{}/qdrant", host, port);
        info!("   📊 GraphQL API: http://{}:{}/graphql", host, port);
        info!(
            "   🎮 GraphQL Playground: http://{}:{}/graphiql",
            host, port
        );
        info!("   📊 Dashboard: http://{}:{}/dashboard/", host, port);
        if self.auth_handler_state.is_some() {
            info!("   🔐 Auth API: http://{}:{}/auth", host, port);
        }
        if self.hub_manager.is_some() {
            info!("   🌐 HiveHub: Cluster mode enabled (internal service access)");
        }

        // Bind and start the server
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
        info!(
            "✅ MCP server (StreamableHTTP) with REST API listening on {}:{}",
            host, port
        );

        // Create shutdown signal for axum graceful shutdown
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn task to listen for Ctrl+C and trigger shutdown
        tokio::spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                info!("🛑 Received shutdown signal (Ctrl+C)");
                let _ = shutdown_tx.send(());
            }
        });

        // Serve the application with graceful shutdown
        let server_handle = axum::serve(listener, app).with_graceful_shutdown(async {
            shutdown_rx.await.ok();
            info!("🛑 Graceful shutdown signal received, stopping HTTP server...");
        });

        // Spawn server task
        let server_task = tokio::spawn(async move {
            if let Err(e) = server_handle.await {
                error!("❌ Server error: {}", e);
            } else {
                info!("✅ HTTP server stopped");
            }
        });

        // Get abort handle before moving server_task (for emergency shutdown)
        let server_task_abort = server_task.abort_handle();

        // Wait for HTTP server to stop (this will block until Ctrl+C is pressed)
        // When shutdown signal is received, the server will stop gracefully
        // No timeout here - server should run indefinitely until Ctrl+C
        match server_task.await {
            Ok(_) => {
                info!("✅ HTTP server stopped gracefully");
            }
            Err(e) => {
                error!("❌ HTTP server task join error: {}", e);
                // Force abort as fallback
                server_task_abort.abort();
            }
        }

        // Now shutdown all background tasks AFTER HTTP server has stopped
        info!("🛑 Stopping all background tasks...");

        // Background collection loading task (non-blocking)
        if let Ok(mut bg_task) = self.background_task.try_lock() {
            if let Some((handle, cancel_tx)) = bg_task.take() {
                let _ = cancel_tx.send(true);
                handle.abort();
                info!("✅ Background task aborted");
            }
        }

        // File watcher cancellation (non-blocking)
        if let Ok(mut cancel) = self.file_watcher_cancel.try_lock() {
            if let Some(cancel_tx) = cancel.take() {
                let _ = cancel_tx.send(true);
            }
        }

        // File watcher task (non-blocking)
        if let Ok(mut fw_task) = self.file_watcher_task.try_lock() {
            if let Some(handle) = fw_task.take() {
                handle.abort();
                info!("✅ File watcher task aborted");
            }
        }

        // File watcher system (non-blocking)
        if let Ok(mut fw_system) = self.file_watcher_system.try_lock() {
            fw_system.take(); // Just drop it
            info!("✅ File watcher system dropped");
        }

        // gRPC server task (non-blocking)
        if let Ok(mut grpc_task) = self.grpc_task.try_lock() {
            if let Some(handle) = grpc_task.take() {
                handle.abort();
                info!("✅ gRPC server task aborted");
            }
        }

        // System collector task (non-blocking)
        if let Ok(mut sys_task) = self.system_collector_task.try_lock() {
            if let Some(handle) = sys_task.take() {
                handle.abort();
                info!("✅ System collector task aborted");
            }
        }

        // Force save all data before shutdown to prevent data loss
        // This ensures any changes made since the last auto-save are persisted
        if let Some(auto_save) = &self.auto_save_manager {
            info!("💾 Forcing final save before shutdown...");
            match auto_save.force_save().await {
                Ok(_) => info!("✅ Final save completed successfully"),
                Err(e) => warn!("⚠️ Final save failed (data may be lost): {}", e),
            }
        }

        // Auto save task (non-blocking) - abort AFTER force_save
        if let Ok(mut auto_task) = self.auto_save_task.try_lock() {
            if let Some(handle) = auto_task.take() {
                handle.abort();
                info!("✅ Auto save task aborted");
            }
        }

        // Auto save manager shutdown (non-blocking, no await)
        if let Some(auto_save) = &self.auto_save_manager {
            auto_save.shutdown();
        }

        info!("✅ Server stopped");
        Ok(())
    }

    /// Create MCP router with StreamableHTTP transport (rmcp 0.8.1)
    /// In production mode (is_production=true), validates authentication before processing requests
    async fn create_mcp_router(
        &self,
        is_production: bool,
        auth_state: Option<auth_handlers::AuthHandlerState>,
    ) -> Router {
        use std::sync::Arc;

        use hyper::service::Service;
        use hyper_util::service::TowerToHyperService;
        use rmcp::transport::streamable_http_server::StreamableHttpService;
        use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

        // Create MCP service handler
        let store = self.store.clone();
        let embedding_manager = self.embedding_manager.clone();
        let cluster_manager = self.cluster_manager.clone();

        // Create StreamableHTTP service
        let streamable_service = StreamableHttpService::new(
            move || {
                Ok(VectorizerMcpService {
                    store: store.clone(),
                    embedding_manager: embedding_manager.clone(),
                    cluster_manager: cluster_manager.clone(),
                })
            },
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // Convert to axum service and create router
        let hyper_service = TowerToHyperService::new(streamable_service);

        // Create router with the MCP endpoint
        // In production mode, wrap with authentication check
        if is_production && auth_state.is_some() {
            let auth_state = auth_state.unwrap();
            let auth_manager = auth_state.auth_manager.clone();

            Router::new().route(
                "/mcp",
                axum::routing::any(
                    move |req: axum::extract::Request| {
                        let mut service = hyper_service.clone();
                        let auth_mgr = auth_manager.clone();

                        // Extract credentials synchronously before async block
                        let (jwt_token, api_key) = extract_auth_credentials(&req);

                        async move {
                            // Check authentication in production mode
                            if !check_mcp_auth_with_credentials(jwt_token, api_key, &auth_mgr).await
                            {
                                return axum::response::Response::builder()
                                    .status(axum::http::StatusCode::UNAUTHORIZED)
                                    .header("Content-Type", "application/json")
                                    .body(axum::body::Body::from(
                                        r#"{"error":"unauthorized","message":"Authentication required for MCP access in production mode."}"#
                                    ))
                                    .unwrap();
                            }

                            // Forward request to hyper service
                            match service.call(req).await {
                                Ok(response) => {
                                    // Convert BoxBody to axum Body
                                    let (parts, body) = response.into_parts();
                                    axum::response::Response::from_parts(
                                        parts,
                                        axum::body::Body::new(body),
                                    )
                                }
                                Err(_) => axum::response::Response::builder()
                                    .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(axum::body::Body::from("Internal server error"))
                                    .unwrap(),
                            }
                        }
                    },
                ),
            )
        } else {
            Router::new().route(
                "/mcp",
                axum::routing::any(move |req: axum::extract::Request| {
                    let mut service = hyper_service.clone();
                    async move {
                        // Forward request to hyper service
                        match service.call(req).await {
                            Ok(response) => {
                                // Convert BoxBody to axum Body
                                let (parts, body) = response.into_parts();
                                axum::response::Response::from_parts(
                                    parts,
                                    axum::body::Body::new(body),
                                )
                            }
                            Err(_) => axum::response::Response::builder()
                                .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                                .body(axum::body::Body::from("Internal server error"))
                                .unwrap(),
                        }
                    }
                }),
            )
        }
    }

    /// Start gRPC server
    async fn start_grpc_server(
        host: &str,
        port: u16,
        store: Arc<VectorStore>,
        cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
        snapshot_manager: Option<Arc<crate::storage::SnapshotManager>>,
    ) -> anyhow::Result<()> {
        use tonic::transport::Server;

        use crate::grpc::VectorizerGrpcService;
        use crate::grpc::vectorizer::vectorizer_service_server::VectorizerServiceServer;

        let addr = format!("{}:{}", host, port).parse()?;
        let service = VectorizerGrpcService::new(store.clone());

        info!("🚀 Starting gRPC server on {}", addr);

        let mut server_builder =
            Server::builder().add_service(VectorizerServiceServer::new(service));

        // Add ClusterService if cluster is enabled
        if let Some(cluster_mgr) = cluster_manager {
            use crate::cluster::ClusterGrpcService;
            use crate::grpc::cluster::cluster_service_server::ClusterServiceServer;

            info!("🔗 Adding Cluster gRPC service");
            let cluster_service = ClusterGrpcService::new(store.clone(), cluster_mgr);
            server_builder = server_builder.add_service(ClusterServiceServer::new(cluster_service));
        }

        // Add Qdrant-compatible gRPC services
        {
            use crate::grpc::QdrantGrpcService;
            use crate::grpc::qdrant_proto::collections_server::CollectionsServer;
            use crate::grpc::qdrant_proto::points_server::PointsServer;
            use crate::grpc::qdrant_proto::snapshots_server::SnapshotsServer;

            info!("🔗 Adding Qdrant-compatible gRPC services (Collections, Points, Snapshots)");
            let qdrant_service = if let Some(sm) = snapshot_manager {
                QdrantGrpcService::with_snapshot_manager(store.clone(), sm)
            } else {
                QdrantGrpcService::new(store.clone())
            };

            // Add all Qdrant services using the same service instance (it implements all traits)
            server_builder = server_builder
                .add_service(CollectionsServer::new(qdrant_service.clone()))
                .add_service(PointsServer::new(qdrant_service.clone()))
                .add_service(SnapshotsServer::new(qdrant_service));
        }

        server_builder.serve(addr).await?;

        Ok(())
    }
}

/// Extract auth credentials from request headers (sync part)
/// Returns (Option<jwt_token>, Option<api_key>)
fn extract_auth_credentials(req: &axum::extract::Request) -> (Option<String>, Option<String>) {
    use axum::http::header::AUTHORIZATION;

    let mut jwt_token: Option<String> = None;
    let mut api_key: Option<String> = None;

    // Try to get authorization header
    if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            // Check for Bearer token (JWT)
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                jwt_token = Some(token.to_string());
            } else {
                // Try as direct API key
                api_key = Some(auth_str.to_string());
            }
        }
    }

    // Check for X-API-Key header (if no API key found yet)
    if api_key.is_none() {
        if let Some(api_key_header) = req.headers().get("X-API-Key") {
            if let Ok(key) = api_key_header.to_str() {
                api_key = Some(key.to_string());
            }
        }
    }

    // Check for API key in query parameters (if no API key found yet)
    if api_key.is_none() {
        if let Some(query) = req.uri().query() {
            for param in query.split('&') {
                if let Some(key) = param.strip_prefix("api_key=") {
                    api_key = Some(key.to_string());
                    break;
                }
            }
        }
    }

    (jwt_token, api_key)
}

/// Check authentication for MCP/UMICP requests in production mode
/// Returns true if authentication is valid, false otherwise
async fn check_mcp_auth_with_credentials(
    jwt_token: Option<String>,
    api_key: Option<String>,
    auth_manager: &std::sync::Arc<crate::auth::AuthManager>,
) -> bool {
    // Try JWT first
    if let Some(token) = jwt_token {
        if auth_manager.validate_jwt(&token).is_ok() {
            return true;
        }
    }

    // Try API key
    if let Some(key) = api_key {
        if auth_manager.validate_api_key(&key).await.is_ok() {
            return true;
        }
    }

    false
}

/// Security headers middleware
///
/// Adds standard security headers to all responses:
/// - X-Content-Type-Options: nosniff - Prevents MIME type sniffing
/// - X-Frame-Options: DENY - Prevents clickjacking
/// - X-XSS-Protection: 1; mode=block - XSS filter (legacy browsers)
/// - Content-Security-Policy: default-src 'self' - CSP for dashboard
/// - Referrer-Policy: strict-origin-when-cross-origin - Controls referrer info
/// - Permissions-Policy: geolocation=(), microphone=(), camera=() - Disables sensitive APIs
async fn security_headers_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );

    // Prevent clickjacking (allow framing for dashboard)
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        "SAMEORIGIN".parse().unwrap(),
    );

    // XSS protection for legacy browsers
    headers.insert(
        axum::http::HeaderName::from_static("x-xss-protection"),
        "1; mode=block".parse().unwrap(),
    );

    // Content Security Policy - allow self and inline for dashboard
    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' ws: wss:".parse().unwrap(),
    );

    // Referrer policy
    headers.insert(
        axum::http::header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Permissions policy - disable sensitive APIs
    headers.insert(
        axum::http::HeaderName::from_static("permissions-policy"),
        "geolocation=(), microphone=(), camera=(), payment=()"
            .parse()
            .unwrap(),
    );

    response
}

/// Get File Watcher metrics endpoint
pub async fn get_file_watcher_metrics(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<FileWatcherMetrics>, crate::server::error_middleware::ErrorResponse> {
    use crate::server::error_middleware::ErrorResponse;

    // Get the file watcher system from the state
    let watcher_lock = state.file_watcher_system.lock().await;

    if let Some(watcher_system) = watcher_lock.as_ref() {
        let metrics = watcher_system.get_metrics().await;
        return Ok(Json(metrics));
    }

    // Return empty/default metrics if File Watcher is not available
    use std::collections::HashMap;

    use crate::file_watcher::metrics::*;

    let default_metrics = FileWatcherMetrics {
        timing: TimingMetrics {
            avg_file_processing_ms: 0.0,
            avg_discovery_ms: 0.0,
            avg_sync_ms: 0.0,
            uptime_seconds: 0,
            last_activity: None,
            peak_processing_ms: 0,
        },
        files: FileMetrics {
            total_files_processed: 0,
            files_processed_success: 0,
            files_processed_error: 0,
            files_skipped: 0,
            files_in_progress: 0,
            files_discovered: 0,
            files_removed: 0,
            files_indexed_realtime: 0,
        },
        system: SystemMetrics {
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            thread_count: 0,
            active_file_handles: 0,
            disk_io_ops_per_sec: 0,
            network_io_bytes_per_sec: 0,
        },
        network: NetworkMetrics {
            total_api_requests: 0,
            successful_api_requests: 0,
            failed_api_requests: 0,
            avg_api_response_ms: 0.0,
            peak_api_response_ms: 0,
            active_connections: 0,
        },
        status: StatusMetrics {
            total_errors: 0,
            errors_by_type: HashMap::new(),
            current_status: "initializing".to_string(),
            last_error: None,
            health_score: 0,
            restart_count: 0,
        },
        collections: HashMap::new(),
    };

    Ok(Json(default_metrics))
}

/// MCP Service implementation
#[derive(Clone)]
struct VectorizerMcpService {
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
    cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
}

impl rmcp::ServerHandler for VectorizerMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        use rmcp::model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo};

        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "vectorizer-server".to_string(),
                title: Some("HiveLLM Vectorizer Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                website_url: Some("https://github.com/hivellm/hivellm".to_string()),
                icons: None,
            },
            instructions: Some("HiveLLM Vectorizer - High-performance semantic search and vector database system with MCP + REST API.".to_string()),
        }
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<
        Output = Result<rmcp::model::ListToolsResult, rmcp::model::ErrorData>,
    > + Send
    + '_ {
        async move {
            use rmcp::model::ListToolsResult;

            let tools = mcp_tools::get_mcp_tools();

            Ok(ListToolsResult {
                tools,
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<
        Output = Result<rmcp::model::CallToolResult, rmcp::model::ErrorData>,
    > + Send
    + '_ {
        async move {
            mcp_handlers::handle_mcp_tool(
                request,
                self.store.clone(),
                self.embedding_manager.clone(),
                self.cluster_manager.clone(),
            )
            .await
        }
    }

    fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<
        Output = Result<rmcp::model::ListResourcesResult, rmcp::model::ErrorData>,
    > + Send
    + '_ {
        async move {
            use rmcp::model::ListResourcesResult;
            Ok(ListResourcesResult {
                resources: vec![],
                next_cursor: None,
            })
        }
    }
}

/// Load file watcher configuration from workspace.yml
async fn load_file_watcher_config() -> anyhow::Result<crate::file_watcher::FileWatcherConfig> {
    match crate::file_watcher::FileWatcherConfig::from_yaml_file("workspace.yml") {
        Ok(config) => {
            info!(
                "Loaded file watcher configuration from workspace: watch_paths={:?}, exclude_patterns={:?}",
                config.watch_paths, config.exclude_patterns
            );
            Ok(config)
        }
        Err(e) => {
            info!(
                "Failed to load workspace configuration: {}, using default file watcher config",
                e
            );
            Ok(crate::file_watcher::FileWatcherConfig::default())
        }
    }
}

/// Load workspace collections using the new file_loader module
pub async fn load_workspace_collections(
    store: &Arc<VectorStore>,
    embedding_manager: &Arc<EmbeddingManager>,
    mut cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<usize> {
    use std::path::Path;

    use crate::file_loader::{FileLoader, LoaderConfig};
    use crate::workspace::manager::WorkspaceManager;

    // Look for workspace configuration file
    let workspace_file = Path::new("workspace.yml");
    info!(
        "Checking for workspace file at: {}",
        workspace_file.display()
    );
    if !workspace_file.exists() {
        info!("No workspace configuration file found at workspace.yml");
        return Ok(0);
    }

    info!("Found workspace configuration file, loading...");

    // Load workspace configuration
    let workspace_manager = match WorkspaceManager::load_from_file(workspace_file) {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to load workspace configuration: {}", e);
            return Ok(0);
        }
    };

    info!(
        "Workspace loaded with {} projects",
        workspace_manager.config().projects.len()
    );

    let mut indexed_count = 0;

    // Check if using .vecdb format - if so, verify collections in archive first
    let data_dir = std::path::PathBuf::from("./data");
    let vecdb_path = data_dir.join("vectorizer.vecdb");
    let using_vecdb = vecdb_path.exists();

    let mut existing_in_vecdb = std::collections::HashSet::new();
    if using_vecdb {
        use crate::storage::StorageReader;
        info!(
            "🔍 .vecdb file exists at {}, attempting to read...",
            vecdb_path.display()
        );
        match StorageReader::new(&data_dir) {
            Ok(reader) => match reader.list_collections() {
                Ok(collections) => {
                    existing_in_vecdb = collections.iter().cloned().collect();
                    info!(
                        "📦 Found {} collections in .vecdb archive: {:?}",
                        existing_in_vecdb.len(),
                        existing_in_vecdb
                    );
                }
                Err(e) => {
                    warn!("Failed to list collections from .vecdb: {}", e);
                }
            },
            Err(e) => {
                warn!("Failed to create StorageReader for .vecdb: {}", e);
            }
        }
    } else {
        info!("📦 .vecdb file does not exist, will create after indexing");
    }

    // Process each enabled project
    for project in workspace_manager.enabled_projects() {
        // Check for cancellation
        if *cancel_rx.borrow() {
            info!("🛑 Workspace loading cancelled by user");
            break;
        }

        info!("Processing project: {}", project.name);

        for collection in &project.collections {
            // Check for cancellation
            if *cancel_rx.borrow() {
                info!("🛑 Workspace loading cancelled by user");
                break;
            }

            info!("Processing collection: {}", collection.name);

            // Check if collection already exists in .vecdb archive
            if using_vecdb && existing_in_vecdb.contains(&collection.name) {
                // Collection exists in .vecdb - FORCE LOAD it into memory (same as when no cache)
                if !store.has_collection_in_memory(&collection.name) {
                    info!(
                        "📥 FORCE LOADING collection '{}' from .vecdb into memory...",
                        collection.name
                    );

                    // Use the SAME method as when .vecdb doesn't exist - load directly from .vecdb
                    use crate::storage::StorageReader;
                    match StorageReader::new(&std::path::PathBuf::from("./data")) {
                        Ok(reader) => {
                            let vector_store_path = format!("{}_vector_store.bin", collection.name);
                            match reader.read_file(&vector_store_path) {
                                Ok(data) => {
                                    // Try to deserialize as PersistedVectorStore first (correct format)
                                    let persisted = match serde_json::from_slice::<
                                        crate::persistence::PersistedVectorStore,
                                    >(
                                        &data
                                    ) {
                                        Ok(persisted_store) => {
                                            // Extract the first collection from the store
                                            persisted_store.collections.into_iter().next()
                                        }
                                        Err(_) => {
                                            // Fallback: try deserializing as PersistedCollection directly (legacy format)
                                            serde_json::from_slice::<
                                                crate::persistence::PersistedCollection,
                                            >(&data)
                                            .ok()
                                        }
                                    };

                                    if let Some(mut persisted) = persisted {
                                        // BACKWARD COMPATIBILITY: If name is empty, infer from filename
                                        if persisted.name.is_empty() {
                                            persisted.name = collection.name.clone();
                                        }

                                        // Use EXACT config from .vecdb (not workspace config!)
                                        let config = persisted.config.clone().unwrap_or_else(|| {
                                                warn!("⚠️  Collection '{}' has no config in .vecdb, using default", collection.name);
                                                crate::models::CollectionConfig::default()
                                            });
                                        let vector_count = persisted.vectors.len();

                                        info!(
                                            "📥 Loading collection '{}' from .vecdb with {} vectors...",
                                            collection.name, vector_count
                                        );

                                        // Convert vectors FIRST (before creating collection)
                                        info!(
                                            "🔄 Converting {} persisted vectors to runtime format...",
                                            persisted.vectors.len()
                                        );
                                        let vectors: Vec<crate::models::Vector> = persisted
                                            .vectors
                                            .into_iter()
                                            .filter_map(|pv| match pv.into_runtime() {
                                                Ok(v) => Some(v),
                                                Err(e) => {
                                                    warn!(
                                                        "Failed to convert persisted vector: {}",
                                                        e
                                                    );
                                                    None
                                                }
                                            })
                                            .collect();

                                        info!(
                                            "🔄 Converted {} vectors successfully",
                                            vectors.len()
                                        );

                                        // Create collection with config FROM .vecdb
                                        if let Err(e) =
                                            store.create_collection(&collection.name, config)
                                        {
                                            // Collection might already exist from lazy loading - just load vectors with HNSW
                                            warn!(
                                                "Collection '{}' already exists (maybe from lazy loading), loading vectors with HNSW anyway: {}",
                                                collection.name, e
                                            );
                                            if let Ok(mut collection_ref) =
                                                store.get_collection_mut(&collection.name)
                                            {
                                                info!(
                                                    "🔄 Loading {} vectors with HNSW index into existing collection '{}'...",
                                                    vectors.len(),
                                                    collection.name
                                                );
                                                // Use fast_load_vectors() to build HNSW index properly
                                                if let Err(e) =
                                                    collection_ref.fast_load_vectors(vectors)
                                                {
                                                    warn!(
                                                        "❌ FAILED to load vectors with HNSW into collection '{}': {}",
                                                        collection.name, e
                                                    );
                                                } else {
                                                    // Enable graph for this collection automatically
                                                    if let Err(e) = store
                                                        .enable_graph_for_collection(
                                                            &collection.name,
                                                        )
                                                    {
                                                        warn!(
                                                            "⚠️  Failed to enable graph for collection '{}': {} (continuing anyway)",
                                                            collection.name, e
                                                        );
                                                    } else {
                                                        info!(
                                                            "✅ Graph enabled for collection '{}'",
                                                            collection.name
                                                        );
                                                    }

                                                    info!(
                                                        "✅ Collection '{}' loaded from .vecdb with {} vectors + HNSW index",
                                                        collection.name, vector_count
                                                    );
                                                    indexed_count += 1;
                                                }
                                            }
                                            continue;
                                        }

                                        // Collection created successfully - now load vectors with HNSW index
                                        if let Ok(mut collection_ref) =
                                            store.get_collection_mut(&collection.name)
                                        {
                                            info!(
                                                "🔄 Loading {} vectors with HNSW index into collection '{}'...",
                                                vectors.len(),
                                                collection.name
                                            );
                                            // Use fast_load_vectors() to build HNSW index properly
                                            if let Err(e) =
                                                collection_ref.fast_load_vectors(vectors)
                                            {
                                                warn!(
                                                    "❌ FAILED to load vectors with HNSW into collection '{}': {}",
                                                    collection.name, e
                                                );
                                            } else {
                                                // Enable graph for this collection automatically
                                                if let Err(e) = store
                                                    .enable_graph_for_collection(&collection.name)
                                                {
                                                    warn!(
                                                        "⚠️  Failed to enable graph for collection '{}': {} (continuing anyway)",
                                                        collection.name, e
                                                    );
                                                } else {
                                                    info!(
                                                        "✅ Graph enabled for collection '{}'",
                                                        collection.name
                                                    );
                                                }

                                                info!(
                                                    "✅ Collection '{}' loaded from .vecdb with {} vectors + HNSW index",
                                                    collection.name, vector_count
                                                );
                                                indexed_count += 1;
                                            }
                                        } else {
                                            warn!(
                                                "❌ FAILED to get collection '{}' after creation!",
                                                collection.name
                                            );
                                        }
                                    } else {
                                        debug!(
                                            "Failed to deserialize collection '{}' from .vecdb: no collection found",
                                            collection.name
                                        );
                                    }
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to read collection '{}' from .vecdb: {}",
                                        collection.name, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to create StorageReader for collection '{}': {}",
                                collection.name, e
                            );
                        }
                    }
                } else {
                    info!(
                        "✅ Collection '{}' already in memory, skipping",
                        collection.name
                    );
                }
                continue;
            }

            // Check if collection already exists in store memory (WITHOUT lazy loading)
            if store.has_collection_in_memory(&collection.name) {
                info!(
                    "✅ Collection '{}' already exists in memory, skipping",
                    collection.name
                );
                continue;
            }

            // Get project path
            let project_path = match workspace_manager.get_project_path(&project.name) {
                Ok(path) => path,
                Err(e) => {
                    warn!("Failed to get project path for '{}': {}", project.name, e);
                    continue;
                }
            };

            // Use FileLoader to index files
            let mut loader_config = LoaderConfig {
                max_chunk_size: 2048,
                chunk_overlap: 256,
                include_patterns: collection.processing.include_patterns.clone(),
                exclude_patterns: collection.processing.exclude_patterns.clone(),
                embedding_dimension: collection.embedding.dimension,
                embedding_type: "bm25".to_string(),
                collection_name: collection.name.clone(),
                max_file_size: 1024 * 1024, // 1MB
            };

            // CRITICAL: Always enforce hardcoded exclusions (Python cache, binaries, etc.)
            loader_config.ensure_hardcoded_excludes();

            // Create embedding manager for this collection
            let mut coll_embedding_manager = crate::embedding::EmbeddingManager::new();
            let bm25 = crate::embedding::Bm25Embedding::new(collection.embedding.dimension);
            coll_embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
            coll_embedding_manager.set_default_provider("bm25")?;

            let mut loader =
                FileLoader::with_embedding_manager(loader_config, coll_embedding_manager);

            match loader
                .load_and_index_project(&project_path.to_string_lossy(), &store)
                .await
            {
                Ok(file_count) => {
                    if file_count > 0 {
                        info!(
                            "Indexed {} vectors for collection '{}'",
                            file_count, collection.name
                        );
                        indexed_count += 1;
                    } else {
                        info!(
                            "Collection '{}' already exists in .vecdb, no indexing needed",
                            collection.name
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to index collection '{}': {}", collection.name, e);
                }
            }
        }
    }

    Ok(indexed_count)
}
