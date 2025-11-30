//! GraphQL type definitions for Vectorizer
//!
//! This module defines all GraphQL types used in the API.

use async_graphql::{Enum, InputObject, Object, SimpleObject};
use serde_json::Value as JsonValue;

// =============================================================================
// ENUMS
// =============================================================================

/// Distance metric for vector similarity calculations
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlDistanceMetric {
    /// Cosine similarity (normalized dot product)
    Cosine,
    /// Euclidean distance (L2 norm)
    Euclidean,
    /// Dot product (inner product)
    DotProduct,
}

impl From<crate::models::DistanceMetric> for GqlDistanceMetric {
    fn from(metric: crate::models::DistanceMetric) -> Self {
        match metric {
            crate::models::DistanceMetric::Cosine => GqlDistanceMetric::Cosine,
            crate::models::DistanceMetric::Euclidean => GqlDistanceMetric::Euclidean,
            crate::models::DistanceMetric::DotProduct => GqlDistanceMetric::DotProduct,
        }
    }
}

impl From<GqlDistanceMetric> for crate::models::DistanceMetric {
    fn from(metric: GqlDistanceMetric) -> Self {
        match metric {
            GqlDistanceMetric::Cosine => crate::models::DistanceMetric::Cosine,
            GqlDistanceMetric::Euclidean => crate::models::DistanceMetric::Euclidean,
            GqlDistanceMetric::DotProduct => crate::models::DistanceMetric::DotProduct,
        }
    }
}

/// Storage type for collections
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlStorageType {
    /// In-memory storage (fastest, limited by RAM)
    Memory,
    /// Memory-mapped storage (slower, limited by disk)
    Mmap,
}

impl From<crate::models::StorageType> for GqlStorageType {
    fn from(storage: crate::models::StorageType) -> Self {
        match storage {
            crate::models::StorageType::Memory => GqlStorageType::Memory,
            crate::models::StorageType::Mmap => GqlStorageType::Mmap,
        }
    }
}

// =============================================================================
// OUTPUT TYPES
// =============================================================================

/// HNSW index configuration
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlHnswConfig {
    /// Number of bidirectional links per node
    pub m: i32,
    /// Size of dynamic list during construction
    pub ef_construction: i32,
    /// Size of dynamic list during search
    pub ef_search: i32,
}

impl From<&crate::models::HnswConfig> for GqlHnswConfig {
    fn from(config: &crate::models::HnswConfig) -> Self {
        Self {
            m: config.m as i32,
            ef_construction: config.ef_construction as i32,
            ef_search: config.ef_search as i32,
        }
    }
}

/// Collection configuration
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlCollectionConfig {
    /// Vector dimension
    pub dimension: i32,
    /// Distance metric for similarity
    pub metric: GqlDistanceMetric,
    /// HNSW index configuration
    pub hnsw_config: GqlHnswConfig,
    /// Storage type
    pub storage_type: Option<GqlStorageType>,
    /// Whether sharding is enabled
    pub sharding_enabled: bool,
    /// Number of shards (if sharding enabled)
    pub shard_count: Option<i32>,
    /// Whether graph is enabled
    pub graph_enabled: bool,
}

impl From<&crate::models::CollectionConfig> for GqlCollectionConfig {
    fn from(config: &crate::models::CollectionConfig) -> Self {
        Self {
            dimension: config.dimension as i32,
            metric: config.metric.into(),
            hnsw_config: (&config.hnsw_config).into(),
            storage_type: config.storage_type.map(|s| s.into()),
            sharding_enabled: config.sharding.is_some(),
            shard_count: config.sharding.as_ref().map(|s| s.shard_count as i32),
            graph_enabled: config.graph.as_ref().map(|g| g.enabled).unwrap_or(false),
        }
    }
}

/// Collection information
#[derive(Clone, Debug)]
pub struct GqlCollection {
    pub name: String,
    pub tenant_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub vector_count: i32,
    pub document_count: i32,
    pub config: GqlCollectionConfig,
}

#[Object]
impl GqlCollection {
    /// Collection name (unique identifier)
    async fn name(&self) -> &str {
        &self.name
    }

    /// Tenant ID for multi-tenancy support
    async fn tenant_id(&self) -> Option<&str> {
        self.tenant_id.as_deref()
    }

    /// Creation timestamp
    async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    /// Last update timestamp
    async fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }

    /// Number of vectors in the collection
    async fn vector_count(&self) -> i32 {
        self.vector_count
    }

    /// Number of indexed documents
    async fn document_count(&self) -> i32 {
        self.document_count
    }

    /// Collection configuration
    async fn config(&self) -> &GqlCollectionConfig {
        &self.config
    }
}

impl From<crate::models::CollectionMetadata> for GqlCollection {
    fn from(meta: crate::models::CollectionMetadata) -> Self {
        Self {
            name: meta.name,
            tenant_id: meta.tenant_id,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            vector_count: meta.vector_count as i32,
            document_count: meta.document_count as i32,
            config: (&meta.config).into(),
        }
    }
}

/// Vector with payload
#[derive(Clone, Debug)]
pub struct GqlVector {
    pub id: String,
    pub data: Vec<f32>,
    pub payload: Option<JsonValue>,
}

#[Object]
impl GqlVector {
    /// Vector unique identifier
    async fn id(&self) -> &str {
        &self.id
    }

    /// Vector data as array of floats
    async fn data(&self) -> &[f32] {
        &self.data
    }

    /// Vector dimension
    async fn dimension(&self) -> i32 {
        self.data.len() as i32
    }

    /// Payload as JSON
    async fn payload(&self) -> Option<async_graphql::Json<JsonValue>> {
        self.payload.clone().map(async_graphql::Json)
    }
}

impl From<crate::models::Vector> for GqlVector {
    fn from(v: crate::models::Vector) -> Self {
        Self {
            id: v.id,
            data: v.data,
            payload: v.payload.map(|p| p.data),
        }
    }
}

/// Search result with score
#[derive(Clone, Debug)]
pub struct GqlSearchResult {
    pub id: String,
    pub score: f32,
    pub vector: Option<Vec<f32>>,
    pub payload: Option<JsonValue>,
}

#[Object]
impl GqlSearchResult {
    /// Vector ID
    async fn id(&self) -> &str {
        &self.id
    }

    /// Similarity score
    async fn score(&self) -> f32 {
        self.score
    }

    /// Vector data (if requested)
    async fn vector(&self) -> Option<&[f32]> {
        self.vector.as_deref()
    }

    /// Payload as JSON
    async fn payload(&self) -> Option<async_graphql::Json<JsonValue>> {
        self.payload.clone().map(async_graphql::Json)
    }
}

impl From<crate::models::SearchResult> for GqlSearchResult {
    fn from(r: crate::models::SearchResult) -> Self {
        Self {
            id: r.id,
            score: r.score,
            vector: r.vector,
            payload: r.payload.map(|p| p.data),
        }
    }
}

/// Server statistics
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlServerStats {
    /// Server version
    pub version: String,
    /// Total number of collections
    pub collection_count: i32,
    /// Total number of vectors across all collections
    pub total_vectors: i64,
    /// Server uptime in seconds
    pub uptime_seconds: i64,
    /// Memory usage in bytes
    pub memory_usage_bytes: i64,
}

// =============================================================================
// GRAPH TYPES
// =============================================================================

/// Graph relationship type
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlRelationshipType {
    /// Documents are semantically similar
    SimilarTo,
    /// Document references another document
    References,
    /// Document contains another document
    Contains,
    /// Document is derived from another document
    DerivedFrom,
}

impl From<crate::db::graph::RelationshipType> for GqlRelationshipType {
    fn from(rt: crate::db::graph::RelationshipType) -> Self {
        match rt {
            crate::db::graph::RelationshipType::SimilarTo => GqlRelationshipType::SimilarTo,
            crate::db::graph::RelationshipType::References => GqlRelationshipType::References,
            crate::db::graph::RelationshipType::Contains => GqlRelationshipType::Contains,
            crate::db::graph::RelationshipType::DerivedFrom => GqlRelationshipType::DerivedFrom,
        }
    }
}

impl From<GqlRelationshipType> for crate::db::graph::RelationshipType {
    fn from(rt: GqlRelationshipType) -> Self {
        match rt {
            GqlRelationshipType::SimilarTo => crate::db::graph::RelationshipType::SimilarTo,
            GqlRelationshipType::References => crate::db::graph::RelationshipType::References,
            GqlRelationshipType::Contains => crate::db::graph::RelationshipType::Contains,
            GqlRelationshipType::DerivedFrom => crate::db::graph::RelationshipType::DerivedFrom,
        }
    }
}

/// Graph node representing a document/file
#[derive(Clone, Debug)]
pub struct GqlNode {
    pub id: String,
    pub node_type: String,
    pub metadata: JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[Object]
impl GqlNode {
    /// Node unique identifier
    async fn id(&self) -> &str {
        &self.id
    }

    /// Node type (e.g., "document", "file", "chunk")
    async fn node_type(&self) -> &str {
        &self.node_type
    }

    /// Node metadata as JSON
    async fn metadata(&self) -> async_graphql::Json<JsonValue> {
        async_graphql::Json(self.metadata.clone())
    }

    /// Creation timestamp
    async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
}

impl From<crate::db::graph::Node> for GqlNode {
    fn from(n: crate::db::graph::Node) -> Self {
        Self {
            id: n.id,
            node_type: n.node_type,
            metadata: serde_json::to_value(&n.metadata).unwrap_or(JsonValue::Null),
            created_at: n.created_at,
        }
    }
}

/// Graph edge representing a relationship between nodes
#[derive(Clone, Debug)]
pub struct GqlEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relationship_type: GqlRelationshipType,
    pub weight: f32,
    pub metadata: JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[Object]
impl GqlEdge {
    /// Edge unique identifier
    async fn id(&self) -> &str {
        &self.id
    }

    /// Source node ID
    async fn source(&self) -> &str {
        &self.source
    }

    /// Target node ID
    async fn target(&self) -> &str {
        &self.target
    }

    /// Relationship type
    async fn relationship_type(&self) -> GqlRelationshipType {
        self.relationship_type
    }

    /// Edge weight (e.g., similarity score)
    async fn weight(&self) -> f32 {
        self.weight
    }

    /// Edge metadata as JSON
    async fn metadata(&self) -> async_graphql::Json<JsonValue> {
        async_graphql::Json(self.metadata.clone())
    }

    /// Creation timestamp
    async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
}

impl From<crate::db::graph::Edge> for GqlEdge {
    fn from(e: crate::db::graph::Edge) -> Self {
        Self {
            id: e.id,
            source: e.source,
            target: e.target,
            relationship_type: e.relationship_type.into(),
            weight: e.weight,
            metadata: serde_json::to_value(&e.metadata).unwrap_or(JsonValue::Null),
            created_at: e.created_at,
        }
    }
}

/// Related node with hop distance and cumulative weight
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlRelatedNode {
    /// The related node
    pub node: GqlNode,
    /// Number of hops from source
    pub hops: i32,
    /// Cumulative weight from source
    pub weight: f32,
}

/// Graph statistics
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlGraphStats {
    /// Total number of nodes
    pub node_count: i32,
    /// Total number of edges
    pub edge_count: i32,
    /// Whether graph is enabled for this collection
    pub enabled: bool,
}

/// Paginated results wrapper
#[derive(SimpleObject, Clone, Debug)]
#[graphql(concrete(name = "VectorPage", params(GqlVector)))]
#[graphql(concrete(name = "SearchResultPage", params(GqlSearchResult)))]
#[graphql(concrete(name = "NodePage", params(GqlNode)))]
#[graphql(concrete(name = "EdgePage", params(GqlEdge)))]
pub struct GqlPage<T: async_graphql::ObjectType + Send + Sync> {
    /// Items in this page
    pub items: Vec<T>,
    /// Total count of items
    pub total_count: i32,
    /// Whether there are more items
    pub has_next_page: bool,
    /// Cursor for next page (if available)
    pub next_cursor: Option<String>,
}

// =============================================================================
// INPUT TYPES
// =============================================================================

/// Input for creating a new collection
#[derive(InputObject, Clone, Debug)]
pub struct CreateCollectionInput {
    /// Collection name (unique identifier)
    pub name: String,
    /// Vector dimension
    pub dimension: i32,
    /// Distance metric (defaults to Cosine)
    #[graphql(default)]
    pub metric: Option<GqlDistanceMetric>,
    /// HNSW M parameter (defaults to 16)
    #[graphql(default)]
    pub hnsw_m: Option<i32>,
    /// HNSW ef_construction parameter (defaults to 200)
    #[graphql(default)]
    pub hnsw_ef_construction: Option<i32>,
    /// Enable sharding with specified shard count
    #[graphql(default)]
    pub shard_count: Option<i32>,
    /// Enable graph relationships
    #[graphql(default)]
    pub enable_graph: Option<bool>,
    /// Force CPU backend (disables GPU acceleration, useful for graph-enabled collections)
    #[graphql(default)]
    pub force_cpu: Option<bool>,
}

/// Input for upserting a single vector
#[derive(InputObject, Clone, Debug)]
pub struct UpsertVectorInput {
    /// Vector unique identifier
    pub id: String,
    /// Vector data as array of floats
    pub data: Vec<f32>,
    /// Optional payload as JSON
    #[graphql(default)]
    pub payload: Option<async_graphql::Json<JsonValue>>,
}

/// Input for batch upserting vectors
#[derive(InputObject, Clone, Debug)]
pub struct UpsertVectorsInput {
    /// Collection name
    pub collection: String,
    /// Vectors to upsert
    pub vectors: Vec<UpsertVectorInput>,
}

/// Input for semantic search
#[derive(InputObject, Clone, Debug)]
pub struct SearchInput {
    /// Collection to search in
    pub collection: String,
    /// Query vector
    pub vector: Vec<f32>,
    /// Number of results (default: 10)
    #[graphql(default_with = "10")]
    pub limit: i32,
    /// Include vector data in results
    #[graphql(default)]
    pub include_vectors: Option<bool>,
    /// Payload filter as JSON
    #[graphql(default)]
    pub filter: Option<async_graphql::Json<JsonValue>>,
    /// Minimum score threshold
    #[graphql(default)]
    pub score_threshold: Option<f32>,
}

/// Input for scrolling through vectors
#[derive(InputObject, Clone, Debug)]
pub struct ScrollInput {
    /// Collection name
    pub collection: String,
    /// Maximum number of items to return
    #[graphql(default_with = "100")]
    pub limit: i32,
    /// Cursor for pagination
    #[graphql(default)]
    pub cursor: Option<String>,
    /// Include vector data
    #[graphql(default)]
    pub include_vectors: Option<bool>,
    /// Payload filter as JSON
    #[graphql(default)]
    pub filter: Option<async_graphql::Json<JsonValue>>,
}

/// Result of a mutation operation
#[derive(Clone, Debug)]
pub struct MutationResult {
    /// Whether the operation succeeded
    pub is_success: bool,
    /// Optional message
    pub message: Option<String>,
    /// Number of affected items (for batch operations)
    pub affected_count: Option<i32>,
}

#[Object]
impl MutationResult {
    /// Whether the operation succeeded
    async fn success(&self) -> bool {
        self.is_success
    }

    /// Optional message
    async fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Number of affected items (for batch operations)
    async fn affected_count(&self) -> Option<i32> {
        self.affected_count
    }
}

impl MutationResult {
    pub fn ok() -> Self {
        Self {
            is_success: true,
            message: None,
            affected_count: None,
        }
    }

    pub fn ok_with_message(msg: impl Into<String>) -> Self {
        Self {
            is_success: true,
            message: Some(msg.into()),
            affected_count: None,
        }
    }

    pub fn ok_with_count(count: i32) -> Self {
        Self {
            is_success: true,
            message: None,
            affected_count: Some(count),
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            is_success: false,
            message: Some(msg.into()),
            affected_count: None,
        }
    }
}

// =============================================================================
// GRAPH INPUT TYPES
// =============================================================================

/// Input for creating a graph edge
#[derive(InputObject, Clone, Debug)]
pub struct CreateEdgeInput {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Relationship type
    pub relationship_type: GqlRelationshipType,
    /// Edge weight (default: 1.0)
    #[graphql(default_with = "1.0")]
    pub weight: f32,
}

/// Input for discovering edges based on similarity
#[derive(InputObject, Clone, Debug)]
pub struct DiscoverEdgesInput {
    /// Similarity threshold for creating edges (0.0 to 1.0)
    #[graphql(default_with = "0.7")]
    pub similarity_threshold: f32,
    /// Maximum edges per node
    #[graphql(default_with = "10")]
    pub max_edges_per_node: i32,
}

// =============================================================================
// WORKSPACE TYPES
// =============================================================================

/// Workspace directory entry
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlWorkspace {
    /// Directory path
    pub path: String,
    /// Associated collection name
    pub collection_name: String,
    /// Whether the workspace is currently indexed
    pub indexed: bool,
}

/// Workspace configuration
#[derive(SimpleObject, Clone, Debug)]
pub struct GqlWorkspaceConfig {
    /// Global settings as JSON
    pub global_settings: async_graphql::Json<JsonValue>,
    /// Projects configuration as JSON
    pub projects: async_graphql::Json<JsonValue>,
}

/// Input for adding a workspace
#[derive(InputObject, Clone, Debug)]
pub struct AddWorkspaceInput {
    /// Directory path to index
    pub path: String,
    /// Collection name to store vectors
    pub collection_name: String,
}
