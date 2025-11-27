//! Qdrant Snapshot API handlers
//!
//! This module provides handlers for the Qdrant Snapshots API endpoints.

use std::time::Instant;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use tracing::{error, info};

use super::VectorizerServer;
use super::error_middleware::{ErrorResponse, create_error_response, create_not_found_error};
use crate::models::qdrant::snapshot::{
    QdrantCreateSnapshotResponse, QdrantDeleteSnapshotResponse, QdrantListSnapshotsResponse,
    QdrantSnapshotDescription, QdrantUploadSnapshotResponse,
};

/// List snapshots for a specific collection
/// GET /qdrant/collections/{name}/snapshots
pub async fn list_collection_snapshots(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
) -> Result<Json<QdrantListSnapshotsResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        collection = %collection_name,
        "Qdrant Snapshots API: Listing collection snapshots"
    );

    // Verify collection exists
    state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    // Get snapshot manager from server state
    let snapshot_manager = state.snapshot_manager.as_ref().ok_or_else(|| {
        create_error_response(
            "Snapshot manager not initialized",
            "Snapshots not available",
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // List snapshots
    let snapshots = snapshot_manager.list_snapshots().map_err(|e| {
        create_error_response(
            &format!("Failed to list snapshots: {}", e),
            "Snapshot list failed",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // Convert to Qdrant format
    let result: Vec<QdrantSnapshotDescription> = snapshots
        .into_iter()
        .map(|s| QdrantSnapshotDescription {
            name: s.id,
            creation_time: Some(s.created_at.to_rfc3339()),
            size: s.size_bytes,
            checksum: None,
        })
        .collect();

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        collection = %collection_name,
        count = result.len(),
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Snapshots API: Listed collection snapshots"
    );

    Ok(Json(QdrantListSnapshotsResponse {
        result,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Create a snapshot for a specific collection
/// POST /qdrant/collections/{name}/snapshots
pub async fn create_collection_snapshot(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
) -> Result<Json<QdrantCreateSnapshotResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        collection = %collection_name,
        "Qdrant Snapshots API: Creating collection snapshot"
    );

    // Verify collection exists
    state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    // Get snapshot manager from server state
    let snapshot_manager = state.snapshot_manager.as_ref().ok_or_else(|| {
        create_error_response(
            "Snapshot manager not initialized",
            "Snapshots not available",
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Create snapshot
    let snapshot = snapshot_manager.create_snapshot().map_err(|e| {
        error!("Failed to create snapshot: {}", e);
        create_error_response(
            &format!("Failed to create snapshot: {}", e),
            "Snapshot creation failed",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        collection = %collection_name,
        snapshot_id = %snapshot.id,
        size_bytes = snapshot.size_bytes,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Snapshots API: Created collection snapshot"
    );

    Ok(Json(QdrantCreateSnapshotResponse {
        result: QdrantSnapshotDescription {
            name: snapshot.id,
            creation_time: Some(snapshot.created_at.to_rfc3339()),
            size: snapshot.size_bytes,
            checksum: None,
        },
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Delete a snapshot for a specific collection
/// DELETE /qdrant/collections/{name}/snapshots/{snapshot_name}
pub async fn delete_collection_snapshot(
    State(state): State<VectorizerServer>,
    Path((collection_name, snapshot_name)): Path<(String, String)>,
) -> Result<Json<QdrantDeleteSnapshotResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        collection = %collection_name,
        snapshot = %snapshot_name,
        "Qdrant Snapshots API: Deleting collection snapshot"
    );

    // Verify collection exists
    state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    // Get snapshot manager from server state
    let snapshot_manager = state.snapshot_manager.as_ref().ok_or_else(|| {
        create_error_response(
            "Snapshot manager not initialized",
            "Snapshots not available",
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Delete snapshot
    let deleted = snapshot_manager
        .delete_snapshot(&snapshot_name)
        .map_err(|e| {
            error!("Failed to delete snapshot: {}", e);
            create_error_response(
                &format!("Failed to delete snapshot: {}", e),
                "Snapshot deletion failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    if !deleted {
        return Err(create_not_found_error("snapshot", &snapshot_name));
    }

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        collection = %collection_name,
        snapshot = %snapshot_name,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Snapshots API: Deleted collection snapshot"
    );

    Ok(Json(QdrantDeleteSnapshotResponse {
        result: true,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// List all snapshots (global)
/// GET /qdrant/snapshots
pub async fn list_all_snapshots(
    State(state): State<VectorizerServer>,
) -> Result<Json<QdrantListSnapshotsResponse>, ErrorResponse> {
    let start = Instant::now();
    info!("Qdrant Snapshots API: Listing all snapshots");

    // Get snapshot manager from server state
    let snapshot_manager = state.snapshot_manager.as_ref().ok_or_else(|| {
        create_error_response(
            "Snapshot manager not initialized",
            "Snapshots not available",
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // List snapshots
    let snapshots = snapshot_manager.list_snapshots().map_err(|e| {
        create_error_response(
            &format!("Failed to list snapshots: {}", e),
            "Snapshot list failed",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // Convert to Qdrant format
    let result: Vec<QdrantSnapshotDescription> = snapshots
        .into_iter()
        .map(|s| QdrantSnapshotDescription {
            name: s.id,
            creation_time: Some(s.created_at.to_rfc3339()),
            size: s.size_bytes,
            checksum: None,
        })
        .collect();

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        count = result.len(),
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Snapshots API: Listed all snapshots"
    );

    Ok(Json(QdrantListSnapshotsResponse {
        result,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Create a full snapshot (global)
/// POST /qdrant/snapshots
pub async fn create_full_snapshot(
    State(state): State<VectorizerServer>,
) -> Result<Json<QdrantCreateSnapshotResponse>, ErrorResponse> {
    let start = Instant::now();
    info!("Qdrant Snapshots API: Creating full snapshot");

    // Get snapshot manager from server state
    let snapshot_manager = state.snapshot_manager.as_ref().ok_or_else(|| {
        create_error_response(
            "Snapshot manager not initialized",
            "Snapshots not available",
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Create snapshot
    let snapshot = snapshot_manager.create_snapshot().map_err(|e| {
        error!("Failed to create snapshot: {}", e);
        create_error_response(
            &format!("Failed to create snapshot: {}", e),
            "Snapshot creation failed",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        snapshot_id = %snapshot.id,
        size_bytes = snapshot.size_bytes,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Snapshots API: Created full snapshot"
    );

    Ok(Json(QdrantCreateSnapshotResponse {
        result: QdrantSnapshotDescription {
            name: snapshot.id,
            creation_time: Some(snapshot.created_at.to_rfc3339()),
            size: snapshot.size_bytes,
            checksum: None,
        },
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Recover a collection from a snapshot
/// POST /qdrant/collections/{name}/snapshots/recover
pub async fn recover_collection_snapshot(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<crate::models::qdrant::snapshot::QdrantRecoverSnapshotRequest>,
) -> Result<Json<crate::models::qdrant::snapshot::QdrantRecoverSnapshotResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        collection = %collection_name,
        location = %request.location,
        "Qdrant Snapshots API: Recovering collection from snapshot"
    );

    // Get snapshot manager from server state
    let snapshot_manager = state.snapshot_manager.as_ref().ok_or_else(|| {
        create_error_response(
            "Snapshot manager not initialized",
            "Snapshots not available",
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // The location could be a snapshot ID or a path
    // For now, we treat it as a snapshot ID
    snapshot_manager
        .restore_snapshot(&request.location)
        .map_err(|e| {
            error!("Failed to recover from snapshot: {}", e);
            create_error_response(
                &format!("Failed to recover from snapshot: {}", e),
                "Snapshot recovery failed",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        collection = %collection_name,
        location = %request.location,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Snapshots API: Recovered collection from snapshot"
    );

    Ok(Json(
        crate::models::qdrant::snapshot::QdrantRecoverSnapshotResponse {
            result: true,
            status: "ok".to_string(),
            time: elapsed,
        },
    ))
}

/// Upload a snapshot for a specific collection
/// POST /qdrant/collections/{name}/snapshots/upload
pub async fn upload_collection_snapshot(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    body: Bytes,
) -> Result<Json<QdrantUploadSnapshotResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        collection = %collection_name,
        size_bytes = body.len(),
        "Qdrant Snapshots API: Uploading collection snapshot"
    );

    // Verify collection exists
    state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    // Get snapshot manager from server state
    let snapshot_manager = state.snapshot_manager.as_ref().ok_or_else(|| {
        create_error_response(
            "Snapshot manager not initialized",
            "Snapshots not available",
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;

    // Validate data size
    if body.is_empty() {
        return Err(create_error_response(
            "Empty snapshot data",
            "No data provided",
            StatusCode::BAD_REQUEST,
        ));
    }

    // Import the snapshot
    let snapshot = snapshot_manager.import_snapshot(&body).map_err(|e| {
        error!("Failed to import snapshot: {}", e);
        create_error_response(
            &format!("Failed to import snapshot: {}", e),
            "Snapshot import failed",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        collection = %collection_name,
        snapshot_id = %snapshot.id,
        size_bytes = snapshot.size_bytes,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Snapshots API: Uploaded collection snapshot"
    );

    Ok(Json(QdrantUploadSnapshotResponse {
        result: QdrantSnapshotDescription {
            name: snapshot.id,
            creation_time: Some(snapshot.created_at.to_rfc3339()),
            size: snapshot.size_bytes,
            checksum: None,
        },
        status: "ok".to_string(),
        time: elapsed,
    }))
}
