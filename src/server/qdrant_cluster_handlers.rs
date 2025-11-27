//! Qdrant Cluster API handlers
//!
//! This module provides handlers for the Qdrant Cluster API endpoints.
//! Vectorizer operates as a single-node server, so cluster operations are simulated
//! for API compatibility.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use axum::extract::{Path, State};
use axum::response::Json;
use tracing::{info, warn};

use super::VectorizerServer;
use super::error_middleware::ErrorResponse;
use crate::models::qdrant::cluster::{
    QdrantClusterRecoverResponse, QdrantClusterStatus, QdrantClusterStatusResponse,
    QdrantGetMetadataKeyResponse, QdrantListMetadataKeysResponse, QdrantPeerInfo, QdrantPeerState,
    QdrantRaftInfo, QdrantRemovePeerResponse, QdrantUpdateMetadataKeyRequest,
    QdrantUpdateMetadataKeyResponse,
};

// Static peer ID for this single-node instance
static PEER_ID: AtomicU64 = AtomicU64::new(1);

/// Get cluster status
/// GET /qdrant/cluster
pub async fn get_cluster_status(
    State(_state): State<VectorizerServer>,
) -> Result<Json<QdrantClusterStatusResponse>, ErrorResponse> {
    let start = Instant::now();
    info!("Qdrant Cluster API: Getting cluster status");

    let peer_id = PEER_ID.load(Ordering::SeqCst);

    // Build peer info for this single node
    let mut peers = HashMap::new();
    peers.insert(
        peer_id.to_string(),
        QdrantPeerInfo {
            uri: "http://localhost:7777".to_string(),
            state: Some(QdrantPeerState::Active),
        },
    );

    // Build Raft info (simulated for single-node)
    let raft_info = QdrantRaftInfo {
        term: 1,
        commit: 0,
        pending_operations: 0,
        leader: Some(peer_id),
        role: Some("Leader".to_string()),
        is_voter: true,
    };

    let cluster_status = QdrantClusterStatus {
        status: "enabled".to_string(),
        peer_id,
        peers,
        raft_info: Some(raft_info),
        consensus_thread_status: None,
        message_send_failures: None,
    };

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        peer_id = peer_id,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Cluster API: Retrieved cluster status"
    );

    Ok(Json(QdrantClusterStatusResponse {
        result: cluster_status,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Recover current peer
/// POST /qdrant/cluster/recover
pub async fn cluster_recover(
    State(_state): State<VectorizerServer>,
) -> Result<Json<QdrantClusterRecoverResponse>, ErrorResponse> {
    let start = Instant::now();
    info!("Qdrant Cluster API: Recovering current peer");

    // For a single-node server, recovery is a no-op but we simulate success
    warn!("Cluster recovery requested on single-node Vectorizer instance (no-op)");

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Cluster API: Recovery completed (simulated)"
    );

    Ok(Json(QdrantClusterRecoverResponse {
        result: true,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Remove peer from cluster
/// DELETE /qdrant/cluster/peer/{peer_id}
pub async fn remove_peer(
    State(_state): State<VectorizerServer>,
    Path(peer_id): Path<u64>,
) -> Result<Json<QdrantRemovePeerResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(peer_id = peer_id, "Qdrant Cluster API: Removing peer");

    let current_peer_id = PEER_ID.load(Ordering::SeqCst);

    if peer_id == current_peer_id {
        warn!(
            peer_id = peer_id,
            "Cannot remove self from cluster on single-node instance"
        );
    } else {
        warn!(
            peer_id = peer_id,
            "Peer not found in single-node cluster (no-op)"
        );
    }

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        peer_id = peer_id,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Cluster API: Remove peer completed (simulated)"
    );

    Ok(Json(QdrantRemovePeerResponse {
        result: true,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// List metadata keys
/// GET /qdrant/cluster/metadata/keys
pub async fn list_metadata_keys(
    State(_state): State<VectorizerServer>,
) -> Result<Json<QdrantListMetadataKeysResponse>, ErrorResponse> {
    let start = Instant::now();
    info!("Qdrant Cluster API: Listing metadata keys");

    // Vectorizer doesn't maintain cluster metadata, return empty list
    let keys: Vec<String> = vec![];

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        key_count = keys.len(),
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Cluster API: Listed metadata keys"
    );

    Ok(Json(QdrantListMetadataKeysResponse {
        result: keys,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Get metadata key value
/// GET /qdrant/cluster/metadata/keys/{key}
pub async fn get_metadata_key(
    State(_state): State<VectorizerServer>,
    Path(key): Path<String>,
) -> Result<Json<QdrantGetMetadataKeyResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(key = %key, "Qdrant Cluster API: Getting metadata key");

    // Vectorizer doesn't maintain cluster metadata, return null
    let elapsed = start.elapsed().as_secs_f64();
    info!(
        key = %key,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Cluster API: Retrieved metadata key"
    );

    Ok(Json(QdrantGetMetadataKeyResponse {
        result: serde_json::Value::Null,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Update metadata key value
/// PUT /qdrant/cluster/metadata/keys/{key}
pub async fn update_metadata_key(
    State(_state): State<VectorizerServer>,
    Path(key): Path<String>,
    Json(request): Json<QdrantUpdateMetadataKeyRequest>,
) -> Result<Json<QdrantUpdateMetadataKeyResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        key = %key,
        value = ?request.value,
        "Qdrant Cluster API: Updating metadata key"
    );

    // Vectorizer doesn't maintain cluster metadata, acknowledge but don't persist
    warn!(
        key = %key,
        "Cluster metadata update acknowledged but not persisted (single-node mode)"
    );

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        key = %key,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Cluster API: Updated metadata key (simulated)"
    );

    Ok(Json(QdrantUpdateMetadataKeyResponse {
        result: true,
        status: "ok".to_string(),
        time: elapsed,
    }))
}
