//! REST API endpoints for cluster management

use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::cluster::{ClusterManager, NodeId};
use crate::db::VectorStore;
use crate::error::VectorizerError;

/// Cluster API state
#[derive(Clone)]
pub struct ClusterApiState {
    /// Cluster manager
    pub cluster_manager: Arc<ClusterManager>,
    /// Vector store
    pub store: Arc<VectorStore>,
}

/// Response for cluster node information
#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterNodeResponse {
    pub id: String,
    pub address: String,
    pub grpc_port: u16,
    pub status: String,
    pub shard_count: usize,
    pub metadata: NodeMetadataResponse,
}

/// Node metadata response
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeMetadataResponse {
    pub version: Option<String>,
    pub capabilities: Vec<String>,
    pub vector_count: usize,
    pub memory_usage: u64,
    pub cpu_usage: f32,
}

/// Response for list of cluster nodes
#[derive(Debug, Serialize, Deserialize)]
pub struct ListNodesResponse {
    pub nodes: Vec<ClusterNodeResponse>,
}

/// Response for shard distribution
#[derive(Debug, Serialize, Deserialize)]
pub struct ShardDistributionResponse {
    pub shard_to_node: Vec<ShardNodeMapping>,
    pub node_shard_counts: Vec<NodeShardCount>,
}

/// Shard to node mapping
#[derive(Debug, Serialize, Deserialize)]
pub struct ShardNodeMapping {
    pub shard_id: u32,
    pub node_id: String,
}

/// Node shard count
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeShardCount {
    pub node_id: String,
    pub shard_count: usize,
}

/// Request to add a node
#[derive(Debug, Serialize, Deserialize)]
pub struct AddNodeRequest {
    pub id: String,
    pub address: String,
    pub grpc_port: u16,
}

/// Request to trigger rebalancing
#[derive(Debug, Serialize, Deserialize)]
pub struct RebalanceRequest {
    pub force: Option<bool>,
}

/// Response for rebalancing
#[derive(Debug, Serialize, Deserialize)]
pub struct RebalanceResponse {
    pub success: bool,
    pub message: String,
    pub shards_moved: Option<usize>,
}

/// Create cluster API router
pub fn create_cluster_router() -> Router<ClusterApiState> {
    Router::new()
        .route("/api/v1/cluster/nodes", get(list_nodes))
        .route("/api/v1/cluster/nodes/:node_id", get(get_node))
        .route("/api/v1/cluster/nodes", post(add_node))
        .route("/api/v1/cluster/nodes/:node_id", delete(remove_node))
        .route(
            "/api/v1/cluster/shard-distribution",
            get(get_shard_distribution),
        )
        .route("/api/v1/cluster/rebalance", post(trigger_rebalance))
}

/// List all cluster nodes
async fn list_nodes(
    State(state): State<ClusterApiState>,
) -> Result<Json<ListNodesResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!("REST: List cluster nodes");

    let nodes = state.cluster_manager.get_nodes();
    let node_responses: Vec<ClusterNodeResponse> = nodes
        .into_iter()
        .map(|node| ClusterNodeResponse {
            id: node.id.as_str().to_string(),
            address: node.address.clone(),
            grpc_port: node.grpc_port,
            status: format!("{:?}", node.status).to_lowercase(),
            shard_count: node.shard_count(),
            metadata: NodeMetadataResponse {
                version: node.metadata.version.clone(),
                capabilities: node.metadata.capabilities.clone(),
                vector_count: node.metadata.vector_count,
                memory_usage: node.metadata.memory_usage,
                cpu_usage: node.metadata.cpu_usage,
            },
        })
        .collect();

    Ok(Json(ListNodesResponse {
        nodes: node_responses,
    }))
}

/// Get node information
async fn get_node(
    State(state): State<ClusterApiState>,
    Path(node_id): Path<String>,
) -> Result<Json<ClusterNodeResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!("REST: Get cluster node {}", node_id);

    let node_id_obj = NodeId::new(node_id.clone());
    let node = state
        .cluster_manager
        .get_node(&node_id_obj)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Node {} not found", node_id)
                })),
            )
        })?;

    Ok(Json(ClusterNodeResponse {
        id: node.id.as_str().to_string(),
        address: node.address.clone(),
        grpc_port: node.grpc_port,
        status: format!("{:?}", node.status).to_lowercase(),
        shard_count: node.shard_count(),
        metadata: NodeMetadataResponse {
            version: node.metadata.version.clone(),
            capabilities: node.metadata.capabilities.clone(),
            vector_count: node.metadata.vector_count,
            memory_usage: node.metadata.memory_usage,
            cpu_usage: node.metadata.cpu_usage,
        },
    }))
}

/// Add a node to the cluster
async fn add_node(
    State(state): State<ClusterApiState>,
    Json(request): Json<AddNodeRequest>,
) -> Result<Json<ClusterNodeResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("REST: Add cluster node {}", request.id);

    let node_id = NodeId::new(request.id.clone());
    let mut node = crate::cluster::ClusterNode::new(
        node_id.clone(),
        request.address.clone(),
        request.grpc_port,
    );
    node.mark_active();

    state.cluster_manager.add_node(node.clone());

    Ok(Json(ClusterNodeResponse {
        id: node.id.as_str().to_string(),
        address: node.address.clone(),
        grpc_port: node.grpc_port,
        status: format!("{:?}", node.status).to_lowercase(),
        shard_count: node.shard_count(),
        metadata: NodeMetadataResponse {
            version: node.metadata.version.clone(),
            capabilities: node.metadata.capabilities.clone(),
            vector_count: node.metadata.vector_count,
            memory_usage: node.metadata.memory_usage,
            cpu_usage: node.metadata.cpu_usage,
        },
    }))
}

/// Remove a node from the cluster
async fn remove_node(
    State(state): State<ClusterApiState>,
    Path(node_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("REST: Remove cluster node {}", node_id);

    let node_id_obj = NodeId::new(node_id.clone());
    let removed = state.cluster_manager.remove_node(&node_id_obj);

    if removed.is_some() {
        Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Node {} removed from cluster", node_id)
        })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Node {} not found", node_id)
            })),
        ))
    }
}

/// Get shard distribution across nodes
async fn get_shard_distribution(
    State(state): State<ClusterApiState>,
) -> Result<Json<ShardDistributionResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!("REST: Get shard distribution");

    let shard_router = state.cluster_manager.shard_router();
    let nodes = state.cluster_manager.get_nodes();

    let mut shard_to_node = Vec::new();
    let mut node_shard_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for node in &nodes {
        let shards = shard_router.get_shards_for_node(&node.id);
        let count = shards.len();
        node_shard_counts.insert(node.id.as_str().to_string(), count);

        for shard_id in shards {
            shard_to_node.push(ShardNodeMapping {
                shard_id: shard_id.as_u32(),
                node_id: node.id.as_str().to_string(),
            });
        }
    }

    let node_shard_counts_vec: Vec<NodeShardCount> = node_shard_counts
        .into_iter()
        .map(|(node_id, shard_count)| NodeShardCount {
            node_id,
            shard_count,
        })
        .collect();

    Ok(Json(ShardDistributionResponse {
        shard_to_node,
        node_shard_counts: node_shard_counts_vec,
    }))
}

/// Trigger shard rebalancing
async fn trigger_rebalance(
    State(state): State<ClusterApiState>,
    Json(request): Json<RebalanceRequest>,
) -> Result<Json<RebalanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!(
        "REST: Trigger shard rebalancing (force: {:?})",
        request.force
    );

    // Get current cluster state
    let nodes = state.cluster_manager.get_nodes();
    let active_nodes: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n.status, crate::cluster::NodeStatus::Active))
        .collect();

    if active_nodes.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "No active nodes available for rebalancing"
            })),
        ));
    }

    // Get all shards from the shard router
    let shard_router = state.cluster_manager.shard_router();
    let current_distribution = get_current_shard_distribution(&shard_router, &nodes);

    // Calculate target distribution (balanced)
    let total_shards: usize = current_distribution.values().map(|v| v.len()).sum();
    let num_active_nodes = active_nodes.len();

    if total_shards == 0 {
        return Ok(Json(RebalanceResponse {
            success: true,
            message: "No shards to rebalance".to_string(),
            shards_moved: Some(0),
        }));
    }

    // Calculate ideal shards per node
    let shards_per_node = total_shards / num_active_nodes;
    let extra_shards = total_shards % num_active_nodes;

    // Check if rebalancing is needed
    let mut imbalanced = false;
    for (i, node) in active_nodes.iter().enumerate() {
        let current_count = current_distribution
            .get(&node.id.as_str().to_string())
            .map(|v| v.len())
            .unwrap_or(0);
        let target_count = shards_per_node + if i < extra_shards { 1 } else { 0 };

        // Allow 1 shard difference without triggering rebalance (unless forced)
        if (current_count as isize - target_count as isize).abs() > 1
            || request.force.unwrap_or(false)
        {
            imbalanced = true;
            break;
        }
    }

    if !imbalanced {
        return Ok(Json(RebalanceResponse {
            success: true,
            message: "Cluster is already balanced".to_string(),
            shards_moved: Some(0),
        }));
    }

    // Perform rebalancing
    let mut shards_moved = 0;

    // Collect all shards
    let all_shards: Vec<crate::db::sharding::ShardId> = current_distribution
        .values()
        .flat_map(|shards| shards.iter().cloned())
        .collect();

    // Get active node IDs
    let active_node_ids: Vec<NodeId> = active_nodes.iter().map(|n| n.id.clone()).collect();

    // Use the router's rebalance function
    shard_router.rebalance(&all_shards, &active_node_ids);

    // Count how many shards moved
    let new_distribution = get_current_shard_distribution(&shard_router, &nodes);
    for (node_id, old_shards) in &current_distribution {
        if let Some(new_shards) = new_distribution.get(node_id) {
            let old_set: std::collections::HashSet<_> = old_shards.iter().collect();
            let new_set: std::collections::HashSet<_> = new_shards.iter().collect();

            // Count shards that are no longer on this node
            shards_moved += old_set.difference(&new_set).count();
        }
    }

    info!("Rebalancing complete: {} shards moved", shards_moved);

    Ok(Json(RebalanceResponse {
        success: true,
        message: format!(
            "Rebalancing completed successfully. {} shards redistributed across {} nodes.",
            shards_moved, num_active_nodes
        ),
        shards_moved: Some(shards_moved),
    }))
}

/// Get current shard distribution across nodes
fn get_current_shard_distribution(
    shard_router: &std::sync::Arc<crate::cluster::DistributedShardRouter>,
    nodes: &[crate::cluster::ClusterNode],
) -> std::collections::HashMap<String, Vec<crate::db::sharding::ShardId>> {
    let mut distribution = std::collections::HashMap::new();

    for node in nodes {
        let shards = shard_router.get_shards_for_node(&node.id);
        distribution.insert(node.id.as_str().to_string(), shards);
    }

    distribution
}
