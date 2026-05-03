//! Replication REST API handlers

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

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
    pub stats: Option<vectorizer::replication::ReplicationStats>,
    pub replicas: Option<Vec<vectorizer::replication::ReplicaInfo>>,
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
        "master" => vectorizer::replication::NodeRole::Master,
        "replica" => vectorizer::replication::NodeRole::Replica,
        "standalone" => vectorizer::replication::NodeRole::Standalone,
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

// ---------------------------------------------------------------------------
// Phase-15 cluster admin handlers
// ---------------------------------------------------------------------------

/// Request body for POST /cluster/failover
#[derive(Debug, Deserialize)]
pub struct ClusterFailoverRequest {
    /// ID of the replica to promote to primary.
    pub replica_id: String,
    /// Maximum tolerated WAL-segment lag before rejecting the failover.
    /// Defaults to `DEFAULT_MAX_FAILOVER_LAG_SEGMENTS` (1).
    pub max_lag_segments: Option<u64>,
}

/// POST /cluster/failover — promote a replica to primary.
///
/// Performs a pre-flight WAL-lag check and returns 409 if the replica is
/// too far behind.  See `vectorizer::replication::state` for the residual
/// loss-window caveat.
pub async fn cluster_failover(
    State(state): State<VectorizerServer>,
    Json(request): Json<ClusterFailoverRequest>,
) -> Result<Json<Value>, ErrorResponse> {
    let master = state.master_node.as_ref().ok_or_else(|| {
        create_bad_request_error("Failover requires this node to be running as master")
    })?;

    let max_lag = request
        .max_lag_segments
        .unwrap_or(vectorizer::replication::DEFAULT_MAX_FAILOVER_LAG_SEGMENTS);

    match vectorizer::replication::state::failover_to(master, &request.replica_id, max_lag) {
        Ok(report) => {
            info!(
                "Failover completed: replica '{}' promoted (lag={})",
                report.promoted_replica_id, report.residual_lag_operations
            );
            Ok(Json(json!(report)))
        }
        Err(vectorizer::replication::ReplicationError::LagTooHigh {
            lag_operations,
            max_allowed,
            ..
        }) => Err(super::error_middleware::ErrorResponse::new(
            "lag_too_high".to_string(),
            format!(
                "Replica lag {} exceeds max allowed {}. Drain writes or increase max_lag_segments.",
                lag_operations, max_allowed
            ),
            axum::http::StatusCode::CONFLICT,
        )),
        Err(e) => Err(create_bad_request_error(&e.to_string())),
    }
}

/// POST /cluster/replicas/{id}/resync — force a full resync on a replica.
pub async fn cluster_resync_replica(
    State(state): State<VectorizerServer>,
    Path(replica_id): Path<String>,
) -> Result<Json<Value>, ErrorResponse> {
    let master = state.master_node.as_ref().ok_or_else(|| {
        create_bad_request_error("Resync requires this node to be running as master")
    })?;

    let report = vectorizer::replication::state::force_resync(master, &replica_id)
        .map_err(|e| create_bad_request_error(&e.to_string()))?;

    info!(
        "Force-resync initiated for replica '{}' at offset={}",
        report.replica_id, report.snapshot_offset
    );

    Ok(Json(json!(report)))
}

/// Request body for POST /cluster/peers
#[derive(Debug, Deserialize)]
pub struct AddPeerRequest {
    /// Address of the new peer (host:port).
    pub address: String,
    /// Role: "member" (default) or "observer".
    #[serde(default)]
    pub role: String,
}

/// POST /cluster/peers — add a peer to the cluster.
pub async fn cluster_add_peer(
    State(state): State<VectorizerServer>,
    Json(request): Json<AddPeerRequest>,
) -> Result<Json<Value>, ErrorResponse> {
    let cluster_mgr = state
        .cluster_manager
        .as_ref()
        .ok_or_else(|| create_bad_request_error("Cluster mode is not enabled on this node"))?;

    let role = if request.role.to_lowercase() == "observer" {
        vectorizer::cluster::rebalance::PeerRole::Observer
    } else {
        vectorizer::cluster::rebalance::PeerRole::Member
    };

    let info = vectorizer::cluster::rebalance::add_peer(cluster_mgr, request.address, role)
        .map_err(|e| create_bad_request_error(&e.to_string()))?;

    info!("Peer '{}' added to cluster", info.node_id);
    Ok(Json(json!(info)))
}

/// POST /cluster/rebalance — trigger a shard rebalance.
pub async fn cluster_rebalance(
    State(state): State<VectorizerServer>,
) -> Result<Json<Value>, ErrorResponse> {
    let cluster_mgr = state
        .cluster_manager
        .as_ref()
        .ok_or_else(|| create_bad_request_error("Cluster mode is not enabled on this node"))?;

    let job = vectorizer::cluster::rebalance::rebalance(cluster_mgr)
        .map_err(|e| create_bad_request_error(&e.to_string()))?;

    info!("Rebalance job {} started", job.job_id);
    Ok(Json(json!(job)))
}

/// GET /cluster/rebalance/status — return progress of the active rebalance.
pub async fn cluster_rebalance_status(
    State(_state): State<VectorizerServer>,
) -> Result<Json<Value>, ErrorResponse> {
    match vectorizer::cluster::rebalance::rebalance_status() {
        Some(job) => Ok(Json(json!(job))),
        None => Ok(Json(json!({
            "status": "idle",
            "message": "No rebalance has been triggered on this node"
        }))),
    }
}
