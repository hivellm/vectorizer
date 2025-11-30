//! Qdrant Sharding API handlers
//!
//! This module provides handlers for the Qdrant Sharding API endpoints.

use std::time::Instant;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use tracing::{error, info, warn};

use super::VectorizerServer;
use super::error_middleware::{ErrorResponse, create_error_response, create_not_found_error};
use crate::db::sharding::ShardId;
use crate::models::qdrant::sharding::{
    QdrantCreateShardKeyRequest, QdrantCreateShardKeyResponse, QdrantDeleteShardKeyRequest,
    QdrantDeleteShardKeyResponse, QdrantListShardKeysResponse, QdrantLocalShardInfo,
    QdrantShardKeyInfo, QdrantShardKeyValue, QdrantShardKeysResult, QdrantShardState,
};

/// Create a shard key for a collection
/// PUT /qdrant/collections/{name}/shards
pub async fn create_shard_key(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantCreateShardKeyRequest>,
) -> Result<Json<QdrantCreateShardKeyResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        collection = %collection_name,
        shard_key = %request.shard_key,
        "Qdrant Sharding API: Creating shard key"
    );

    // Verify collection exists
    state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    // Get the shard key as a numeric ID for the internal sharding system
    let shard_id = match &request.shard_key {
        QdrantShardKeyValue::Integer(i) => ShardId::new(*i as u32),
        QdrantShardKeyValue::String(s) => {
            // Hash the string to get a shard ID
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            ShardId::new((hasher.finish() % u32::MAX as u64) as u32)
        }
    };

    // Get the sharded collection or create sharding configuration
    // For now, we'll just acknowledge the request since Vectorizer uses automatic sharding
    // In a real implementation, this would configure the shard router

    let shards_number = request.shards_number.unwrap_or(1);
    let _replication_factor = request.replication_factor.unwrap_or(1);

    info!(
        collection = %collection_name,
        shard_key = %request.shard_key,
        shard_id = shard_id.as_u32(),
        shards_number = shards_number,
        "Shard key creation acknowledged (using automatic sharding)"
    );

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        collection = %collection_name,
        shard_key = %request.shard_key,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Sharding API: Created shard key"
    );

    Ok(Json(QdrantCreateShardKeyResponse {
        result: true,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// Delete a shard key from a collection
/// POST /qdrant/collections/{name}/shards/delete
pub async fn delete_shard_key(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
    Json(request): Json<QdrantDeleteShardKeyRequest>,
) -> Result<Json<QdrantDeleteShardKeyResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        collection = %collection_name,
        shard_key = %request.shard_key,
        "Qdrant Sharding API: Deleting shard key"
    );

    // Verify collection exists
    state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    // In Vectorizer, sharding is automatic and managed internally
    // This endpoint acknowledges the request for API compatibility
    warn!(
        collection = %collection_name,
        shard_key = %request.shard_key,
        "Shard key deletion acknowledged (Vectorizer uses automatic sharding)"
    );

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        collection = %collection_name,
        shard_key = %request.shard_key,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Sharding API: Deleted shard key"
    );

    Ok(Json(QdrantDeleteShardKeyResponse {
        result: true,
        status: "ok".to_string(),
        time: elapsed,
    }))
}

/// List shard keys for a collection
/// GET /qdrant/collections/{name}/shards
pub async fn list_shard_keys(
    State(state): State<VectorizerServer>,
    Path(collection_name): Path<String>,
) -> Result<Json<QdrantListShardKeysResponse>, ErrorResponse> {
    let start = Instant::now();
    info!(
        collection = %collection_name,
        "Qdrant Sharding API: Listing shard keys"
    );

    // Verify collection exists and get info
    let collection = state
        .store
        .get_collection(&collection_name)
        .map_err(|_| create_not_found_error("collection", &collection_name))?;

    // Get collection stats for shard info
    let vector_count = collection.vector_count() as u64;

    // Vectorizer uses automatic sharding - report the default shard
    // In a full implementation, this would query the shard router
    let shard_info = QdrantShardKeyInfo {
        shard_key: QdrantShardKeyValue::String("_default".to_string()),
        shards_number: 1,
        replication_factor: 1,
        local_shards: vec![QdrantLocalShardInfo {
            shard_id: 0,
            points_count: vector_count,
            state: QdrantShardState::Active,
        }],
        remote_shards: vec![],
    };

    let elapsed = start.elapsed().as_secs_f64();
    info!(
        collection = %collection_name,
        shard_count = 1,
        elapsed_ms = elapsed * 1000.0,
        "Qdrant Sharding API: Listed shard keys"
    );

    Ok(Json(QdrantListShardKeysResponse {
        result: QdrantShardKeysResult {
            keys: vec![shard_info],
        },
        status: "ok".to_string(),
        time: elapsed,
    }))
}
