//! GraphQL API Tests
//!
//! This module contains unit and integration tests for the GraphQL API.

#[cfg(test)]
mod unit_tests {
    use crate::api::graphql::types::*;
    use crate::models::{CollectionConfig, CollectionMetadata, DistanceMetric, HnswConfig, Vector};

    // =========================================================================
    // Type Conversion Tests
    // =========================================================================

    #[test]
    fn test_distance_metric_conversion() {
        // Test From<DistanceMetric> for GqlDistanceMetric
        assert_eq!(
            GqlDistanceMetric::from(DistanceMetric::Cosine),
            GqlDistanceMetric::Cosine
        );
        assert_eq!(
            GqlDistanceMetric::from(DistanceMetric::Euclidean),
            GqlDistanceMetric::Euclidean
        );
        assert_eq!(
            GqlDistanceMetric::from(DistanceMetric::DotProduct),
            GqlDistanceMetric::DotProduct
        );

        // Test From<GqlDistanceMetric> for DistanceMetric
        assert_eq!(
            DistanceMetric::from(GqlDistanceMetric::Cosine),
            DistanceMetric::Cosine
        );
        assert_eq!(
            DistanceMetric::from(GqlDistanceMetric::Euclidean),
            DistanceMetric::Euclidean
        );
        assert_eq!(
            DistanceMetric::from(GqlDistanceMetric::DotProduct),
            DistanceMetric::DotProduct
        );
    }

    #[test]
    fn test_relationship_type_conversion() {
        use crate::db::graph::RelationshipType;

        // Test From<RelationshipType> for GqlRelationshipType
        assert_eq!(
            GqlRelationshipType::from(RelationshipType::SimilarTo),
            GqlRelationshipType::SimilarTo
        );
        assert_eq!(
            GqlRelationshipType::from(RelationshipType::References),
            GqlRelationshipType::References
        );
        assert_eq!(
            GqlRelationshipType::from(RelationshipType::Contains),
            GqlRelationshipType::Contains
        );
        assert_eq!(
            GqlRelationshipType::from(RelationshipType::DerivedFrom),
            GqlRelationshipType::DerivedFrom
        );

        // Test From<GqlRelationshipType> for RelationshipType
        assert_eq!(
            RelationshipType::from(GqlRelationshipType::SimilarTo),
            RelationshipType::SimilarTo
        );
        assert_eq!(
            RelationshipType::from(GqlRelationshipType::References),
            RelationshipType::References
        );
        assert_eq!(
            RelationshipType::from(GqlRelationshipType::Contains),
            RelationshipType::Contains
        );
        assert_eq!(
            RelationshipType::from(GqlRelationshipType::DerivedFrom),
            RelationshipType::DerivedFrom
        );
    }

    #[test]
    fn test_hnsw_config_conversion() {
        let config = HnswConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            ..Default::default()
        };

        let gql_config = GqlHnswConfig::from(&config);

        assert_eq!(gql_config.m, 16);
        assert_eq!(gql_config.ef_construction, 200);
        assert_eq!(gql_config.ef_search, 100);
    }

    #[test]
    fn test_vector_conversion() {
        let vector = Vector::new("test-id".to_string(), vec![0.1, 0.2, 0.3]);
        let gql_vector = GqlVector::from(vector);

        assert_eq!(gql_vector.id, "test-id");
        assert_eq!(gql_vector.data, vec![0.1, 0.2, 0.3]);
        assert!(gql_vector.payload.is_none());
    }

    #[test]
    fn test_vector_with_payload_conversion() {
        use crate::models::Payload;

        let payload = Payload::new(serde_json::json!({"key": "value"}));
        let vector = Vector::with_payload("test-id".to_string(), vec![0.1, 0.2, 0.3], payload);
        let gql_vector = GqlVector::from(vector);

        assert_eq!(gql_vector.id, "test-id");
        assert!(gql_vector.payload.is_some());
        let payload_value = gql_vector.payload.unwrap();
        assert_eq!(payload_value["key"], "value");
    }

    #[test]
    fn test_search_result_conversion() {
        use crate::models::SearchResult;

        let result = SearchResult {
            id: "result-1".to_string(),
            score: 0.95,
            vector: Some(vec![0.1, 0.2, 0.3]),
            payload: None,
        };

        let gql_result = GqlSearchResult::from(result);

        assert_eq!(gql_result.id, "result-1");
        assert_eq!(gql_result.score, 0.95);
        assert!(gql_result.vector.is_some());
        assert_eq!(gql_result.vector.unwrap(), vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_collection_config_conversion() {
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            ..Default::default()
        };

        let gql_config = GqlCollectionConfig::from(&config);

        assert_eq!(gql_config.dimension, 384);
        assert_eq!(gql_config.metric, GqlDistanceMetric::Cosine);
        assert!(!gql_config.sharding_enabled);
        assert!(!gql_config.graph_enabled);
    }

    #[test]
    fn test_collection_metadata_conversion() {
        let metadata = CollectionMetadata {
            name: "test-collection".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            vector_count: 100,
            document_count: 50,
            config: CollectionConfig::default(),
        };

        let gql_collection = GqlCollection::from(metadata);

        assert_eq!(gql_collection.name, "test-collection");
        assert_eq!(gql_collection.tenant_id, Some("tenant-1".to_string()));
        assert_eq!(gql_collection.vector_count, 100);
        assert_eq!(gql_collection.document_count, 50);
    }

    // =========================================================================
    // MutationResult Tests
    // =========================================================================

    #[test]
    fn test_mutation_result_ok() {
        let result = MutationResult::ok();
        assert!(result.is_success);
        assert!(result.message.is_none());
        assert!(result.affected_count.is_none());
    }

    #[test]
    fn test_mutation_result_ok_with_message() {
        let result = MutationResult::ok_with_message("Operation successful");
        assert!(result.is_success);
        assert_eq!(result.message, Some("Operation successful".to_string()));
        assert!(result.affected_count.is_none());
    }

    #[test]
    fn test_mutation_result_ok_with_count() {
        let result = MutationResult::ok_with_count(42);
        assert!(result.is_success);
        assert!(result.message.is_none());
        assert_eq!(result.affected_count, Some(42));
    }

    #[test]
    fn test_mutation_result_err() {
        let result = MutationResult::err("Something went wrong");
        assert!(!result.is_success);
        assert_eq!(result.message, Some("Something went wrong".to_string()));
        assert!(result.affected_count.is_none());
    }

    // =========================================================================
    // Graph Type Tests
    // =========================================================================

    #[test]
    fn test_graph_node_conversion() {
        use std::collections::HashMap;

        use crate::db::graph::Node;

        let mut metadata = HashMap::new();
        metadata.insert("file_path".to_string(), serde_json::json!("/path/to/file"));

        let node = Node {
            id: "node-1".to_string(),
            node_type: "document".to_string(),
            metadata,
            created_at: chrono::Utc::now(),
        };

        let gql_node = GqlNode::from(node);

        assert_eq!(gql_node.id, "node-1");
        assert_eq!(gql_node.node_type, "document");
    }

    #[test]
    fn test_graph_edge_conversion() {
        use crate::db::graph::{Edge, RelationshipType};

        let edge = Edge::new(
            "edge-1".to_string(),
            "source-node".to_string(),
            "target-node".to_string(),
            RelationshipType::SimilarTo,
            0.85,
        );

        let gql_edge = GqlEdge::from(edge);

        assert_eq!(gql_edge.id, "edge-1");
        assert_eq!(gql_edge.source, "source-node");
        assert_eq!(gql_edge.target, "target-node");
        assert_eq!(gql_edge.relationship_type, GqlRelationshipType::SimilarTo);
        assert_eq!(gql_edge.weight, 0.85);
    }

    // =========================================================================
    // Input Type Tests
    // =========================================================================

    #[test]
    fn test_create_collection_input_defaults() {
        let input = CreateCollectionInput {
            name: "test".to_string(),
            dimension: 128,
            metric: None,
            hnsw_m: None,
            hnsw_ef_construction: None,
            shard_count: None,
            enable_graph: None,
        };

        assert_eq!(input.name, "test");
        assert_eq!(input.dimension, 128);
        assert!(input.metric.is_none());
        assert!(input.hnsw_m.is_none());
        assert!(input.enable_graph.is_none());
    }

    #[test]
    fn test_upsert_vector_input() {
        let input = UpsertVectorInput {
            id: "vec-1".to_string(),
            data: vec![0.1, 0.2, 0.3],
            payload: None,
        };

        assert_eq!(input.id, "vec-1");
        assert_eq!(input.data, vec![0.1, 0.2, 0.3]);
        assert!(input.payload.is_none());
    }

    #[test]
    fn test_search_input() {
        let input = SearchInput {
            collection: "test-collection".to_string(),
            vector: vec![0.1, 0.2, 0.3],
            limit: 10,
            include_vectors: Some(true),
            filter: None,
            score_threshold: Some(0.7),
        };

        assert_eq!(input.collection, "test-collection");
        assert_eq!(input.limit, 10);
        assert_eq!(input.score_threshold, Some(0.7));
    }

    #[test]
    fn test_create_edge_input() {
        let input = CreateEdgeInput {
            source: "node-a".to_string(),
            target: "node-b".to_string(),
            relationship_type: GqlRelationshipType::References,
            weight: 0.9,
        };

        assert_eq!(input.source, "node-a");
        assert_eq!(input.target, "node-b");
        assert_eq!(input.relationship_type, GqlRelationshipType::References);
        assert_eq!(input.weight, 0.9);
    }

    #[test]
    fn test_add_workspace_input() {
        let input = AddWorkspaceInput {
            path: "/path/to/workspace".to_string(),
            collection_name: "my-collection".to_string(),
        };

        assert_eq!(input.path, "/path/to/workspace");
        assert_eq!(input.collection_name, "my-collection");
    }
}

#[cfg(test)]
mod schema_tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use crate::api::graphql::{VectorizerSchema, create_schema};
    use crate::db::VectorStore;
    use crate::embedding::EmbeddingManager;

    fn create_test_schema() -> (VectorizerSchema, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(VectorStore::new());
        let embedding_manager = Arc::new(EmbeddingManager::new());
        let start_time = std::time::Instant::now();

        let schema = create_schema(store, embedding_manager, start_time);
        (schema, temp_dir)
    }

    #[tokio::test]
    async fn test_schema_creation() {
        let (schema, _temp_dir) = create_test_schema();

        // Schema should be created successfully
        // We can verify by executing a simple introspection query
        let query = r#"
            {
                __schema {
                    queryType {
                        name
                    }
                    mutationType {
                        name
                    }
                }
            }
        "#;

        let result = schema.execute(query).await;
        assert!(
            result.errors.is_empty(),
            "Schema introspection failed: {:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_collections_query() {
        let (schema, _temp_dir) = create_test_schema();

        let query = r#"
            {
                collections {
                    name
                    vectorCount
                }
            }
        "#;

        let result = schema.execute(query).await;
        assert!(
            result.errors.is_empty(),
            "Collections query failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert!(data["collections"].is_array());
    }

    #[tokio::test]
    async fn test_stats_query() {
        let (schema, _temp_dir) = create_test_schema();

        let query = r#"
            {
                stats {
                    version
                    collectionCount
                    totalVectors
                    uptimeSeconds
                }
            }
        "#;

        let result = schema.execute(query).await;
        assert!(
            result.errors.is_empty(),
            "Stats query failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert!(data["stats"]["version"].is_string());
        assert!(data["stats"]["collectionCount"].is_number());
    }

    #[tokio::test]
    async fn test_create_collection_mutation() {
        let (schema, _temp_dir) = create_test_schema();

        let mutation = r#"
            mutation {
                createCollection(input: {
                    name: "test-graphql-collection"
                    dimension: 128
                }) {
                    name
                    config {
                        dimension
                        metric
                    }
                }
            }
        "#;

        let result = schema.execute(mutation).await;
        assert!(
            result.errors.is_empty(),
            "Create collection mutation failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert_eq!(data["createCollection"]["name"], "test-graphql-collection");
        assert_eq!(data["createCollection"]["config"]["dimension"], 128);
    }

    #[tokio::test]
    async fn test_delete_collection_mutation() {
        let (schema, _temp_dir) = create_test_schema();

        // First create a collection
        let create_mutation = r#"
            mutation {
                createCollection(input: {
                    name: "collection-to-delete"
                    dimension: 64
                }) {
                    name
                }
            }
        "#;
        schema.execute(create_mutation).await;

        // Then delete it
        let delete_mutation = r#"
            mutation {
                deleteCollection(name: "collection-to-delete") {
                    success
                    message
                }
            }
        "#;

        let result = schema.execute(delete_mutation).await;
        assert!(
            result.errors.is_empty(),
            "Delete collection mutation failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert_eq!(data["deleteCollection"]["success"], true);
    }

    #[tokio::test]
    async fn test_collection_not_found() {
        let (schema, _temp_dir) = create_test_schema();

        let query = r#"
            {
                collection(name: "non-existent-collection") {
                    name
                }
            }
        "#;

        let result = schema.execute(query).await;
        // Should return null for non-existent collection, not an error
        assert!(result.errors.is_empty());

        let data = result.data.into_json().unwrap();
        assert!(data["collection"].is_null());
    }

    #[tokio::test]
    async fn test_upsert_and_get_vector() {
        let (schema, _temp_dir) = create_test_schema();

        // Create collection first
        let create_mutation = r#"
            mutation {
                createCollection(input: {
                    name: "vector-test-collection"
                    dimension: 3
                }) {
                    name
                }
            }
        "#;
        schema.execute(create_mutation).await;

        // Upsert a vector
        let upsert_mutation = r#"
            mutation {
                upsertVector(
                    collection: "vector-test-collection"
                    input: {
                        id: "test-vector-1"
                        data: [0.1, 0.2, 0.3]
                    }
                ) {
                    id
                    dimension
                }
            }
        "#;

        let result = schema.execute(upsert_mutation).await;
        assert!(
            result.errors.is_empty(),
            "Upsert vector failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert_eq!(data["upsertVector"]["id"], "test-vector-1");
        assert_eq!(data["upsertVector"]["dimension"], 3);

        // Get the vector back
        let get_query = r#"
            {
                vector(collection: "vector-test-collection", id: "test-vector-1") {
                    id
                    data
                }
            }
        "#;

        let result = schema.execute(get_query).await;
        assert!(
            result.errors.is_empty(),
            "Get vector failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert_eq!(data["vector"]["id"], "test-vector-1");
    }

    #[tokio::test]
    async fn test_graph_stats_no_graph() {
        let (schema, _temp_dir) = create_test_schema();

        // Create collection without graph
        let create_mutation = r#"
            mutation {
                createCollection(input: {
                    name: "no-graph-collection"
                    dimension: 64
                }) {
                    name
                }
            }
        "#;
        schema.execute(create_mutation).await;

        let query = r#"
            {
                graphStats(collection: "no-graph-collection") {
                    nodeCount
                    edgeCount
                    enabled
                }
            }
        "#;

        let result = schema.execute(query).await;
        assert!(
            result.errors.is_empty(),
            "Graph stats query failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert_eq!(data["graphStats"]["enabled"], false);
        assert_eq!(data["graphStats"]["nodeCount"], 0);
        assert_eq!(data["graphStats"]["edgeCount"], 0);
    }

    #[tokio::test]
    async fn test_enable_graph_mutation() {
        let (schema, _temp_dir) = create_test_schema();

        // Create collection
        let create_mutation = r#"
            mutation {
                createCollection(input: {
                    name: "graph-enable-test"
                    dimension: 64
                }) {
                    name
                }
            }
        "#;
        schema.execute(create_mutation).await;

        // Enable graph
        let enable_mutation = r#"
            mutation {
                enableGraph(collection: "graph-enable-test") {
                    success
                    message
                }
            }
        "#;

        let result = schema.execute(enable_mutation).await;
        assert!(
            result.errors.is_empty(),
            "Enable graph mutation failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert_eq!(
            data["enableGraph"]["success"], true,
            "Enable graph failed with message: {:?}",
            data["enableGraph"]["message"]
        );

        // Verify graph is enabled
        let stats_query = r#"
            {
                graphStats(collection: "graph-enable-test") {
                    enabled
                }
            }
        "#;

        let result = schema.execute(stats_query).await;
        let data = result.data.into_json().unwrap();
        assert_eq!(data["graphStats"]["enabled"], true);
    }

    #[tokio::test]
    async fn test_workspace_config_query() {
        let (schema, _temp_dir) = create_test_schema();

        let query = r#"
            {
                workspaceConfig {
                    globalSettings
                    projects
                }
            }
        "#;

        let result = schema.execute(query).await;
        assert!(
            result.errors.is_empty(),
            "Workspace config query failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert!(data["workspaceConfig"]["globalSettings"].is_object());
    }

    #[tokio::test]
    async fn test_workspaces_query() {
        let (schema, _temp_dir) = create_test_schema();

        let query = r#"
            {
                workspaces {
                    path
                    collectionName
                    indexed
                }
            }
        "#;

        let result = schema.execute(query).await;
        assert!(
            result.errors.is_empty(),
            "Workspaces query failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert!(data["workspaces"].is_array());
    }

    #[tokio::test]
    async fn test_add_workspace_mutation() {
        let (schema, _temp_dir) = create_test_schema();

        let mutation = r#"
            mutation {
                addWorkspace(input: {
                    path: "/test/workspace"
                    collectionName: "test-collection"
                }) {
                    success
                    message
                }
            }
        "#;

        let result = schema.execute(mutation).await;
        assert!(
            result.errors.is_empty(),
            "Add workspace mutation failed: {:?}",
            result.errors
        );

        let data = result.data.into_json().unwrap();
        assert_eq!(data["addWorkspace"]["success"], true);
    }

    #[tokio::test]
    async fn test_query_depth_limit() {
        let (schema, _temp_dir) = create_test_schema();

        // Create a deeply nested query that exceeds depth limit (10)
        let deep_query = r#"
            {
                collections {
                    name
                    config {
                        hnswConfig {
                            m
                            efConstruction
                            efSearch
                        }
                    }
                }
            }
        "#;

        // This query has depth 4, should work
        let result = schema.execute(deep_query).await;
        assert!(
            result.errors.is_empty(),
            "Deep query should work within limit"
        );
    }
}

#[cfg(test)]
mod error_handling_tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use crate::api::graphql::create_schema;
    use crate::db::VectorStore;
    use crate::embedding::EmbeddingManager;

    fn create_test_schema() -> (crate::api::graphql::VectorizerSchema, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(VectorStore::new());
        let embedding_manager = Arc::new(EmbeddingManager::new());
        let start_time = std::time::Instant::now();

        let schema = create_schema(store, embedding_manager, start_time);
        (schema, temp_dir)
    }

    #[tokio::test]
    async fn test_invalid_collection_for_search() {
        let (schema, _temp_dir) = create_test_schema();

        let query = r#"
            {
                search(input: {
                    collection: "non-existent"
                    vector: [0.1, 0.2, 0.3]
                    limit: 10
                }) {
                    id
                    score
                }
            }
        "#;

        let result = schema.execute(query).await;
        // Should return an error for non-existent collection
        assert!(
            !result.errors.is_empty(),
            "Should return error for non-existent collection"
        );
    }

    #[tokio::test]
    async fn test_invalid_vector_id() {
        let (schema, _temp_dir) = create_test_schema();

        // Create collection first
        let create_mutation = r#"
            mutation {
                createCollection(input: {
                    name: "error-test-collection"
                    dimension: 3
                }) {
                    name
                }
            }
        "#;
        schema.execute(create_mutation).await;

        // Try to get non-existent vector
        let query = r#"
            {
                vector(collection: "error-test-collection", id: "non-existent-vector") {
                    id
                }
            }
        "#;

        let result = schema.execute(query).await;
        // Should return null, not error
        assert!(result.errors.is_empty());

        let data = result.data.into_json().unwrap();
        assert!(data["vector"].is_null());
    }

    #[tokio::test]
    async fn test_graph_not_enabled_error() {
        let (schema, _temp_dir) = create_test_schema();

        // Create collection without graph
        let create_mutation = r#"
            mutation {
                createCollection(input: {
                    name: "no-graph-error-test"
                    dimension: 64
                }) {
                    name
                }
            }
        "#;
        schema.execute(create_mutation).await;

        // Try to query graph nodes
        let query = r#"
            {
                graphNodes(collection: "no-graph-error-test") {
                    items {
                        id
                    }
                }
            }
        "#;

        let result = schema.execute(query).await;
        // Should return an error because graph is not enabled
        assert!(
            !result.errors.is_empty(),
            "Should return error when graph not enabled"
        );
        assert!(
            result.errors[0].message.contains("Graph not enabled"),
            "Error message should mention graph not enabled"
        );
    }

    #[tokio::test]
    async fn test_delete_non_existent_collection() {
        let (schema, _temp_dir) = create_test_schema();

        let mutation = r#"
            mutation {
                deleteCollection(name: "definitely-does-not-exist") {
                    success
                    message
                }
            }
        "#;

        let result = schema.execute(mutation).await;
        // Should return success: false, not an error
        assert!(result.errors.is_empty());

        let data = result.data.into_json().unwrap();
        assert_eq!(data["deleteCollection"]["success"], false);
    }

    #[tokio::test]
    async fn test_upsert_to_non_existent_collection() {
        let (schema, _temp_dir) = create_test_schema();

        let mutation = r#"
            mutation {
                upsertVector(
                    collection: "non-existent-collection"
                    input: {
                        id: "test-vector"
                        data: [0.1, 0.2, 0.3]
                    }
                ) {
                    id
                }
            }
        "#;

        let result = schema.execute(mutation).await;
        // Should return an error
        assert!(
            !result.errors.is_empty(),
            "Should return error for non-existent collection"
        );
    }
}
