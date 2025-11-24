//! Graph database module for relationship tracking and traversal
//!
//! This module provides a lightweight in-memory graph structure for tracking
//! relationships between documents/files based on semantic similarity, metadata,
//! and explicit relationships.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{Result, VectorizerError};

/// Relationship types between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    /// Documents are semantically similar
    SimilarTo,
    /// Document references another document
    References,
    /// Document contains another document
    Contains,
    /// Document is derived from another document
    DerivedFrom,
}

impl RelationshipType {
    /// Get all relationship types
    pub fn all() -> Vec<RelationshipType> {
        vec![
            RelationshipType::SimilarTo,
            RelationshipType::References,
            RelationshipType::Contains,
            RelationshipType::DerivedFrom,
        ]
    }
}

/// Graph node representing a document/file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique node identifier (typically file_path or vector ID)
    pub id: String,
    /// Node type (e.g., "document", "file", "chunk")
    pub node_type: String,
    /// Node metadata (file path, creation time, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Node {
    /// Create a new node
    pub fn new(id: String, node_type: String) -> Self {
        Self {
            id,
            node_type,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create a node from vector ID and payload
    pub fn from_vector(vector_id: &str, payload: Option<&crate::models::Payload>) -> Self {
        let mut node = Self::new(vector_id.to_string(), "document".to_string());

        if let Some(payload) = payload {
            // Extract file_path if available
            if let Some(file_path) = payload.data.get("file_path") {
                node.metadata
                    .insert("file_path".to_string(), file_path.clone());
            }

            // Copy other relevant metadata
            if let Some(obj) = payload.data.as_object() {
                for (key, value) in obj {
                    if key != "file_path" {
                        node.metadata.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        node
    }
}

/// Graph edge representing a relationship between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Unique edge identifier
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Edge weight (e.g., similarity score)
    pub weight: f32,
    /// Edge metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Edge {
    /// Create a new edge
    pub fn new(
        id: String,
        source: String,
        target: String,
        relationship_type: RelationshipType,
        weight: f32,
    ) -> Self {
        Self {
            id,
            source,
            target,
            relationship_type,
            weight,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create an edge with similarity weight
    pub fn with_similarity(source: String, target: String, similarity_score: f32) -> Self {
        let id = format!(
            "{}:{}:{}",
            source,
            target,
            RelationshipType::SimilarTo as u8
        );
        Self::new(
            id,
            source,
            target,
            RelationshipType::SimilarTo,
            similarity_score,
        )
    }
}

/// Graph structure for storing nodes and edges
#[derive(Debug, Clone)]
pub struct Graph {
    /// Collection name this graph belongs to
    collection_name: String,
    /// Nodes indexed by ID
    nodes: Arc<RwLock<HashMap<String, Node>>>,
    /// Edges indexed by ID
    edges: Arc<RwLock<HashMap<String, Edge>>>,
    /// Adjacency list: node_id -> Vec<edge_id>
    adjacency_list: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Reverse adjacency list: node_id -> Vec<edge_id> (for incoming edges)
    reverse_adjacency_list: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl Graph {
    /// Create a new empty graph
    pub fn new(collection_name: String) -> Self {
        Self {
            collection_name,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
            adjacency_list: Arc::new(RwLock::new(HashMap::new())),
            reverse_adjacency_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get collection name
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Add a node to the graph
    pub fn add_node(&self, node: Node) -> Result<()> {
        let mut nodes = self.nodes.write();
        if nodes.contains_key(&node.id) {
            // Update existing node
            debug!("Updating existing node '{}' in graph", node.id);
        } else {
            info!(
                "Adding node '{}' to graph '{}'",
                node.id, self.collection_name
            );
        }
        nodes.insert(node.id.clone(), node);
        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<Node> {
        let nodes = self.nodes.read();
        nodes.get(node_id).cloned()
    }

    /// Remove a node and all its edges
    pub fn remove_node(&self, node_id: &str) -> Result<()> {
        let mut nodes = self.nodes.write();
        let mut edges = self.edges.write();
        let mut adjacency = self.adjacency_list.write();
        let mut reverse_adjacency = self.reverse_adjacency_list.write();

        if !nodes.contains_key(node_id) {
            return Err(VectorizerError::NotFound(format!(
                "Node '{}' not found",
                node_id
            )));
        }

        // Remove all edges connected to this node
        let edge_ids_to_remove: Vec<String> = {
            let outgoing = adjacency.get(node_id).cloned().unwrap_or_default();
            let incoming = reverse_adjacency.get(node_id).cloned().unwrap_or_default();
            [outgoing, incoming].concat()
        };

        for edge_id in &edge_ids_to_remove {
            if let Some(edge) = edges.get(edge_id) {
                // Remove from adjacency lists
                if let Some(neighbors) = adjacency.get_mut(&edge.source) {
                    neighbors.retain(|id| id != edge_id);
                }
                if let Some(neighbors) = reverse_adjacency.get_mut(&edge.target) {
                    neighbors.retain(|id| id != edge_id);
                }
                edges.remove(edge_id);
            }
        }

        // Remove from adjacency lists
        adjacency.remove(node_id);
        reverse_adjacency.remove(node_id);

        // Remove node
        nodes.remove(node_id);

        info!(
            "Removed node '{}' and {} edges from graph",
            node_id,
            edge_ids_to_remove.len()
        );
        Ok(())
    }

    /// Add an edge to the graph
    pub fn add_edge(&self, edge: Edge) -> Result<()> {
        // Verify both nodes exist
        let nodes = self.nodes.read();
        if !nodes.contains_key(&edge.source) {
            return Err(VectorizerError::NotFound(format!(
                "Source node '{}' not found",
                edge.source
            )));
        }
        if !nodes.contains_key(&edge.target) {
            return Err(VectorizerError::NotFound(format!(
                "Target node '{}' not found",
                edge.target
            )));
        }
        drop(nodes);

        let mut edges = self.edges.write();
        let mut adjacency = self.adjacency_list.write();
        let mut reverse_adjacency = self.reverse_adjacency_list.write();

        // Check if edge already exists
        if edges.contains_key(&edge.id) {
            // Update existing edge
            debug!("Updating existing edge '{}' in graph", edge.id);
        } else {
            info!(
                "Adding edge '{}' ({:?}) from '{}' to '{}' in graph",
                edge.id, edge.relationship_type, edge.source, edge.target
            );
        }

        edges.insert(edge.id.clone(), edge.clone());

        // Update adjacency lists
        adjacency
            .entry(edge.source.clone())
            .or_insert_with(Vec::new)
            .push(edge.id.clone());
        reverse_adjacency
            .entry(edge.target.clone())
            .or_insert_with(Vec::new)
            .push(edge.id.clone());

        Ok(())
    }

    /// Remove an edge from the graph
    pub fn remove_edge(&self, edge_id: &str) -> Result<()> {
        let mut edges = self.edges.write();
        let mut adjacency = self.adjacency_list.write();
        let mut reverse_adjacency = self.reverse_adjacency_list.write();

        let edge = edges
            .get(edge_id)
            .ok_or_else(|| VectorizerError::NotFound(format!("Edge '{}' not found", edge_id)))?;

        // Remove from adjacency lists
        if let Some(neighbors) = adjacency.get_mut(&edge.source) {
            neighbors.retain(|id| id != edge_id);
        }
        if let Some(neighbors) = reverse_adjacency.get_mut(&edge.target) {
            neighbors.retain(|id| id != edge_id);
        }

        edges.remove(edge_id);
        info!("Removed edge '{}' from graph", edge_id);
        Ok(())
    }

    /// Get neighbors of a node
    pub fn get_neighbors(
        &self,
        node_id: &str,
        relationship_type: Option<RelationshipType>,
    ) -> Result<Vec<(Node, Edge)>> {
        let nodes = self.nodes.read();
        let edges = self.edges.read();
        let adjacency = self.adjacency_list.read();

        if !nodes.contains_key(node_id) {
            return Err(VectorizerError::NotFound(format!(
                "Node '{}' not found",
                node_id
            )));
        }

        let edge_ids = adjacency.get(node_id).cloned().unwrap_or_default();
        let mut neighbors = Vec::new();

        for edge_id in edge_ids {
            if let Some(edge) = edges.get(&edge_id) {
                // Filter by relationship type if specified
                if let Some(rel_type) = relationship_type {
                    if edge.relationship_type != rel_type {
                        continue;
                    }
                }

                if let Some(target_node) = nodes.get(&edge.target) {
                    neighbors.push((target_node.clone(), edge.clone()));
                }
            }
        }

        Ok(neighbors)
    }

    /// Find all nodes related to a given node within N hops
    pub fn find_related(
        &self,
        node_id: &str,
        max_hops: usize,
        relationship_type: Option<RelationshipType>,
    ) -> Result<Vec<(Node, usize, f32)>> {
        // BFS traversal
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut results = Vec::new();

        queue.push_back((node_id.to_string(), 0, 1.0)); // (node_id, hop_count, cumulative_weight)
        visited.insert(node_id.to_string());

        while let Some((current_id, hop_count, cumulative_weight)) = queue.pop_front() {
            if hop_count > max_hops {
                continue;
            }

            // Get neighbors
            let neighbors = self.get_neighbors(&current_id, relationship_type)?;

            for (neighbor, edge) in neighbors {
                if !visited.contains(&neighbor.id) {
                    visited.insert(neighbor.id.clone());
                    let new_weight = cumulative_weight * edge.weight;
                    results.push((neighbor.clone(), hop_count + 1, new_weight));

                    if hop_count + 1 < max_hops {
                        queue.push_back((neighbor.id.clone(), hop_count + 1, new_weight));
                    }
                }
            }
        }

        // Sort by weight (descending)
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// Find shortest path between two nodes
    pub fn find_path(&self, source: &str, target: &str) -> Result<Vec<Node>> {
        // BFS to find shortest path
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(source.to_string());
        visited.insert(source.to_string());

        while let Some(current) = queue.pop_front() {
            if current == target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node_id = target.to_string();
                let nodes = self.nodes.read();

                while let Some(node) = nodes.get(&node_id) {
                    path.push(node.clone());
                    if let Some(parent_id) = parent.get(&node_id) {
                        node_id = parent_id.clone();
                    } else {
                        break;
                    }
                }

                path.reverse();
                return Ok(path);
            }

            // Get neighbors
            let neighbors = self.get_neighbors(&current, None)?;
            for (neighbor, _) in neighbors {
                if !visited.contains(&neighbor.id) {
                    visited.insert(neighbor.id.clone());
                    parent.insert(neighbor.id.clone(), current.clone());
                    queue.push_back(neighbor.id.clone());
                }
            }
        }

        Err(VectorizerError::NotFound(format!(
            "No path found from '{}' to '{}'",
            source, target
        )))
    }

    /// Get all nodes in the graph
    pub fn get_all_nodes(&self) -> Vec<Node> {
        let nodes = self.nodes.read();
        nodes.values().cloned().collect()
    }

    /// Get all edges in the graph
    pub fn get_all_edges(&self) -> Vec<Edge> {
        let edges = self.edges.read();
        edges.values().cloned().collect()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        let nodes = self.nodes.read();
        nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        let edges = self.edges.read();
        edges.len()
    }

    /// Get connected components (nodes that are reachable from each other)
    pub fn get_connected_components(&self) -> Vec<Vec<String>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();
        let nodes = self.nodes.read();

        for node_id in nodes.keys() {
            if !visited.contains(node_id) {
                // BFS to find all connected nodes
                let mut component = Vec::new();
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(node_id.clone());
                visited.insert(node_id.clone());

                while let Some(current) = queue.pop_front() {
                    component.push(current.clone());

                    let neighbors = self.get_neighbors(&current, None).unwrap_or_default();
                    for (neighbor, _) in neighbors {
                        if !visited.contains(&neighbor.id) {
                            visited.insert(neighbor.id.clone());
                            queue.push_back(neighbor.id.clone());
                        }
                    }
                }

                components.push(component);
            }
        }

        components
    }
}
