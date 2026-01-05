//! Setup Wizard Handlers
//!
//! REST API handlers for the setup wizard functionality

use axum::{extract::State, response::Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::server::{VectorizerServer, error_middleware::ErrorResponse};
use crate::workspace::project_analyzer::{analyze_directory, ProjectAnalysis};
use crate::workspace::setup_config::{ApplyConfigRequest, write_workspace_config};
use crate::workspace::templates::{get_templates, get_template_by_id, ConfigTemplate};

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
pub async fn get_setup_status(
    State(state): State<VectorizerServer>,
) -> Json<SetupStatus> {
    let workspace_exists = std::path::Path::new("workspace.yml").exists();
    
    // Check if running in Docker
    let in_docker = std::env::var("VECTORIZER_DOCKER").is_ok() 
        || std::path::Path::new("/.dockerenv").exists();
    
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
            .and_then(|config| config.get("projects").and_then(|p| p.as_array().map(|a| a.len())))
            .unwrap_or(0)
    } else {
        0
    };
    
    let collection_count = state.store.list_collections().len();
    
    // Needs setup if no workspace config and no collections
    let needs_setup = !workspace_exists && collection_count == 0;
    
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
/// Apply the setup configuration (create workspace.yml and optionally trigger indexing)
/// Apply the setup configuration (create workspace.yml and optionally trigger indexing)
pub async fn apply_setup_config(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<ApplyConfigRequest>,
) -> ApiResult<Json<Value>> {
    info!("📝 Applying setup configuration with {} projects", payload.projects.len());
    
    // Use shared logic to write configuration
    match write_workspace_config(&payload, "workspace.yml") {
        Ok(_) => {
            info!("✅ workspace.yml created successfully");
            Ok(Json(json!({
                "success": true,
                "message": "Setup configuration applied successfully",
                "workspace_file": "workspace.yml",
                "projects_count": payload.projects.len(),
            })))
        }
        Err(e) => {
            error!("❌ Failed to write workspace.yml: {}", e);
            Err(ErrorResponse::new(
                "WRITE_FAILED".to_string(),
                format!("Failed to write workspace.yml: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// GET /setup/verify
///
/// Verify that setup was completed successfully
pub async fn verify_setup(
    State(state): State<VectorizerServer>,
) -> Json<Value> {
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
pub async fn update_workspace_config(
    Json(payload): Json<Value>,
) -> ApiResult<Json<Value>> {
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
        println!("║  👉 {}/setup                                       ║", base_url);
        println!("║                                                                  ║");
        println!("║  Or use the CLI:                                                 ║");
        println!("║  $ vectorizer-cli setup /path/to/your/project                    ║");
        println!("║                                                                  ║");
        println!("║  Quick links:                                                    ║");
        println!("║  • Dashboard:     {}/overview                      ║", base_url);
        println!("║  • API Docs:      {}/docs                          ║", base_url);
        println!("║  • Health Check:  {}/health                        ║", base_url);
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
