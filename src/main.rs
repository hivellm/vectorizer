//! Vectorizer server entry point

use clap::Parser;
use tracing::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vectorizer - High-performance vector database
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 15002)]
    port: u16,

    /// Host to bind to
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Project directory to load and index automatically
    #[arg(long)]
    project: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments first
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("vectorizer=debug,tower_http=debug,axum=debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    println!("🚀 Starting Vectorizer v{}", vectorizer::VERSION);
    println!("🔗 Binding to {}:{}", args.host, args.port);
    println!("📁 Project argument: {:?}", args.project);
    println!("⚙️ Config argument: {:?}", args.config);
    
    info!("Starting Vectorizer v{}", vectorizer::VERSION);
    info!("Binding to {}:{}", args.host, args.port);
    info!("Project argument: {:?}", args.project);
    info!("Config argument: {:?}", args.config);

    // Check for legacy data and offer migration
    let data_dir = std::path::Path::new("./data");
    if data_dir.exists() {
        let migrator = vectorizer::storage::StorageMigrator::new(data_dir, 6);
        
        if migrator.needs_migration() {
            println!("\n⚠️  Legacy data format detected!");
            println!("📦 The new .vecdb format offers:");
            println!("   • Better compression and performance");
            println!("   • Atomic operations and crash recovery");
            println!("   • Built-in snapshots and backups");
            println!("\n❓ Do you want to migrate to the new format now? (Y/n): ");
            
            use std::io::{stdin, stdout, Write};
            stdout().flush().unwrap();
            
            let mut response = String::new();
            stdin().read_line(&mut response).unwrap();
            let response = response.trim().to_lowercase();
            
            if response.is_empty() || response == "y" || response == "yes" {
                println!("\n🔄 Starting migration...");
                
                match migrator.migrate() {
                    Ok(result) => {
                        println!("✅ Migration completed successfully!");
                        println!("   Collections migrated: {}", result.collections_migrated);
                        if let Some(backup) = result.backup_path {
                            println!("   Backup saved to: {}", backup.display());
                            println!("   You can delete the backup after verifying everything works.");
                        }
                        println!();
                        info!("Migration completed: {}", result.message);
                    }
                    Err(e) => {
                        eprintln!("❌ Migration failed: {}", e);
                        eprintln!("   The vectorizer will continue using the legacy format.");
                        eprintln!("   You can try migrating manually later with: vectorizer-admin storage compact");
                        info!("Migration failed: {}", e);
                    }
                }
            } else {
                println!("⏭️  Skipping migration. Using legacy format.");
                println!("   You can migrate later with: vectorizer-admin storage compact\n");
            }
        }
    }

    // Initialize vector store
    let store = vectorizer::VectorStore::new();
    info!("Vector store initialized");

    // Load project documents if specified
    let embedding_manager = if let Some(project_path) = &args.project {
        println!("📁 Loading project from: {}", project_path);
        info!("Loading project from: {}", project_path);

        // Load configuration from vectorize.yml if available
        let config = if let Some(config_path) = &args.config {
            if std::path::Path::new(config_path).exists() {
                match load_loader_config_from_yaml(config_path) {
                    Ok(config) => {
                        println!("✅ Loaded configuration from {}", config_path);
                        config
                    }
                    Err(e) => {
                        vectorizer::document_loader::LoaderConfig::default()
                    }
                }
            } else {
                println!("⚠️ Config file {} not found. Using defaults.", config_path);
                vectorizer::document_loader::LoaderConfig::default()
            }
        } else {
            vectorizer::document_loader::LoaderConfig::default()
        };
        
        let mut loader = vectorizer::document_loader::DocumentLoader::new_with_summarization(config, None);

        match loader.load_project(project_path, &store) {
            Ok(count) => {
                println!("✅ Successfully loaded {} document chunks", count);
                info!("Successfully loaded {} document chunks", count);

                // Print collection statistics
                if let Ok(stats) = loader.get_stats(&store) {
                    info!(
                        "Collection stats: {}",
                        serde_json::to_string_pretty(&stats)?
                    );
                }

                // Extract the embedding manager from the loader
                let mut manager = vectorizer::embedding::EmbeddingManager::new();

                // If a tokenizer exists, load it to ensure vocabulary persistence
                let base = std::path::Path::new(project_path).join(".vectorizer");
                let try_load = |name: &str, f: &mut dyn FnMut(&std::path::Path) -> bool| {
                    let path = base.join(name);
                    if path.exists() {
                        let ok = f(&path);
                        if ok {
                            eprintln!("Tokenizer loaded: {}", path.to_string_lossy());
                        }
                    }
                };

                // Try load tokenizer for the configured provider
                let _ = (|| {
                    let cfg = vectorizer::document_loader::LoaderConfig::default();
                    match cfg.embedding_type.as_str() {
                        "bm25" => {
                            try_load("tokenizer.bm25.json", &mut |p| {
                                if let Some(prov) = manager.get_provider_mut("bm25") {
                                    if let Some(bm25) =
                                        prov.as_any_mut()
                                            .downcast_mut::<vectorizer::embedding::Bm25Embedding>()
                                    {
                                        return bm25.load_vocabulary_json(p).is_ok();
                                    }
                                }
                                false
                            });
                        }
                        "tfidf" => {
                            try_load("tokenizer.tfidf.json", &mut |p| {
                                if let Some(prov) = manager.get_provider_mut("tfidf") {
                                    if let Some(tfidf) =
                                        prov.as_any_mut()
                                            .downcast_mut::<vectorizer::embedding::TfIdfEmbedding>()
                                    {
                                        return tfidf.load_vocabulary_json(p).is_ok();
                                    }
                                }
                                false
                            });
                        }
                        "bagofwords" => {
                            try_load("tokenizer.bow.json", &mut |p| {
                                if let Some(prov) = manager.get_provider_mut("bagofwords") {
                                    if let Some(bow) = prov
                                        .as_any_mut()
                                        .downcast_mut::<vectorizer::embedding::BagOfWordsEmbedding>(
                                    ) {
                                        return bow.load_vocabulary_json(p).is_ok();
                                    }
                                }
                                false
                            });
                        }
                        "charngram" => {
                            try_load("tokenizer.charngram.json", &mut |p| {
                                if let Some(prov) = manager.get_provider_mut("charngram") {
                                    if let Some(cng) = prov
                                        .as_any_mut()
                                        .downcast_mut::<vectorizer::embedding::CharNGramEmbedding>(
                                    ) {
                                        return cng.load_vocabulary_json(p).is_ok();
                                    }
                                }
                                false
                            });
                        }
                        _ => {}
                    }
                    Some(())
                })();

                manager
            }
            Err(e) => {
                println!("❌ Failed to load project: {}", e);
                eprintln!("Failed to load project: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Create a default embedding manager if no project is loaded
        println!("🔧 No project specified, using default embedding manager");
        let mut embedding_manager = vectorizer::embedding::EmbeddingManager::new();
        
        // Register default providers
        let tfidf = Box::new(vectorizer::embedding::TfIdfEmbedding::new(512));
        let bm25 = Box::new(vectorizer::embedding::Bm25Embedding::new(512));
        embedding_manager.register_provider("tfidf".to_string(), tfidf);
        embedding_manager.register_provider("bm25".to_string(), bm25);
        embedding_manager.set_default_provider("bm25").unwrap();
        
        embedding_manager
    };

    // Load summarization configuration if available
    let summarization_config = if let Some(config_path) = &args.config {
        if std::path::Path::new(config_path).exists() {
            match load_summarization_config_from_yaml(config_path) {
                Ok(config) => {
                    println!("✅ Loaded summarization configuration from {}", config_path);
                    info!("Loaded summarization configuration from {}", config_path);
                    Some(config)
                }
                Err(e) => {
                    Some(vectorizer::summarization::SummarizationConfig::default())
                }
            }
        } else {
            println!("⚠️ Config file {} not found. Using default summarization config.", config_path);
            Some(vectorizer::summarization::SummarizationConfig::default())
        }
    } else {
        Some(vectorizer::summarization::SummarizationConfig::default())
    };


    // Create and start the HTTP server
    let server = vectorizer::api::VectorizerServer::new(
        &args.host,
        args.port,
        store.into(),
        embedding_manager,
        summarization_config,
    );

    info!("Starting REST API server...");
    server.start().await?;

    Ok(())
}

/// Load LoaderConfig from vectorize.yml
fn load_loader_config_from_yaml(config_path: &str) -> Result<vectorizer::document_loader::LoaderConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(config_path)?;
    let yaml_config: YamlConfig = serde_yaml::from_str(&content)?;
    
    // Convert to LoaderConfig
    let mut config = vectorizer::document_loader::LoaderConfig::default();
    
    // Find the first project and its first collection
    if let Some(project) = yaml_config.projects.first() {
        if let Some(collection) = project.collections.first() {
            config.collection_name = collection.name.clone();
            config.max_chunk_size = collection.chunk_size;
            config.chunk_overlap = collection.chunk_overlap;
            config.embedding_type = collection.embedding_provider.clone();
            config.embedding_dimension = collection.dimension;
            config.include_patterns = collection.include_patterns.clone();
            config.exclude_patterns = collection.exclude_patterns.clone();
        }
    }
    
    Ok(config)
}

/// Load SummarizationConfig from YAML file
fn load_summarization_config_from_yaml(config_path: &str) -> Result<vectorizer::summarization::SummarizationConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(config_path)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    vectorizer::summarization::SummarizationConfig::from_yaml(&yaml_value)
        .map_err(|e| format!("Failed to parse summarization config: {}", e).into())
}

#[derive(Debug, Deserialize)]
struct YamlConfig {
    projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
struct Project {
    name: String,
    path: String,
    collections: Vec<Collection>,
}

#[derive(Debug, Deserialize)]
struct Collection {
    name: String,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    chunk_size: usize,
    chunk_overlap: usize,
    embedding_provider: String,
    dimension: usize,
}
