//! Integration tests for Graph REST API endpoints
//!
//! These tests verify:
//! - Graph REST endpoints work correctly
//! - Request/response formats
//! - Error handling
//! - Graph operations via HTTP
//!
//! Note: These tests require a running server or use direct API calls.
//! For now, we test the graph functionality through the VectorStore directly.

use tracing::info;
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, GraphConfig, HnswConfig, QuantizationConfig,
};

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
        encryption: None,
    }
}

#[test]
fn test_graph_rest_api_functionality() {
    // Test that graph functionality works through VectorStore
    // This verifies the underlying functionality that REST endpoints use

    let store = VectorStore::new();

    // Create collection with graph enabled (CPU-only for deterministic tests)
    store
        .create_collection_cpu_only("test_graph_rest", create_test_collection_config())
        .unwrap();

    // Insert vectors to create nodes
    store
        .insert(
            "test_graph_rest",
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

    // Verify graph exists and has nodes
    let collection = store.get_collection("test_graph_rest").unwrap();
    match &*collection {
        vectorizer::db::CollectionType::Cpu(c) => {
            let graph = c.get_graph();
            assert!(graph.is_some(), "Graph should be enabled");
            let graph = graph.unwrap();
            assert!(
                graph.node_count() >= 2,
                "Graph should have at least 2 nodes"
            );
        }
        _ => panic!("Expected CPU collection"),
    }
}

#[test]
fn test_graph_discovery_creates_edges_and_api_returns_them() {
    // Test that discovery creates edges and they are returned by the API

    let store = VectorStore::new();
    let collection_name = "test_discovery_edges_api";

    // Create collection with graph enabled (CPU-only for deterministic tests)
    store
        .create_collection_cpu_only(collection_name, create_test_collection_config())
        .unwrap();

    // Insert vectors with varying similarity
    store
        .insert(
            collection_name,
            vec![
                vectorizer::models::Vector {
                    id: "doc1".to_string(),
                    data: vec![1.0; 128], // Similar vectors
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "doc2".to_string(),
                    data: vec![1.0; 128], // Similar to doc1
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "doc3".to_string(),
                    data: vec![0.1; 128], // Different vector
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    // Get graph and verify initial state
    let collection = store.get_collection(collection_name).unwrap();
    let graph = match &*collection {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };

    let initial_edge_count = graph.edge_count();
    assert_eq!(initial_edge_count, 0, "Initially should have no edges");

    // Discover edges for the collection
    let config = vectorizer::models::AutoRelationshipConfig {
        similarity_threshold: 0.5, // Lower threshold to ensure edges are created
        max_per_node: 10,
        enabled_types: vec!["SIMILAR_TO".to_string()],
    };

    let vectorizer::db::CollectionType::Cpu(cpu_collection) = &*collection else {
        panic!("Expected CPU collection")
    };

    // Discover edges for entire collection
    let stats = vectorizer::db::graph_relationship_discovery::discover_edges_for_collection(
        graph.as_ref(),
        cpu_collection,
        &config,
    )
    .expect("Discovery should succeed");

    // Verify edges were created
    assert!(
        stats.total_edges_created > 0,
        "Should have created at least some edges. Created: {}",
        stats.total_edges_created
    );

    // Verify edges are in the graph
    let collection_after = store.get_collection(collection_name).unwrap();
    let graph_after = match &*collection_after {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };

    let final_edge_count = graph_after.edge_count();
    assert!(
        final_edge_count > 0,
        "Graph should have edges after discovery. Edge count: {final_edge_count}"
    );

    // Verify edges can be retrieved via get_all_edges (simulating API endpoint)
    let all_edges = graph_after.get_all_edges();
    assert_eq!(
        all_edges.len(),
        final_edge_count,
        "get_all_edges should return all edges. Expected: {}, Got: {}",
        final_edge_count,
        all_edges.len()
    );

    // Verify specific edges exist (doc1 and doc2 should be similar)
    let doc1_neighbors = graph_after.get_neighbors("doc1", None).unwrap_or_default();
    let has_doc2_as_neighbor = doc1_neighbors
        .iter()
        .any(|(node, edge)| edge.target == "doc2" || node.id == "doc2");

    assert!(
        has_doc2_as_neighbor,
        "doc1 should have doc2 as neighbor after discovery. Neighbors: {:?}",
        doc1_neighbors
            .iter()
            .map(|(n, e)| (n.id.clone(), e.target.clone()))
            .collect::<Vec<_>>()
    );

    // Verify edge details - check for edge in either direction since discovery order is non-deterministic
    let has_edge_between_doc1_and_doc2 = all_edges.iter().any(|e| {
        (e.source == "doc1" && e.target == "doc2") || (e.source == "doc2" && e.target == "doc1")
    });
    assert!(
        has_edge_between_doc1_and_doc2,
        "Should have edge between doc1 and doc2 (in either direction). Edges: {:?}",
        all_edges
            .iter()
            .map(|e| format!("{} -> {}", e.source, e.target))
            .collect::<Vec<_>>()
    );

    info!(
        "✅ Discovery created {} edges, API can retrieve {} edges",
        stats.total_edges_created,
        all_edges.len()
    );
}

#[test]
fn test_graph_discovery_via_api_and_list_edges_returns_them() {
    // Test that after calling discovery via API simulation, list_edges returns the edges
    // This simulates the actual API flow

    use std::sync::Arc;

    let store = Arc::new(VectorStore::new());
    let collection_name = "test_api_discovery_flow";

    // Create collection with graph enabled (CPU-only for deterministic tests)
    store
        .create_collection_cpu_only(collection_name, create_test_collection_config())
        .unwrap();

    // Insert vectors
    store
        .insert(
            collection_name,
            vec![
                vectorizer::models::Vector {
                    id: "api_doc1".to_string(),
                    data: vec![1.0; 128],
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "api_doc2".to_string(),
                    data: vec![1.0; 128], // Similar to api_doc1
                    sparse: None,
                    payload: None,
                },
                vectorizer::models::Vector {
                    id: "api_doc3".to_string(),
                    data: vec![0.1; 128], // Different
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    // Verify initial state - no edges
    let collection_before = store.get_collection(collection_name).unwrap();
    let graph_before = match &*collection_before {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };
    assert_eq!(graph_before.edge_count(), 0, "Should start with no edges");

    // We can't easily call async functions from sync test, so we'll use the underlying function
    // But let's verify the graph directly after discovery
    let collection_for_discovery = store.get_collection(collection_name).unwrap();
    let graph_for_discovery = match &*collection_for_discovery {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };

    let config = vectorizer::models::AutoRelationshipConfig {
        similarity_threshold: 0.5,
        max_per_node: 10,
        enabled_types: vec!["SIMILAR_TO".to_string()],
    };

    let vectorizer::db::CollectionType::Cpu(cpu_collection) = &*collection_for_discovery else {
        panic!("Expected CPU collection")
    };

    // Discover edges (simulating API call)
    let stats = vectorizer::db::graph_relationship_discovery::discover_edges_for_collection(
        graph_for_discovery.as_ref(),
        cpu_collection,
        &config,
    )
    .expect("Discovery should succeed");

    assert!(
        stats.total_edges_created > 0,
        "Discovery should have created edges. Created: {}",
        stats.total_edges_created
    );

    // Now verify that list_edges would return them (simulating API call)
    // Get collection again to simulate a new API request
    let collection_after = store.get_collection(collection_name).unwrap();
    let graph_after = match &*collection_after {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };

    // Simulate what list_edges does
    let edges = graph_after.get_all_edges();

    assert!(
        !edges.is_empty(),
        "list_edges should return edges after discovery. Got {} edges",
        edges.len()
    );

    assert_eq!(
        edges.len(),
        graph_after.edge_count(),
        "get_all_edges should return same count as edge_count(). Got {} vs {}",
        edges.len(),
        graph_after.edge_count()
    );

    // Verify specific edges - check for edge in either direction since discovery order is non-deterministic
    // When processing api_doc1, it should find api_doc2 as similar and vice versa
    // The actual direction depends on the order nodes are processed (HashMap iteration order)
    let has_edge_between_doc1_and_doc2 = edges.iter().any(|e| {
        (e.source == "api_doc1" && e.target == "api_doc2")
            || (e.source == "api_doc2" && e.target == "api_doc1")
    });
    assert!(
        has_edge_between_doc1_and_doc2,
        "Should have edge between api_doc1 and api_doc2 (in either direction). Edges: {:?}",
        edges
            .iter()
            .map(|e| format!("{} -> {}", e.source, e.target))
            .collect::<Vec<_>>()
    );

    info!(
        "✅ API flow test: Discovery created {} edges, list_edges returns {} edges",
        stats.total_edges_created,
        edges.len()
    );
}
