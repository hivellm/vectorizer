//! Integration tests for graph functionality

use vectorizer::db::graph::{Edge, Graph, Node, RelationshipType};
use vectorizer::db::{CollectionType, VectorStore};
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, GraphConfig, HnswConfig,
    QuantizationConfig,
};

fn create_test_collection_config() -> CollectionConfig {
    CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            seed: Some(42),
        },
        quantization: QuantizationConfig::SQ { bits: 8 },
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        graph: Some(GraphConfig {
            enabled: true,
            auto_relationship: Default::default(),
        }),
    }
}

#[test]
fn test_graph_creation() {
    let graph = Graph::new("test_collection".to_string());
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_graph_add_node() {
    let graph = Graph::new("test_collection".to_string());
    let node = Node::new("node1".to_string(), "document".to_string());

    assert!(graph.add_node(node.clone()).is_ok());
    assert_eq!(graph.node_count(), 1);

    let retrieved = graph.get_node("node1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "node1");
}

#[test]
fn test_graph_add_edge() {
    let graph = Graph::new("test_collection".to_string());

    let node1 = Node::new("node1".to_string(), "document".to_string());
    let node2 = Node::new("node2".to_string(), "document".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();

    let edge = Edge::new(
        "edge1".to_string(),
        "node1".to_string(),
        "node2".to_string(),
        RelationshipType::SimilarTo,
        0.85,
    );

    assert!(graph.add_edge(edge.clone()).is_ok());
    assert_eq!(graph.edge_count(), 1);

    // Verify edge exists by checking neighbors
    let neighbors = graph.get_neighbors("node1", None).unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].1.id, "edge1");
}

#[test]
fn test_graph_get_neighbors() {
    let graph = Graph::new("test_collection".to_string());

    let node1 = Node::new("node1".to_string(), "document".to_string());
    let node2 = Node::new("node2".to_string(), "document".to_string());
    let node3 = Node::new("node3".to_string(), "document".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph.add_node(node3).unwrap();

    let edge1 = Edge::new(
        "edge1".to_string(),
        "node1".to_string(),
        "node2".to_string(),
        RelationshipType::SimilarTo,
        0.85,
    );
    let edge2 = Edge::new(
        "edge2".to_string(),
        "node1".to_string(),
        "node3".to_string(),
        RelationshipType::References,
        0.90,
    );

    graph.add_edge(edge1).unwrap();
    graph.add_edge(edge2).unwrap();

    let neighbors = graph.get_neighbors("node1", None).unwrap();
    assert_eq!(neighbors.len(), 2);
}

#[test]
fn test_graph_find_related() {
    let graph = Graph::new("test_collection".to_string());

    let node1 = Node::new("node1".to_string(), "document".to_string());
    let node2 = Node::new("node2".to_string(), "document".to_string());
    let node3 = Node::new("node3".to_string(), "document".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph.add_node(node3).unwrap();

    let edge1 = Edge::new(
        "edge1".to_string(),
        "node1".to_string(),
        "node2".to_string(),
        RelationshipType::SimilarTo,
        0.85,
    );
    let edge2 = Edge::new(
        "edge2".to_string(),
        "node2".to_string(),
        "node3".to_string(),
        RelationshipType::SimilarTo,
        0.80,
    );

    graph.add_edge(edge1).unwrap();
    graph.add_edge(edge2).unwrap();

    let related = graph.find_related("node1", 2, None).unwrap();
    assert!(related.len() >= 2); // node2 (1 hop) and node3 (2 hops)
}

#[test]
fn test_graph_find_path() {
    let graph = Graph::new("test_collection".to_string());

    let node1 = Node::new("node1".to_string(), "document".to_string());
    let node2 = Node::new("node2".to_string(), "document".to_string());
    let node3 = Node::new("node3".to_string(), "document".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph.add_node(node3).unwrap();

    let edge1 = Edge::new(
        "edge1".to_string(),
        "node1".to_string(),
        "node2".to_string(),
        RelationshipType::SimilarTo,
        0.85,
    );
    let edge2 = Edge::new(
        "edge2".to_string(),
        "node2".to_string(),
        "node3".to_string(),
        RelationshipType::SimilarTo,
        0.80,
    );

    graph.add_edge(edge1).unwrap();
    graph.add_edge(edge2).unwrap();

    let path = graph.find_path("node1", "node3").unwrap();
    assert_eq!(path.len(), 3); // node1 -> node2 -> node3
    assert_eq!(path[0].id, "node1");
    assert_eq!(path[1].id, "node2");
    assert_eq!(path[2].id, "node3");
}

#[test]
fn test_graph_remove_node() {
    let graph = Graph::new("test_collection".to_string());

    let node1 = Node::new("node1".to_string(), "document".to_string());
    let node2 = Node::new("node2".to_string(), "document".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();

    let edge = Edge::new(
        "edge1".to_string(),
        "node1".to_string(),
        "node2".to_string(),
        RelationshipType::SimilarTo,
        0.85,
    );

    graph.add_edge(edge).unwrap();
    assert_eq!(graph.edge_count(), 1);

    graph.remove_node("node1").unwrap();
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0); // Edge should be removed too
}

#[test]
fn test_graph_remove_edge() {
    let graph = Graph::new("test_collection".to_string());

    let node1 = Node::new("node1".to_string(), "document".to_string());
    let node2 = Node::new("node2".to_string(), "document".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();

    let edge = Edge::new(
        "edge1".to_string(),
        "node1".to_string(),
        "node2".to_string(),
        RelationshipType::SimilarTo,
        0.85,
    );

    graph.add_edge(edge).unwrap();
    assert_eq!(graph.edge_count(), 1);

    graph.remove_edge("edge1").unwrap();
    assert_eq!(graph.edge_count(), 0);
    assert_eq!(graph.node_count(), 2); // Nodes should remain
}

#[test]
fn test_collection_with_graph() {
    let store = VectorStore::new();
    let config = create_test_collection_config();

    store
        .create_collection("test_graph_collection", config.clone())
        .unwrap();

    let collection = store.get_collection("test_graph_collection").unwrap();
    match &*collection {
        CollectionType::Cpu(c) => {
            let graph = c.get_graph();
            assert!(graph.is_some(), "Graph should be enabled for collection");
        }
        _ => panic!("Expected CPU collection"),
    }
}

#[test]
fn test_graph_get_all_nodes() {
    let graph = Graph::new("test_collection".to_string());

    for i in 1..=5 {
        let node = Node::new(format!("node{i}"), "document".to_string());
        graph.add_node(node).unwrap();
    }

    let all_nodes = graph.get_all_nodes();
    assert_eq!(all_nodes.len(), 5);
}

#[test]
fn test_graph_get_connected_components() {
    let graph = Graph::new("test_collection".to_string());

    // Create two disconnected components
    let node1 = Node::new("node1".to_string(), "document".to_string());
    let node2 = Node::new("node2".to_string(), "document".to_string());
    let node3 = Node::new("node3".to_string(), "document".to_string());
    let node4 = Node::new("node4".to_string(), "document".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph.add_node(node3).unwrap();
    graph.add_node(node4).unwrap();

    // Component 1: node1 <-> node2
    let edge1 = Edge::new(
        "edge1".to_string(),
        "node1".to_string(),
        "node2".to_string(),
        RelationshipType::SimilarTo,
        0.85,
    );

    // Component 2: node3 <-> node4
    let edge2 = Edge::new(
        "edge2".to_string(),
        "node3".to_string(),
        "node4".to_string(),
        RelationshipType::SimilarTo,
        0.80,
    );

    graph.add_edge(edge1).unwrap();
    graph.add_edge(edge2).unwrap();

    // Test that we can find paths between connected nodes
    let path1 = graph.find_path("node1", "node2");
    assert!(path1.is_ok());

    let path2 = graph.find_path("node3", "node4");
    assert!(path2.is_ok());

    // But no path between disconnected components
    let path3 = graph.find_path("node1", "node3");
    assert!(path3.is_err());
}
