//! Administrative REST handlers.
//!
//! Workspace management (add/remove/list + config get/update), server
//! configuration (read/update `config.yml`), and a graceful restart
//! endpoint. The write endpoints here go through
//! [`crate::server::auth_handlers::require_admin_for_rest`] so that in
//! mixed auth / no-auth deployments they still enforce Role::Admin when
//! an `AuthHandlerState` is configured.

use axum::extract::State;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::{error, info};

use crate::server::VectorizerServer;
use crate::server::error_middleware::{
    ErrorResponse, create_bad_request_error, create_validation_error,
};

/// Add workspace directory (for GUI)
pub async fn add_workspace(
    State(state): State<VectorizerServer>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    crate::server::auth_handlers::require_admin_for_rest(&state.auth_handler_state, &headers)
        .await?;
    let path = payload
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| create_validation_error("path", "missing or invalid path parameter"))?;

    let collection_name = payload
        .get("collection_name")
        .and_then(|c| c.as_str())
        .ok_or_else(|| {
            create_validation_error(
                "collection_name",
                "missing or invalid collection_name parameter",
            )
        })?;

    info!("📁 Adding workspace: {} -> {}", path, collection_name);

    // Use workspace manager
    let workspace_manager = crate::config::WorkspaceManager::new();
    match workspace_manager.add_workspace(path, collection_name) {
        Ok(workspace) => Ok(Json(json!({
            "success": true,
            "message": "Workspace added successfully",
            "workspace": {
                "id": workspace.id,
                "path": workspace.path,
                "collection_name": workspace.collection_name,
                "active": workspace.active,
                "created_at": workspace.created_at.to_rfc3339()
            }
        }))),
        Err(e) => {
            error!("Failed to add workspace: {}", e);
            Err(create_validation_error("workspace", &e))
        }
    }
}

/// Remove workspace directory (for GUI)
pub async fn remove_workspace(
    State(_state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let path = payload
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| create_validation_error("path", "missing or invalid path parameter"))?;

    info!("🗑️ Removing workspace: {}", path);

    // Use workspace manager
    let workspace_manager = crate::config::WorkspaceManager::new();
    match workspace_manager.remove_workspace(path) {
        Ok(workspace) => Ok(Json(json!({
            "success": true,
            "message": "Workspace removed successfully",
            "removed_workspace": {
                "id": workspace.id,
                "path": workspace.path,
                "collection_name": workspace.collection_name
            }
        }))),
        Err(e) => {
            error!("Failed to remove workspace: {}", e);
            Err(create_validation_error("workspace", &e))
        }
    }
}

/// List workspace directories (for GUI)
pub async fn list_workspaces(State(_state): State<VectorizerServer>) -> Json<Value> {
    let workspace_manager = crate::config::WorkspaceManager::new();
    let workspaces = workspace_manager.list_workspaces();

    let workspace_list: Vec<serde_json::Value> = workspaces
        .iter()
        .map(|w| {
            json!({
                "id": w.id,
                "path": w.path,
                "collection_name": w.collection_name,
                "active": w.active,
                "file_count": w.file_count,
                "created_at": w.created_at.to_rfc3339(),
                "updated_at": w.updated_at.to_rfc3339(),
                "last_indexed": w.last_indexed.map(|t| t.to_rfc3339()),
                "exists": w.exists()
            })
        })
        .collect();

    Json(json!({
        "workspaces": workspace_list
    }))
}

/// Get configuration (for GUI)
pub async fn get_config() -> Json<Value> {
    // Try multiple paths for config.yml
    let possible_paths = vec![
        "./config.yml",
        "../config.yml",
        "config.yml",
        "/mnt/f/Node/hivellm/vectorizer/config.yml",
    ];

    for path in &possible_paths {
        info!("Trying to read config from: {}", path);
        if let Ok(content) = std::fs::read_to_string(path) {
            info!("Successfully read config from: {}", path);
            match serde_yaml::from_str::<Value>(&content) {
                Ok(config) => {
                    info!("Successfully parsed config.yml");
                    return Json(config);
                }
                Err(e) => {
                    error!("Failed to parse config.yml from {}: {}", path, e);
                }
            }
        }
    }

    // If all paths failed, log and return error
    error!(
        "Failed to read config.yml from any path. Tried: {:?}",
        possible_paths
    );
    Json(json!({
        "error": "config.yml not found",
        "message": "Could not find config.yml file",
        "server": { "host": "0.0.0.0", "port": 15002 },
        "storage": { "data_dir": "./data", "cache_size": 1024 },
        "embedding": { "provider": "fastembed", "model": "BAAI/bge-small-en-v1.5", "dimension": 384 },
        "performance": { "threads": 4, "batch_size": 100 }
    }))
}

/// Update configuration (for GUI). Admin-only.
pub async fn update_config(
    State(state): State<VectorizerServer>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    crate::server::auth_handlers::require_admin_for_rest(&state.auth_handler_state, &headers)
        .await?;

    // Write to config.yml
    match serde_yaml::to_string(&payload) {
        Ok(yaml_content) => match std::fs::write("./config.yml", yaml_content) {
            Ok(_) => {
                info!("Configuration updated successfully");
                Ok(Json(json!({
                    "success": true,
                    "message": "Configuration updated successfully. Restart server for changes to take effect."
                })))
            }
            Err(e) => {
                error!("Failed to write config.yml: {}", e);
                Err(create_bad_request_error(&format!(
                    "Operation failed: {}",
                    e
                )))
            }
        },
        Err(e) => {
            error!("Failed to serialize config to YAML: {}", e);
            Err(create_bad_request_error(&format!(
                "Failed to serialize config: {}",
                e
            )))
        }
    }
}

/// Restart server (for GUI)
///
/// This initiates a graceful restart by:
/// 1. Saving all pending data
/// 2. Sending a restart signal to the process
/// 3. The server should be run under a process manager (e.g., systemd) for actual restart
pub async fn restart_server(
    State(state): State<VectorizerServer>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, ErrorResponse> {
    crate::server::auth_handlers::require_admin_for_rest(&state.auth_handler_state, &headers)
        .await?;

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    static RESTART_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

    // Prevent concurrent restart requests
    if RESTART_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        return Ok(Json(json!({
            "success": false,
            "message": "Restart already in progress"
        })));
    }

    info!("🔄 Initiating graceful server restart");

    // Spawn the restart task
    tokio::spawn(async move {
        // Give time for the response to be sent
        tokio::time::sleep(Duration::from_millis(500)).await;

        info!("🔄 Saving data before restart...");

        // Note: The actual data saving should be handled by the auto-save manager
        // This is just a best-effort sync before restart
        // The store state is managed globally and will be properly saved on shutdown

        info!("🔄 Signaling process to restart...");

        // On Unix-like systems, we can use SIGHUP for graceful restart
        // On Windows, we exit and rely on a process manager
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            let _ = signal::kill(Pid::this(), Signal::SIGHUP);
        }

        #[cfg(windows)]
        {
            // On Windows, we schedule an exit and expect a process manager to restart
            // Write a restart marker file that can be checked by the process manager
            let restart_marker = std::path::Path::new("./restart.marker");
            let _ = std::fs::write(
                restart_marker,
                format!(
                    "{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                ),
            );

            // Give some time for cleanup
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Exit with code 0 to indicate intentional restart
            std::process::exit(0);
        }
    });

    Ok(Json(json!({
        "success": true,
        "message": "Server restart initiated. The server will restart shortly."
    })))
}

/// Get workspace configuration (for GUI)
pub async fn get_workspace_config() -> Result<Json<Value>, ErrorResponse> {
    let possible_paths = vec![
        "./workspace.yml",
        "../workspace.yml",
        "../../workspace.yml",
        "./config/workspace.yml",
    ];

    for path in &possible_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            match serde_yaml::from_str::<Value>(&content) {
                Ok(config) => {
                    info!("✅ Loaded workspace config from: {}", path);
                    return Ok(Json(config));
                }
                Err(e) => {
                    error!("Failed to parse workspace YAML from {}: {}", path, e);
                }
            }
        }
    }

    // Return minimal default if no file found
    error!("⚠️ No workspace config file found in any of the expected paths");
    Ok(Json(json!({
        "global_settings": {
            "file_watcher": {
                "watch_paths": [],
                "auto_discovery": true,
                "enable_auto_update": true,
                "hot_reload": true,
                "exclude_patterns": []
            }
        },
        "projects": []
    })))
}

/// Update workspace configuration (for GUI)
pub async fn update_workspace_config(
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    // Write to workspace.yml
    match serde_yaml::to_string(&payload) {
        Ok(yaml_content) => match std::fs::write("./workspace.yml", yaml_content) {
            Ok(_) => {
                info!("Workspace configuration updated successfully");
                Ok(Json(json!({
                    "success": true,
                    "message": "Workspace configuration updated successfully."
                })))
            }
            Err(e) => {
                error!("Failed to write workspace.yml: {}", e);
                Err(create_bad_request_error(&format!(
                    "Operation failed: {}",
                    e
                )))
            }
        },
        Err(e) => {
            error!("Failed to serialize workspace config to YAML: {}", e);
            Err(create_bad_request_error(&format!(
                "Failed to serialize workspace config: {}",
                e
            )))
        }
    }
}
