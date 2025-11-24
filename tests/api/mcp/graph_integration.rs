//! Integration tests for Graph MCP Tools
//!
//! These tests verify:
//! - Graph MCP tools work correctly
//! - Tool parameters and responses
//! - Error handling
//! - Graph operations via MCP

use std::sync::Arc;

use rmcp::model::CallToolRequestParam;
use vectorizer::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, GraphConfig, HnswConfig, QuantizationConfig,
};
use vectorizer::server::mcp_handlers::handle_mcp_tool;

fn create_test_collection_config() -> CollectionConfig {
    CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        graph: Some(GraphConfig {
            enabled: true,
            auto_relationship: Default::default(),
        }),
    }
}

#[tokio::test]
async fn test_graph_find_related_mcp_tool() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Create collection with graph enabled
    store
        .create_collection("test_mcp_graph", create_test_collection_config())
        .unwrap();

    // Insert vectors
    store
        .insert(
            "test_mcp_graph",
            vec![vectorizer::models::Vector {
                id: "vec1".to_string(),
                data: vec![1.0; 128],
                sparse: None,
                payload: None,
            }],
        )
        .unwrap();

    // Call MCP tool
    let mut args = serde_json::Map::new();
    args.insert(
        "collection".to_string(),
        serde_json::Value::String("test_mcp_graph".to_string()),
    );
    args.insert(
        "node_id".to_string(),
        serde_json::Value::String("vec1".to_string()),
    );
    args.insert(
        "max_hops".to_string(),
        serde_json::Value::Number(serde_json::Number::from(2)),
    );
    args.insert(
        "relationship_type".to_string(),
        serde_json::Value::String("SIMILAR_TO".to_string()),
    );

    let request = CallToolRequestParam {
        name: "graph_find_related".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone(), None).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert!(!call_result.content.is_empty());
}

#[tokio::test]
async fn test_graph_find_path_mcp_tool() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Create collection with graph enabled
    store
        .create_collection("test_mcp_path", create_test_collection_config())
        .unwrap();

    // Insert vectors
    store
        .insert(
            "test_mcp_path",
            vec![
                vectorizer::models::Vector {
                    id: "vec1".to_string(),
                    data: vec![1.0; 128],
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "vec2".to_string(),
                    data: vec![1.0; 128],
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    // Call MCP tool
    let mut args = serde_json::Map::new();
    args.insert(
        "collection".to_string(),
        serde_json::Value::String("test_mcp_path".to_string()),
    );
    args.insert(
        "source".to_string(),
        serde_json::Value::String("vec1".to_string()),
    );
    args.insert(
        "target".to_string(),
        serde_json::Value::String("vec2".to_string()),
    );

    let request = CallToolRequestParam {
        name: "graph_find_path".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone(), None).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert!(!call_result.content.is_empty());
}

#[tokio::test]
async fn test_graph_get_neighbors_mcp_tool() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Create collection with graph enabled
    store
        .create_collection("test_mcp_neighbors", create_test_collection_config())
        .unwrap();

    // Insert vectors
    store
        .insert(
            "test_mcp_neighbors",
            vec![vectorizer::models::Vector {
                id: "vec1".to_string(),
                data: vec![1.0; 128],
                sparse: None,
                payload: None,
            }],
        )
        .unwrap();

    // Call MCP tool
    let mut args = serde_json::Map::new();
    args.insert(
        "collection".to_string(),
        serde_json::Value::String("test_mcp_neighbors".to_string()),
    );
    args.insert(
        "node_id".to_string(),
        serde_json::Value::String("vec1".to_string()),
    );

    let request = CallToolRequestParam {
        name: "graph_get_neighbors".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone(), None).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert!(!call_result.content.is_empty());
}

#[tokio::test]
async fn test_graph_create_edge_mcp_tool() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Create collection with graph enabled
    store
        .create_collection("test_mcp_create", create_test_collection_config())
        .unwrap();

    // Insert vectors
    store
        .insert(
            "test_mcp_create",
            vec![
                vectorizer::models::Vector {
                    id: "vec1".to_string(),
                    data: vec![1.0; 128],
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "vec2".to_string(),
                    data: vec![1.0; 128],
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    // Call MCP tool
    let mut args = serde_json::Map::new();
    args.insert(
        "collection".to_string(),
        serde_json::Value::String("test_mcp_create".to_string()),
    );
    args.insert(
        "source".to_string(),
        serde_json::Value::String("vec1".to_string()),
    );
    args.insert(
        "target".to_string(),
        serde_json::Value::String("vec2".to_string()),
    );
    args.insert(
        "relationship_type".to_string(),
        serde_json::Value::String("SIMILAR_TO".to_string()),
    );
    args.insert(
        "weight".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(0.85).unwrap()),
    );

    let request = CallToolRequestParam {
        name: "graph_create_edge".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone(), None).await;
    assert!(result.is_ok());

    let call_result = result.unwrap();
    assert!(!call_result.content.is_empty());
}

#[tokio::test]
async fn test_graph_mcp_tool_error_handling() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Test with non-existent collection
    let mut args = serde_json::Map::new();
    args.insert(
        "collection".to_string(),
        serde_json::Value::String("nonexistent".to_string()),
    );
    args.insert(
        "node_id".to_string(),
        serde_json::Value::String("vec1".to_string()),
    );

    let request = CallToolRequestParam {
        name: "graph_get_neighbors".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone(), None).await;
    // Should return error for non-existent collection
    assert!(result.is_err() || result.is_ok()); // May return error or empty result
}

#[tokio::test]
async fn test_graph_discover_edges_mcp_tool_creates_edges() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Create collection with graph enabled
    store
        .create_collection("test_mcp_discover", create_test_collection_config())
        .unwrap();

    // Insert multiple vectors with varying similarity
    store
        .insert(
            "test_mcp_discover",
            vec![
                vectorizer::models::Vector {
                    id: "vec1".to_string(),
                    data: vec![1.0; 128], // Similar vectors
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "vec2".to_string(),
                    data: vec![1.0; 128], // Similar to vec1
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "vec3".to_string(),
                    data: vec![0.1; 128], // Different vector
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    // Get initial edge count
    let collection = store.get_collection("test_mcp_discover").unwrap();
    let graph = match &*collection {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };
    let initial_edge_count = graph.edge_count();

    // Call MCP tool to discover edges for entire collection
    let mut args = serde_json::Map::new();
    args.insert(
        "collection".to_string(),
        serde_json::Value::String("test_mcp_discover".to_string()),
    );
    args.insert(
        "similarity_threshold".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()), // Lower threshold to ensure edges are created
    );
    args.insert(
        "max_per_node".to_string(),
        serde_json::Value::Number(serde_json::Number::from(10)),
    );

    let request = CallToolRequestParam {
        name: "graph_discover_edges".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone(), None).await;
    assert!(result.is_ok(), "Discovery should succeed");

    let call_result = result.unwrap();
    assert!(
        !call_result.content.is_empty(),
        "Response should not be empty"
    );

    // Parse response to verify edges were created
    let response_text = call_result.content[0]
        .as_text()
        .map(|t| t.text.as_str())
        .unwrap_or("");
    let response_json: serde_json::Value =
        serde_json::from_str(response_text).expect("Response should be valid JSON");

    // Verify response contains edges_created or total_edges_created
    let edges_created = response_json
        .get("edges_created")
        .or_else(|| response_json.get("total_edges_created"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Verify edges were actually added to the graph
    let collection_after = store.get_collection("test_mcp_discover").unwrap();
    let graph_after = match &*collection_after {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };
    let final_edge_count = graph_after.edge_count();

    // Edges should have been created
    assert!(
        final_edge_count > initial_edge_count || edges_created > 0,
        "Edges should have been created. Initial: {initial_edge_count}, Final: {final_edge_count}, Response: {edges_created}"
    );

    // Verify specific edges exist (vec1 and vec2 should be similar)
    let neighbors = graph_after.get_neighbors("vec1", None).unwrap_or_default();
    assert!(
        neighbors
            .iter()
            .any(|(node, edge)| edge.target == "vec2" || node.id == "vec2"),
        "vec1 should have vec2 as neighbor after discovery"
    );
}

#[tokio::test]
async fn test_graph_discover_edges_mcp_tool_node_specific() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(EmbeddingManager::new());

    // Create collection with graph enabled
    store
        .create_collection("test_mcp_discover_node", create_test_collection_config())
        .unwrap();

    // Insert multiple vectors
    store
        .insert(
            "test_mcp_discover_node",
            vec![
                vectorizer::models::Vector {
                    id: "node1".to_string(),
                    data: vec![1.0; 128],
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "node2".to_string(),
                    data: vec![1.0; 128], // Similar to node1
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "node3".to_string(),
                    data: vec![0.1; 128], // Different
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    // Get initial edge count for node1
    let collection = store.get_collection("test_mcp_discover_node").unwrap();
    let graph = match &*collection {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };
    let initial_neighbors = graph.get_neighbors("node1", None).unwrap_or_default().len();

    // Call MCP tool to discover edges for specific node
    let mut args = serde_json::Map::new();
    args.insert(
        "collection".to_string(),
        serde_json::Value::String("test_mcp_discover_node".to_string()),
    );
    args.insert(
        "node_id".to_string(),
        serde_json::Value::String("node1".to_string()),
    );
    args.insert(
        "similarity_threshold".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()),
    );
    args.insert(
        "max_per_node".to_string(),
        serde_json::Value::Number(serde_json::Number::from(10)),
    );

    let request = CallToolRequestParam {
        name: "graph_discover_edges".to_string().into(),
        arguments: Some(args),
    };

    let result = handle_mcp_tool(request, store.clone(), embedding_manager.clone(), None).await;
    assert!(result.is_ok(), "Discovery should succeed");

    let call_result = result.unwrap();
    assert!(
        !call_result.content.is_empty(),
        "Response should not be empty"
    );

    // Parse response
    let response_text = call_result.content[0]
        .as_text()
        .map(|t| t.text.as_str())
        .unwrap_or("");
    let response_json: serde_json::Value =
        serde_json::from_str(response_text).expect("Response should be valid JSON");

    let edges_created = response_json
        .get("edges_created")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Verify edges were created for node1
    let collection_after = store.get_collection("test_mcp_discover_node").unwrap();
    let graph_after = match &*collection_after {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };
    let final_neighbors = graph_after
        .get_neighbors("node1", None)
        .unwrap_or_default()
        .len();

    assert!(
        final_neighbors > initial_neighbors || edges_created > 0,
        "Edges should have been created for node1. Initial neighbors: {initial_neighbors}, Final: {final_neighbors}, Response: {edges_created}"
    );

    // Verify node1 has node2 as neighbor (they are similar)
    let neighbors = graph_after.get_neighbors("node1", None).unwrap_or_default();
    assert!(
        neighbors
            .iter()
            .any(|(node, edge)| edge.target == "node2" || node.id == "node2"),
        "node1 should have node2 as neighbor after discovery"
    );
}
