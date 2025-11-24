//! Automatic relationship discovery for graph
//!
//! This module provides functionality to automatically discover and create
//! relationships between documents based on semantic similarity, metadata,
//! and payload information.

use tracing::{debug, info, warn};

use crate::db::graph::{Edge, Graph, Node, RelationshipType};
use crate::error::{Result, VectorizerError};
use crate::models::{AutoRelationshipConfig, Vector};

/// Discover and create relationships for a newly inserted vector
pub fn discover_relationships(
    graph: &Graph,
    vector: &Vector,
    collection: &impl GraphRelationshipHelper,
    config: &AutoRelationshipConfig,
) -> Result<()> {
    // Create or update node for this vector
    let node = Node::from_vector(&vector.id, vector.payload.as_ref());
    graph.add_node(node)?;

    // Discover relationships based on similarity
    if is_relationship_type_enabled("SIMILAR_TO", config) {
        discover_similarity_relationships(graph, vector, collection, config)?;
    }

    // Discover relationships based on metadata
    if let Some(payload) = &vector.payload {
        if is_relationship_type_enabled("REFERENCES", config) {
            discover_reference_relationships(graph, &vector.id, payload)?;
        }
        if is_relationship_type_enabled("CONTAINS", config) {
            discover_contains_relationships(graph, &vector.id, payload)?;
        }
        if is_relationship_type_enabled("DERIVED_FROM", config) {
            discover_derived_from_relationships(graph, &vector.id, payload)?;
        }
    }

    Ok(())
}

/// Discover SIMILAR_TO relationships based on semantic similarity
fn discover_similarity_relationships(
    graph: &Graph,
    vector: &Vector,
    collection: &impl GraphRelationshipHelper,
    config: &AutoRelationshipConfig,
) -> Result<()> {
    // Search for similar vectors (limit to max_per_node + some buffer for filtering)
    // Limit search to small number to avoid timeout during insertion
    let search_limit = (config.max_per_node * 2).min(20); // Cap at 20 for fast insertion
    let similar_vectors = collection.search_similar_vectors(&vector.data, search_limit)?;

    // Filter by similarity threshold and limit to max_per_node
    let mut relationships_created = 0;
    for (similar_id, similarity_score) in similar_vectors {
        // Skip self
        if similar_id == vector.id {
            continue;
        }

        // Check similarity threshold
        if similarity_score < config.similarity_threshold {
            continue;
        }

        // Ensure target node exists
        if graph.get_node(&similar_id).is_none() {
            // Create node for similar vector if it doesn't exist
            if let Ok(similar_vector) = collection.get_vector(&similar_id) {
                let similar_node = Node::from_vector(&similar_id, similar_vector.payload.as_ref());
                graph.add_node(similar_node)?;
            } else {
                continue; // Skip if we can't get the vector
            }
        }

        // Create SIMILAR_TO edge
        let edge = Edge::with_similarity(vector.id.clone(), similar_id, similarity_score);
        graph.add_edge(edge)?;
        relationships_created += 1;

        if relationships_created >= config.max_per_node {
            break;
        }
    }

    if relationships_created > 0 {
        debug!(
            "Created {} SIMILAR_TO relationships for vector '{}'",
            relationships_created, vector.id
        );
    }

    Ok(())
}

/// Discover REFERENCES relationships from payload metadata
pub fn discover_reference_relationships(
    graph: &Graph,
    source_id: &str,
    payload: &crate::models::Payload,
) -> Result<()> {
    if let Some(references) = payload.data.get("references") {
        if let Some(ref_array) = references.as_array() {
            for ref_value in ref_array {
                if let Some(ref_path) = ref_value.as_str() {
                    // Try to find node by file_path or create one
                    let target_id = find_or_create_node_by_file_path(graph, ref_path)?;
                    if target_id != source_id {
                        let edge = Edge::new(
                            format!("{}:{}:REFERENCES", source_id, target_id),
                            source_id.to_string(),
                            target_id,
                            RelationshipType::References,
                            1.0, // References have weight 1.0
                        );
                        graph.add_edge(edge)?;
                        debug!(
                            "Created REFERENCES relationship from '{}' to '{}'",
                            source_id, ref_path
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/// Discover CONTAINS relationships from payload metadata
pub fn discover_contains_relationships(
    graph: &Graph,
    source_id: &str,
    payload: &crate::models::Payload,
) -> Result<()> {
    if let Some(contains) = payload.data.get("contains") {
        if let Some(contains_array) = contains.as_array() {
            for contained_value in contains_array {
                if let Some(contained_path) = contained_value.as_str() {
                    let target_id = find_or_create_node_by_file_path(graph, contained_path)?;
                    if target_id != source_id {
                        let edge = Edge::new(
                            format!("{}:{}:CONTAINS", source_id, target_id),
                            source_id.to_string(),
                            target_id,
                            RelationshipType::Contains,
                            1.0,
                        );
                        graph.add_edge(edge)?;
                        debug!(
                            "Created CONTAINS relationship from '{}' to '{}'",
                            source_id, contained_path
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/// Discover DERIVED_FROM relationships from payload metadata
pub fn discover_derived_from_relationships(
    graph: &Graph,
    source_id: &str,
    payload: &crate::models::Payload,
) -> Result<()> {
    if let Some(derived_from) = payload.data.get("derived_from") {
        if let Some(derived_path) = derived_from.as_str() {
            let target_id = find_or_create_node_by_file_path(graph, derived_path)?;
            if target_id != source_id {
                let edge = Edge::new(
                    format!("{}:{}:DERIVED_FROM", source_id, target_id),
                    source_id.to_string(),
                    target_id,
                    RelationshipType::DerivedFrom,
                    1.0,
                );
                graph.add_edge(edge)?;
                debug!(
                    "Created DERIVED_FROM relationship from '{}' to '{}'",
                    source_id, derived_path
                );
            }
        }
    }
    Ok(())
}

/// Find or create a node by file path
fn find_or_create_node_by_file_path(graph: &Graph, file_path: &str) -> Result<String> {
    // First, try to find existing node by file_path metadata
    let all_nodes = graph.get_all_nodes();
    for node in all_nodes {
        if let Some(node_file_path) = node.metadata.get("file_path") {
            if let Some(node_path_str) = node_file_path.as_str() {
                if node_path_str == file_path {
                    return Ok(node.id.clone());
                }
            }
        }
    }

    // If not found, create a new node with file_path as ID
    let mut node = Node::new(file_path.to_string(), "document".to_string());
    node.metadata.insert(
        "file_path".to_string(),
        serde_json::Value::String(file_path.to_string()),
    );
    graph.add_node(node.clone())?;
    Ok(node.id)
}

/// Check if a relationship type is enabled in config
pub fn is_relationship_type_enabled(rel_type: &str, config: &AutoRelationshipConfig) -> bool {
    config.enabled_types.iter().any(|t| t == rel_type)
}

/// Trait for collections to provide graph relationship discovery helpers
pub trait GraphRelationshipHelper {
    /// Search for similar vectors and return (id, similarity_score) pairs
    fn search_similar_vectors(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f32)>>;

    /// Get a vector by ID
    fn get_vector(&self, vector_id: &str) -> Result<Vector>;
}
