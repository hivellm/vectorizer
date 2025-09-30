//! Vectorizer Server - Main server with MCP integration
//!
//! This binary provides the main Vectorizer server with integrated MCP support
//! for IDE integration and AI model communication.

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};
use vectorizer::{
    api::{
        VectorizerServer,
        handlers::{AppState, IndexingProgressState},
    },
    cache::CacheConfig,
    db::VectorStore,
    document_loader::{DocumentLoader, LoaderConfig},
    process_manager::{initialize_process_management, cleanup_process_management},
    logging,
};

/// Update indexing progress for a collection
fn update_indexing_progress(
    indexing_progress: &IndexingProgressState,
    collection_name: &str,
    status: &str,
    progress: f32,
    total_documents: usize,
    processed_documents: usize,
) {
    indexing_progress.update(
        collection_name,
        status,
        progress,
        total_documents,
        processed_documents,
    );
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

    // Initialize process management first
    let ports = vec![args.port];
    // Temporarily disabled to fix server startup issues
    // let _process_logger = initialize_process_management("vectorizer-server", &ports)?;

    // Initialize centralized logging
    if let Err(e) = logging::init_logging("vectorizer-server") {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("Starting Vectorizer Server with dashboard");

    // Create a global vector store for AppState (will be populated by background indexing)
    // Try to load CUDA config from config.yml
    let cuda_config = {
        use serde_yaml;

        match std::fs::read_to_string("config.yml") {
            Ok(content) => {
                match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    Ok(yaml) => {
                        if let Some(cuda_section) = yaml.get("cuda") {
                            match serde_yaml::from_value::<vectorizer::cuda::CudaConfig>(cuda_section.clone()) {
                                Ok(mut config) => {
                                    println!("✅ Loaded CUDA config for vectorizer-server:");
                                    println!("   - enabled: {}", config.enabled);
                                    println!("   - device_id: {}", config.device_id);
                                    println!("   - memory_limit_mb: {}", config.memory_limit_mb);

                                    // Override with defaults if not specified
                                    if config.memory_limit_mb == 0 {
                                        config.memory_limit_mb = 4096; // 4GB default
                                    }

                                    config
                                }
                            Err(e) => {
                                println!("⚠️ Failed to parse CUDA config section: {}. Using CPU-only mode.", e);
                                let mut config = vectorizer::cuda::CudaConfig::default();
                                config.enabled = false;
                                config
                            }
                            }
                        } else {
                            println!("ℹ️ No CUDA section in config.yml for vectorizer-server, using CPU-only mode");
                            let mut config = vectorizer::cuda::CudaConfig::default();
                            config.enabled = false;
                            config
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Failed to parse config.yml as YAML: {}. Using default CUDA config", e);
                        vectorizer::cuda::CudaConfig::default()
                    }
                }
            }
            Err(_) => {
                println!("ℹ️ No config.yml found for vectorizer-server, using default CUDA config");
                vectorizer::cuda::CudaConfig::default()
            }
        }
    };

    let app_vector_store = Arc::new(VectorStore::new_with_cuda_config(cuda_config));

    // Create shared indexing progress tracker
    let indexing_progress = IndexingProgressState::new();

    // Create app state with indexing progress
    let mut app_state = AppState::new_with_progress(
        Arc::clone(&app_vector_store),
        vectorizer::embedding::EmbeddingManager::new(),
        indexing_progress.clone(),
        None, // summarization_config
    );

    // Initialize collections as pending (dashboard will show them immediately)
    if let Some(workspace_path) = args.workspace.clone() {
        // Set environment variable for workspace loading
        unsafe {
            std::env::set_var("VECTORIZER_WORKSPACE_INFO", &workspace_path);
        }

        // Load workspace collections to know what to track
        let workspace_collections = AppState::load_workspace_collections();

        // Initialize all collections as "pending" so dashboard shows them immediately
        for collection in &workspace_collections {
            update_indexing_progress(
                &indexing_progress,
                &collection.name,
                "pending",
                0.0,
                0,
                0,
            );
        }

        info!("✅ Dashboard ready with {} collections initialized", workspace_collections.len());
    }

    // Initialize GRPC client
    info!("🔗 Initializing GRPC client for communication with vzr...");
    if let Err(e) = app_state.init_grpc_client().await {
        warn!("⚠️ Failed to initialize GRPC client: {}. Server will use local store only.", e);
    }

    // Start HTTP server
    info!(
        "🚀 Starting Vectorizer HTTP server on {}:{}...",
        args.host, args.port
    );
    let server = VectorizerServer::new_with_state(&args.host, args.port, app_state);

    // No background indexing here - vzr handles that and replicates status to servers

    // Start server
    info!("🎯 Vectorizer server ready - dashboard accessible immediately");
    
    // Setup cleanup on exit
    let cleanup_guard = scopeguard::guard((), |_| {
        cleanup_process_management("vectorizer-server");
    });
    
    let result = server.start().await;
    
    // Cleanup will be called automatically when cleanup_guard goes out of scope
    drop(cleanup_guard);

    result
}
