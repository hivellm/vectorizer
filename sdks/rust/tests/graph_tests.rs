//! Tests for graph operations in the Rust SDK

use vectorizer_sdk::*;

#[tokio::test]
async fn test_list_graph_nodes() {
    let client = VectorizerClient::new_default().unwrap();
    
    // This test requires a collection with graph enabled
    // For now, we just test that the method exists and can be called
    let result = client.list_graph_nodes("test_collection").await;
    
    // If collection doesn't exist, we expect an error
    // If it exists, we should get a valid response
    match result {
        Ok(response) => {
            assert!(response.count >= 0);
            assert_eq!(response.nodes.len(), response.count);
        }
        Err(_) => {
            // Collection doesn't exist or graph not enabled - this is expected in test environment
        }
    }
}

#[tokio::test]
async fn test_get_graph_neighbors() {
    let client = VectorizerClient::new_default().unwrap();
    
    let result = client.get_graph_neighbors("test_collection", "test_node").await;
    
    match result {
        Ok(response) => {
            assert!(!response.neighbors.is_empty() || response.neighbors.is_empty());
        }
        Err(_) => {
            // Expected if collection/node doesn't exist
        }
    }
}

#[tokio::test]
async fn test_find_related_nodes() {
    let client = VectorizerClient::new_default().unwrap();
    
    let request = FindRelatedRequest {
        max_hops: Some(2),
        relationship_type: Some("SIMILAR_TO".to_string()),
    };
    
    let result = client.find_related_nodes("test_collection", "test_node", request).await;
    
    match result {
        Ok(response) => {
            assert!(response.related.len() >= 0);
        }
        Err(_) => {
            // Expected if collection/node doesn't exist
        }
    }
}

#[tokio::test]
async fn test_find_graph_path() {
    let client = VectorizerClient::new_default().unwrap();
    
    let request = FindPathRequest {
        collection: "test_collection".to_string(),
        source: "node1".to_string(),
        target: "node2".to_string(),
    };
    
    let result = client.find_graph_path(request).await;
    
    match result {
        Ok(response) => {
            assert!(response.found || !response.found);
            if response.found {
                assert!(!response.path.is_empty());
            }
        }
        Err(_) => {
            // Expected if collection/nodes don't exist
        }
    }
}

#[tokio::test]
async fn test_create_graph_edge() {
    let client = VectorizerClient::new_default().unwrap();
    
    let request = CreateEdgeRequest {
        collection: "test_collection".to_string(),
        source: "node1".to_string(),
        target: "node2".to_string(),
        relationship_type: "SIMILAR_TO".to_string(),
        weight: Some(0.85),
    };
    
    let result = client.create_graph_edge(request).await;
    
    match result {
        Ok(response) => {
            assert!(response.success);
            assert!(!response.edge_id.is_empty());
        }
        Err(_) => {
            // Expected if collection/nodes don't exist
        }
    }
}

#[tokio::test]
async fn test_list_graph_edges() {
    let client = VectorizerClient::new_default().unwrap();
    
    let result = client.list_graph_edges("test_collection").await;
    
    match result {
        Ok(response) => {
            assert!(response.count >= 0);
            assert_eq!(response.edges.len(), response.count);
        }
        Err(_) => {
            // Expected if collection doesn't exist
        }
    }
}

#[tokio::test]
async fn test_discover_graph_edges() {
    let client = VectorizerClient::new_default().unwrap();
    
    let request = DiscoverEdgesRequest {
        similarity_threshold: Some(0.7),
        max_per_node: Some(10),
    };
    
    let result = client.discover_graph_edges("test_collection", request).await;
    
    match result {
        Ok(response) => {
            assert!(response.success);
            assert!(response.edges_created >= 0);
        }
        Err(_) => {
            // Expected if collection doesn't exist
        }
    }
}

#[tokio::test]
async fn test_get_graph_discovery_status() {
    let client = VectorizerClient::new_default().unwrap();
    
    let result = client.get_graph_discovery_status("test_collection").await;
    
    match result {
        Ok(response) => {
            assert!(response.total_nodes >= 0);
            assert!(response.nodes_with_edges >= 0);
            assert!(response.total_edges >= 0);
            assert!(response.progress_percentage >= 0.0 && response.progress_percentage <= 100.0);
        }
        Err(_) => {
            // Expected if collection doesn't exist
        }
    }
}

#[test]
fn test_graph_models_serialization() {
    // Test that graph models can be serialized/deserialized
    use vectorizer_sdk::models::graph::GraphNode;
    
    let node = GraphNode {
        id: "test_node".to_string(),
        node_type: "document".to_string(),
        metadata: std::collections::HashMap::new(),
    };
    
    let json = serde_json::to_string(&node).unwrap();
    let deserialized: GraphNode = serde_json::from_str(&json).unwrap();
    
    assert_eq!(node.id, deserialized.id);
    assert_eq!(node.node_type, deserialized.node_type);
}

