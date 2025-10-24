//! Replication REST API handlers

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{error, info};

use super::VectorizerServer;

/// Request to configure replication
#[derive(Debug, Deserialize)]
pub struct ConfigureReplicationRequest {
    pub role: String,                   // "master", "replica", or "standalone"
    pub bind_address: Option<String>,   // For master
    pub master_address: Option<String>, // For replica
    pub heartbeat_interval: Option<u64>,
    pub log_size: Option<usize>,
}

/// Replication status response
#[derive(Debug, Serialize)]
pub struct ReplicationStatusResponse {
    pub role: String,
    pub enabled: bool,
    pub stats: Option<crate::replication::ReplicationStats>,
    pub replicas: Option<Vec<crate::replication::ReplicaInfo>>,
}

/// Get replication status
pub async fn get_replication_status(
    State(state): State<VectorizerServer>,
) -> Result<Json<ReplicationStatusResponse>, (StatusCode, String)> {
    // Check if replication is configured (stored in VectorStore metadata)
    let role_str = state
        .store
        .get_metadata("replication_role")
        .unwrap_or_else(|| "standalone".to_string());

    let role = match role_str.as_str() {
        "master" => crate::replication::NodeRole::Master,
        "replica" => crate::replication::NodeRole::Replica,
        _ => crate::replication::NodeRole::Standalone,
    };

    let enabled = role != crate::replication::NodeRole::Standalone;

    // Retrieve stats and replicas from metadata if available
    // Note: This is a temporary implementation until server stores actual node instances
    let stats = if enabled {
        // Create basic stats from stored metadata
        Some(crate::replication::ReplicationStats {
            role,
            lag_ms: state
                .store
                .get_metadata("replication_lag_ms")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            bytes_sent: state
                .store
                .get_metadata("replication_bytes_sent")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            bytes_received: state
                .store
                .get_metadata("replication_bytes_received")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            last_sync: std::time::SystemTime::now(),
            operations_pending: 0,
            snapshot_size: 0,
            connected_replicas: if role == crate::replication::NodeRole::Master {
                Some(0) // Will be updated when master node is integrated
            } else {
                None
            },
            master_offset: 0,
            replica_offset: 0,
            lag_operations: 0,
            total_replicated: 0,
        })
    } else {
        None
    };

    let replicas = if role == crate::replication::NodeRole::Master {
        Some(Vec::new()) // Empty list until master node is integrated
    } else {
        None
    };

    let response = ReplicationStatusResponse {
        role: format!("{:?}", role),
        enabled,
        stats,
        replicas,
    };

    Ok(Json(response))
}

/// Configure replication
pub async fn configure_replication(
    State(state): State<VectorizerServer>,
    Json(request): Json<ConfigureReplicationRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    info!("Configuring replication: {:?}", request);

    // Parse role
    let role = match request.role.as_str() {
        "master" => crate::replication::NodeRole::Master,
        "replica" => crate::replication::NodeRole::Replica,
        "standalone" => crate::replication::NodeRole::Standalone,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid role: {}", request.role),
            ));
        }
    };

    // Store configuration in VectorStore metadata
    state
        .store
        .set_metadata("replication_role", format!("{:?}", role));

    if let Some(bind_addr) = &request.bind_address {
        state
            .store
            .set_metadata("replication_bind_address", bind_addr.clone());
    }

    if let Some(master_addr) = &request.master_address {
        state
            .store
            .set_metadata("replication_master_address", master_addr.clone());
    }

    if let Some(interval) = request.heartbeat_interval {
        state
            .store
            .set_metadata("replication_heartbeat_interval", interval.to_string());
    }

    if let Some(log_size) = request.log_size {
        state
            .store
            .set_metadata("replication_log_size", log_size.to_string());
    }

    info!("Replication configured successfully. Role: {:?}", role);
    info!("⚠️  Server restart required for replication to take effect");

    Ok(Json(json!({
        "success": true,
        "role": format!("{:?}", role),
        "message": "Replication configured successfully. Server restart required."
    })))
}

/// Get replication statistics
pub async fn get_replication_stats(
    State(state): State<VectorizerServer>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Check if replication is enabled
    let role_str = state
        .store
        .get_metadata("replication_role")
        .unwrap_or_else(|| "standalone".to_string());

    if role_str == "standalone" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Replication not enabled".to_string(),
        ));
    }

    let role = match role_str.as_str() {
        "Master" => crate::replication::NodeRole::Master,
        "Replica" => crate::replication::NodeRole::Replica,
        _ => crate::replication::NodeRole::Standalone,
    };

    // Create stats from metadata (temporary until actual node instances are integrated)
    let stats = crate::replication::ReplicationStats {
        role,
        lag_ms: state
            .store
            .get_metadata("replication_lag_ms")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        bytes_sent: state
            .store
            .get_metadata("replication_bytes_sent")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        bytes_received: state
            .store
            .get_metadata("replication_bytes_received")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        last_sync: std::time::SystemTime::now(),
        operations_pending: 0,
        snapshot_size: 0,
        connected_replicas: if role == crate::replication::NodeRole::Master {
            Some(0)
        } else {
            None
        },
        master_offset: 0,
        replica_offset: 0,
        lag_operations: 0,
        total_replicated: 0,
    };

    Ok(Json(json!(stats)))
}

/// List connected replicas (master only)
pub async fn list_replicas(
    State(state): State<VectorizerServer>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let role_str = state
        .store
        .get_metadata("replication_role")
        .unwrap_or_else(|| "standalone".to_string());

    if role_str != "master" && role_str != "Master" {
        return Err((
            StatusCode::BAD_REQUEST,
            "This endpoint is only available on master nodes".to_string(),
        ));
    }

    // Return empty replica list (temporary until actual master node is integrated)
    // In production, this would query the actual MasterNode instance
    let replicas: Vec<crate::replication::ReplicaInfo> = Vec::new();

    Ok(Json(json!({
        "replicas": replicas,
        "count": replicas.len(),
        "message": "Replica list will populate when replication is actively running"
    })))
}
