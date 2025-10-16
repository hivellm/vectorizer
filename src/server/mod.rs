pub mod mcp_tools;
pub mod mcp_handlers;
mod discovery_handlers;
pub mod rest_handlers;
pub mod file_operations_handlers;

pub use mcp_tools::get_mcp_tools;
pub use mcp_handlers::handle_mcp_tool;

// Re-export main server types from the original implementation
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    Router,
    routing::{get, post, delete},
    extract::State,
    response::Json,
    http::StatusCode,
};
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use tracing::{info, error, warn};
use crate::file_watcher::{FileWatcherMetrics, FileWatcherSystem, MetricsCollector};

/// Global server state to share between endpoints
#[derive(Clone)]
pub struct ServerState {
    pub file_watcher_system: Arc<tokio::sync::Mutex<Option<FileWatcherSystem>>>,
}

use crate::{
    VectorStore,
    embedding::EmbeddingManager,
    workspace::{WorkspaceManager, WorkspaceConfig},
};

/// Vectorizer server state
#[derive(Clone)]
pub struct VectorizerServer {
    pub store: Arc<VectorStore>,
    pub embedding_manager: Arc<EmbeddingManager>,
    pub start_time: std::time::Instant,
    pub file_watcher_system: Arc<tokio::sync::Mutex<Option<crate::file_watcher::FileWatcherSystem>>>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub auto_save_manager: Option<Arc<crate::db::AutoSaveManager>>,
    background_task: Arc<tokio::sync::Mutex<Option<(tokio::task::JoinHandle<()>, tokio::sync::watch::Sender<bool>)>>>,
}

impl VectorizerServer {
    /// Create a new vectorizer server
    pub async fn new() -> anyhow::Result<Self> {
        info!("🔧 Initializing Vectorizer Server...");
        
        // Initialize VectorStore with auto-save enabled
        let vector_store = VectorStore::new_auto();
        let store_arc = Arc::new(vector_store);
        
        info!("🔍 PRE_INIT: Creating embedding manager...");
        let mut embedding_manager = EmbeddingManager::new();
        info!("🔍 PRE_INIT: Creating BM25 embedding...");
        let bm25 = crate::embedding::Bm25Embedding::new(512);
        info!("🔍 PRE_INIT: Registering BM25 provider...");
        embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
        info!("🔍 PRE_INIT: Setting default provider...");
        embedding_manager.set_default_provider("bm25")?;
        info!("✅ PRE_INIT: Embedding manager configured");

        info!("✅ Vectorizer Server initialized successfully - starting background collection loading");
        info!("🔍 STEP 1: Server initialization completed, proceeding to file watcher setup");
        info!("🔍 STEP 1.1: About to initialize file watcher embedding manager...");

        // Initialize file watcher if enabled
        info!("🔍 STEP 2: Initializing file watcher embedding manager...");
        let mut embedding_manager_for_watcher = EmbeddingManager::new();
        let bm25_for_watcher = crate::embedding::Bm25Embedding::new(512);
        embedding_manager_for_watcher.register_provider("bm25".to_string(), Box::new(bm25_for_watcher));
        embedding_manager_for_watcher.set_default_provider("bm25")?;
        info!("✅ STEP 2: File watcher embedding manager initialized");
        
        info!("🔍 STEP 3: Creating Arc wrappers for file watcher components...");
        let embedding_manager_for_watcher_arc = Arc::new(RwLock::new(embedding_manager_for_watcher));
        let file_watcher_arc = embedding_manager_for_watcher_arc.clone();
        let store_for_watcher = store_arc.clone();
        info!("✅ STEP 3: Arc wrappers created successfully");
        
        info!("🔍 STEP 4: Checking if file watcher is enabled...");
        
        // Check if file watcher is enabled in config before starting
        let file_watcher_enabled = std::fs::read_to_string("config.yml")
            .ok()
            .and_then(|content| serde_yaml::from_str::<serde_yaml::Value>(&content).ok())
            .and_then(|config| {
                config.get("file_watcher")
                    .and_then(|fw| fw.get("enabled"))
                    .and_then(|enabled| enabled.as_bool())
            })
            .unwrap_or(false); // Default to disabled if not found
        
        let watcher_system_arc = Arc::new(tokio::sync::Mutex::new(None::<crate::file_watcher::FileWatcherSystem>));
        let watcher_system_for_task = watcher_system_arc.clone();
        let watcher_system_for_server = watcher_system_arc.clone();
        
        if file_watcher_enabled {
            info!("✅ File watcher is ENABLED in config - starting...");
            tokio::task::spawn(async move {
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
            
            // Keep the task alive by waiting indefinitely
            // This ensures the file watcher continues running
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                info!("🔍 File watcher is still running...");
            }
            });
        } else {
            info!("⏭️  File watcher is DISABLED in config - skipping initialization");
        }

        // Create cancellation token for background task
        let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
        
        // Start background collection loading and workspace indexing
        let store_for_loading = store_arc.clone();
        let embedding_manager_for_loading = Arc::new(embedding_manager);
        let watcher_system_for_loading = watcher_system_arc.clone();
        let background_handle = tokio::task::spawn(async move {
            println!("📦 Background task started - loading collections and checking workspace...");
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
                        config.get("workspace")
                            .and_then(|ws| ws.get("auto_load_collections"))
                            .and_then(|enabled| enabled.as_bool())
                    })
                    .unwrap_or(false)
            };
            
            // Load all persisted collections in background
            let persisted_count = if should_auto_load {
                info!("🔍 COLLECTION_LOAD_STEP_1: Auto-load ENABLED - loading all persisted collections...");
                match store_for_loading.load_all_persisted_collections() {
                Ok(count) => {
                    if count > 0 {
                        println!("✅ Background loading completed - {} collections loaded", count);
                        info!("✅ COLLECTION_LOAD_STEP_2: Background loading completed - {} collections loaded", count);
                        
                        // Update file watcher with loaded collections
                        info!("🔍 COLLECTION_LOAD_STEP_3: Updating file watcher with loaded collections...");
                        if let Some(watcher_system) = watcher_system_for_loading.lock().await.as_ref() {
                            let collections = store_for_loading.list_collections();
                            for collection_name in collections {
                                if let Err(e) = watcher_system.update_with_collection(&collection_name).await {
                                    warn!("⚠️ Failed to update file watcher with collection '{}': {}", collection_name, e);
                                } else {
                                    info!("✅ Updated file watcher with collection: {}", collection_name);
                                }
                            }
                            
                            // Discover and index existing files after collections are loaded
                            info!("🔍 COLLECTION_LOAD_STEP_4: Starting file discovery for existing files...");
                            match watcher_system.discover_existing_files().await {
                                Ok(result) => {
                                    info!("✅ File discovery completed: {} files indexed, {} skipped, {} errors", 
                                          result.stats.files_indexed, result.stats.files_skipped, result.stats.files_errors);
                                }
                                Err(e) => {
                                    warn!("⚠️ File discovery failed: {}", e);
                                }
                            }
                            
                            // Sync with collections to remove orphaned files
                            info!("🔍 COLLECTION_LOAD_STEP_5: Starting collection sync...");
                            match watcher_system.sync_with_collections().await {
                                Ok(result) => {
                                    info!("✅ Collection sync completed: {} orphaned files removed", 
                                          result.stats.orphaned_files_removed);
                                }
                                Err(e) => {
                                    warn!("⚠️ Collection sync failed: {}", e);
                                }
                            }
                        } else {
                            warn!("⚠️ File watcher not available for update");
                        }
                        
                        count
                    } else {
                        println!("ℹ️  Background loading completed - no persisted collections found");
                        info!("✅ COLLECTION_LOAD_STEP_2: Background loading completed - no persisted collections found");
                        
                        // Even with no persisted collections, try to discover existing files
                        info!("🔍 COLLECTION_LOAD_STEP_3: No persisted collections, attempting conservative file discovery...");
                        
                        // Wait for file watcher to be available (with timeout)
                        let mut attempts = 0;
                        let max_attempts = 10; // Conservative timeout
                        
                        loop {
                            if let Some(watcher_system) = watcher_system_for_loading.lock().await.as_ref() {
                                info!("🔍 COLLECTION_LOAD_STEP_4: Starting conservative file discovery...");
                                match watcher_system.discover_existing_files().await {
                                    Ok(result) => {
                                        info!("✅ File discovery completed: {} files indexed, {} skipped, {} errors", 
                                              result.stats.files_indexed, result.stats.files_skipped, result.stats.files_errors);
                                    }
                                    Err(e) => {
                                        warn!("⚠️ File discovery failed: {}", e);
                                    }
                                }
                                
                                // Perform comprehensive synchronization
                                info!("🔍 COLLECTION_LOAD_STEP_5: Starting comprehensive synchronization...");
                                let sync_start = std::time::Instant::now();
                                match watcher_system.comprehensive_sync().await {
                                    Ok((sync_result, unindexed_files)) => {
                                        let sync_time_ms = sync_start.elapsed().as_millis() as u64;
                                        
                                        // Record sync metrics
                                        watcher_system.record_sync(
                                            sync_result.stats.orphaned_files_removed as u64,
                                            unindexed_files.len() as u64,
                                            sync_time_ms
                                        ).await;
                                        
                                        info!("✅ Comprehensive sync completed: {} orphaned files removed, {} unindexed files detected", 
                                              sync_result.stats.orphaned_files_removed, unindexed_files.len());
                                        
                                        if !unindexed_files.is_empty() {
                                            info!("📄 Unindexed files detected: {:?}", unindexed_files);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("⚠️ Comprehensive sync failed: {}", e);
                                        watcher_system.record_error("sync_error", &e.to_string()).await;
                                    }
                                }
                                
                                break;
                            } else {
                                attempts += 1;
                                if attempts >= max_attempts {
                                    warn!("⚠️ File watcher not available after {} seconds, skipping discovery", max_attempts);
                                    break;
                                }
                                info!("⏳ Waiting for file watcher to be available... (attempt {}/{})", attempts, max_attempts);
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            }
                        }
                        
                        0
                    }
                },
                Err(e) => {
                    println!("⚠️  Failed to load persisted collections in background: {}", e);
                    warn!("⚠️  Failed to load persisted collections in background: {}", e);
                    0
                }
                }
            } else {
                println!("⏭️  Auto-load DISABLED - collections will be loaded on first access (lazy loading)");
                info!("⏭️  Auto-load DISABLED - collections will be loaded on first access (lazy loading)");
                0
            };

            // Check for cancellation before workspace loading
            if *cancel_rx.borrow() {
                info!("Background task cancelled before workspace loading");
                return;
            }
            
            // Check for workspace configuration and reindex if needed
            // (data_dir and vecdb_path already declared above)
            
            match load_workspace_collections(&store_for_loading, &embedding_manager_for_loading, cancel_rx.clone()).await {
                Ok(workspace_count) => {
        if workspace_count > 0 {
            println!("✅ Workspace loading completed - {} collections indexed/loaded", workspace_count);
            info!("✅ Workspace loading completed - {} collections indexed/loaded", workspace_count);
            
            // Check if there are .bin files created during indexing
            use crate::storage::StorageCompactor;
            let compactor = StorageCompactor::new(&data_dir, 6, 1000);
            
            // Count .bin files to see if we need to compact
            let bin_count = std::fs::read_dir(&data_dir)
                .ok()
                .map(|entries| entries.filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("bin"))
                    .count())
                .unwrap_or(0);
            
            if bin_count > 0 {
                println!("📦 Found {} .bin files - compacting to vectorizer.vecdb from memory...", bin_count);
                info!("📦 Found {} .bin files - compacting to vectorizer.vecdb from memory...", bin_count);
                info!("🔍 DEBUG: bin_count = {}, workspace_count = {}", bin_count, workspace_count);
                info!("🔍 DEBUG: data_dir = {}", data_dir.display());
                
                info!("🔍 DEBUG: Starting compact_from_memory...");
                
                // Compact directly FROM MEMORY (no raw files needed)
                match compactor.compact_from_memory(&store_for_loading) {
                    Ok(index) => {
                        println!("✅ Compaction complete:");
                        println!("   Collections: {}", index.collection_count());
                        println!("   Total vectors: {}", index.total_vectors());
                        println!("   Compressed size: {} MB", index.compressed_size / 1_048_576);
                        println!("   🗑️  vectorizer.vecdb created from memory - no raw files needed");
                        
                        info!("✅ First compaction complete - created vectorizer.vecdb from memory");
                        info!("   Collections: {}, Vectors: {}", index.collection_count(), index.total_vectors());
                        info!("   Only vectorizer.vecdb and vectorizer.vecidx exist");
                        
                        // Verify the file was created
                        if vecdb_path.exists() {
                            let metadata = std::fs::metadata(&vecdb_path).unwrap();
                            println!("   📊 vectorizer.vecdb size: {} MB", metadata.len() / 1_048_576);
                            info!("   📊 vectorizer.vecdb size: {} bytes", metadata.len());
                        } else {
                            error!("❌ CRITICAL: vectorizer.vecdb was NOT created!");
                        }
                        
                        // Remove any temporary .bin files that might have been created during indexing
                        match compactor.remove_raw_files() {
                            Ok(count) if count > 0 => {
                                println!("   🗑️  Removed {} temporary raw files", count);
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
                        println!("❌ Compaction failed: {}", e);
                        error!("❌ Compaction from memory failed: {}", e);
                        error!("   Error details: {:?}", e);
                        error!("   Will retry on next startup");
                    }
                }
            } else {
                // No .bin files - either loaded from .vecdb or nothing to compact
                println!("ℹ️  No .bin files found - skipping compaction");
                info!("ℹ️  No .bin files found - vectorizer.vecdb is up to date");
            }
                    } else {
                        println!("ℹ️  All collections already exist - no indexing needed");
                        info!("ℹ️  All collections already exist - no indexing needed");
                    }
                },
                Err(e) => {
                    println!("⚠️  Failed to process workspace: {}", e);
                    warn!("⚠️  Failed to process workspace: {}", e);
                }
            }
            
            // NOW enable auto-save after all collections are loaded
            println!("🔄 Enabling auto-save after successful initialization");
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
        let _auto_save_handle = auto_save_manager.start();
        info!("✅ AutoSaveManager started (5min save + 1h snapshot intervals)");

        Ok(Self {
            store: store_arc,
            embedding_manager: Arc::new(final_embedding_manager),
            start_time: std::time::Instant::now(),
            file_watcher_system: watcher_system_for_server,
            metrics_collector: Arc::new(MetricsCollector::new()),
            auto_save_manager: Some(auto_save_manager),
            background_task: Arc::new(tokio::sync::Mutex::new(Some((background_handle, cancel_tx)))),
        })
    }
    
    /// Start the server
    pub async fn start(&self, host: &str, port: u16) -> anyhow::Result<()> {
        info!("🚀 Starting Vectorizer Server on {}:{}", host, port);

        // Create server state for metrics endpoint
        let server_state = ServerState {
            file_watcher_system: self.file_watcher_system.clone(),
        };

        // Create MCP router (main server) using StreamableHTTP transport
        info!("🔧 Creating MCP router with StreamableHTTP transport (rmcp 0.8.1)...");
        let mcp_router = self.create_mcp_router().await;
        info!("✅ MCP router created (StreamableHTTP)");

        // Create REST API router to add to MCP
        let metrics_collector_1 = self.metrics_collector.clone();
        let metrics_router = Router::new()
            .route("/metrics", get(get_file_watcher_metrics))
            .with_state(Arc::new(server_state))
            .layer(axum::middleware::from_fn(move |req: axum::extract::Request, next: axum::middleware::Next| {
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
            }));
        
        let metrics_collector_2 = self.metrics_collector.clone();
        let rest_routes = Router::new()
            // Health and stats
            .route("/health", get(rest_handlers::health_check))
            .route("/stats", get(rest_handlers::get_stats))
            .route("/indexing/progress", get(rest_handlers::get_indexing_progress))
            
            // GUI-specific endpoints
            .route("/api/status", get(rest_handlers::get_status))
            .route("/api/logs", get(rest_handlers::get_logs))
            .route("/api/collections/{name}/force-save", post(rest_handlers::force_save_collection))
            .route("/api/workspace/add", post(rest_handlers::add_workspace))
            .route("/api/workspace/remove", post(rest_handlers::remove_workspace))
            .route("/api/workspace/list", get(rest_handlers::list_workspaces))
            .route("/api/workspace/config", get(rest_handlers::get_workspace_config))
            .route("/api/workspace/config", post(rest_handlers::update_workspace_config))
            .route("/api/config", get(rest_handlers::get_config))
            .route("/api/config", post(rest_handlers::update_config))
            .route("/admin/restart", post(rest_handlers::restart_server))
            .route("/api/backups", get(rest_handlers::list_backups))
            .route("/api/backups/create", post(rest_handlers::create_backup))
            .route("/api/backups/restore", post(rest_handlers::restore_backup))
            .route("/api/backups/directory", get(rest_handlers::get_backup_directory))
            
            // Collection management
            .route("/collections", get(rest_handlers::list_collections))
            .route("/collections", post(rest_handlers::create_collection))
            .route("/collections/{name}", get(rest_handlers::get_collection))
            .route("/collections/{name}", delete(rest_handlers::delete_collection))
            
            // Vector operations - single
            .route("/search", post(rest_handlers::search_vectors))
            .route("/collections/{name}/search", post(rest_handlers::search_vectors))
            .route("/collections/{name}/search/text", post(rest_handlers::search_vectors_by_text))
            .route("/collections/{name}/search/file", post(rest_handlers::search_by_file))
            .route("/insert", post(rest_handlers::insert_text))
            .route("/update", post(rest_handlers::update_vector))
            .route("/delete", post(rest_handlers::delete_vector))
            .route("/embed", post(rest_handlers::embed_text))
            .route("/vector", post(rest_handlers::get_vector))
            .route("/collections/{name}/vectors", get(rest_handlers::list_vectors))
            .route("/collections/{name}/vectors/{id}", get(rest_handlers::get_vector))
            .route("/collections/{name}/vectors/{id}", delete(rest_handlers::delete_vector))
            
            // Vector operations - batch
            .route("/batch_insert", post(rest_handlers::batch_insert_texts))
            .route("/insert_texts", post(rest_handlers::insert_texts))
            .route("/batch_search", post(rest_handlers::batch_search_vectors))
            .route("/batch_update", post(rest_handlers::batch_update_vectors))
            .route("/batch_delete", post(rest_handlers::batch_delete_vectors))
            
            // Intelligent search routes
            .route("/intelligent_search", post(rest_handlers::intelligent_search))
            .route("/multi_collection_search", post(rest_handlers::multi_collection_search))
            .route("/semantic_search", post(rest_handlers::semantic_search))
            .route("/contextual_search", post(rest_handlers::contextual_search))
            
            // Discovery routes
            .route("/discover", post(rest_handlers::discover))
            .route("/discovery/filter_collections", post(rest_handlers::filter_collections))
            .route("/discovery/score_collections", post(rest_handlers::score_collections))
            .route("/discovery/expand_queries", post(rest_handlers::expand_queries))
            .route("/discovery/broad_discovery", post(rest_handlers::broad_discovery))
            .route("/discovery/semantic_focus", post(rest_handlers::semantic_focus))
            .route("/discovery/promote_readme", post(rest_handlers::promote_readme))
            .route("/discovery/compress_evidence", post(rest_handlers::compress_evidence))
            .route("/discovery/build_answer_plan", post(rest_handlers::build_answer_plan))
            .route("/discovery/render_llm_prompt", post(rest_handlers::render_llm_prompt))
            
            // File Operations routes
            .route("/file/content", post(rest_handlers::get_file_content))
            .route("/file/list", post(rest_handlers::list_files_in_collection))
            .route("/file/summary", post(rest_handlers::get_file_summary))
            .route("/file/chunks", post(rest_handlers::get_file_chunks_ordered))
            .route("/file/outline", post(rest_handlers::get_project_outline))
            .route("/file/related", post(rest_handlers::get_related_files))
            .route("/file/search_by_type", post(rest_handlers::search_by_file_type))
            
            // Dashboard - serve static files
            .nest_service("/dashboard", ServeDir::new("dashboard"))
            .fallback_service(ServeDir::new("dashboard"))
            
            .layer(CorsLayer::permissive())
            .layer(axum::middleware::from_fn(move |req: axum::extract::Request, next: axum::middleware::Next| {
                let metrics = metrics_collector_2.clone();
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
            }))
            .with_state(self.clone());

        // Create UMICP state
        let umicp_state = crate::umicp::UmicpState {
            store: self.store.clone(),
            embedding_manager: self.embedding_manager.clone(),
        };
        
        // Create UMICP routes (needs custom state)
        let umicp_routes = Router::new()
            .route("/umicp", post(crate::umicp::transport::umicp_handler))
            .route("/umicp/health", get(crate::umicp::health_check))
            .with_state(umicp_state);
        
        // Merge all routes - UMICP first so it doesn't get masked
        let app = Router::new()
            .merge(umicp_routes)
            .merge(mcp_router)
            .merge(rest_routes)
            .merge(metrics_router);

        info!("🌐 Vectorizer Server available at:");
        info!("   📡 MCP StreamableHTTP: http://{}:{}/mcp", host, port);
        info!("   🔌 REST API: http://{}:{}", host, port);
        info!("   🔗 UMICP: http://{}:{}/umicp", host, port);
        info!("   📊 Dashboard: http://{}:{}/", host, port);

        // Bind and start the server
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
        info!("✅ MCP server (StreamableHTTP) with REST API listening on {}:{}", host, port);
        
        // Set up graceful shutdown
        let shutdown_signal = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
            info!("🛑 Received shutdown signal (Ctrl+C)");
        };
        
        // Serve the application with graceful shutdown
        let server_handle = axum::serve(listener, app);
        
        // Wait for either the server to complete or shutdown signal
        tokio::select! {
            result = server_handle => {
                match result {
                    Ok(_) => info!("✅ Server completed normally"),
                    Err(e) => error!("❌ Server error: {}", e),
                }
            }
            _ = shutdown_signal => {
                info!("🛑 Shutdown signal received, stopping server...");
            }
        }
        
        // Cancel and await background collection loading task if still running
        info!("🛑 Stopping background collection loading task...");
        if let Some((handle, cancel_tx)) = self.background_task.lock().await.take() {
            // Send cancellation signal
            let _ = cancel_tx.send(true);
            info!("📤 Sent cancellation signal to background task");
            
            // CRITICAL: Wait for task to finish gracefully (with timeout)
            // This ensures all collections are fully loaded before shutdown/compaction
            info!("⏳ Waiting for background task to complete (max 10 seconds)...");
            let timeout_duration = tokio::time::Duration::from_secs(10);
            match tokio::time::timeout(timeout_duration, handle).await {
                Ok(_) => {
                    info!("✅ Background task completed gracefully");
                }
                Err(_) => {
                    warn!("⚠️  Background task did not complete within timeout - some collections may be incomplete");
                }
            }
        }
        
        // Force final save before shutdown
        if let Some(auto_save) = &self.auto_save_manager {
            info!("💾 Performing final save before shutdown...");
            match auto_save.force_save().await {
                Ok(_) => info!("✅ Final save completed successfully"),
                Err(e) => warn!("⚠️  Final save failed: {}", e),
            }
            auto_save.shutdown();
        }
        
        // Graceful shutdown of file watcher
        info!("🛑 Starting graceful shutdown of file watcher...");
        if let Some(watcher_system) = self.file_watcher_system.lock().await.as_ref() {
            if let Err(e) = watcher_system.stop().await {
                error!("❌ Failed to stop file watcher gracefully: {}", e);
            } else {
                info!("✅ File watcher stopped gracefully");
            }
        }
        
        info!("✅ Server stopped gracefully");
        Ok(())
    }

    /// Create MCP router with StreamableHTTP transport (rmcp 0.8.1)
    async fn create_mcp_router(&self) -> Router {
        use rmcp::transport::streamable_http_server::{
            StreamableHttpService, session::local::LocalSessionManager,
        };
        use hyper_util::service::TowerToHyperService;
        use hyper::service::Service;
        use std::sync::Arc;
        
        // Create MCP service handler
        let store = self.store.clone();
        let embedding_manager = self.embedding_manager.clone();
        
        // Create StreamableHTTP service
        let streamable_service = StreamableHttpService::new(
            move || {
                Ok(VectorizerMcpService {
                    store: store.clone(),
                    embedding_manager: embedding_manager.clone(),
                })
            },
            LocalSessionManager::default().into(),
            Default::default(),
        );
        
        // Convert to axum service and create router
        let hyper_service = TowerToHyperService::new(streamable_service);
        
        // Create router with the MCP endpoint
        Router::new()
            .route("/mcp", axum::routing::any(move |req: axum::extract::Request| {
                let mut service = hyper_service.clone();
                async move {
                    // Forward request to hyper service
                    match service.call(req).await {
                        Ok(response) => Ok(response),
                        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
                    }
                }
            }))
    }
}

/// Get File Watcher metrics endpoint
/// Get File Watcher metrics endpoint
pub async fn get_file_watcher_metrics(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<FileWatcherMetrics>, (StatusCode, String)> {
    // Get the file watcher system from the state
    let watcher_lock = state.file_watcher_system.lock().await;
    
    if let Some(watcher_system) = watcher_lock.as_ref() {
        let metrics = watcher_system.get_metrics().await;
        return Ok(Json(metrics));
    }
    
    // Return empty/default metrics if File Watcher is not available
    use crate::file_watcher::metrics::*;
    use std::collections::HashMap;
    
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
}

impl rmcp::ServerHandler for VectorizerMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        use rmcp::model::{ServerInfo, ProtocolVersion, ServerCapabilities, Implementation};
        
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
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, rmcp::model::ErrorData>> + Send + '_ {
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
    ) -> impl std::future::Future<Output = Result<rmcp::model::CallToolResult, rmcp::model::ErrorData>> + Send + '_ {
        async move {
            mcp_handlers::handle_mcp_tool(
                request,
                self.store.clone(),
                self.embedding_manager.clone(),
            ).await
        }
    }

    fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListResourcesResult, rmcp::model::ErrorData>> + Send + '_ {
        async move {
            use rmcp::model::ListResourcesResult;
            Ok(ListResourcesResult {
                resources: vec![],
                next_cursor: None,
            })
        }
    }

}

/// Load file watcher configuration from vectorize-workspace.yml
async fn load_file_watcher_config() -> anyhow::Result<crate::file_watcher::FileWatcherConfig> {
    match crate::file_watcher::FileWatcherConfig::from_yaml_file("vectorize-workspace.yml") {
        Ok(config) => {
            info!("Loaded file watcher configuration from workspace: watch_paths={:?}, exclude_patterns={:?}", 
                  config.watch_paths, config.exclude_patterns);
            Ok(config)
        }
        Err(e) => {
            info!("Failed to load workspace configuration: {}, using default file watcher config", e);
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
    use crate::workspace::manager::WorkspaceManager;
    use crate::file_loader::{FileLoader, LoaderConfig};

    // Look for workspace configuration file
    let workspace_file = Path::new("vectorize-workspace.yml");
    info!("Checking for workspace file at: {}", workspace_file.display());
    if !workspace_file.exists() {
        info!("No workspace configuration file found at vectorize-workspace.yml");
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

    info!("Workspace loaded with {} projects", workspace_manager.config().projects.len());

    let mut indexed_count = 0;

    // Check if using .vecdb format - if so, verify collections in archive first
    let data_dir = std::path::PathBuf::from("./data");
    let vecdb_path = data_dir.join("vectorizer.vecdb");
    let using_vecdb = vecdb_path.exists();
    
    let mut existing_in_vecdb = std::collections::HashSet::new();
    if using_vecdb {
        use crate::storage::StorageReader;
        info!("🔍 .vecdb file exists at {}, attempting to read...", vecdb_path.display());
        match StorageReader::new(&data_dir) {
            Ok(reader) => {
                match reader.list_collections() {
                    Ok(collections) => {
                        existing_in_vecdb = collections.iter().cloned().collect();
                        info!("📦 Found {} collections in .vecdb archive: {:?}", existing_in_vecdb.len(), existing_in_vecdb);
                    }
                    Err(e) => {
                        warn!("Failed to list collections from .vecdb: {}", e);
                    }
                }
            }
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
                    info!("📥 FORCE LOADING collection '{}' from .vecdb into memory...", collection.name);
                    
                    // Use the SAME method as when .vecdb doesn't exist - load directly from .vecdb
                    use crate::storage::StorageReader;
                    match StorageReader::new(&std::path::PathBuf::from("./data")) {
                        Ok(reader) => {
                            let vector_store_path = format!("{}_vector_store.bin", collection.name);
                            match reader.read_file(&vector_store_path) {
                                Ok(data) => {
                                    match serde_json::from_slice::<crate::persistence::PersistedCollection>(&data) {
                                        Ok(persisted) => {
                                            // Use EXACT config from .vecdb (not workspace config!)
                                            let config = persisted.config.clone().unwrap_or_else(|| {
                                                warn!("⚠️  Collection '{}' has no config in .vecdb, using default", collection.name);
                                                crate::models::CollectionConfig::default()
                                            });
                                            let vector_count = persisted.vectors.len();
                                            
                                            info!("📥 Loading collection '{}' from .vecdb with {} vectors...", collection.name, vector_count);
                                            
                                            // Convert vectors FIRST (before creating collection)
                                            info!("🔄 Converting {} persisted vectors to runtime format...", persisted.vectors.len());
                                            let vectors: Vec<crate::models::Vector> = persisted.vectors
                                                .into_iter()
                                                .filter_map(|pv| {
                                                    match pv.into_runtime() {
                                                        Ok(v) => Some(v),
                                                        Err(e) => {
                                                            warn!("Failed to convert persisted vector: {}", e);
                                                            None
                                                        }
                                                    }
                                                })
                                                .collect();
                                            
                                            info!("🔄 Converted {} vectors successfully", vectors.len());
                                            
                                            // Create collection with config FROM .vecdb
                                            if let Err(e) = store.create_collection(&collection.name, config) {
                                                // Collection might already exist from lazy loading - just load vectors with HNSW
                                                warn!("Collection '{}' already exists (maybe from lazy loading), loading vectors with HNSW anyway: {}", collection.name, e);
                                                if let Ok(collection_ref) = store.get_collection(&collection.name) {
                                                    info!("🔄 Loading {} vectors with HNSW index into existing collection '{}'...", vectors.len(), collection.name);
                                                    // Use fast_load_vectors() to build HNSW index properly
                                                    if let Err(e) = collection_ref.fast_load_vectors(vectors) {
                                                        warn!("❌ FAILED to load vectors with HNSW into collection '{}': {}", collection.name, e);
                                                    } else {
                                                        info!("✅ Collection '{}' loaded from .vecdb with {} vectors + HNSW index", collection.name, vector_count);
                                                        indexed_count += 1;
                                                    }
                                                }
                                                continue;
                                            }
                                            
                                            // Collection created successfully - now load vectors with HNSW index
                                            if let Ok(collection_ref) = store.get_collection(&collection.name) {
                                                info!("🔄 Loading {} vectors with HNSW index into collection '{}'...", vectors.len(), collection.name);
                                                // Use fast_load_vectors() to build HNSW index properly
                                                if let Err(e) = collection_ref.fast_load_vectors(vectors) {
                                                    warn!("❌ FAILED to load vectors with HNSW into collection '{}': {}", collection.name, e);
                                                } else {
                                                    info!("✅ Collection '{}' loaded from .vecdb with {} vectors + HNSW index", collection.name, vector_count);
                                                    indexed_count += 1;
                                                }
                                            } else {
                                                warn!("❌ FAILED to get collection '{}' after creation!", collection.name);
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to deserialize collection '{}' from .vecdb: {}", collection.name, e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to read collection '{}' from .vecdb: {}", collection.name, e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to create StorageReader for collection '{}': {}", collection.name, e);
                        }
                    }
                } else {
                    info!("✅ Collection '{}' already in memory, skipping", collection.name);
                }
                continue;
            }
            
            // Check if collection already exists in store memory (WITHOUT lazy loading)
            if store.has_collection_in_memory(&collection.name) {
                info!("✅ Collection '{}' already exists in memory, skipping", collection.name);
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

            let mut loader = FileLoader::with_embedding_manager(loader_config, coll_embedding_manager);

            match loader.load_and_index_project(&project_path.to_string_lossy(), &store).await {
                Ok(file_count) => {
                    if file_count > 0 {
                        info!("Indexed {} vectors for collection '{}'", file_count, collection.name);
                        indexed_count += 1;
                    } else {
                        info!("Collection '{}' already exists in .vecdb, no indexing needed", collection.name);
                    }
                },
                Err(e) => {
                    warn!("Failed to index collection '{}': {}", collection.name, e);
                }
            }
        }
    }

    Ok(indexed_count)
}
