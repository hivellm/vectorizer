//! Vectorizer Server - Main server with MCP integration
//!
//! This binary provides the main Vectorizer server with integrated MCP support
//! for IDE integration and AI model communication.

use clap::Parser;
use tracing::{info, warn, error};
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
use tokio::task;
use std::collections::HashMap;
use vectorizer::{
    api::{handlers::AppState, types::IndexingStatus, VectorizerServer},
    db::VectorStore,
    document_loader::{DocumentLoader, LoaderConfig},
    cache::{CacheManager, CacheConfig, CacheResult},
};

/// Update indexing progress for a collection
fn update_indexing_progress(
    indexing_progress: &Arc<std::sync::Mutex<HashMap<String, IndexingStatus>>>,
    collection_name: &str,
    status: &str,
    progress: f32,
    total_documents: usize,
    processed_documents: usize,
) {
    info!("🔄 Updating progress for '{}' to status '{}' progress {:.1}%", collection_name, status, progress);
    let mut progress_map = indexing_progress.lock().unwrap();
    let old_count = progress_map.len();
    progress_map.insert(collection_name.to_string(), IndexingStatus {
        status: status.to_string(),
        progress,
        total_documents,
        processed_documents,
        estimated_time_remaining: None, // Could be calculated based on progress rate
        last_updated: chrono::Utc::now().to_rfc3339(),
    });
    let new_count = progress_map.len();
    info!("📊 Progress map updated: {} -> {} entries", old_count, new_count);
}

#[derive(Parser)]
#[command(name = "vectorizer-server")]
#[command(about = "Vectorizer HTTP Server with document loading capabilities")]
struct Args {
    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind the server to
    #[arg(long, default_value = "15001")]
    port: u16,

    /// Project directory to load and vectorize (legacy single project mode)
    #[arg(long)]
    project: Option<String>,

    /// Workspace configuration file path (for multi-project mode)
    #[arg(long)]
    workspace: Option<String>,

    /// Configuration file path
    #[arg(long, default_value = "config.yml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging to file (unique per port to avoid conflicts)
    let log_filename = format!("vectorizer-server-{}.log", args.port);
    let log_file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_filename) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open log file {}: {}", log_filename, e);
            std::process::exit(1);
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter("vectorizer=info")
        .with_writer(move || log_file.try_clone().expect("Failed to clone log file"))
        .init();

    info!("Starting Vectorizer Server with dashboard");

    // Configuration loading disabled for now
    let config = serde_yaml::Value::Null;

    // Note: Vector stores are now loaded per-project in load_workspace_projects
    // Create a global vector store for AppState (will be populated by background indexing)
    let app_vector_store = Arc::new(VectorStore::new());

    // Create shared indexing progress tracker
    let indexing_progress: Arc<std::sync::Mutex<HashMap<String, IndexingStatus>>> = Arc::new(std::sync::Mutex::new(HashMap::new()));

    // Create default embedding manager
    let embedding_manager = {
        let config = LoaderConfig::default();
        let loader = DocumentLoader::new(config);
        loader.into_embedding_manager()
    };

    // Create app state with indexing progress
    let app_state = AppState::new_with_progress(Arc::clone(&app_vector_store), embedding_manager, Arc::clone(&indexing_progress));
    
    // Start HTTP server immediately
    info!("🚀 Starting Vectorizer HTTP server on {}:{}...", args.host, args.port);
    let server = VectorizerServer::new_with_state(&args.host, args.port, app_state);
    
    // Clone variables for the background thread
    let workspace_path_clone = args.workspace.clone();
    let indexing_progress_clone = Arc::clone(&indexing_progress);
    let app_vector_store_clone = Arc::clone(&app_vector_store);
    
    // Start indexing in a separate thread
    if let Some(workspace_path) = workspace_path_clone {
        std::thread::spawn(move || {
            info!("🔄 Starting workspace indexing in background thread...");
            
            // Load workspace collections to know what to track
            let workspace_collections = AppState::load_workspace_collections();
            
            // Initialize all collections as "pending"
            for collection in &workspace_collections {
                update_indexing_progress(&indexing_progress_clone, &collection.name, "pending", 0.0, 0, 0);
            }
            
            // Process collections one by one with progress updates
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(load_workspace_projects(&workspace_path, Arc::clone(&app_vector_store_clone), Some(&indexing_progress_clone))) {
                Ok(loaded_collections) => {
                    info!("✅ Background workspace indexing completed: {} collections loaded", loaded_collections);
            }
            Err(e) => {
                    error!("❌ Background workspace indexing failed: {}", e);
                    
                    // Mark remaining collections as failed
                    for collection in &workspace_collections {
                        let status = {
                            let progress = indexing_progress_clone.lock().unwrap();
                            progress.get(&collection.name).map(|s| s.status.clone()).unwrap_or_default()
                        };
                        if status == "pending" || status == "indexing" {
                            update_indexing_progress(&indexing_progress_clone, &collection.name, "failed", 0.0, 0, 0);
                        }
                    }
                }
            }
        });
    }
    
    // Start the server (this will run indefinitely)
    info!("🎯 Vectorizer server ready - collections will be indexed in background");
    server.start().await?;

    Ok(())
}

/// Load all projects from a workspace configuration with cache management
async fn load_workspace_projects(
    workspace_path: &str,
    app_vector_store: Arc<VectorStore>,
    progress_tracker: Option<&Arc<std::sync::Mutex<HashMap<String, IndexingStatus>>>>,
) -> anyhow::Result<usize> {
    info!("Loading workspace info from: {}", workspace_path);

    let workspace_content = std::fs::read_to_string(workspace_path)
        .map_err(|e| anyhow::anyhow!("Failed to read workspace file: {}", e))?;

    let workspace_json: serde_json::Value = serde_json::from_str(&workspace_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse workspace JSON: {}", e))?;

    let projects = workspace_json.get("projects")
        .and_then(|p| p.as_array())
        .ok_or_else(|| anyhow::anyhow!("No projects found in workspace configuration"))?;

    let mut total_collections = 0;

    for project_value in projects {
        if let Some(project_name) = project_value.get("name").and_then(|n| n.as_str()) {
            info!("Loading project: {}", project_name);
            
            let project_path = project_value.get("path")
                .and_then(|p| p.as_str())
                .unwrap_or(".");

            // Load collections for this project
            if let Some(collections) = project_value.get("collections").and_then(|c| c.as_array()) {
                for collection in collections {
                    if let Some(collection_name) = collection.get("name").and_then(|n| n.as_str()) {

                        info!("Loading collection '{}' for project '{}'", collection_name, project_name);

                        // Create loader config for this collection
                        let loader_config = LoaderConfig {
                            collection_name: collection_name.to_string(),
                            max_chunk_size: collection.get("chunk_size")
                                .and_then(|c| c.as_u64())
                                .unwrap_or(512) as usize,
                            chunk_overlap: collection.get("chunk_overlap")
                                .and_then(|c| c.as_u64())
                                .unwrap_or(50) as usize,
                            allowed_extensions: vec![], // Will use include_patterns instead
                            include_patterns: collection.get("processing")
                                .and_then(|p| p.get("include_patterns"))
                                .and_then(|i| i.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_else(|| vec!["**/*.md".to_string(), "**/*.txt".to_string()]),
                            exclude_patterns: collection.get("processing")
                                .and_then(|p| p.get("exclude_patterns"))
                                .and_then(|e| e.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_else(|| vec!["**/node_modules/**".to_string(), "**/target/**".to_string()]),
                            embedding_dimension: 512,
                            embedding_type: "bm25".to_string(),
                            max_file_size: 10 * 1024 * 1024, // 10MB
                        };

                        // Update progress: starting this collection
                        if let Some(tracker) = progress_tracker {
                            update_indexing_progress(tracker, collection_name, "processing", 10.0, 0, 0);
                        }

                        // Create cache configuration
                        let cache_config = CacheConfig {
                            cache_path: PathBuf::from(project_path).join(".vectorizer/cache"),
                            validation_level: vectorizer::cache::ValidationLevel::Basic,
                            cleanup: vectorizer::cache::CleanupConfig::default(),
                            compression: vectorizer::cache::CompressionConfig::default(),
                            max_size_bytes: 5 * 1024 * 1024 * 1024, // 5GB
                            ttl_seconds: 7 * 24 * 60 * 60, // 7 days
                        };

                        info!("🔧 Creating cache config for '{}' at path: {:?}", collection_name, cache_config.cache_path);

                        // Create loader with cache management
                        let mut loader = match DocumentLoader::new_with_cache(loader_config.clone(), cache_config).await {
                            Ok(loader) => {
                                info!("✅ Successfully created loader with cache for '{}'", collection_name);
                                loader
                            },
                            Err(e) => {
                                warn!("❌ Failed to create loader with cache for '{}': {}, falling back to regular loader", collection_name, e);
                                DocumentLoader::new(loader_config)
                            }
                        };

                        info!("🔍 Cache manager present for '{}': {}", collection_name, loader.cache_manager.is_some());

                        // Each collection gets its own vector store file
                        let collection_vector_store_path = PathBuf::from(project_path)
                            .join(".vectorizer")
                            .join(format!("{}_vector_store.bin", collection_name));
                        
                        let (collection_vector_store, already_loaded) = if collection_vector_store_path.exists() {
                            info!("Loading existing collection vector store from: {:?}", collection_vector_store_path);
                            match VectorStore::load(&collection_vector_store_path) {
                                Ok(store) => {
                                    let collections = store.list_collections();
                                    info!("Loaded VectorStore contains {} collections: {:?}", collections.len(), collections);
                                    
                                    let has_vectors = if let Ok(coll) = store.get_collection(collection_name) {
                                        let count = coll.vector_count();
                                        info!("Successfully loaded collection '{}' with {} vectors", collection_name, count);
                                        count > 0
                                    } else {
                                        warn!("Collection '{}' not found in loaded VectorStore!", collection_name);
                                        false
                                    };
                                    (Arc::new(store), has_vectors)
                                }
                                Err(e) => {
                                    warn!("Failed to load collection vector store: {}, creating new one", e);
                                    (Arc::new(VectorStore::new()), false)
                                }
                            }
                        } else {
                            info!("No existing collection vector store found, creating new one for '{}'", collection_name);
                            (Arc::new(VectorStore::new()), false)
                        };
                        
                        info!("🔍 Using collection vector store for '{}' (already_loaded: {})", collection_name, already_loaded);

                        // Skip processing if already loaded from persistent store
                        let result = if already_loaded {
                            info!("✅ Collection '{}' already loaded from persistent store, skipping reprocessing", collection_name);
                            if let Ok(coll) = collection_vector_store.get_collection(collection_name) {
                                Ok(coll.vector_count())
                            } else {
                                Ok(0)
                            }
                        } else {
                            // Try cache-enabled loading first, fall back to regular loading
                            if loader.cache_manager.is_some() {
                                info!("🚀 Attempting cache-enabled loading for '{}'", collection_name);
                                info!("🔍 About to call load_project_with_cache for '{}'", collection_name);
                                let cache_result = loader.load_project_with_cache(project_path, &collection_vector_store).await;
                                info!("🔍 load_project_with_cache completed for '{}'", collection_name);
                                match cache_result {
                                    Ok(count) => {
                                        info!("✅ Cache-enabled loading succeeded for '{}' with {} vectors", collection_name, count);
                                        Ok(count)
                                    },
                                    Err(e) => {
                                        warn!("❌ Cache-enabled loading failed for '{}': {}, falling back to regular loading", collection_name, e);
                                        loader.load_project(project_path, &collection_vector_store)
                                    }
                                }
                            } else {
                                info!("⚠️ No cache manager for '{}', using regular loading", collection_name);
                                loader.load_project(project_path, &collection_vector_store)
                            }
                        };

                        match result {
                            Ok(vector_count) => {
                                info!("Successfully loaded collection '{}' with {} vectors", collection_name, vector_count);
                                total_collections += 1;

                                // Save collection-specific vector store only if it wasn't already loaded
                                if !already_loaded {
                                    let store_to_save = Arc::clone(&collection_vector_store);
                                    let save_path = collection_vector_store_path.clone();
                                    tokio::task::spawn_blocking(move || {
                                        if let Err(e) = store_to_save.save(&save_path) {
                                            warn!("Failed to save collection vector store to {:?}: {}", save_path, e);
                                        } else {
                                            info!("Collection vector store saved successfully to {:?}", save_path);
                                        }
                                    });
                                }
                                
                                // Merge freshly loaded/cached collection into global app vector store directly from memory
                                if let Ok(src_collection) = collection_vector_store.get_collection(collection_name) {
                                    let meta = src_collection.metadata();
                                    // Create collection in app store if missing
                                    if app_vector_store.get_collection(collection_name).is_err() {
                                        let _ = app_vector_store.create_collection(collection_name, meta.config.clone());
                                    }
                                    let vectors = src_collection.get_all_vectors();
                                    if let Err(e) = app_vector_store.insert(collection_name, vectors) {
                                        warn!("Failed to merge collection '{}' into app store: {}", collection_name, e);
                                    } else if let Ok(dest) = app_vector_store.get_collection(collection_name) {
                                        info!(
                                            "Merged collection '{}' into app store with {} vectors",
                                            collection_name,
                                            dest.vector_count()
                                        );
                                    }
                                }

                                // Update progress: completed
                                if let Some(tracker) = progress_tracker {
                                    if vector_count > 0 {
                                        update_indexing_progress(tracker, collection_name, "completed", 100.0, 1, 1);
                                    }
                                }
                }
                Err(e) => {
                                warn!("Failed to load collection '{}': {}", collection_name, e);

                                // Update progress: failed
                                if let Some(tracker) = progress_tracker {
                                    update_indexing_progress(tracker, collection_name, "failed", 0.0, 0, 0);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(total_collections)
}

/// Load a single project (legacy mode)
fn load_single_project(project_path: &str, vector_store: Arc<VectorStore>) -> anyhow::Result<()> {
    info!("Loading single project from: {}", project_path);

    // Create default loader config
    let loader_config = LoaderConfig {
        collection_name: "documents".to_string(),
        max_chunk_size: 512,
        chunk_overlap: 50,
        allowed_extensions: vec!["*.rs".to_string(), "*.md".to_string(), "*.txt".to_string()],
        include_patterns: vec![],
        exclude_patterns: vec![],
        embedding_dimension: 512,
        embedding_type: "bm25".to_string(),
        max_file_size: 10 * 1024 * 1024, // 10MB
    };

    let mut loader = DocumentLoader::new(loader_config);
    loader.load_project(project_path, &vector_store)?;

    info!("Successfully loaded single project");
    Ok(())
}
