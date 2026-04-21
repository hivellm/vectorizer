//! Extracted unit tests (phase3 test-extraction).
//!
//! Wired from `src/models/qdrant/search.rs` via the `#[path]` attribute.

#![allow(clippy::unwrap_used, clippy::expect_used)]

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
