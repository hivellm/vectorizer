mod search_model_tests {
    use std::collections::HashMap;

    use vectorizer::models::qdrant::point::QdrantPointId;
    use vectorizer::models::qdrant::search::{
        QdrantDirection, QdrantFusionMethod, QdrantPrefetch, QdrantQuery, QdrantRecommendStrategy,
        QdrantScoredPoint, QdrantSearchParams, QdrantWithPayload, QdrantWithVector,
    };

    #[test]
    fn test_recommend_strategy_serialization() {
        let avg = QdrantRecommendStrategy::AverageVector;
        let best = QdrantRecommendStrategy::BestScore;

        let avg_json = serde_json::to_string(&avg).unwrap();
        let best_json = serde_json::to_string(&best).unwrap();

        assert!(avg_json.contains("average_vector"));
        assert!(best_json.contains("best_score"));
    }

    #[test]
    fn test_direction_serialization() {
        let asc = QdrantDirection::Asc;
        let desc = QdrantDirection::Desc;

        let asc_json = serde_json::to_string(&asc).unwrap();
        let desc_json = serde_json::to_string(&desc).unwrap();

        assert!(asc_json.contains("asc"));
        assert!(desc_json.contains("desc"));
    }

    #[test]
    fn test_fusion_method_serialization() {
        let rrf = QdrantFusionMethod::Rrf;
        let dbsf = QdrantFusionMethod::Dbsf;

        let rrf_json = serde_json::to_string(&rrf).unwrap();
        let dbsf_json = serde_json::to_string(&dbsf).unwrap();

        assert!(rrf_json.contains("rrf"));
        assert!(dbsf_json.contains("dbsf"));
    }

    #[test]
    fn test_with_payload_bool() {
        let with = QdrantWithPayload::Bool(true);
        let json = serde_json::to_string(&with).unwrap();
        assert_eq!(json, "true");
    }

    #[test]
    fn test_with_payload_include() {
        let include = QdrantWithPayload::Include(vec!["field1".to_string(), "field2".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("field1"));
        assert!(json.contains("field2"));
    }

    #[test]
    fn test_with_vector_bool() {
        let with = QdrantWithVector::Bool(false);
        let json = serde_json::to_string(&with).unwrap();
        assert_eq!(json, "false");
    }

    #[test]
    fn test_with_vector_include() {
        let include = QdrantWithVector::Include(vec!["dense".to_string(), "sparse".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("dense"));
        assert!(json.contains("sparse"));
    }

    #[test]
    fn test_search_params() {
        let params = QdrantSearchParams {
            hnsw_ef: Some(128),
            exact: Some(false),
            quantization: None,
            indexed_only: Some(true),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"hnsw_ef\":128"));
        assert!(json.contains("\"exact\":false"));
        assert!(json.contains("\"indexed_only\":true"));
    }

    #[test]
    fn test_scored_point() {
        let point = QdrantScoredPoint {
            id: QdrantPointId::Numeric(123),
            vector: None,
            payload: Some(HashMap::new()),
            score: 0.95,
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("123"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_query_vector() {
        let query = QdrantQuery::Vector(vec![0.1, 0.2, 0.3]);
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("0.1"));
        assert!(json.contains("0.2"));
        assert!(json.contains("0.3"));
    }

    #[test]
    fn test_query_point_id() {
        let query = QdrantQuery::PointId(QdrantPointId::Uuid("abc-123".to_string()));
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("abc-123"));
    }

    #[test]
    fn test_prefetch_basic() {
        let prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.5, 0.5])),
            filter: None,
            limit: Some(100),
            using: Some("dense".to_string()),
            prefetch: None,
            score_threshold: Some(0.5),
            params: None,
        };

        let json = serde_json::to_string(&prefetch).unwrap();
        assert!(json.contains("0.5"));
        assert!(json.contains("\"limit\":100"));
        assert!(json.contains("dense"));
    }

    #[test]
    fn test_prefetch_nested() {
        // Test nested prefetch (prefetch within prefetch)
        let inner_prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2, 0.3])),
            filter: None,
            limit: Some(50),
            using: Some("sparse".to_string()),
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let outer_prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.4, 0.5, 0.6])),
            filter: None,
            limit: Some(100),
            using: Some("dense".to_string()),
            prefetch: Some(vec![inner_prefetch]),
            score_threshold: Some(0.7),
            params: None,
        };

        let json = serde_json::to_string(&outer_prefetch).unwrap();
        assert!(json.contains("prefetch"));
        assert!(json.contains("sparse"));
        assert!(json.contains("dense"));
        assert!(json.contains("\"limit\":50"));
        assert!(json.contains("\"limit\":100"));
    }

    #[test]
    fn test_prefetch_with_params() {
        let params = QdrantSearchParams {
            hnsw_ef: Some(128),
            exact: Some(false),
            quantization: None,
            indexed_only: Some(true),
        };

        let prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2])),
            filter: None,
            limit: Some(200),
            using: None,
            prefetch: None,
            score_threshold: None,
            params: Some(params),
        };

        let json = serde_json::to_string(&prefetch).unwrap();
        assert!(json.contains("hnsw_ef"));
        assert!(json.contains("128"));
        assert!(json.contains("indexed_only"));
    }

    #[test]
    fn test_prefetch_multiple_stages() {
        // Test multiple prefetch stages (array of prefetch)
        let prefetch1 = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2])),
            filter: None,
            limit: Some(100),
            using: Some("dense".to_string()),
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let prefetch2 = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.3, 0.4])),
            filter: None,
            limit: Some(50),
            using: Some("sparse".to_string()),
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let prefetch_array = vec![prefetch1, prefetch2];
        let json = serde_json::to_string(&prefetch_array).unwrap();

        assert!(json.contains("dense"));
        assert!(json.contains("sparse"));
        // Should be an array
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }
}

// ============================================================================
// Unit Tests for Query API Models
// ============================================================================

#[cfg(test)]
mod query_api_model_tests {
    use vectorizer::models::qdrant::point::QdrantPointId;
    use vectorizer::models::qdrant::search::{
        QdrantBatchQueryRequest, QdrantPrefetch, QdrantQuery, QdrantQueryGroupsRequest,
        QdrantQueryRequest, QdrantWithPayload, QdrantWithVector,
    };

    #[test]
    fn test_query_request_simple_vector() {
        let request = QdrantQueryRequest {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2, 0.3, 0.4])),
            prefetch: None,
            filter: None,
            limit: Some(10),
            offset: None,
            with_payload: Some(QdrantWithPayload::Bool(true)),
            with_vector: Some(QdrantWithVector::Bool(false)),
            score_threshold: Some(0.5),
            using: None,
            lookup_from: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("0.1"));
        assert!(json.contains("\"limit\":10"));
        assert!(json.contains("\"score_threshold\":0.5"));
    }

    #[test]
    fn test_query_request_with_point_id() {
        let request = QdrantQueryRequest {
            query: Some(QdrantQuery::PointId(QdrantPointId::Numeric(123))),
            prefetch: None,
            filter: None,
            limit: Some(5),
            offset: None,
            with_payload: None,
            with_vector: None,
            score_threshold: None,
            using: Some("default".to_string()),
            lookup_from: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("123"));
        assert!(json.contains("\"using\":\"default\""));
    }

    #[test]
    fn test_query_request_with_prefetch() {
        let prefetch = QdrantPrefetch {
            query: Some(QdrantQuery::Vector(vec![0.5, 0.5])),
            filter: None,
            limit: Some(100),
            using: Some("dense".to_string()),
            prefetch: None,
            score_threshold: None,
            params: None,
        };

        let request = QdrantQueryRequest {
            query: None, // No query when using prefetch with fusion
            prefetch: Some(vec![prefetch]),
            filter: None,
            limit: Some(10),
            offset: None,
            with_payload: Some(QdrantWithPayload::Bool(true)),
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("prefetch"));
        assert!(json.contains("dense"));
    }

    #[test]
    fn test_batch_query_request() {
        let query1 = QdrantQueryRequest {
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
        };

        let query2 = QdrantQueryRequest {
            query: Some(QdrantQuery::Vector(vec![0.3, 0.4])),
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

        let batch = QdrantBatchQueryRequest {
            searches: vec![query1, query2],
        };

        let json = serde_json::to_string(&batch).unwrap();
        assert!(json.contains("searches"));
        assert!(json.contains("0.1"));
        assert!(json.contains("0.3"));
    }

    #[test]
    fn test_query_groups_request() {
        let request = QdrantQueryGroupsRequest {
            query: Some(QdrantQuery::Vector(vec![0.1, 0.2, 0.3])),
            prefetch: None,
            filter: None,
            group_by: "category".to_string(),
            group_size: Some(3),
            limit: Some(10),
            with_payload: Some(QdrantWithPayload::Bool(true)),
            with_vector: None,
            score_threshold: None,
            using: None,
            lookup_from: None,
            with_lookup: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"group_by\":\"category\""));
        assert!(json.contains("\"group_size\":3"));
        assert!(json.contains("\"limit\":10"));
    }

    #[test]
    fn test_with_payload_selector() {
        let include =
            QdrantWithPayload::Include(vec!["name".to_string(), "description".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("name"));
        assert!(json.contains("description"));
    }

    #[test]
    fn test_with_vector_selector() {
        let include = QdrantWithVector::Include(vec!["dense".to_string(), "sparse".to_string()]);
        let json = serde_json::to_string(&include).unwrap();
        assert!(json.contains("dense"));
        assert!(json.contains("sparse"));
    }
}
