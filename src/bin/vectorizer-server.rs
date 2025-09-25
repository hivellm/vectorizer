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

    // Try to load existing vector store first
    let vector_store_path = if let Some(project_path) = &args.project {
        PathBuf::from(project_path).join(".vectorizer").join("vector_store.bin")
    } else {
        PathBuf::from(".vectorizer").join("vector_store.bin")
    };

    info!("Vector store path: {:?}", vector_store_path);
    info!("Vector store path exists: {}", vector_store_path.exists());

    let vector_store = if vector_store_path.exists() {
        info!("Loading existing vector store from: {:?}", vector_store_path);
        match VectorStore::load(&vector_store_path) {
            Ok(store) => {
                info!("Successfully loaded vector store with {} collections", store.list_collections().len());
                Arc::new(store)
            }
            Err(e) => {
                warn!("Failed to load vector store from {:?}: {}, creating new one", vector_store_path, e);
                Arc::new(VectorStore::new())
            }
        }
    } else {
        info!("No existing vector store found, creating new one");
        Arc::new(VectorStore::new())
    };

    // Create shared indexing progress tracker
    let indexing_progress: Arc<std::sync::Mutex<HashMap<String, IndexingStatus>>> = Arc::new(std::sync::Mutex::new(HashMap::new()));

    // Create default embedding manager
    let embedding_manager = {
        let config = LoaderConfig::default();
        let loader = DocumentLoader::new(config);
        loader.into_embedding_manager()
    };

    // Load workspace synchronously FIRST
    if let Some(workspace_path) = &args.workspace {
        info!("🔄 Starting workspace indexing synchronously...");

        // Load workspace collections to know what to track
        let workspace_collections = AppState::load_workspace_collections();

        // Initialize all collections as "indexing"
        for collection in &workspace_collections {
            update_indexing_progress(&indexing_progress, &collection.name, "indexing", 0.0, 0, 0);
        }

        match load_workspace_projects(workspace_path, Arc::clone(&vector_store), Some(&indexing_progress)) {
            Ok(loaded_collections) => {
                info!("✅ Workspace indexing completed: {} collections loaded", loaded_collections);

                // Mark all collections as completed
                for collection in &workspace_collections {
                    update_indexing_progress(&indexing_progress, &collection.name, "completed", 100.0, 1, 1);
                }
            }
            Err(e) => {
                error!("❌ Workspace indexing failed: {}", e);

                // Mark all collections as failed
                for collection in &workspace_collections {
                    update_indexing_progress(&indexing_progress, &collection.name, "failed", 0.0, 0, 0);
                }
                return Err(e.into());
            }
        }
    }

    // Now start HTTP server with completed indexing state
    info!("🚀 Starting Vectorizer HTTP server...");
    let server = VectorizerServer::new(&args.host, args.port, vector_store, embedding_manager);
    server.start().await?;

    Ok(())
}

/// Load all projects from a workspace configuration
fn load_workspace_projects(
    workspace_path: &str,
    vector_store: Arc<VectorStore>,
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

            // Load collections for this project
            if let Some(collections) = project_value.get("collections").and_then(|c| c.as_array()) {
                for collection in collections {
                    if let Some(collection_name) = collection.get("name").and_then(|n| n.as_str()) {
                        let project_path = project_value.get("path")
                            .and_then(|p| p.as_str())
                            .unwrap_or(".");

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

                        let mut loader = DocumentLoader::new(loader_config);
                        match loader.load_project(project_path, &vector_store) {
                            Ok(_) => {
                                info!("Successfully loaded collection '{}' with documents", collection_name);
                                total_collections += 1;

                                // Update progress: completed
                                if let Some(tracker) = progress_tracker {
                                    update_indexing_progress(tracker, collection_name, "completed", 100.0, 1, 1);
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
