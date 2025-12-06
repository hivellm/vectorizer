//! REST API handlers for HiveHub user-scoped backups
//!
//! These endpoints provide backup/restore functionality for HiveHub cluster mode,
//! with all operations scoped to the authenticated user.

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{error, info, warn};
use uuid::Uuid;

use super::VectorizerServer;
use super::error_middleware::ErrorResponse;
use crate::hub::backup::{RestoreResult, UserBackupInfo};
use crate::monitoring::metrics::METRICS;

/// Query parameters for list backups
#[derive(Debug, Deserialize)]
pub struct ListBackupsQuery {
    /// User ID to list backups for (required for HiveHub)
    pub user_id: Uuid,
}

/// Request body for creating a backup
#[derive(Debug, Deserialize)]
pub struct CreateBackupRequest {
    /// User ID who owns the backup
    pub user_id: Uuid,
    /// Human-readable backup name
    pub name: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Optional list of collection names (None = all user's collections)
    #[serde(default)]
    pub collections: Option<Vec<String>>,
}

/// Request body for restoring a backup
#[derive(Debug, Deserialize)]
pub struct RestoreBackupRequest {
    /// User ID
    pub user_id: Uuid,
    /// Backup ID to restore
    pub backup_id: Uuid,
    /// Whether to overwrite existing collections
    #[serde(default)]
    pub overwrite: bool,
}

/// Query parameters for download/delete backup
#[derive(Debug, Deserialize)]
pub struct BackupQuery {
    /// User ID
    pub user_id: Uuid,
}

/// Query parameters for upload backup
#[derive(Debug, Deserialize)]
pub struct UploadBackupQuery {
    /// User ID
    pub user_id: Uuid,
    /// Optional backup name (overrides name in uploaded file)
    #[serde(default)]
    pub name: Option<String>,
}

/// Response for backup operations
#[derive(Debug, Serialize)]
pub struct BackupResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup: Option<UserBackupInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backups: Option<Vec<UserBackupInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore_result: Option<RestoreResult>,
}

/// List all backups for a user
///
/// GET /api/hub/backups?user_id=<uuid>
pub async fn list_user_backups(
    State(state): State<VectorizerServer>,
    Query(query): Query<ListBackupsQuery>,
) -> Result<Json<BackupResponse>, ErrorResponse> {
    let backup_manager = state.backup_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "BACKUP_DISABLED".to_string(),
            "HiveHub backup functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    let backups = backup_manager
        .list_backups(&query.user_id)
        .await
        .map_err(|e| {
            ErrorResponse::new(
                "BACKUP_LIST_ERROR".to_string(),
                format!("Failed to list backups: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    info!(
        "Listed {} backups for user {}",
        backups.len(),
        query.user_id
    );

    Ok(Json(BackupResponse {
        success: true,
        message: format!("Found {} backups", backups.len()),
        backup: None,
        backups: Some(backups),
        restore_result: None,
    }))
}

/// Get backup info by ID
///
/// GET /api/hub/backups/:backup_id?user_id=<uuid>
pub async fn get_user_backup(
    State(state): State<VectorizerServer>,
    Path(backup_id): Path<Uuid>,
    Query(query): Query<BackupQuery>,
) -> Result<Json<BackupResponse>, ErrorResponse> {
    let backup_manager = state.backup_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "BACKUP_DISABLED".to_string(),
            "HiveHub backup functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    let backup = backup_manager
        .get_backup(&query.user_id, &backup_id)
        .await
        .map_err(|e| {
            ErrorResponse::new(
                "BACKUP_NOT_FOUND".to_string(),
                format!("Backup not found: {}", e),
                StatusCode::NOT_FOUND,
            )
        })?;

    Ok(Json(BackupResponse {
        success: true,
        message: "Backup found".to_string(),
        backup: Some(backup),
        backups: None,
        restore_result: None,
    }))
}

/// Create a new backup for a user
///
/// POST /api/hub/backups
pub async fn create_user_backup(
    State(state): State<VectorizerServer>,
    Json(request): Json<CreateBackupRequest>,
) -> Result<Json<BackupResponse>, ErrorResponse> {
    let backup_manager = state.backup_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "BACKUP_DISABLED".to_string(),
            "HiveHub backup functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Check quota if HubManager is enabled
    if let Some(ref hub_manager) = state.hub_manager {
        info!(
            "Creating backup for user {} via HiveHub cluster mode",
            request.user_id
        );

        // Check storage quota before creating backup
        // Estimate backup size based on user's collections
        let estimated_size = backup_manager
            .estimate_backup_size(&request.user_id, request.collections.as_deref())
            .await
            .unwrap_or(0);

        if estimated_size > 0 {
            let tenant_id = request.user_id.to_string();
            match hub_manager
                .check_quota(&tenant_id, crate::hub::QuotaType::Storage, estimated_size)
                .await
            {
                Ok(allowed) if !allowed => {
                    METRICS
                        .hub_backup_operations_total
                        .with_label_values(&["create", "quota_exceeded"])
                        .inc();
                    return Err(ErrorResponse::new(
                        "QUOTA_EXCEEDED".to_string(),
                        "Storage quota exceeded. Cannot create backup.".to_string(),
                        StatusCode::TOO_MANY_REQUESTS,
                    ));
                }
                Err(e) => {
                    warn!("Failed to check quota for backup: {}", e);
                    // Continue with backup creation - fail open
                }
                _ => {}
            }
        }
    }

    let backup = backup_manager
        .create_backup(
            request.user_id,
            request.name.clone(),
            request.description,
            request.collections,
        )
        .await
        .map_err(|e| {
            METRICS
                .hub_backup_operations_total
                .with_label_values(&["create", "error"])
                .inc();
            ErrorResponse::new(
                "BACKUP_CREATE_ERROR".to_string(),
                format!("Failed to create backup: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    METRICS
        .hub_backup_operations_total
        .with_label_values(&["create", "success"])
        .inc();

    info!(
        "Created backup '{}' ({}) for user {}",
        backup.name, backup.id, request.user_id
    );

    Ok(Json(BackupResponse {
        success: true,
        message: format!("Backup '{}' created successfully", backup.name),
        backup: Some(backup),
        backups: None,
        restore_result: None,
    }))
}

/// Download a backup file
///
/// GET /api/hub/backups/:backup_id/download?user_id=<uuid>
pub async fn download_user_backup(
    State(state): State<VectorizerServer>,
    Path(backup_id): Path<Uuid>,
    Query(query): Query<BackupQuery>,
) -> Result<Response, ErrorResponse> {
    let backup_manager = state.backup_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "BACKUP_DISABLED".to_string(),
            "HiveHub backup functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Get backup info first for filename
    let backup_info = backup_manager
        .get_backup(&query.user_id, &backup_id)
        .await
        .map_err(|e| {
            ErrorResponse::new(
                "BACKUP_NOT_FOUND".to_string(),
                format!("Backup not found: {}", e),
                StatusCode::NOT_FOUND,
            )
        })?;

    // Download backup data
    let data = backup_manager
        .download_backup(&query.user_id, &backup_id)
        .await
        .map_err(|e| {
            ErrorResponse::new(
                "BACKUP_DOWNLOAD_ERROR".to_string(),
                format!("Failed to download backup: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    // Generate filename
    let filename = format!(
        "{}_{}.backup.gz",
        backup_info.name.replace(' ', "_"),
        backup_id
    );

    info!(
        "Downloaded backup {} for user {} ({} bytes)",
        backup_id,
        query.user_id,
        data.len()
    );

    // Return as file download
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/gzip"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        data,
    )
        .into_response())
}

/// Restore a backup
///
/// POST /api/hub/backups/restore
pub async fn restore_user_backup(
    State(state): State<VectorizerServer>,
    Json(request): Json<RestoreBackupRequest>,
) -> Result<Json<BackupResponse>, ErrorResponse> {
    let backup_manager = state.backup_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "BACKUP_DISABLED".to_string(),
            "HiveHub backup functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Check quota if HubManager is enabled (for restored collections)
    if let Some(ref hub_manager) = state.hub_manager {
        info!(
            "Restoring backup {} for user {} via HiveHub cluster mode",
            request.backup_id, request.user_id
        );
    }

    let result = backup_manager
        .restore_backup(&request.user_id, &request.backup_id, request.overwrite)
        .await
        .map_err(|e| {
            METRICS
                .hub_backup_operations_total
                .with_label_values(&["restore", "error"])
                .inc();
            ErrorResponse::new(
                "BACKUP_RESTORE_ERROR".to_string(),
                format!("Failed to restore backup: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let success = result.errors.is_empty();
    let status = if success { "success" } else { "partial" };
    METRICS
        .hub_backup_operations_total
        .with_label_values(&["restore", status])
        .inc();
    let message = if success {
        format!(
            "Restored {} collections with {} vectors",
            result.collections_restored.len(),
            result.vectors_restored
        )
    } else {
        format!(
            "Restored {} collections with {} errors",
            result.collections_restored.len(),
            result.errors.len()
        )
    };

    info!(
        "Restored backup {} for user {}: {} collections, {} vectors, {} errors",
        request.backup_id,
        request.user_id,
        result.collections_restored.len(),
        result.vectors_restored,
        result.errors.len()
    );

    Ok(Json(BackupResponse {
        success,
        message,
        backup: None,
        backups: None,
        restore_result: Some(result),
    }))
}

/// Delete a backup
///
/// DELETE /api/hub/backups/:backup_id?user_id=<uuid>
pub async fn delete_user_backup(
    State(state): State<VectorizerServer>,
    Path(backup_id): Path<Uuid>,
    Query(query): Query<BackupQuery>,
) -> Result<Json<BackupResponse>, ErrorResponse> {
    let backup_manager = state.backup_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "BACKUP_DISABLED".to_string(),
            "HiveHub backup functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    backup_manager
        .delete_backup(&query.user_id, &backup_id)
        .await
        .map_err(|e| {
            METRICS
                .hub_backup_operations_total
                .with_label_values(&["delete", "error"])
                .inc();
            ErrorResponse::new(
                "BACKUP_DELETE_ERROR".to_string(),
                format!("Failed to delete backup: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    METRICS
        .hub_backup_operations_total
        .with_label_values(&["delete", "success"])
        .inc();

    info!("Deleted backup {} for user {}", backup_id, query.user_id);

    Ok(Json(BackupResponse {
        success: true,
        message: format!("Backup {} deleted successfully", backup_id),
        backup: None,
        backups: None,
        restore_result: None,
    }))
}

/// Upload a backup file
///
/// POST /api/hub/backups/upload?user_id=<uuid>&name=<optional>
pub async fn upload_user_backup(
    State(state): State<VectorizerServer>,
    Query(query): Query<UploadBackupQuery>,
    body: Bytes,
) -> Result<Json<BackupResponse>, ErrorResponse> {
    let backup_manager = state.backup_manager.as_ref().ok_or_else(|| {
        ErrorResponse::new(
            "BACKUP_DISABLED".to_string(),
            "HiveHub backup functionality is not enabled".to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    if body.is_empty() {
        return Err(ErrorResponse::new(
            "INVALID_REQUEST".to_string(),
            "Backup data is empty".to_string(),
            StatusCode::BAD_REQUEST,
        ));
    }

    let backup = backup_manager
        .upload_backup(query.user_id, body.to_vec(), query.name)
        .await
        .map_err(|e| {
            METRICS
                .hub_backup_operations_total
                .with_label_values(&["upload", "error"])
                .inc();
            ErrorResponse::new(
                "BACKUP_UPLOAD_ERROR".to_string(),
                format!("Failed to upload backup: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    METRICS
        .hub_backup_operations_total
        .with_label_values(&["upload", "success"])
        .inc();

    info!(
        "Uploaded backup '{}' ({}) for user {}",
        backup.name, backup.id, query.user_id
    );

    Ok(Json(BackupResponse {
        success: true,
        message: format!("Backup '{}' uploaded successfully", backup.name),
        backup: Some(backup),
        backups: None,
        restore_result: None,
    }))
}
