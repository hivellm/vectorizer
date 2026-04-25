//! Server bootstrap — `VectorizerServer::new` and
//! [`VectorizerServer::new_with_root_config`].
//!
//! Runs once at process start and:
//! 1. Initializes monitoring / telemetry
//! 2. Creates the `VectorStore` (with auto-save enabled)
//! 3. Optionally runs startup cleanup of empty collections
//! 4. Configures embedding managers (server-side + file-watcher side)
//! 5. Starts the file watcher task (if enabled in config)
//! 6. Spawns a background task that loads persisted collections and
//!    indexes the workspace (compacting raw files into `vectorizer.vecdb`
//!    once loaded)
//! 7. Initializes auto-save + system metrics + query cache
//! 8. Brings up cluster / replication / Raft / HA when configured
//! 9. Resolves `max_request_size_mb` from config
//! 10. Initializes auth (optional JWT secret generation on first boot)
//! 11. Initializes HiveHub (and its backup + MCP gateway companions)
//! 12. Assembles the final `VectorizerServer` struct
//!
//! Each step logs progress with the `🔍 STEP N` pattern used by
//! operators to pinpoint where boot hangs on a troubled deployment.

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use vectorizer::VectorStore;
use vectorizer::embedding::{EmbeddingManager, EmbeddingProvider};
use vectorizer::file_watcher::MetricsCollector;

use super::workspace_loader::{load_file_watcher_config, load_workspace_collections};
use crate::server::{AuthHandlerState, RootUserConfig, VectorizerServer};

/// Parse `embedding.model` from the top-level of `config.yml` and return
/// the canonical provider name + dimension. Falls back to `"bm25"` when
/// the field is missing, empty, or the config file can't be read.
///
/// Recognized values:
///   - `"bm25"` (default) — handled by the caller via `Bm25Embedding::new(dim)`
///   - `"fastembed:<model-id>"` — resolved via
///     `vectorizer::embedding::providers::fastembed::parse_model_id`
///
/// An unknown prefix / unresolvable fastembed id returns `Err` so boot
/// fails fast instead of silently falling back to BM25.
fn resolve_embedding_model_name(config_path: &str) -> anyhow::Result<String> {
    let Ok(content) = std::fs::read_to_string(config_path) else {
        return Ok("bm25".to_string());
    };
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
        return Ok("bm25".to_string());
    };
    let model = value
        .get("embedding")
        .and_then(|e| e.get("model"))
        .and_then(|m| m.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "bm25".to_string());
    Ok(model)
}

/// Build the default embedding provider for a fresh `EmbeddingManager`
/// according to `config.embedding.model`. Returns `(name, dimension,
/// boxed_provider)`.
///
/// The server bootstrap calls this three times — once for the main
/// server, once for the file-watcher embedding manager, once for the
/// "final" embedding manager the `VectorizerServer` struct holds. All
/// three must point at the same provider shape so text indexed by the
/// file watcher lands in the same embedding space as text indexed via
/// `POST /insert`.
fn build_default_provider(
    config_path: &str,
) -> anyhow::Result<(String, usize, Box<dyn EmbeddingProvider>)> {
    let model = resolve_embedding_model_name(config_path)?;

    if let Some(fastembed_id) = model.strip_prefix("fastembed:") {
        let cache_dir = vectorizer_core::paths::data_dir().join("fastembed");
        let provider = vectorizer::embedding::providers::try_build_fastembed_provider(
            fastembed_id,
            cache_dir.clone(),
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;
        let dim = provider.dimension();
        let name = format!("fastembed:{}", fastembed_id);
        info!(
            "🧠 Embedding provider: {} (fastembed, dim={}, cache_dir={})",
            name,
            dim,
            cache_dir.display()
        );
        return Ok((name, dim, provider));
    }

    if model == "bm25" {
        info!("🧠 Embedding provider: bm25 (default, dim=512)");
        return Ok((
            "bm25".to_string(),
            512,
            Box::new(vectorizer::embedding::Bm25Embedding::new(512)),
        ));
    }

    Err(anyhow::anyhow!(
        "Unknown embedding model '{}'. Supported prefixes: \"bm25\" (default), \
         \"fastembed:<model-id>\" (requires the fastembed Cargo feature at \
         compile time). See docs/specs/EMBEDDING.md for the full matrix.",
        model
    ))
}

impl VectorizerServer {
    /// Create a new vectorizer server
    pub async fn new() -> anyhow::Result<Self> {
        Self::new_with_root_config(RootUserConfig::default()).await
    }

    /// Create a new vectorizer server with root user configuration
    pub async fn new_with_root_config(root_config: RootUserConfig) -> anyhow::Result<Self> {
        info!("🔧 Initializing Vectorizer Server...");

        // Surface which SIMD backend the dispatcher picked for this
        // process so operators can confirm the binary is using the
        // expected vector instructions (avx2/avx512/neon/sve/wasm128/
        // scalar). Logged once because dispatch caches via OnceLock.
        info!(
            "⚙️  SIMD backend selected: {}",
            vectorizer_core::simd::selected_backend_name()
        );

        // Fail fast on a desynced capability registry: a duplicate id, a
        // duplicate (method, path), or a Both/RestOnly/McpOnly mismatch
        // would silently desync the MCP tool list and the REST router.
        // See `crate::server::capabilities`.
        crate::server::capabilities::assert_inventory_invariants()
            .map_err(|e| anyhow::anyhow!("capability registry invariant violation: {}", e))?;

        // Get config path from root_config or use the layout default.
        //
        // phase4_consolidate-repo-layout moved the canonical config
        // into `config/config.yml`. The legacy `./config.yml` path is
        // still honoured for one release with a one-shot deprecation
        // warning so operators with a pre-v3.x layout aren't broken
        // by the move; the shim disappears in v3.1.
        let config_path = root_config.config_path.unwrap_or_else(|| {
            let canonical = std::path::Path::new("config/config.yml");
            let legacy = std::path::Path::new("config.yml");
            if canonical.exists() {
                "config/config.yml".to_string()
            } else if legacy.exists() {
                tracing::warn!(
                    "Loading config from legacy path ./config.yml — this fallback is \
                     scheduled for removal in v3.1. Move your config to ./config/config.yml \
                     to silence this warning."
                );
                "config.yml".to_string()
            } else {
                "config/config.yml".to_string()
            }
        });

        // Layered config loader (`base → mode → env → CLI`). When the
        // operator sets `VECTORIZER_MODE=production` (or `dev`,
        // `cluster`, `hub`), this validates the merged config eagerly
        // so a typo in the override surfaces at boot rather than
        // randomly later. The base + every mode override file already
        // pass the strict serde schema, so a successful merge is the
        // signal we need.
        if let Some(mode) = vectorizer::config::layered::mode_from_env() {
            match vectorizer::config::layered::load_layered(
                std::path::Path::new(&config_path),
                vectorizer::config::layered::LayeredOptions {
                    mode: Some(mode.clone()),
                    modes_dir: None,
                },
            ) {
                Ok(_) => info!(
                    "📑 VECTORIZER_MODE={} — config validated through the layered loader",
                    mode
                ),
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "VECTORIZER_MODE={} requested but the merged config did not validate: {}",
                        mode,
                        e
                    ));
                }
            }
        }

        // Initialize monitoring system
        if let Err(e) = vectorizer::monitoring::init() {
            warn!("Failed to initialize monitoring system: {}", e);
        }

        // Try to initialize OpenTelemetry (optional, graceful degradation)
        if let Err(e) = vectorizer::monitoring::telemetry::try_init("vectorizer", None) {
            warn!("OpenTelemetry not available: {}", e);
        }

        // Initialize VectorStore with auto-save enabled
        let vector_store = VectorStore::new_auto();
        let store_arc = Arc::new(vector_store);

        // Check if we should cleanup empty collections on startup
        let should_cleanup = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| {
                serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content).ok()
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
        let (provider_name, _provider_dim, provider) = build_default_provider(&config_path)?;
        info!(
            "🔍 PRE_INIT: Registering '{}' provider (dim {})",
            provider_name, _provider_dim
        );
        embedding_manager.register_provider(provider_name.clone(), provider);
        embedding_manager.set_default_provider(&provider_name)?;
        info!("✅ PRE_INIT: Embedding manager configured");

        info!(
            "✅ Vectorizer Server initialized successfully - starting background collection loading"
        );
        info!("🔍 STEP 1: Server initialization completed, proceeding to file watcher setup");
        info!("🔍 STEP 1.1: About to initialize file watcher embedding manager...");

        // Initialize file watcher if enabled
        info!("🔍 STEP 2: Initializing file watcher embedding manager...");
        let mut embedding_manager_for_watcher = EmbeddingManager::new();
        let (watcher_provider_name, _watcher_dim, watcher_provider) =
            build_default_provider(&config_path)?;
        embedding_manager_for_watcher
            .register_provider(watcher_provider_name.clone(), watcher_provider);
        embedding_manager_for_watcher.set_default_provider(&watcher_provider_name)?;
        info!(
            "✅ STEP 2: File watcher embedding manager initialized with '{}'",
            watcher_provider_name
        );

        info!("🔍 STEP 3: Creating Arc wrappers for file watcher components...");
        let embedding_manager_for_watcher_arc =
            Arc::new(RwLock::new(embedding_manager_for_watcher));
        let file_watcher_arc = embedding_manager_for_watcher_arc.clone();
        let store_for_watcher = store_arc.clone();
        info!("✅ STEP 3: Arc wrappers created successfully");

        info!("🔍 STEP 4: Checking if file watcher is enabled...");

        // Load cluster config for file watcher check
        let cluster_config_for_watcher = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| {
                serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content)
                    .ok()
                    .map(|config| config.cluster)
            })
            .unwrap_or_default();

        // Check if file watcher is enabled in config before starting
        // Also check if cluster mode requires file watcher to be disabled
        let file_watcher_enabled = std::fs::read_to_string(&config_path)
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
            None::<vectorizer::file_watcher::FileWatcherSystem>,
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
                    vectorizer::file_watcher::FileWatcherConfig::default()
                });

                let mut watcher_system = vectorizer::file_watcher::FileWatcherSystem::new(
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
        let config_path_for_background = config_path.clone();
        let background_handle = tokio::task::spawn(async move {
            let config_path = config_path_for_background;
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
                std::fs::read_to_string(&config_path)
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
                        use vectorizer::storage::StorageCompactor;
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
                                        match std::fs::metadata(&vecdb_path) {
                                            Ok(metadata) => info!(
                                                "   📊 vectorizer.vecdb size: {} bytes",
                                                metadata.len()
                                            ),
                                            Err(e) => warn!(
                                                "   ⚠️  Could not stat vectorizer.vecdb: {}",
                                                e
                                            ),
                                        }
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
            // Swallow the computed count to preserve the original log flow;
            // operators read it in `load_all_persisted_collections` above.
            let _ = persisted_count;
        });

        // Create final embedding manager for the server struct
        let mut final_embedding_manager = EmbeddingManager::new();
        let (final_provider_name, _final_dim, final_provider) =
            build_default_provider(&config_path)?;
        final_embedding_manager.register_provider(final_provider_name.clone(), final_provider);
        final_embedding_manager.set_default_provider(&final_provider_name)?;

        // Initialize AutoSaveManager (5min save + 1h snapshot intervals)
        info!("🔄 Initializing AutoSaveManager...");
        let auto_save_manager =
            Arc::new(vectorizer::db::AutoSaveManager::new(store_arc.clone(), 1));

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
        let system_collector = vectorizer::monitoring::SystemCollector::new(store_arc.clone());
        let system_collector_handle = system_collector.start();
        info!("✅ System metrics collector started");

        // Initialize query cache
        info!("💾 Initializing query cache...");
        let cache_config = vectorizer::cache::query_cache::QueryCacheConfig::default();
        let max_size = cache_config.max_size;
        let ttl_seconds = cache_config.ttl_seconds;
        let query_cache = Arc::new(vectorizer::cache::query_cache::QueryCache::new(
            cache_config,
        ));
        info!(
            "✅ Query cache initialized (max_size: {}, ttl: {}s)",
            max_size, ttl_seconds
        );

        // Initialize cluster manager if cluster is enabled
        let (cluster_manager, cluster_client_pool, cluster_config_ref) = {
            // Try to load cluster config from config.yml or use default
            let cluster_config = std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|content| {
                    serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content)
                        .ok()
                        .map(|config| config.cluster)
                })
                .unwrap_or_default();

            if cluster_config.enabled {
                info!("🔗 Initializing cluster manager...");

                // Validate cluster configuration
                let validator = vectorizer::cluster::ClusterConfigValidator::new();

                // Also load file watcher config for validation
                let file_watcher_config = std::fs::read_to_string(&config_path)
                    .ok()
                    .and_then(|content| {
                        serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content)
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

                match vectorizer::cluster::ClusterManager::new(cluster_config.clone()) {
                    Ok(manager) => {
                        let manager_arc = Arc::new(manager);
                        let timeout = std::time::Duration::from_millis(cluster_config.timeout_ms);
                        let client_pool =
                            Arc::new(vectorizer::cluster::ClusterClientPool::new(timeout));

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

        // Initialize replication if configured
        let (master_node, replica_node) = {
            let repl_yaml = std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|content| {
                    serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content)
                        .ok()
                        .map(|c| c.replication)
                })
                .unwrap_or_default();

            if repl_yaml.enabled || repl_yaml.role == "master" || repl_yaml.role == "replica" {
                let repl_config = repl_yaml.to_replication_config();
                match repl_config.role {
                    vectorizer::replication::NodeRole::Master => {
                        info!("🔄 Initializing replication as MASTER...");
                        match vectorizer::replication::MasterNode::new(
                            repl_config,
                            store_arc.clone(),
                        ) {
                            Ok(master) => {
                                let master = Arc::new(master);
                                let master_clone = master.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = master_clone.start().await {
                                        error!("❌ Master replication failed: {}", e);
                                    }
                                });
                                info!("✅ Replication master started");
                                (Some(master), None)
                            }
                            Err(e) => {
                                error!("❌ Failed to initialize master: {}", e);
                                (None, None)
                            }
                        }
                    }
                    vectorizer::replication::NodeRole::Replica => {
                        info!("🔄 Initializing replication as REPLICA...");
                        let replica = Arc::new(vectorizer::replication::ReplicaNode::new(
                            repl_config,
                            store_arc.clone(),
                        ));
                        let replica_clone = replica.clone();
                        tokio::spawn(async move {
                            if let Err(e) = replica_clone.start().await {
                                error!("❌ Replica replication failed: {}", e);
                            }
                        });
                        info!("✅ Replication replica started (connecting to master...)");
                        (None, Some(replica))
                    }
                    _ => {
                        info!("ℹ️  Replication mode: standalone");
                        (None, None)
                    }
                }
            } else {
                (None, None)
            }
        };

        // Initialize Raft HA automatically when cluster mode is enabled
        let (raft_manager, ha_manager) = {
            let cluster_enabled = match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content) {
                        Ok(c) => {
                            warn!(
                                "Cluster config parsed: enabled={}, node_id={:?}",
                                c.cluster.enabled, c.cluster.node_id
                            );
                            c.cluster.enabled
                        }
                        Err(e) => {
                            error!("❌ Failed to parse config for cluster check: {}", e);
                            false
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to read config file {}: {}", config_path, e);
                    false
                }
            };

            if cluster_enabled {
                info!("🗳️  Initializing Raft consensus (cluster mode active)...");

                // Derive node_id: use configured raft_node_id, or hash the string node_id, or default to 1.
                // Uses xxh3 for deterministic cross-platform hashing.
                let node_id = std::fs::read_to_string(&config_path)
                    .ok()
                    .and_then(|content| {
                        serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content)
                            .ok()
                            .and_then(|c| {
                                // Prefer explicit raft_node_id
                                c.cluster.raft_node_id.or_else(|| {
                                    // Hash the string node_id to u64 (deterministic)
                                    c.cluster
                                        .node_id
                                        .map(|s| xxhash_rust::xxh3::xxh3_64(s.as_bytes()))
                                })
                            })
                    })
                    .unwrap_or(1);

                match vectorizer::cluster::raft_node::RaftManager::new(node_id).await {
                    Ok(mgr) => {
                        let mgr = Arc::new(mgr);

                        // Bootstrap Raft cluster with all configured members.
                        // Build the member map from the cluster.servers config so
                        // all nodes participate in the initial election.
                        let cluster_servers = match std::fs::read_to_string(&config_path) {
                            Ok(content) => {
                                match serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(
                                    &content,
                                ) {
                                    Ok(c) => {
                                        warn!(
                                            "Cluster servers from config: {} servers",
                                            c.cluster.servers.len()
                                        );
                                        c.cluster.servers
                                    }
                                    Err(e) => {
                                        error!(
                                            "❌ Failed to parse config for cluster servers: {}",
                                            e
                                        );
                                        vec![]
                                    }
                                }
                            }
                            Err(e) => {
                                error!("❌ Failed to read config: {}", e);
                                vec![]
                            }
                        };

                        warn!(
                            "Cluster servers count: {} (need >1 for multi-node)",
                            cluster_servers.len()
                        );

                        if cluster_servers.len() > 1 {
                            // Wait for at least 1 peer to be resolvable via DNS
                            // before bootstrapping Raft. In Kubernetes, headless
                            // service DNS takes a few seconds after pod creation.
                            let my_id_str = std::fs::read_to_string(&config_path)
                                .ok()
                                .and_then(|c| {
                                    serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&c)
                                        .ok()
                                        .and_then(|c| c.cluster.node_id)
                                })
                                .unwrap_or_default();

                            warn!("⏳ Waiting for peer DNS resolution before Raft bootstrap...");
                            for attempt in 1..=30 {
                                let mut resolved = 0;
                                for server in &cluster_servers {
                                    if server.id == my_id_str {
                                        continue; // Skip self
                                    }
                                    let addr = format!("{}:{}", server.address, server.grpc_port);
                                    if tokio::net::lookup_host(&addr).await.is_ok() {
                                        resolved += 1;
                                    }
                                }
                                if resolved > 0 {
                                    warn!(
                                        "✅ {} peer(s) resolvable via DNS (attempt {})",
                                        resolved, attempt
                                    );
                                    break;
                                }
                                if attempt % 5 == 0 {
                                    warn!(
                                        "⏳ Still waiting for peer DNS... (attempt {}/30)",
                                        attempt
                                    );
                                }
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            }

                            let mut members = std::collections::BTreeMap::new();
                            for server in &cluster_servers {
                                let sid = xxhash_rust::xxh3::xxh3_64(server.id.as_bytes());
                                members.insert(
                                    sid,
                                    vectorizer::cluster::raft_node::RaftNodeInfo {
                                        address: server.address.clone(),
                                        grpc_port: server.grpc_port,
                                    },
                                );
                            }

                            // Only the lowest-ordinal node bootstraps Raft. The
                            // previous "every node attempts initialize, others get
                            // 'already initialized'" pattern was wrong: openraft's
                            // `initialize` writes to the *local* log/vote
                            // independently on each node, so calling it
                            // simultaneously on N nodes produces N divergent
                            // term-1 logs (each node votes for self) and the
                            // subsequent election permanently rejects every vote
                            // due to log inconsistency. By gating on the
                            // lowest-ordinal hostname (`<sts>-0` in a Kubernetes
                            // StatefulSet, or the alphabetically-first server id
                            // outside K8s), exactly one node bootstraps; the
                            // others wait, accept the membership log entry the
                            // bootstrap node propagates, and join cleanly.
                            let bootstrap_id = cluster_servers
                                .iter()
                                .map(|s| s.id.clone())
                                .min()
                                .unwrap_or_default();
                            let should_bootstrap = my_id_str == bootstrap_id;
                            if should_bootstrap {
                                warn!(
                                    bootstrap_id = %bootstrap_id,
                                    "🗳️ Calling initialize_cluster with {} members (this node is the bootstrap node)...",
                                    members.len()
                                );
                                match mgr.initialize_cluster(members).await {
                                    Ok(_) => warn!("✅ Raft cluster initialized successfully"),
                                    Err(e) => warn!(
                                        "Raft cluster bootstrap: {} (may be normal if already initialized)",
                                        e
                                    ),
                                }
                            } else {
                                warn!(
                                    bootstrap_id = %bootstrap_id,
                                    my_id = %my_id_str,
                                    "⏸️ Skipping initialize_cluster — waiting for bootstrap node to propagate membership"
                                );
                            }
                            // Register all node addresses in the Raft state machine
                            // so that resolve_leader_addr() can find them. Only the
                            // leader can propose; followers get ForwardToLeader (ok).
                            let mgr_clone = mgr.clone();
                            let servers_clone = cluster_servers.clone();
                            tokio::spawn(async move {
                                // Retry registering nodes until this node becomes
                                // leader and can propose. In a 3-node StatefulSet,
                                // the election may take 30-60s as pods start sequentially.
                                for attempt in 1..=12 {
                                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                                    // Only the leader can propose — check first.
                                    if !mgr_clone.is_leader().await {
                                        // Nudge openraft only when this node has been
                                        // stuck in `Candidate` for the warm-up window.
                                        //
                                        // Previously this checked `current_leader.is_some()`
                                        // on the metrics snapshot, but that field is
                                        // populated only after the leader has received
                                        // a quorum-ack — there is a real interval where
                                        // openraft has elected a leader and is following
                                        // it (state = Follower, vote committed) yet
                                        // `current_leader` is still `None`. Triggering
                                        // an election then was a false positive that
                                        // showed up as repeated "No leader after Ns"
                                        // warnings on healthy clusters (live test on
                                        // ermes prod, v3.0.11). Gating on
                                        // `state == Candidate` is the actual
                                        // "stuck — no quorum exists" signal.
                                        use openraft::ServerState;
                                        use openraft::rt::WatchReceiver as _;
                                        let metrics =
                                            mgr_clone.raft().metrics().borrow_watched().clone();
                                        let stuck_in_candidate = matches!(
                                            metrics.state,
                                            ServerState::Candidate
                                        );
                                        if stuck_in_candidate && attempt >= 3 {
                                            let _ = mgr_clone.raft().trigger().elect().await;
                                            tracing::warn!(
                                                attempt,
                                                state = ?metrics.state,
                                                "Stuck in Candidate after {}s — nudging election",
                                                attempt * 10
                                            );
                                        } else {
                                            tracing::debug!(
                                                attempt,
                                                state = ?metrics.state,
                                                current_leader = ?metrics.current_leader,
                                                "Not leader on this node — waiting for current leader to propose AddNode"
                                            );
                                        }
                                        continue;
                                    }

                                    let mut all_ok = true;
                                    for server in &servers_clone {
                                        let sid = xxhash_rust::xxh3::xxh3_64(server.id.as_bytes());
                                        match mgr_clone
                                            .propose(
                                                vectorizer::cluster::raft_node::ClusterCommand::AddNode {
                                                    node_id: sid,
                                                    address: server.address.clone(),
                                                    grpc_port: server.grpc_port,
                                                },
                                            )
                                            .await
                                        {
                                            Ok(_) => tracing::info!(
                                                "Registered node {} in Raft state machine",
                                                server.id
                                            ),
                                            Err(e) => {
                                                tracing::debug!(
                                                    "Could not register node {}: {}",
                                                    server.id,
                                                    e
                                                );
                                                all_ok = false;
                                            }
                                        }
                                    }
                                    if all_ok {
                                        tracing::info!(
                                            "All {} nodes registered in Raft state machine",
                                            servers_clone.len()
                                        );
                                        break;
                                    }
                                }
                            });
                        } else {
                            // Single-node cluster
                            if let Err(e) = mgr.initialize_single().await {
                                warn!("Raft single-node bootstrap: {}", e);
                            }
                        }

                        // Load replication config for HA role transitions
                        let repl_yaml = std::fs::read_to_string(&config_path)
                            .ok()
                            .and_then(|content| {
                                serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(
                                    &content,
                                )
                                .ok()
                                .map(|c| c.replication)
                            })
                            .unwrap_or_default();
                        let repl_config = repl_yaml.to_replication_config();

                        let ha = Arc::new(vectorizer::cluster::HaManager::new(
                            node_id,
                            store_arc.clone(),
                            repl_config,
                        ));

                        info!(
                            "✅ Raft node ready (node_id={}, members={})",
                            node_id,
                            cluster_servers.len()
                        );

                        // Start Raft leadership watcher — bridges consensus
                        // elections to HA role transitions (MasterNode ↔ ReplicaNode).
                        let watcher =
                            vectorizer::cluster::RaftWatcher::new(mgr.clone(), ha.clone());
                        let _watcher_handle = watcher.start();
                        info!("🔭 Raft watcher started — automatic failover enabled");

                        (Some(mgr), Some(ha))
                    }
                    Err(e) => {
                        error!("❌ Failed to initialize Raft: {}", e);
                        (None, None)
                    }
                }
            } else {
                // Even without cluster mode, create HaManager if replication is active
                // This enforces read-only on replicas
                let repl_yaml = std::fs::read_to_string(&config_path)
                    .ok()
                    .and_then(|content| {
                        serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content)
                            .ok()
                            .map(|c| c.replication)
                    })
                    .unwrap_or_default();

                if repl_yaml.role == "replica" {
                    let repl_config = repl_yaml.to_replication_config();
                    // Use node_id=999 so set_leader with id=0 marks us as Follower
                    let ha = Arc::new(vectorizer::cluster::HaManager::new(
                        999,
                        store_arc.clone(),
                        repl_config.clone(),
                    ));
                    // Set leader as remote node (id=0) → this node becomes Follower
                    let leader_url = repl_config
                        .master_address
                        .map(|addr| format!("http://{}:{}", addr.ip(), 15002))
                        .unwrap_or_else(|| "http://leader:15002".to_string());
                    ha.leader_router.set_leader(0, leader_url);
                    info!("🔒 Replica mode: writes will be redirected to leader");
                    (None, Some(ha))
                } else {
                    (None, None)
                }
            }
        };

        // Load API config for max request size
        let max_request_size_mb = std::fs::read_to_string(&config_path)
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
            let mut auth_config = std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|content| {
                    serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content)
                        .ok()
                        .map(|config| config.auth)
                })
                .unwrap_or_default();

            // Override with environment variables if set (for Docker)
            if let Ok(enabled) = std::env::var("VECTORIZER_AUTH_ENABLED") {
                auth_config.enabled = enabled.to_lowercase() == "true";
            }
            if let Ok(secret) = std::env::var("VECTORIZER_JWT_SECRET") {
                auth_config.jwt_secret = vectorizer::auth::Secret::new(secret);
            }

            // Opt-in auto-generated JWT secret for first-boot UX. Triggered by
            // either the CLI flag (plumbed via RootUserConfig) or the env var.
            // Only fires when the operator hasn't already set a secret — an
            // explicit VECTORIZER_JWT_SECRET or config.yml value always wins.
            let env_opt_in = std::env::var("VECTORIZER_AUTO_GEN_JWT_SECRET")
                .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
                .unwrap_or(false);
            let opt_in = root_config.auto_generate_jwt_secret || env_opt_in;
            if opt_in && auth_config.jwt_secret.expose_secret().is_empty() {
                let key_path = vectorizer::auth::persistence::AuthPersistence::get_data_dir()
                    .join("jwt_secret.key");
                match vectorizer::auth::jwt_secret::load_or_generate(&key_path) {
                    Ok(secret) => {
                        // Path-only — never log the secret value itself.
                        info!(
                            "🔑 Using auto-generated JWT secret persisted at {}",
                            key_path.display()
                        );
                        auth_config.jwt_secret = vectorizer::auth::Secret::new(secret);
                    }
                    Err(e) => {
                        error!(
                            "Failed to load or generate JWT secret at {}: {}",
                            key_path.display(),
                            e
                        );
                    }
                }
            }

            if auth_config.enabled {
                info!("🔐 Initializing authentication system...");
                match vectorizer::auth::AuthManager::new(auth_config) {
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
            let hub_config = match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content) {
                        Ok(config) => {
                            info!(
                                "✅ Loaded hub config from config.yml: enabled={}",
                                config.hub.enabled
                            );
                            config.hub
                        }
                        Err(e) => {
                            warn!("⚠️  Failed to parse config.yml for hub config: {}", e);
                            vectorizer::hub::HubConfig::default()
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to read config.yml for hub config: {}", e);
                    vectorizer::hub::HubConfig::default()
                }
            };

            if hub_config.enabled {
                info!("🌐 Initializing HiveHub integration...");
                match vectorizer::hub::HubManager::new(hub_config).await {
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
            let backup_config = vectorizer::hub::BackupConfig::default();
            match vectorizer::hub::UserBackupManager::new(backup_config, store_arc.clone()) {
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
            let gateway = vectorizer::hub::McpHubGateway::new(hub_mgr.clone());
            info!("✅ MCP Hub Gateway initialized");
            Some(Arc::new(gateway))
        } else {
            None
        };

        // VectorizerRPC binary listener — opt-in via `rpc.enabled` in
        // config.yml. The listener spawns its own background tasks per
        // accepted connection; nothing else in `Self` needs to retain a
        // handle to it (the listener owns its TcpListener for life).
        let embedding_manager_arc = Arc::new(final_embedding_manager);
        let rpc_config = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| {
                serde_yaml::from_str::<vectorizer::config::VectorizerConfig>(&content).ok()
            })
            .map(|cfg| cfg.rpc)
            .unwrap_or_default();
        if rpc_config.enabled {
            let bind = format!("{}:{}", rpc_config.host, rpc_config.port);
            match bind.parse::<std::net::SocketAddr>() {
                Ok(addr) => {
                    let rpc_state = crate::protocol::rpc::server::RpcState {
                        store: store_arc.clone(),
                        embedding_manager: embedding_manager_arc.clone(),
                        auth: auth_handler_state.clone(),
                    };
                    if let Err(e) = crate::protocol::rpc::spawn_rpc_listener(rpc_state, addr).await
                    {
                        error!("Failed to spawn VectorizerRPC listener on {}: {}", addr, e);
                    } else {
                        info!("✅ VectorizerRPC listener bound to {}", addr);
                    }
                }
                Err(e) => {
                    warn!(
                        "Invalid RPC bind address '{}': {} — RPC listener not started",
                        bind, e
                    );
                }
            }
        } else {
            debug!("VectorizerRPC listener disabled in config");
        }

        Ok(Self {
            store: store_arc,
            embedding_manager: embedding_manager_arc,
            start_time: std::time::Instant::now(),
            file_watcher_system: watcher_system_for_server,
            metrics_collector: Arc::new(MetricsCollector::new()),
            auto_save_manager: Some(auto_save_manager),
            master_node,
            replica_node,
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
                Some(Arc::new(vectorizer::storage::SnapshotManager::new(
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
            raft_manager,
            ha_manager,
        })
    }
}
