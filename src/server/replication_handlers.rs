//! Replication REST API handlers

use axum::extract::{Path, State};
use axum::response::Json;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{error, info};

use super::VectorizerServer;
use super::error_middleware::{ErrorResponse, create_bad_request_error};

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
) -> Result<Json<ReplicationStatusResponse>, ErrorResponse> {
    // Check if replication nodes are configured
    let (role, enabled, stats, replicas) = if let Some(master) = &state.master_node {
        // Master node is configured
        let stats = master.get_stats();
        let replicas = master.get_replicas();
        ("Master", true, Some(stats), Some(replicas))
    } else if let Some(replica) = &state.replica_node {
        // Replica node is configured
        let stats = replica.get_stats();
        ("Replica", true, Some(stats), None)
    } else {
        // Fallback to metadata for backwards compatibility
        let role_str = state
            .store
            .get_metadata("replication_role")
            .unwrap_or_else(|| "standalone".to_string());

        let role = match role_str.as_str() {
            "master" | "Master" => "Master",
            "replica" | "Replica" => "Replica",
            _ => "Standalone",
        };

        let enabled = role != "Standalone";
        (role, enabled, None, None)
    };

    let response = ReplicationStatusResponse {
        role: role.to_string(),
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
) -> Result<Json<Value>, ErrorResponse> {
    info!("Configuring replication: {:?}", request);

    // Parse role
    let role = match request.role.as_str() {
        "master" => crate::replication::NodeRole::Master,
        "replica" => crate::replication::NodeRole::Replica,
        "standalone" => crate::replication::NodeRole::Standalone,
        _ => {
            return Err(create_bad_request_error(&format!(
                "Invalid role: {}. Must be 'master', 'replica', or 'standalone'",
                request.role
            )));
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
) -> Result<Json<Value>, ErrorResponse> {
    // Get stats from actual replication nodes
    let stats = if let Some(master) = &state.master_node {
        master.get_stats()
    } else if let Some(replica) = &state.replica_node {
        replica.get_stats()
    } else {
        return Err(create_bad_request_error("Replication not enabled"));
    };

    Ok(Json(json!(stats)))
}

/// List connected replicas (master only)
pub async fn list_replicas(
    State(state): State<VectorizerServer>,
) -> Result<Json<Value>, ErrorResponse> {
    // Get replica list from actual master node
    let replicas = if let Some(master) = &state.master_node {
        master.get_replicas()
    } else {
        return Err(create_bad_request_error(
            "This endpoint is only available on master nodes",
        ));
    };

    Ok(Json(json!({
        "replicas": replicas,
        "count": replicas.len()
    })))
}
