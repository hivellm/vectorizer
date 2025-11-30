//! Graph models for the Vectorizer SDK

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Graph node representing a document/file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node identifier
    pub id: String,
    /// Node type (e.g., "document", "file", "chunk")
    pub node_type: String,
    /// Node metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Graph edge representing a relationship between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Edge identifier
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Relationship type
    pub relationship_type: String,
    /// Edge weight (0.0 to 1.0)
    pub weight: f32,
    /// Edge metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: String,
}

/// Relationship type between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Neighbor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborInfo {
    /// Neighbor node
    pub node: GraphNode,
    /// Edge connecting to neighbor
    pub edge: GraphEdge,
}

/// Related node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedNodeInfo {
    /// Related node
    pub node: GraphNode,
    /// Distance in hops
    pub distance: usize,
    /// Relationship weight
    pub weight: f32,
}

/// Request to find related nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindRelatedRequest {
    /// Maximum number of hops
    pub max_hops: Option<usize>,
    /// Relationship type filter
    pub relationship_type: Option<String>,
}

/// Response for finding related nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindRelatedResponse {
    /// List of related nodes
    pub related: Vec<RelatedNodeInfo>,
}

/// Request to find path between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindPathRequest {
    /// Collection name
    pub collection: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
}

/// Response for finding path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindPathResponse {
    /// Path as list of nodes
    pub path: Vec<GraphNode>,
    /// Whether path was found
    pub found: bool,
}

/// Request to create an edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEdgeRequest {
    /// Collection name
    pub collection: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Relationship type
    pub relationship_type: String,
    /// Optional edge weight
    pub weight: Option<f32>,
}

/// Response for creating an edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEdgeResponse {
    /// Created edge ID
    pub edge_id: String,
    /// Success status
    pub success: bool,
    /// Status message
    pub message: String,
}

/// Response for listing nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNodesResponse {
    /// List of nodes
    pub nodes: Vec<GraphNode>,
    /// Total count
    pub count: usize,
}

/// Response for getting neighbors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNeighborsResponse {
    /// List of neighbors
    pub neighbors: Vec<NeighborInfo>,
}

/// Response for listing edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEdgesResponse {
    /// List of edges
    pub edges: Vec<GraphEdge>,
    /// Total count
    pub count: usize,
}

/// Request to discover edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverEdgesRequest {
    /// Similarity threshold (0.0 to 1.0)
    pub similarity_threshold: Option<f32>,
    /// Maximum edges per node
    pub max_per_node: Option<usize>,
}

/// Response for discovering edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverEdgesResponse {
    /// Success status
    pub success: bool,
    /// Number of edges created
    pub edges_created: usize,
    /// Status message
    pub message: String,
}

/// Response for discovery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStatusResponse {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Number of nodes with edges
    pub nodes_with_edges: usize,
    /// Total number of edges
    pub total_edges: usize,
    /// Progress percentage
    pub progress_percentage: f64,
}
