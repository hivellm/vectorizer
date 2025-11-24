//! Graph MCP tool handlers

use rmcp::model::{CallToolRequestParam, CallToolResult, Content, ErrorData};
use serde_json::json;
use std::sync::Arc;

use crate::db::{CollectionType, VectorStore};
use crate::db::graph::RelationshipType;

/// Get graph from a collection type (if enabled)
fn get_collection_graph_from_type(collection: &CollectionType) -> Option<&crate::db::graph::Graph> {
    match collection {
        crate::db::CollectionType::Cpu(c) => {
            c.get_graph().map(|arc| arc.as_ref())
        }
        _ => None, // Graph only supported for CPU collections for now
    }
}

/// Handle graph_list_nodes tool
pub async fn handle_graph_list_nodes(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args: serde_json::Value = serde_json::from_str(
        request.arguments.as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default()
            .as_str()
    )
    .map_err(|e| ErrorData::invalid_params(format!("Invalid arguments: {}", e), None))?;

    let collection_name = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: collection", None))?;

    let collection = store.get_collection(collection_name)
        .map_err(|e| ErrorData::internal_error(format!("Collection '{}' not found: {}", collection_name, e), None))?;

    let graph = get_collection_graph_from_type(&collection)
        .ok_or_else(|| ErrorData::invalid_params(format!("Graph not enabled for collection '{}'", collection_name), None))?;

    let nodes = graph.get_all_nodes();
    let count = nodes.len();

    Ok(CallToolResult::success(vec![Content::text(json!({
        "nodes": nodes,
        "count": count
    }).to_string())]))
}

/// Handle graph_get_neighbors tool
pub async fn handle_graph_get_neighbors(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args: serde_json::Value = serde_json::from_str(
        request.arguments.as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default()
            .as_str()
    )
    .map_err(|e| ErrorData::invalid_params(format!("Invalid arguments: {}", e), None))?;

    let collection_name = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: collection", None))?;

    let node_id = args.get("node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: node_id", None))?;

    let collection = store.get_collection(collection_name)
        .map_err(|e| ErrorData::internal_error(format!("Collection '{}' not found: {}", collection_name, e), None))?;

    let graph = get_collection_graph_from_type(&collection)
        .ok_or_else(|| ErrorData::invalid_params(format!("Graph not enabled for collection '{}'", collection_name), None))?;

    let neighbors = graph.get_neighbors(node_id, None)
        .map_err(|e| ErrorData::internal_error(format!("Failed to get neighbors: {}", e), None))?;

    let neighbor_infos: Vec<serde_json::Value> = neighbors
        .into_iter()
        .map(|(node, edge)| json!({
            "node": node,
            "edge": edge
        }))
        .collect();

    Ok(CallToolResult::success(vec![Content::text(json!({
        "neighbors": neighbor_infos
    }).to_string())]))
}

/// Handle graph_find_related tool
pub async fn handle_graph_find_related(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args: serde_json::Value = serde_json::from_str(
        request.arguments.as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default()
            .as_str()
    )
    .map_err(|e| ErrorData::invalid_params(format!("Invalid arguments: {}", e), None))?;

    let collection_name = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: collection", None))?;

    let node_id = args.get("node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: node_id", None))?;

    let max_hops = args.get("max_hops")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(2);

    let relationship_type = args.get("relationship_type")
        .and_then(|v| v.as_str())
        .and_then(|s| parse_relationship_type(s));

    let collection = store.get_collection(collection_name)
        .map_err(|e| ErrorData::internal_error(format!("Collection '{}' not found: {}", collection_name, e), None))?;

    let graph = get_collection_graph_from_type(&collection)
        .ok_or_else(|| ErrorData::invalid_params(format!("Graph not enabled for collection '{}'", collection_name), None))?;

    let related = graph.find_related(node_id, max_hops, relationship_type)
        .map_err(|e| ErrorData::internal_error(format!("Failed to find related nodes: {}", e), None))?;

    let related_infos: Vec<serde_json::Value> = related
        .into_iter()
        .map(|(node, distance, weight)| json!({
            "node": node,
            "distance": distance,
            "weight": weight
        }))
        .collect();

    Ok(CallToolResult::success(vec![Content::text(json!({
        "related": related_infos
    }).to_string())]))
}

/// Handle graph_find_path tool
pub async fn handle_graph_find_path(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args: serde_json::Value = serde_json::from_str(
        request.arguments.as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default()
            .as_str()
    )
    .map_err(|e| ErrorData::invalid_params(format!("Invalid arguments: {}", e), None))?;

    let collection_name = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: collection", None))?;

    let source = args.get("source")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: source", None))?;

    let target = args.get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: target", None))?;

    let collection = store.get_collection(collection_name)
        .map_err(|e| ErrorData::internal_error(format!("Collection '{}' not found: {}", collection_name, e), None))?;

    let graph = get_collection_graph_from_type(&collection)
        .ok_or_else(|| ErrorData::invalid_params(format!("Graph not enabled for collection '{}'", collection_name), None))?;

    match graph.find_path(source, target) {
        Ok(path) => Ok(CallToolResult::success(vec![Content::text(json!({
            "path": path,
            "found": true
        }).to_string())])),
        Err(e) => {
            if e.to_string().contains("not found") || e.to_string().contains("No path") {
                Ok(CallToolResult::success(vec![Content::text(json!({
                    "path": [],
                    "found": false,
                    "message": e.to_string()
                }).to_string())]))
            } else {
                Err(ErrorData::internal_error(format!("Failed to find path: {}", e), None))
            }
        }
    }
}

/// Handle graph_create_edge tool
pub async fn handle_graph_create_edge(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args: serde_json::Value = serde_json::from_str(
        request.arguments.as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default()
            .as_str()
    )
    .map_err(|e| ErrorData::invalid_params(format!("Invalid arguments: {}", e), None))?;

    let collection_name = args.get("collection")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: collection", None))?;

    let source = args.get("source")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: source", None))?;

    let target = args.get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: target", None))?;

    let relationship_type_str = args.get("relationship_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: relationship_type", None))?;

    let relationship_type = parse_relationship_type(relationship_type_str)
        .ok_or_else(|| {
            let msg = format!("Invalid relationship type: {}", relationship_type_str);
            ErrorData::invalid_params(msg, None)
        })?;

    let weight = args.get("weight")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0);

    let collection = store.get_collection(collection_name)
        .map_err(|e| ErrorData::internal_error(format!("Collection '{}' not found: {}", collection_name, e), None))?;

    let graph = get_collection_graph_from_type(&collection)
        .ok_or_else(|| ErrorData::invalid_params(format!("Graph not enabled for collection '{}'", collection_name), None))?;

    let edge_id = format!("{}:{}:{:?}", source, target, relationship_type);
    let edge = crate::db::graph::Edge::new(
        edge_id.clone(),
        source.to_string(),
        target.to_string(),
        relationship_type,
        weight,
    );

    graph.add_edge(edge)
        .map_err(|e| ErrorData::internal_error(format!("Failed to create edge: {}", e), None))?;

    Ok(CallToolResult::success(vec![Content::text(json!({
        "edge_id": edge_id,
        "success": true,
        "message": "Edge created successfully"
    }).to_string())]))
}

/// Handle graph_delete_edge tool
pub async fn handle_graph_delete_edge(
    request: CallToolRequestParam,
    store: Arc<VectorStore>,
) -> Result<CallToolResult, ErrorData> {
    let args: serde_json::Value = serde_json::from_str(
        request.arguments.as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default()
            .as_str()
    )
    .map_err(|e| ErrorData::invalid_params(format!("Invalid arguments: {}", e), None))?;

    let edge_id = args.get("edge_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorData::invalid_params("Missing required parameter: edge_id", None))?;

    // Search all collections to find the edge
    let collections = store.list_collections();
    let mut found = false;

    for collection_name in collections {
        if let Ok(collection) = store.get_collection(&collection_name) {
            if let Some(graph) = get_collection_graph_from_type(&collection) {
                if graph.remove_edge(edge_id).is_ok() {
                    found = true;
                    break;
                }
            }
        }
    }

    if !found {
        return Err(ErrorData::invalid_params(format!("Edge '{}' not found", edge_id), None));
    }

    Ok(CallToolResult::success(vec![Content::text(json!({
        "success": true,
        "message": "Edge deleted successfully"
    }).to_string())]))
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

