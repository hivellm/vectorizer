use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use vectorizer::api::handlers::{AppState, IndexingProgressState, update_indexing_progress};
use vectorizer::document_loader::{DocumentLoader, LoaderConfig};
use vectorizer::vector_store::VectorStore;

#[derive(Parser, Debug)]
#[command(name = "vectorizer-indexing")]
#[command(about = "Vectorizer indexing service - manages cache loading and indexing")]
struct Args {
    /// Workspace configuration file
    #[arg(long)]
    workspace: PathBuf,
    
    /// Vector store file path
    #[arg(long)]
    vector_store: PathBuf,
    
    /// Progress update endpoint
    #[arg(long, default_value = "http://127.0.0.1:15001/api/v1/indexing/progress")]
    progress_endpoint: String,
    
    /// Collection name to process (if not provided, processes all)
    #[arg(long)]
    collection: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let args = Args::parse();
    
    println!("🚀 Starting Vectorizer Indexing Service");
    println!("📂 Workspace: {}", args.workspace.display());
    println!("💾 Vector Store: {}", args.vector_store.display());
    
    // Load workspace collections
    let workspace_collections = AppState::load_workspace_collections();
    println!("📊 Found {} collections in workspace", workspace_collections.len());
    
    // Initialize progress tracking
    let indexing_progress = Arc::new(IndexingProgressState::new());
    
    // Load or create vector store
    let vector_store = if args.vector_store.exists() {
        println!("📖 Loading existing vector store...");
        VectorStore::load(&args.vector_store)?
    } else {
        println!("🏗️ Creating new vector store...");
        VectorStore::new()
    };
    let app_vector_store = Arc::new(vector_store);
    
    // Process collections
    if let Some(collection_name) = args.collection {
        // Process single collection
        process_single_collection(
            &collection_name,
            &workspace_collections,
            &app_vector_store,
            &indexing_progress,
        ).await?;
    } else {
        // Process all collections
        process_all_collections(
            &workspace_collections,
            &app_vector_store,
            &indexing_progress,
        ).await?;
    }
    
    // Save vector store
    println!("💾 Saving vector store to: {}", args.vector_store.display());
    app_vector_store.save(&args.vector_store)?;
    
    println!("✅ Indexing completed successfully!");
    Ok(())
}

async fn process_single_collection(
    collection_name: &str,
    workspace_collections: &[vectorizer::api::handlers::CollectionInfo],
    app_vector_store: &Arc<VectorStore>,
    indexing_progress: &Arc<IndexingProgressState>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Processing single collection: {}", collection_name);
    
    // Find collection in workspace
    let collection_info = workspace_collections
        .iter()
        .find(|c| c.name == collection_name)
        .ok_or_else(|| anyhow::anyhow!("Collection '{}' not found in workspace", collection_name))?;
    
    // Initialize progress
    update_indexing_progress(indexing_progress, collection_name, "pending", 0.0, 0, 0);
    
    // Process collection
    process_collection_with_timeout(collection_info, app_vector_store, indexing_progress).await?;
    
    Ok(())
}

async fn process_all_collections(
    workspace_collections: &[vectorizer::api::handlers::CollectionInfo],
    app_vector_store: &Arc<VectorStore>,
    indexing_progress: &Arc<IndexingProgressState>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Processing all {} collections", workspace_collections.len());
    
    // Initialize all collections as pending
    for collection in workspace_collections {
        update_indexing_progress(indexing_progress, &collection.name, "pending", 0.0, 0, 0);
    }
    
    // Process each collection
    for (i, collection) in workspace_collections.iter().enumerate() {
        println!("🔄 Processing collection {}/{}: {}", i + 1, workspace_collections.len(), collection.name);
        
        match process_collection_with_timeout(collection, app_vector_store, indexing_progress).await {
            Ok(_) => {
                println!("✅ Collection '{}' processed successfully", collection.name);
            }
            Err(e) => {
                println!("❌ Collection '{}' failed: {}", collection.name, e);
                update_indexing_progress(indexing_progress, &collection.name, "failed", 0.0, 0, 0);
            }
        }
    }
    
    Ok(())
}

async fn process_collection_with_timeout(
    collection_info: &vectorizer::api::handlers::CollectionInfo,
    app_vector_store: &Arc<VectorStore>,
    indexing_progress: &Arc<IndexingProgressState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection_name = &collection_info.name;
    
    // Set initial status
    update_indexing_progress(indexing_progress, collection_name, "processing", 0.0, 0, 0);
    
    // Create loader config
    let loader_config = LoaderConfig {
        collection_name: collection_name.clone(),
        chunk_size: 512,
        chunk_overlap: 50,
        embedding_model: vectorizer::embedding::EmbeddingModel::MiniLM,
        embedding_dimension: 512,
        distance_metric: vectorizer::vector_store::DistanceMetric::Cosine,
    };
    
    // Create document loader
    let mut loader = DocumentLoader::new(loader_config);
    
    // Process with timeout
    let result = timeout(
        Duration::from_secs(300), // 5 minutes timeout
        process_collection_internal(collection_info, &mut loader, app_vector_store, indexing_progress)
    ).await;
    
    match result {
        Ok(Ok((count, cached))) => {
            let status = if cached { "cached" } else { "completed" };
            update_indexing_progress(indexing_progress, collection_name, status, 100.0, count, count);
            println!("✅ Collection '{}': {} vectors (cached: {})", collection_name, count, cached);
            Ok(())
        }
        Ok(Err(e)) => {
            update_indexing_progress(indexing_progress, collection_name, "failed", 0.0, 0, 0);
            Err(e.into())
        }
        Err(_) => {
            update_indexing_progress(indexing_progress, collection_name, "failed", 0.0, 0, 0);
            Err(anyhow::anyhow!("Timeout processing collection '{}'", collection_name))
        }
    }
}

async fn process_collection_internal(
    collection_info: &vectorizer::api::handlers::CollectionInfo,
    loader: &mut DocumentLoader,
    app_vector_store: &Arc<VectorStore>,
    indexing_progress: &Arc<IndexingProgressState>,
) -> Result<(usize, bool), Box<dyn std::error::Error>> {
    let collection_name = &collection_info.name;
    println!("🔍 Processing collection: {}", collection_name);
    
    // Try to load from cache first
    let cache_path = PathBuf::from(&collection_info.path).join(".vectorizer").join(format!("{}_vector_store.bin", collection_name));
    
    if cache_path.exists() {
        println!("🚀 Cache found for '{}', attempting to load...", collection_name);
        update_indexing_progress(indexing_progress, collection_name, "loading_cache", 10.0, 0, 0);
        
        // Try to load cache with timeout
        let load_result = timeout(
            Duration::from_secs(30), // 30 seconds timeout for cache loading
            load_cache_with_fallback(loader, &cache_path, app_vector_store, collection_name)
        ).await;
        
        match load_result {
            Ok(Ok(count)) => {
                update_indexing_progress(indexing_progress, collection_name, "cached", 100.0, count, count);
                println!("✅ Cache loaded successfully: {} vectors", count);
                return Ok((count, true));
            }
            Ok(Err(e)) => {
                println!("⚠️ Cache loading failed, falling back to indexing: {}", e);
            }
            Err(_) => {
                println!("⚠️ Cache loading timed out, falling back to indexing");
            }
        }
    } else {
        println!("📊 No cache found for '{}', will index from scratch", collection_name);
    }
    
    // Fallback to full indexing
    update_indexing_progress(indexing_progress, collection_name, "indexing", 0.0, 0, 0);
    
    // Process collection
    let count = loader.load_project(&collection_info.path, app_vector_store)?;
    
    update_indexing_progress(indexing_progress, collection_name, "completed", 100.0, count, count);
    println!("✅ Indexing completed: {} vectors", count);
    
    Ok((count, false))
}

async fn load_cache_with_fallback(
    _loader: &DocumentLoader,
    cache_path: &PathBuf,
    app_vector_store: &Arc<VectorStore>,
    collection_name: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    println!("📖 Loading cache from: {}", cache_path.display());
    
    // Load the persisted store
    let persisted_store = VectorStore::load(cache_path)?;
    println!("📖 VectorStore loaded successfully");
    
    // Get collection from persisted store
    let src_collection = persisted_store.get_collection(collection_name)?;
    println!("🔍 Collection retrieved successfully");
    
    // Create collection in app store if it doesn't exist
    let meta = src_collection.metadata();
    if app_vector_store.get_collection(collection_name).is_err() {
        println!("🏗️ Creating collection in app store...");
        app_vector_store.create_collection(collection_name, meta.config.clone())?;
        println!("🏗️ Collection created successfully");
    }
    
    // Copy vectors
    println!("📊 Getting all vectors...");
    let vectors = src_collection.get_all_vectors();
    let vector_count = vectors.len();
    println!("📊 Found {} vectors, inserting into app store...", vector_count);
    app_vector_store.insert(collection_name, vectors)?;
    println!("✅ Successfully loaded {} vectors from cache", vector_count);
    
    Ok(vector_count)
}
