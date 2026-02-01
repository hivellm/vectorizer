//! Setup Wizard Handlers
//!
//! REST API handlers for the setup wizard functionality

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{error, info};

use crate::server::VectorizerServer;
use crate::server::error_middleware::ErrorResponse;
use crate::workspace::project_analyzer::{ProjectAnalysis, analyze_directory};
use crate::workspace::setup_config::{ApplyConfigRequest, write_workspace_config};
use crate::workspace::templates::{ConfigTemplate, get_template_by_id, get_templates};

/// Setup status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatus {
    /// Whether initial setup is needed
    pub needs_setup: bool,
    /// Current server version
    pub version: String,
    /// Deployment type (binary, docker)
    pub deployment_type: String,
    /// Whether workspace.yml exists
    pub has_workspace_config: bool,
    /// Number of existing projects
    pub project_count: usize,
    /// Number of existing collections
    pub collection_count: usize,
}

/// Analyze directory request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    /// Path to the directory to analyze
    pub path: String,
}

// ApplyConfigRequest and related structs imported from crate::workspace::setup_config

/// API Result type
type ApiResult<T> = Result<T, ErrorResponse>;

/// GET /setup/status
///
/// Check if initial setup is needed and return current status
pub async fn get_setup_status(State(state): State<VectorizerServer>) -> Json<SetupStatus> {
    let workspace_exists = std::path::Path::new("workspace.yml").exists();
    
    // Check if running in Docker
    let in_docker =
        std::env::var("VECTORIZER_DOCKER").is_ok() || std::path::Path::new("/.dockerenv").exists();
    
    let deployment_type = if in_docker {
        "docker".to_string()
    } else {
        "binary".to_string()
    };
    
    // Get project count from workspace
    let project_count = if workspace_exists {
        std::fs::read_to_string("workspace.yml")
            .ok()
            .and_then(|content| serde_yaml::from_str::<Value>(&content).ok())
            .and_then(|config| {
                config
                    .get("projects")
                    .and_then(|p| p.as_array().map(|a| a.len()))
            })
            .unwrap_or(0)
    } else {
        0
    };
    
    let collection_count = state.store.list_collections().len();
    
    // Needs setup if no workspace config (regardless of collections)
    // This ensures the setup wizard is shown even if some default collections exist
    let needs_setup = !workspace_exists;
    
    Json(SetupStatus {
        needs_setup,
        version: env!("CARGO_PKG_VERSION").to_string(),
        deployment_type,
        has_workspace_config: workspace_exists,
        project_count,
        collection_count,
    })
}

/// POST /setup/analyze
///
/// Analyze a directory and return project information
pub async fn analyze_project_directory(
    Json(payload): Json<AnalyzeRequest>,
) -> ApiResult<Json<ProjectAnalysis>> {
    info!("🔍 Analyzing directory: {}", payload.path);
    
    match analyze_directory(&payload.path) {
        Ok(analysis) => {
            info!(
                "✅ Analysis complete: {:?} project with {} languages, {} suggested collections",
                analysis.project_types.first(),
                analysis.languages.len(),
                analysis.suggested_collections.len()
            );
            Ok(Json(analysis))
        }
        Err(e) => {
            error!("❌ Failed to analyze directory: {}", e);
            Err(ErrorResponse::new(
                "ANALYSIS_FAILED".to_string(),
                e,
                StatusCode::BAD_REQUEST,
            ))
        }
    }
}

/// POST /setup/apply
///
/// Apply the setup configuration (create workspace.yml, create collections, and index files)
pub async fn apply_setup_config(
    State(state): State<VectorizerServer>,
    Json(payload): Json<ApplyConfigRequest>,
) -> ApiResult<Json<Value>> {
    info!(
        "📝 Applying setup configuration with {} projects",
        payload.projects.len()
    );
    
    // 1. Write workspace.yml first
    if let Err(e) = write_workspace_config(&payload, "workspace.yml") {
        error!("❌ Failed to write workspace.yml: {}", e);
        return Err(ErrorResponse::new(
            "WRITE_FAILED".to_string(),
            format!("Failed to write workspace.yml: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
    info!("✅ workspace.yml created successfully");
    
    // 2. Create collections and index files
    let mut created_collections = Vec::new();
    let mut errors = Vec::new();
    let mut total_vectors = 0;
    
    for project in &payload.projects {
        info!(
            "📁 Processing project: {} at {}",
            project.name, project.path
        );
        
        let project_path = std::path::Path::new(&project.path);
        if !project_path.exists() {
            error!("❌ Project path does not exist: {}", project.path);
            errors.push(format!("Project path does not exist: {}", project.path));
            continue;
        }
        
        for collection in &project.collections {
            let collection_name = &collection.name;
            info!("📦 Processing collection: {}", collection_name);
            
            // Check if collection already exists
            let existing_collections = state.store.list_collections();
            if existing_collections.contains(&collection_name.to_string()) {
                info!(
                    "ℹ️  Collection {} already exists, will reindex",
                    collection_name
                );
            }
            
            // Index project files using FileLoader
            match index_project_with_loader(
                &state.store,
                collection_name,
                &project.path,
                &collection.include_patterns,
                &collection.exclude_patterns,
            )
            .await
            {
                Ok(vector_count) => {
                    info!(
                        "✅ Collection {} indexed with {} vectors",
                        collection_name, vector_count
                    );
                    total_vectors += vector_count;

                    // Enable GraphRAG if requested
                    let graph_enabled = if collection.enable_graph == Some(true) {
                        match state.store.enable_graph_for_collection(collection_name) {
                            Ok(_) => {
                                info!("✅ GraphRAG enabled for collection {}", collection_name);
                                true
                            }
                            Err(e) => {
                                error!(
                                    "⚠️ Failed to enable GraphRAG for collection {}: {}",
                                    collection_name, e
                                );
                                false
                            }
                        }
                    } else {
                        false
                    };

                    created_collections.push(json!({
                        "name": collection_name,
                        "status": "indexed",
                        "vector_count": vector_count,
                        "project_path": project.path,
                        "graph_enabled": graph_enabled,
                    }));
                }
                Err(e) => {
                    error!("❌ Failed to index collection {}: {}", collection_name, e);
                    errors.push(format!(
                        "Failed to index collection {}: {}",
                        collection_name, e
                    ));
                }
            }
        }
    }
    
    let success = errors.is_empty();
    let collections_created = created_collections.len();
    
    let message = if success {
        format!(
            "Setup completed successfully. Indexed {} collections with {} vectors.",
            collections_created, total_vectors
        )
    } else {
        format!("Setup completed with {} errors", errors.len())
    };
    
    info!(
        "✅ Setup apply completed: {} collections, {} vectors, {} errors",
        collections_created,
        total_vectors,
        errors.len()
    );
    
    Ok(Json(json!({
        "success": success,
        "message": message,
        "workspace_file": "workspace.yml",
        "projects_count": payload.projects.len(),
        "collections_created": collections_created,
        "total_vectors": total_vectors,
        "collections": created_collections,
        "errors": errors,
    })))
}

/// Index a project directory using FileLoader
async fn index_project_with_loader(
    store: &std::sync::Arc<crate::db::VectorStore>,
    collection_name: &str,
    project_path: &str,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    use crate::embedding::{Bm25Embedding, EmbeddingManager};
    use crate::file_loader::{FileLoader, LoaderConfig};
    
    info!(
        "🔄 Indexing project {} into collection {}",
        project_path, collection_name
    );
    
    // Build include patterns - if empty, use default patterns for common code/docs
    let effective_include = if include_patterns.is_empty() {
        vec![
            "**/*.rs".to_string(),
            "**/*.py".to_string(),
            "**/*.js".to_string(),
            "**/*.ts".to_string(),
            "**/*.tsx".to_string(),
            "**/*.jsx".to_string(),
            "**/*.go".to_string(),
            "**/*.java".to_string(),
            "**/*.md".to_string(),
            "**/*.txt".to_string(),
            "**/*.yaml".to_string(),
            "**/*.yml".to_string(),
            "**/*.toml".to_string(),
            "**/*.json".to_string(),
        ]
    } else {
        include_patterns.to_vec()
    };
    
    // Build exclude patterns - add default excludes
    let mut effective_exclude = exclude_patterns.to_vec();
    let default_excludes = vec![
        "**/node_modules/**".to_string(),
        "**/target/**".to_string(),
        "**/.git/**".to_string(),
        "**/venv/**".to_string(),
        "**/__pycache__/**".to_string(),
        "**/dist/**".to_string(),
        "**/build/**".to_string(),
        "**/.next/**".to_string(),
        "**/coverage/**".to_string(),
        "**/*.lock".to_string(),
        "**/package-lock.json".to_string(),
        "**/yarn.lock".to_string(),
        "**/pnpm-lock.yaml".to_string(),
        "**/Cargo.lock".to_string(),
    ];
    for pattern in default_excludes {
        if !effective_exclude.contains(&pattern) {
            effective_exclude.push(pattern);
        }
    }
    
    // Create loader config
    let mut loader_config = LoaderConfig {
        max_chunk_size: 2048,
        chunk_overlap: 256,
        include_patterns: effective_include,
        exclude_patterns: effective_exclude,
        embedding_dimension: 512,
        embedding_type: "bm25".to_string(),
        collection_name: collection_name.to_string(),
        max_file_size: 5 * 1024 * 1024, // 5MB
    };
    
    // Ensure hardcoded excludes are applied
    loader_config.ensure_hardcoded_excludes();
    
    // Create embedding manager
    let mut embedding_manager = EmbeddingManager::new();
    let bm25 = Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    if let Err(e) = embedding_manager.set_default_provider("bm25") {
        error!("Failed to set default embedding provider: {}", e);
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to set embedding provider: {}", e),
        )));
    }
    
    // Create FileLoader
    let mut loader = FileLoader::with_embedding_manager(loader_config, embedding_manager);
    
    // Index the project
    match loader.load_and_index_project(project_path, store).await {
        Ok(vector_count) => {
            info!(
                "✅ Indexed {} vectors for collection '{}'",
                vector_count, collection_name
            );
            Ok(vector_count)
        }
        Err(e) => {
            error!("❌ Failed to index project: {}", e);
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Indexing failed: {}", e),
            )))
        }
    }
}

/// GET /setup/verify
///
/// Verify that setup was completed successfully
pub async fn verify_setup(State(state): State<VectorizerServer>) -> Json<Value> {
    let workspace_exists = std::path::Path::new("workspace.yml").exists();
    let collection_count = state.store.list_collections().len();
    
    // Check health endpoint internally
    let health_ok = true; // Server is running if this endpoint responds
    
    // Parse workspace config for verification
    let workspace_status = if workspace_exists {
        match std::fs::read_to_string("workspace.yml") {
            Ok(content) => match serde_yaml::from_str::<Value>(&content) {
                Ok(config) => {
                    let project_count = config
                        .get("projects")
                        .and_then(|p| p.as_array().map(|a| a.len()))
                        .unwrap_or(0);
                    json!({
                        "valid": true,
                        "project_count": project_count,
                    })
                }
                Err(e) => json!({
                    "valid": false,
                    "error": format!("Invalid YAML: {}", e),
                }),
            },
            Err(e) => json!({
                "valid": false,
                "error": format!("Cannot read file: {}", e),
            }),
        }
    } else {
        json!({
            "valid": false,
            "error": "workspace.yml not found",
        })
    };
    
    let setup_complete = workspace_exists && health_ok;
    
    Json(json!({
        "setup_complete": setup_complete,
        "health": {
            "status": if health_ok { "healthy" } else { "unhealthy" },
            "version": env!("CARGO_PKG_VERSION"),
        },
        "workspace": workspace_status,
        "collections": {
            "count": collection_count,
        },
        "next_steps": if setup_complete {
            vec![
                "Restart the server to apply workspace configuration",
                "Visit the dashboard to manage your collections",
                "Use the API or SDK to start inserting data",
            ]
        } else {
            vec![
                "Complete the setup wizard",
                "Add at least one project to your workspace",
            ]
        },
    }))
}

/// GET /workspace/config
/// 
/// Get the current workspace configuration
pub async fn get_workspace_config() -> Json<Value> {
    if let Ok(content) = std::fs::read_to_string("workspace.yml") {
        if let Ok(config) = serde_yaml::from_str::<Value>(&content) {
            return Json(config);
        }
    }
    
    // Return default empty config
    Json(json!({
        "global_settings": {
            "file_watcher": {
                "auto_discovery": true,
                "enable_auto_update": true,
                "hot_reload": true,
                "watch_paths": [],
                "exclude_patterns": [],
            }
        },
        "projects": []
    }))
}

/// POST /workspace/config
///
/// Update the workspace configuration
pub async fn update_workspace_config(Json(payload): Json<Value>) -> ApiResult<Json<Value>> {
    match serde_yaml::to_string(&payload) {
        Ok(yaml_content) => {
            let yaml_with_header = format!(
                "# Vectorizer Workspace Configuration\n\
                 # Updated on {}\n\n{}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                yaml_content
            );
            
            match std::fs::write("workspace.yml", yaml_with_header) {
                Ok(_) => {
                    info!("✅ workspace.yml updated successfully");
                    Ok(Json(json!({
                        "success": true,
                        "message": "Workspace configuration updated successfully",
                    })))
                }
                Err(e) => {
                    error!("❌ Failed to update workspace.yml: {}", e);
                    Err(ErrorResponse::new(
                        "WRITE_FAILED".to_string(),
                        format!("Failed to write workspace.yml: {}", e),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to serialize workspace config: {}", e);
            Err(ErrorResponse::new(
                "SERIALIZATION_FAILED".to_string(),
                format!("Failed to serialize config: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// GET /setup/templates
///
/// Get all available configuration templates
pub async fn get_configuration_templates() -> Json<Vec<ConfigTemplate>> {
    Json(get_templates())
}

/// GET /setup/templates/:id
///
/// Get a specific configuration template by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateIdPath {
    pub id: String,
}

pub async fn get_configuration_template_by_id(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> ApiResult<Json<ConfigTemplate>> {
    match get_template_by_id(&id) {
        Some(template) => Ok(Json(template)),
        None => Err(ErrorResponse::new(
            "TEMPLATE_NOT_FOUND".to_string(),
            format!("Template '{}' not found", id),
            StatusCode::NOT_FOUND,
        )),
    }
}

/// Check if first-time setup is needed and display guidance in terminal
///
/// Call this function during server startup to show the setup wizard URL
/// if no workspace configuration exists.
pub fn display_first_start_guidance(host: &str, port: u16, collection_count: usize) {
    let workspace_exists = std::path::Path::new("workspace.yml").exists();
    let needs_setup = !workspace_exists && collection_count == 0;
    
    if needs_setup {
        let base_url = if host == "0.0.0.0" {
            format!("http://localhost:{}", port)
        } else {
            format!("http://{}:{}", host, port)
        };
        
        println!();
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                                                                  ║");
        println!("║  🚀 Welcome to Vectorizer!                                       ║");
        println!("║                                                                  ║");
        println!("║  First time setup detected.                                      ║");
        println!("║  Configure your workspace using the Setup Wizard:                ║");
        println!("║                                                                  ║");
        println!(
            "║  👉 {}/dashboard/setup                                       ║",
            base_url
        );
        println!("║                                                                  ║");
        println!("║  Or use the CLI:                                                 ║");
        println!("║  $ vectorizer-cli setup /path/to/your/project                    ║");
        println!("║                                                                  ║");
        println!("║  Quick links:                                                    ║");
        println!(
            "║  • Dashboard:     {}/dashboard/overview                      ║",
            base_url
        );
        println!(
            "║  • API Docs:      {}/dashboard/docs                          ║",
            base_url
        );
        println!(
            "║  • Health Check:  {}/health                        ║",
            base_url
        );
        println!("║                                                                  ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");
        println!();
    }
}

/// Check if setup is needed (utility function for other modules)
pub fn needs_setup(collection_count: usize) -> bool {
    let workspace_exists = std::path::Path::new("workspace.yml").exists();
    !workspace_exists && collection_count == 0
}

/// Request to browse directories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseDirectoryRequest {
    /// Path to browse (empty string or "/" for root/home)
    pub path: Option<String>,
}

/// Directory entry in browse response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// Name of the entry
    pub name: String,
    /// Full path
    pub path: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Size in bytes (for files)
    pub size: Option<u64>,
    /// Whether this looks like a project folder
    pub is_project: bool,
}

/// Browse directory response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseDirectoryResponse {
    /// Current directory path
    pub current_path: String,
    /// Parent directory path (None if at root)
    pub parent_path: Option<String>,
    /// List of entries
    pub entries: Vec<DirectoryEntry>,
    /// Whether the path is valid
    pub valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
}

/// POST /setup/browse
///
/// Browse directories for file picker in setup wizard
pub async fn browse_directory(
    Json(payload): Json<BrowseDirectoryRequest>,
) -> Json<BrowseDirectoryResponse> {
    use std::path::{Path, PathBuf};
    
    // Determine the path to browse
    let path_str = payload.path.unwrap_or_default();
    let browse_path: PathBuf = if path_str.is_empty() || path_str == "/" || path_str == "~" {
        // Default to home directory
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    } else if path_str.starts_with('~') {
        // Expand ~ to home directory
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        home.join(&path_str[1..].trim_start_matches('/'))
    } else {
        PathBuf::from(&path_str)
    };
    
    // Verify path exists and is a directory
    if !browse_path.exists() {
        return Json(BrowseDirectoryResponse {
            current_path: path_str,
            parent_path: None,
            entries: vec![],
            valid: false,
            error: Some("Path does not exist".to_string()),
        });
    }
    
    if !browse_path.is_dir() {
        return Json(BrowseDirectoryResponse {
            current_path: path_str,
            parent_path: None,
            entries: vec![],
            valid: false,
            error: Some("Path is not a directory".to_string()),
        });
    }
    
    // Get parent path
    let parent_path = browse_path
        .parent()
        .map(|p| p.to_string_lossy().to_string());
    
    // Read directory entries
    let mut entries: Vec<DirectoryEntry> = Vec::new();
    
    if let Ok(read_dir) = std::fs::read_dir(&browse_path) {
        for entry_result in read_dir {
            if let Ok(entry) = entry_result {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip hidden files (start with .)
                if name.starts_with('.') {
                    continue;
                }
                
                let is_dir = entry_path.is_dir();
                let metadata = entry.metadata().ok();
                
                // Check if this looks like a project folder
                let is_project = if is_dir {
                    check_is_project_folder(&entry_path)
                } else {
                    false
                };
                
                entries.push(DirectoryEntry {
                    name,
                    path: entry_path.to_string_lossy().to_string(),
                    is_directory: is_dir,
                    size: metadata.as_ref().map(|m| m.len()),
                    is_project,
                });
            }
        }
    }
    
    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    
    Json(BrowseDirectoryResponse {
        current_path: browse_path.to_string_lossy().to_string(),
        parent_path,
        entries,
        valid: true,
        error: None,
    })
}

/// Check if a directory looks like a project folder
fn check_is_project_folder(path: &std::path::Path) -> bool {
    // Common project indicators
    let project_files = [
        "Cargo.toml",       // Rust
        "package.json",     // Node.js
        "pyproject.toml",   // Python (modern)
        "setup.py",         // Python (legacy)
        "requirements.txt", // Python
        "go.mod",           // Go
        "pom.xml",          // Java/Maven
        "build.gradle",     // Java/Gradle
        "Gemfile",          // Ruby
        "composer.json",    // PHP
        ".git",             // Git repo
        "README.md",        // Documentation
        "README.rst",
        "Makefile",
    ];
    
    for file in project_files.iter() {
        if path.join(file).exists() {
            return true;
        }
    }
    
    false
}
