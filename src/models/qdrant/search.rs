//! Qdrant search models
//!
//! This module provides data structures for Qdrant search operations,
//! including search requests, responses, and scoring functions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchRequest {
    /// Query vector
    pub vector: Vec<f32>,
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Limit
    pub limit: Option<u32>,
    /// Offset
    pub offset: Option<u32>,
    /// With payload
    pub with_payload: Option<bool>,
    /// With vector
    pub with_vector: Option<bool>,
    /// Score threshold
    pub score_threshold: Option<f32>,
    /// Using
    pub using: Option<String>,
    /// Lookup from
    pub lookup_from: Option<QdrantLookupLocation>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchResponse {
    /// Search results
    pub result: Vec<QdrantScoredPoint>,
}

/// Scored point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScoredPoint {
    /// Point ID
    pub id: QdrantPointId,
    /// Point vector
    pub vector: Option<QdrantVector>,
    /// Point payload
    pub payload: Option<HashMap<String, QdrantValue>>,
    /// Score
    pub score: f32,
}

/// Filter condition (re-exported from point module)
pub use super::point::QdrantFilter;
/// Point ID (re-exported from point module)
pub use super::point::QdrantPointId;
/// Payload value (re-exported from point module)
pub use super::point::QdrantValue;
/// Vector data (re-exported from point module)
pub use super::point::QdrantVector;

/// Lookup location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantLookupLocation {
    /// Collection name
    pub collection: String,
    /// Vector name
    pub vector: Option<String>,
}

/// Recommend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRecommendRequest {
    /// Positive examples
    pub positive: Vec<QdrantPointId>,
    /// Negative examples
    pub negative: Option<Vec<QdrantPointId>>,
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Limit
    pub limit: Option<u32>,
    /// Offset
    pub offset: Option<u32>,
    /// With payload
    pub with_payload: Option<bool>,
    /// With vector
    pub with_vector: Option<bool>,
    /// Score threshold
    pub score_threshold: Option<f32>,
    /// Using
    pub using: Option<String>,
    /// Lookup from
    pub lookup_from: Option<QdrantLookupLocation>,
    /// Strategy
    pub strategy: Option<QdrantRecommendStrategy>,
}

/// Recommend strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantRecommendStrategy {
    /// Average vector
    #[serde(rename = "average_vector")]
    AverageVector,
    /// Best score
    #[serde(rename = "best_score")]
    BestScore,
}

/// Recommend response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRecommendResponse {
    /// Recommend results
    pub result: Vec<QdrantScoredPoint>,
}

/// Scroll request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollRequest {
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Limit
    pub limit: Option<u32>,
    /// Offset
    pub offset: Option<QdrantPointId>,
    /// With payload
    pub with_payload: Option<bool>,
    /// With vector
    pub with_vector: Option<bool>,
    /// Order by
    pub order_by: Option<QdrantOrderBy>,
}

/// Order by
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantOrderBy {
    /// Key
    pub key: String,
    /// Direction
    pub direction: Option<QdrantDirection>,
}

/// Direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantDirection {
    /// Ascending
    #[serde(rename = "asc")]
    Asc,
    /// Descending
    #[serde(rename = "desc")]
    Desc,
}

/// Scroll response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollResponse {
    /// Scroll results
    pub result: QdrantScrollResult,
}

/// Scroll result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollResult {
    /// Points
    pub points: Vec<QdrantPointStruct>,
    /// Next page offset
    pub next_page_offset: Option<QdrantPointId>,
}

/// Point struct (re-exported from point module)
pub use super::point::QdrantPointStruct;

/// Count request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountRequest {
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Exact count
    pub exact: Option<bool>,
}

/// Count response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountResponse {
    /// Count result
    pub result: QdrantCountResult,
}

/// Count result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountResult {
    /// Number of points
    pub count: u64,
}

/// Batch search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchSearchRequest {
    /// Search requests
    pub searches: Vec<QdrantSearchRequest>,
}

/// Batch search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchSearchResponse {
    /// Search results
    pub result: Vec<QdrantSearchResponse>,
}

/// Batch recommend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchRecommendRequest {
    /// Recommend requests
    pub searches: Vec<QdrantRecommendRequest>,
}

/// Batch recommend response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchRecommendResponse {
    /// Recommend results
    pub result: Vec<QdrantRecommendResponse>,
}

// =============================================================================
// Query API Types (Qdrant 1.7+)
// =============================================================================

/// Query request - unified query interface for Qdrant 1.7+
/// Supports: nearest, recommend, discover, context, and fusion queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantQueryRequest {
    /// Query type - can be a vector, point ID, or complex query
    pub query: Option<QdrantQuery>,
    /// Prefetch configuration for multi-stage retrieval
    pub prefetch: Option<Vec<QdrantPrefetch>>,
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Limit
    pub limit: Option<u32>,
    /// Offset
    pub offset: Option<u32>,
    /// With payload
    pub with_payload: Option<QdrantWithPayload>,
    /// With vector
    pub with_vector: Option<QdrantWithVector>,
    /// Score threshold
    pub score_threshold: Option<f32>,
    /// Using - name of the vector to use
    pub using: Option<String>,
    /// Lookup from - collection to lookup vectors from
    pub lookup_from: Option<QdrantLookupLocation>,
    /// Parameters for search
    pub params: Option<QdrantSearchParams>,
}

/// Query type - can be vector, point ID, or complex query
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantQuery {
    /// Nearest neighbor search with vector
    Vector(Vec<f32>),
    /// Nearest neighbor search with point ID
    PointId(QdrantPointId),
    /// Complex query object
    Complex(Box<QdrantComplexQuery>),
}

/// Complex query types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QdrantComplexQuery {
    /// Nearest neighbor search
    Nearest(QdrantNearestQuery),
    /// Recommend based on positive/negative examples
    Recommend(QdrantRecommendQuery),
    /// Discover query
    Discover(QdrantDiscoverQuery),
    /// Context query
    Context(QdrantContextQuery),
    /// Fusion of multiple queries
    Fusion(QdrantFusionQuery),
    /// Order by payload field
    OrderBy(QdrantOrderByQuery),
}

/// Nearest query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantNearestQuery {
    /// Vector or point ID to search near
    pub nearest: QdrantVectorInput,
}

/// Recommend query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRecommendQuery {
    /// Positive examples
    pub positive: Option<Vec<QdrantVectorInput>>,
    /// Negative examples
    pub negative: Option<Vec<QdrantVectorInput>>,
    /// Strategy
    pub strategy: Option<QdrantRecommendStrategy>,
}

/// Discover query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDiscoverQuery {
    /// Target to discover around
    pub target: QdrantVectorInput,
    /// Context pairs for discovery
    pub context: Vec<QdrantContextPair>,
}

/// Context query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantContextQuery {
    /// Context pairs
    pub context: Vec<QdrantContextPair>,
}

/// Fusion query - combines multiple query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantFusionQuery {
    /// Fusion method
    pub fusion: QdrantFusionMethod,
}

/// Order by query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantOrderByQuery {
    /// Key to order by
    pub key: String,
    /// Direction
    pub direction: Option<QdrantDirection>,
    /// Start from value
    pub start_from: Option<serde_json::Value>,
}

/// Fusion methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QdrantFusionMethod {
    /// Reciprocal Rank Fusion
    Rrf,
    /// Distribution-Based Score Fusion
    Dbsf,
}

/// Context pair for discover/context queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantContextPair {
    /// Positive example
    pub positive: QdrantVectorInput,
    /// Negative example
    pub negative: QdrantVectorInput,
}

/// Vector input - can be a vector or point ID
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantVectorInput {
    /// Dense vector
    Vector(Vec<f32>),
    /// Point ID reference
    PointId(QdrantPointId),
}

/// Prefetch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPrefetch {
    /// Query for prefetch
    pub query: Option<QdrantQuery>,
    /// Filter for prefetch
    pub filter: Option<QdrantFilter>,
    /// Limit for prefetch
    pub limit: Option<u32>,
    /// Using vector name
    pub using: Option<String>,
    /// Nested prefetch
    pub prefetch: Option<Vec<QdrantPrefetch>>,
    /// Score threshold
    pub score_threshold: Option<f32>,
    /// Search params
    pub params: Option<QdrantSearchParams>,
}

/// Search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchParams {
    /// HNSW ef parameter
    pub hnsw_ef: Option<u32>,
    /// Exact search (no approximation)
    pub exact: Option<bool>,
    /// Quantization parameters
    pub quantization: Option<QdrantQuantizationSearchParams>,
    /// Indexed only search
    pub indexed_only: Option<bool>,
}

/// Quantization search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantQuantizationSearchParams {
    /// Ignore quantization
    pub ignore: Option<bool>,
    /// Rescore
    pub rescore: Option<bool>,
    /// Oversampling
    pub oversampling: Option<f32>,
}

/// With payload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantWithPayload {
    /// Boolean flag
    Bool(bool),
    /// Include specific fields
    Include(Vec<String>),
    /// Selector
    Selector(QdrantPayloadSelector),
}

/// Payload selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPayloadSelector {
    /// Include fields
    pub include: Option<Vec<String>>,
    /// Exclude fields
    pub exclude: Option<Vec<String>>,
}

/// With vector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantWithVector {
    /// Boolean flag
    Bool(bool),
    /// Include specific vectors
    Include(Vec<String>),
}

/// Query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantQueryResponse {
    /// Query results
    pub result: Vec<QdrantScoredPoint>,
}

/// Batch query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchQueryRequest {
    /// Query requests
    pub searches: Vec<QdrantQueryRequest>,
}

/// Batch query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchQueryResponse {
    /// Query results
    pub result: Vec<QdrantQueryResponse>,
}

/// Query groups request - group results by payload field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantQueryGroupsRequest {
    /// Query type
    pub query: Option<QdrantQuery>,
    /// Prefetch configuration
    pub prefetch: Option<Vec<QdrantPrefetch>>,
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Group by field
    pub group_by: String,
    /// Group size
    pub group_size: Option<u32>,
    /// Limit (number of groups)
    pub limit: Option<u32>,
    /// With payload
    pub with_payload: Option<QdrantWithPayload>,
    /// With vector
    pub with_vector: Option<QdrantWithVector>,
    /// Score threshold
    pub score_threshold: Option<f32>,
    /// Using
    pub using: Option<String>,
    /// Lookup from
    pub lookup_from: Option<QdrantLookupLocation>,
    /// With lookup
    pub with_lookup: Option<QdrantWithLookup>,
}

/// With lookup configuration for group queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantWithLookup {
    /// Collection name only
    Collection(String),
    /// Full lookup config
    Config(QdrantLookupConfig),
}

/// Lookup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantLookupConfig {
    /// Collection name
    pub collection: String,
    /// With payload
    pub with_payload: Option<QdrantWithPayload>,
    /// With vector
    pub with_vector: Option<QdrantWithVector>,
}

/// Query groups response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantQueryGroupsResponse {
    /// Groups results
    pub result: QdrantGroupsResult,
}

/// Groups result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantGroupsResult {
    /// Groups
    pub groups: Vec<QdrantPointGroup>,
}

/// Point group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPointGroup {
    /// Group ID/value
    pub id: serde_json::Value,
    /// Points in this group
    pub hits: Vec<QdrantScoredPoint>,
    /// Optional lookup result
    pub lookup: Option<QdrantPointStruct>,
}

// =============================================================================
// Search Groups API (POST /collections/{name}/points/search/groups)
// =============================================================================

/// Search groups request - groups search results by payload field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchGroupsRequest {
    /// Query vector
    pub vector: Vec<f32>,
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Group by field
    pub group_by: String,
    /// Group size (points per group)
    pub group_size: Option<u32>,
    /// Limit (number of groups)
    pub limit: Option<u32>,
    /// With payload
    pub with_payload: Option<QdrantWithPayload>,
    /// With vector
    pub with_vector: Option<QdrantWithVector>,
    /// Score threshold
    pub score_threshold: Option<f32>,
    /// Using - name of the vector to use
    pub using: Option<String>,
    /// Lookup from
    pub lookup_from: Option<QdrantLookupLocation>,
    /// With lookup - fetch additional data from another collection
    pub with_lookup: Option<QdrantWithLookup>,
    /// Search params
    pub params: Option<QdrantSearchParams>,
}

/// Search groups response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchGroupsResponse {
    /// Groups results
    pub result: QdrantGroupsResult,
}

// =============================================================================
// Search Matrix API (POST /collections/{name}/points/search/matrix/pairs|offsets)
// =============================================================================

/// Search matrix pairs request - compute pairwise distances between sample points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchMatrixPairsRequest {
    /// Sample size - number of random points to sample
    pub sample: Option<u32>,
    /// Limit - max pairs to return
    pub limit: Option<u32>,
    /// Filter to apply when sampling
    pub filter: Option<QdrantFilter>,
    /// Using - name of the vector to use
    pub using: Option<String>,
}

/// Search matrix pairs response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchMatrixPairsResponse {
    /// Matrix pairs results
    pub result: QdrantMatrixPairsResult,
}

/// Matrix pairs result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantMatrixPairsResult {
    /// Pairs of point IDs with their distances
    pub pairs: Vec<QdrantDistancePair>,
}

/// Distance pair - two points and their distance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDistancePair {
    /// First point ID
    pub a: QdrantPointId,
    /// Second point ID
    pub b: QdrantPointId,
    /// Distance/score between points
    pub score: f32,
}

/// Search matrix offsets request - compute distances as offset matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchMatrixOffsetsRequest {
    /// Sample size - number of random points to sample
    pub sample: Option<u32>,
    /// Limit - max entries to return
    pub limit: Option<u32>,
    /// Filter to apply when sampling
    pub filter: Option<QdrantFilter>,
    /// Using - name of the vector to use
    pub using: Option<String>,
}

/// Search matrix offsets response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchMatrixOffsetsResponse {
    /// Matrix offsets results
    pub result: QdrantMatrixOffsetsResult,
}

/// Matrix offsets result - sparse representation of distance matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantMatrixOffsetsResult {
    /// Point IDs in order (row/column indices map to these IDs)
    pub ids: Vec<QdrantPointId>,
    /// Offsets into scores array for each row
    pub offsets: Vec<u64>,
    /// Flat array of scores
    pub scores: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_request_serialization() {
        let request = QdrantSearchRequest {
            vector: vec![0.1, 0.2, 0.3, 0.4],
            filter: None,
            limit: Some(10),
            offset: None,
            with_payload: Some(true),
            with_vector: Some(false),
            score_threshold: Some(0.5),
            using: Some("dense".to_string()),
            lookup_from: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("0.1"));
        assert!(json.contains("dense"));

        let deserialized: QdrantSearchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.limit, Some(10));
        assert_eq!(deserialized.using, Some("dense".to_string()));
    }

    #[test]
    fn test_scored_point_serialization() {
        let scored = QdrantScoredPoint {
            id: QdrantPointId::Uuid("point-1".to_string()),
            vector: Some(QdrantVector::Dense(vec![0.1, 0.2])),
            payload: None,
            score: 0.95,
        };

        let json = serde_json::to_string(&scored).unwrap();
        assert!(json.contains("point-1"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_query_request_with_vector() {
        let request = QdrantQueryRequest {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2, 0.3])),
            prefetch: None,
            filter: None,
            limit: Some(10),
            offset: None,
            with_payload: None,
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("0.1"));

        let deserialized: QdrantQueryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.limit, Some(10));
    }

    #[test]
    fn test_query_request_with_prefetch() {
        let prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2])),
            filter: None,
            limit: Some(100),
            using: None,
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let request = QdrantQueryRequest {
            query: None,
            prefetch: Some(vec![prefetch]),
            filter: None,
            limit: Some(10),
            offset: None,
            with_payload: None,
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("prefetch"));
    }

    #[test]
    fn test_fusion_query() {
        let fusion = QdrantFusionQuery {
            fusion: QdrantFusionMethod::Rrf,
        };

        let json = serde_json::to_string(&fusion).unwrap();
        assert!(json.contains("rrf"));

        // Test DBSF
        let dbsf = QdrantFusionQuery {
            fusion: QdrantFusionMethod::Dbsf,
        };
        let json = serde_json::to_string(&dbsf).unwrap();
        assert!(json.contains("dbsf"));
    }

    #[test]
    fn test_search_groups_request() {
        let request = QdrantSearchGroupsRequest {
            vector: vec![0.1, 0.2, 0.3],
            filter: None,
            group_by: "category".to_string(),
            group_size: Some(3),
            limit: Some(5),
            with_payload: Some(QdrantWithPayload::Bool(true)),
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            with_lookup: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("category"));
        assert!(json.contains("group_size"));
    }

    #[test]
    fn test_matrix_pairs_request() {
        let request = QdrantSearchMatrixPairsRequest {
            sample: Some(100),
            limit: Some(500),
            filter: None,
            using: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("500"));
    }

    #[test]
    fn test_distance_pair() {
        let pair = QdrantDistancePair {
            a: QdrantPointId::Uuid("point-a".to_string()),
            b: QdrantPointId::Uuid("point-b".to_string()),
            score: 0.87,
        };

        let json = serde_json::to_string(&pair).unwrap();
        assert!(json.contains("point-a"));
        assert!(json.contains("point-b"));
        assert!(json.contains("0.87"));
    }

    #[test]
    fn test_query_groups_request() {
        let request = QdrantQueryGroupsRequest {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2])),
            prefetch: None,
            filter: None,
            group_by: "category".to_string(),
            group_size: Some(3),
            limit: Some(10),
            with_payload: None,
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            with_lookup: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("group_by"));
        assert!(json.contains("category"));
    }

    #[test]
    fn test_recommend_strategy() {
        let avg = QdrantRecommendStrategy::AverageVector;
        let json = serde_json::to_string(&avg).unwrap();
        assert!(json.contains("average_vector"));

        let best = QdrantRecommendStrategy::BestScore;
        let json = serde_json::to_string(&best).unwrap();
        assert!(json.contains("best_score"));
    }

    #[test]
    fn test_with_payload_variants() {
        // Bool variant
        let bool_payload = QdrantWithPayload::Bool(true);
        let json = serde_json::to_string(&bool_payload).unwrap();
        assert_eq!(json, "true");

        // Include variant
        let include = QdrantWithPayload::Include(vec!["field1".to_string(), "field2".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("field1"));
    }

    #[test]
    fn test_batch_query_request() {
        let requests = vec![
            QdrantQueryRequest {
                query: Some(QdrantQuery::Vector(vec![0.1, 0.2])),
                prefetch: None,
                filter: None,
                limit: Some(5),
                offset: None,
                with_payload: None,
                with_vector: None,
                score_threshold: None,
                using: None,
                lookup_from: None,
                params: None,
            },
            QdrantQueryRequest {
                query: Some(QdrantQuery::Vector(vec![0.3, 0.4])),
                prefetch: None,
                filter: None,
                limit: Some(5),
                offset: None,
                with_payload: None,
                with_vector: None,
                score_threshold: None,
                using: None,
                lookup_from: None,
                params: None,
            },
        ];

        let batch = QdrantBatchQueryRequest { searches: requests };
        let json = serde_json::to_string(&batch).unwrap();
        assert!(json.contains("searches"));
    }

    #[test]
    fn test_search_params() {
        let params = QdrantSearchParams {
            hnsw_ef: Some(128),
            exact: Some(false),
            quantization: Some(QdrantQuantizationSearchParams {
                ignore: Some(false),
                rescore: Some(true),
                oversampling: Some(2.0),
            }),
            indexed_only: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("hnsw_ef"));
        assert!(json.contains("128"));
        assert!(json.contains("rescore"));
    }
}
