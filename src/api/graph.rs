//! Graph API endpoints for relationship queries and management
//!
//! This module provides REST API endpoints for graph operations including:
//! - Listing nodes in a collection
//! - Getting neighbors of a node
//! - Finding related nodes
//! - Finding paths between nodes
//! - Creating and deleting edges

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::db::graph::{Edge, Graph, Node, RelationshipType};
use crate::db::{CollectionType, VectorStore};
use crate::error::VectorizerError;

/// API state for graph endpoints
#[derive(Clone)]
pub struct GraphApiState {
    pub store: std::sync::Arc<VectorStore>,
}

/// Create the graph API router
pub fn create_graph_router() -> Router<GraphApiState> {
    Router::new()
        .route("/api/v1/graph/nodes/{collection}", get(list_nodes))
        .route(
            "/api/v1/graph/nodes/{collection}/{node_id}/neighbors",
            get(get_neighbors),
        )
        .route(
            "/api/v1/graph/nodes/{collection}/{node_id}/related",
            post(find_related),
        )
        .route("/api/v1/graph/path", post(find_path))
        .route("/api/v1/graph/edges", post(create_edge))
        .route("/api/v1/graph/edges/{edge_id}", delete(delete_edge))
        .route(
            "/api/v1/graph/collections/{collection}/edges",
            get(list_edges),
        )
}

/// Request/Response types

#[derive(Debug, Serialize, Deserialize)]
pub struct ListNodesResponse {
    pub nodes: Vec<Node>,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNeighborsResponse {
    pub neighbors: Vec<NeighborInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NeighborInfo {
    pub node: Node,
    pub edge: Edge,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRelatedRequest {
    pub max_hops: Option<usize>,
    pub relationship_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindRelatedResponse {
    pub related: Vec<RelatedNodeInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedNodeInfo {
    pub node: Node,
    pub distance: usize,
    pub weight: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPathRequest {
    pub collection: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPathResponse {
    pub path: Vec<Node>,
    pub found: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListEdgesResponse {
    pub edges: Vec<EdgeInfo>,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeInfo {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relationship_type: String,
    pub weight: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEdgeRequest {
    pub collection: String,
    pub source: String,
    pub target: String,
    pub relationship_type: String,
    pub weight: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEdgeResponse {
    pub edge_id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteEdgeResponse {
    pub success: bool,
    pub message: String,
}

/// GET /api/v1/graph/nodes/:collection
/// List all nodes in a collection
pub async fn list_nodes(
    State(state): State<GraphApiState>,
    axum::extract::Path(collection_name): axum::extract::Path<String>,
) -> Result<Json<ListNodesResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!("GET /api/v1/graph/nodes/{}", collection_name);

    let collection = state.store.get_collection(&collection_name).map_err(|e| {
        error!("Collection '{}' not found: {}", collection_name, e);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Collection '{}' not found", collection_name)
            })),
        )
    })?;

    // Get graph from collection (only works for CPU collections for now)
    let graph = get_collection_graph_from_type(&collection).ok_or_else(|| {
        error!("Graph not enabled for collection '{}'", collection_name);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Graph not enabled for collection '{}'", collection_name)
            })),
        )
    })?;

    let nodes = graph.get_all_nodes();
    let count = nodes.len();

    Ok(Json(ListNodesResponse { nodes, count }))
}

/// GET /api/v1/graph/nodes/:collection/:node_id/neighbors
/// Get neighbors of a node
pub async fn get_neighbors(
    State(state): State<GraphApiState>,
    axum::extract::Path((collection_name, node_id)): axum::extract::Path<(String, String)>,
) -> Result<Json<GetNeighborsResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!("GET /api/v1/graph/nodes/{}/neighbors", collection_name);

    let collection = state.store.get_collection(&collection_name).map_err(|e| {
        error!("Collection '{}' not found: {}", collection_name, e);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Collection '{}' not found", collection_name)
            })),
        )
    })?;

    let graph = get_collection_graph_from_type(&collection).ok_or_else(|| {
        error!("Graph not enabled for collection '{}'", collection_name);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Graph not enabled for collection '{}'", collection_name)
            })),
        )
    })?;

    let neighbors = graph.get_neighbors(&node_id, None).map_err(|e| {
        error!("Failed to get neighbors for node '{}': {}", node_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get neighbors: {}", e)
            })),
        )
    })?;

    let neighbor_infos: Vec<NeighborInfo> = neighbors
        .into_iter()
        .map(|(node, edge)| NeighborInfo { node, edge })
        .collect();

    Ok(Json(GetNeighborsResponse {
        neighbors: neighbor_infos,
    }))
}

/// POST /api/v1/graph/nodes/:collection/:node_id/related
/// Find related nodes within N hops
pub async fn find_related(
    State(state): State<GraphApiState>,
    Path((collection_name, node_id)): Path<(String, String)>,
    Json(request): Json<FindRelatedRequest>,
) -> Result<Json<FindRelatedResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!("POST /api/v1/graph/nodes/{}/related", collection_name);

    let collection = state.store.get_collection(&collection_name).map_err(|e| {
        error!("Collection '{}' not found: {}", collection_name, e);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Collection '{}' not found", collection_name)
            })),
        )
    })?;

    let graph = get_collection_graph_from_type(&collection).ok_or_else(|| {
        error!("Graph not enabled for collection '{}'", collection_name);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Graph not enabled for collection '{}'", collection_name)
            })),
        )
    })?;

    let max_hops = request.max_hops.unwrap_or(2);
    let relationship_type = request
        .relationship_type
        .as_ref()
        .and_then(|s| parse_relationship_type(s));

    let related = graph
        .find_related(&node_id, max_hops, relationship_type)
        .map_err(|e| {
            error!("Failed to find related nodes for '{}': {}", node_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to find related nodes: {}", e)
                })),
            )
        })?;

    let related_infos: Vec<RelatedNodeInfo> = related
        .into_iter()
        .map(|(node, distance, weight)| RelatedNodeInfo {
            node,
            distance,
            weight,
        })
        .collect();

    Ok(Json(FindRelatedResponse {
        related: related_infos,
    }))
}

/// POST /api/v1/graph/path
/// Find shortest path between two nodes
pub async fn find_path(
    State(state): State<GraphApiState>,
    Json(request): Json<FindPathRequest>,
) -> Result<Json<FindPathResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!(
        "POST /api/v1/graph/path: {} -> {} in collection '{}'",
        request.source, request.target, request.collection
    );

    let collection = state
        .store
        .get_collection(&request.collection)
        .map_err(|e| {
            error!("Collection '{}' not found: {}", request.collection, e);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Collection '{}' not found", request.collection)
                })),
            )
        })?;

    let graph = get_collection_graph_from_type(&collection).ok_or_else(|| {
        error!("Graph not enabled for collection '{}'", request.collection);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Graph not enabled for collection '{}'", request.collection)
            })),
        )
    })?;

    match graph.find_path(&request.source, &request.target) {
        Ok(path) => Ok(Json(FindPathResponse { path, found: true })),
        Err(VectorizerError::NotFound(_)) => Ok(Json(FindPathResponse {
            path: Vec::new(),
            found: false,
        })),
        Err(e) => {
            error!("Failed to find path: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to find path: {}", e)
                })),
            ))
        }
    }
}

/// POST /api/v1/graph/edges
/// Create an explicit edge
pub async fn create_edge(
    State(state): State<GraphApiState>,
    Json(request): Json<CreateEdgeRequest>,
) -> Result<Json<CreateEdgeResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!(
        "POST /api/v1/graph/edges: {} -> {} ({})",
        request.source, request.target, request.relationship_type
    );

    let collection = state
        .store
        .get_collection(&request.collection)
        .map_err(|e| {
            error!("Collection '{}' not found: {}", request.collection, e);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Collection '{}' not found", request.collection)
                })),
            )
        })?;

    let graph = get_collection_graph_from_type(&collection).ok_or_else(|| {
        error!("Graph not enabled for collection '{}'", request.collection);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Graph not enabled for collection '{}'", request.collection)
            })),
        )
    })?;

    let relationship_type =
        parse_relationship_type(&request.relationship_type).ok_or_else(|| {
            error!("Invalid relationship type: {}", request.relationship_type);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Invalid relationship type: {}", request.relationship_type)
                })),
            )
        })?;

    let weight = request.weight.unwrap_or(1.0);
    let edge_id = format!(
        "{}:{}:{:?}",
        request.source, request.target, relationship_type
    );

    let edge = Edge::new(
        edge_id.clone(),
        request.source.clone(),
        request.target.clone(),
        relationship_type,
        weight,
    );

    graph.add_edge(edge).map_err(|e| {
        error!("Failed to create edge: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to create edge: {}", e)
            })),
        )
    })?;

    info!(
        "Created edge '{}' from '{}' to '{}'",
        edge_id, request.source, request.target
    );

    Ok(Json(CreateEdgeResponse {
        edge_id,
        success: true,
        message: "Edge created successfully".to_string(),
    }))
}

/// GET /api/v1/graph/edges/:collection
/// List all edges in a collection
pub async fn list_edges(
    State(state): State<GraphApiState>,
    Path(collection_name): Path<String>,
) -> Result<Json<ListEdgesResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!("GET /api/v1/graph/collections/{}/edges", collection_name);

    let collection = state.store.get_collection(&collection_name).map_err(|e| {
        error!("Collection '{}' not found: {}", collection_name, e);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Collection '{}' not found", collection_name)
            })),
        )
    })?;

    let graph = get_collection_graph_from_type(&collection).ok_or_else(|| {
        error!("Graph not enabled for collection '{}'", collection_name);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Graph not enabled for collection '{}'", collection_name)
            })),
        )
    })?;

    let edges = graph.get_all_edges();

    let edge_infos: Vec<EdgeInfo> = edges
        .into_iter()
        .map(|edge| EdgeInfo {
            id: edge.id.clone(),
            source: edge.source.clone(),
            target: edge.target.clone(),
            relationship_type: match edge.relationship_type {
                RelationshipType::SimilarTo => "SIMILAR_TO".to_string(),
                RelationshipType::References => "REFERENCES".to_string(),
                RelationshipType::Contains => "CONTAINS".to_string(),
                RelationshipType::DerivedFrom => "DERIVED_FROM".to_string(),
            },
            weight: edge.weight,
            metadata: edge.metadata.clone(),
            created_at: edge.created_at,
        })
        .collect();

    Ok(Json(ListEdgesResponse {
        count: edge_infos.len(),
        edges: edge_infos,
    }))
}

/// DELETE /api/v1/graph/edges/:edge_id
/// Delete an edge
pub async fn delete_edge(
    State(state): State<GraphApiState>,
    Path(edge_id): Path<String>,
) -> Result<Json<DeleteEdgeResponse>, (StatusCode, Json<serde_json::Value>)> {
    debug!("DELETE /api/v1/graph/edges/{}", edge_id);

    // Edge ID format: "source:target:RELATIONSHIP_TYPE" or "collection:source:target:RELATIONSHIP_TYPE"
    // For now, we need to search all collections to find the edge
    // TODO: Store edge_id -> collection mapping for faster lookup

    let collections = state.store.list_collections();
    let mut found = false;

    for collection_name in collections {
        if let Ok(collection) = state.store.get_collection(&collection_name) {
            if let Some(graph) = get_collection_graph_from_type(&collection) {
                if graph.remove_edge(&edge_id).is_ok() {
                    found = true;
                    info!(
                        "Deleted edge '{}' from collection '{}'",
                        edge_id, collection_name
                    );
                    break;
                }
            }
        }
    }

    if !found {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Edge '{}' not found", edge_id)
            })),
        ));
    }

    Ok(Json(DeleteEdgeResponse {
        success: true,
        message: "Edge deleted successfully".to_string(),
    }))
}

/// Helper functions

/// Get graph from a collection type (if enabled)
fn get_collection_graph_from_type(collection: &CollectionType) -> Option<&Graph> {
    match collection {
        crate::db::CollectionType::Cpu(c) => c.get_graph().map(|arc| arc.as_ref()),
        _ => None, // Graph only supported for CPU collections for now
    }
}

/// Parse relationship type from string
fn parse_relationship_type(s: &str) -> Option<RelationshipType> {
    match s.to_uppercase().as_str() {
        "SIMILAR_TO" | "SIMILARTO" => Some(RelationshipType::SimilarTo),
        "REFERENCES" => Some(RelationshipType::References),
        "CONTAINS" => Some(RelationshipType::Contains),
        "DERIVED_FROM" | "DERIVEDFROM" => Some(RelationshipType::DerivedFrom),
        _ => None,
    }
}
