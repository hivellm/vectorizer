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

#[test]
fn test_discover_edges_for_node() {
    let store = VectorStore::new();
    let config = create_test_collection_config();

    store
        .create_collection("test_discover", config.clone())
        .unwrap();

    let collection = store.get_collection("test_discover").unwrap();
    let CollectionType::Cpu(cpu_collection) = &*collection else {
        panic!("Expected CPU collection")
    };

    let graph = cpu_collection.get_graph().unwrap();

    // Insert some vectors with similar data
    let mut vec1 = vec![1.0; 128];
    vec1[0] = 0.9;
    let mut vec2 = vec![1.0; 128];
    vec2[0] = 0.95; // Very similar to vec1
    let vec3 = vec![0.0; 128]; // Very different

    cpu_collection
        .insert(vectorizer::models::Vector {
            id: "vec1".to_string(),
            data: vec1.clone(),
            sparse: None,
            payload: None,
        })
        .unwrap();

    cpu_collection
        .insert(vectorizer::models::Vector {
            id: "vec2".to_string(),
            data: vec2.clone(),
            sparse: None,
            payload: None,
        })
        .unwrap();

    cpu_collection
        .insert(vectorizer::models::Vector {
            id: "vec3".to_string(),
            data: vec3.clone(),
            sparse: None,
            payload: None,
        })
        .unwrap();

    // Discover edges for vec1
    let config = vectorizer::models::AutoRelationshipConfig {
        similarity_threshold: 0.7,
        max_per_node: 10,
        enabled_types: vec!["SIMILAR_TO".to_string()],
    };

    let edges_created = vectorizer::db::graph_relationship_discovery::discover_edges_for_node(
        graph.as_ref(),
        "vec1",
        cpu_collection,
        &config,
    )
    .unwrap();

    // Should create at least one edge to vec2 (similar)
    assert!(edges_created > 0);
    assert_eq!(graph.edge_count(), edges_created);
}

#[test]
fn test_discover_edges_for_collection() {
    let store = VectorStore::new();
    let config = create_test_collection_config();

    store
        .create_collection("test_discover_collection", config.clone())
        .unwrap();

    let collection = store.get_collection("test_discover_collection").unwrap();
    let CollectionType::Cpu(cpu_collection) = &*collection else {
        panic!("Expected CPU collection")
    };

    let graph = cpu_collection.get_graph().unwrap();

    // Insert multiple similar vectors
    for i in 0..5 {
        let mut vec_data = vec![1.0; 128];
        vec_data[0] = 0.9 + (i as f32 * 0.01); // Slightly different but similar

        cpu_collection
            .insert(vectorizer::models::Vector {
                id: format!("vec{i}"),
                data: vec_data,
                sparse: None,
                payload: None,
            })
            .unwrap();
    }

    // Discover edges for entire collection
    let config = vectorizer::models::AutoRelationshipConfig {
        similarity_threshold: 0.7,
        max_per_node: 10,
        enabled_types: vec!["SIMILAR_TO".to_string()],
    };

    let stats = vectorizer::db::graph_relationship_discovery::discover_edges_for_collection(
        graph.as_ref(),
        cpu_collection,
        &config,
    )
    .unwrap();

    // Should process all nodes
    assert_eq!(stats.total_nodes, 5);
    assert_eq!(stats.nodes_processed, 5);
    // Should create edges for nodes with similar vectors
    assert!(stats.total_edges_created > 0);
    assert!(stats.nodes_with_edges > 0);
}

#[test]
fn test_graph_persistence_save_and_load() {
    use tempfile::TempDir;

    // Create temporary directory for test
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    // Create graph and add nodes/edges
    let graph = Graph::new("test_persistence".to_string());

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

    // Save graph
    assert!(graph.save_to_file(data_dir).is_ok());

    // Load graph
    let loaded_graph = Graph::load_from_file("test_persistence", data_dir).unwrap();

    // Verify nodes and edges were loaded
    assert_eq!(loaded_graph.node_count(), 2);
    assert_eq!(loaded_graph.edge_count(), 1);

    // Verify nodes exist
    assert!(loaded_graph.get_node("node1").is_some());
    assert!(loaded_graph.get_node("node2").is_some());

    // Verify edge exists
    let neighbors = loaded_graph.get_neighbors("node1", None).unwrap();
    assert_eq!(neighbors.len(), 1);
}

#[test]
fn test_graph_persistence_missing_file() {
    use tempfile::TempDir;

    // Create temporary directory (empty, no graph file)
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    // Load graph from non-existent file should return empty graph
    let loaded_graph = Graph::load_from_file("nonexistent", data_dir).unwrap();

    // Should return empty graph, not error
    assert_eq!(loaded_graph.node_count(), 0);
    assert_eq!(loaded_graph.edge_count(), 0);
}

#[test]
fn test_graph_persistence_corrupted_file() {
    use std::fs;
    use std::io::Write;

    use tempfile::TempDir;

    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();
    let graph_path = data_dir.join("test_corrupted_graph.json");

    // Write corrupted JSON
    let mut file = fs::File::create(&graph_path).unwrap();
    file.write_all(b"{ invalid json }").unwrap();
    drop(file);

    // Load should handle corrupted file gracefully
    let loaded_graph = Graph::load_from_file("test_corrupted", data_dir).unwrap();

    // Should return empty graph, not crash
    assert_eq!(loaded_graph.node_count(), 0);
    assert_eq!(loaded_graph.edge_count(), 0);
}

#[test]
#[ignore] // Performance test - run explicitly with `cargo test -- --ignored`
fn test_graph_discovery_performance_large_collection() {
    use std::time::Instant;

    let store = VectorStore::new();
    let collection_name = "test_perf_discovery";

    // Create collection with graph enabled
    store
        .create_collection(collection_name, create_test_collection_config())
        .unwrap();

    // Insert a large number of vectors (1000 vectors)
    let num_vectors = 1000;
    let mut vectors = Vec::with_capacity(num_vectors);

    for i in 0..num_vectors {
        let payload_data = serde_json::json!({
            "index": i,
            "batch": i / 100
        });
        vectors.push(vectorizer::models::Vector {
            id: format!("vec_{i}"),
            data: vec![(i as f32) / 1000.0; 128], // Varying vectors
            sparse: None,
            payload: Some(vectorizer::models::Payload::new(payload_data)),
        });
    }

    // Insert vectors in batches
    let batch_size = 100;
    for chunk in vectors.chunks(batch_size) {
        store.insert(collection_name, chunk.to_vec()).unwrap();
    }

    // Get collection and graph
    let collection = store.get_collection(collection_name).unwrap();
    let graph = match &*collection {
        CollectionType::Cpu(c) => c.get_graph().unwrap(),
        _ => panic!("Expected CPU collection"),
    };

    // Verify we have nodes
    assert_eq!(graph.node_count(), num_vectors);

    // Test discovery performance
    let config = vectorizer::models::AutoRelationshipConfig {
        similarity_threshold: 0.7,
        max_per_node: 10,
        enabled_types: vec!["SIMILAR_TO".to_string()],
    };

    let start = Instant::now();

    // Discover edges for a subset of nodes (first 100) to keep test reasonable
    let CollectionType::Cpu(cpu_collection) = &*collection else {
        panic!("Expected CPU collection")
    };

    let mut total_edges = 0;
    for i in 0..100.min(num_vectors) {
        let node_id = format!("vec_{i}");
        if let Ok(edges_created) =
            vectorizer::db::graph_relationship_discovery::discover_edges_for_node(
                graph.as_ref(),
                &node_id,
                cpu_collection,
                &config,
            )
        {
            total_edges += edges_created;
        }
    }

    let duration = start.elapsed();

    // Performance assertions
    // Should complete in reasonable time (less than 30 seconds for 100 nodes)
    assert!(
        duration.as_secs() < 30,
        "Discovery took too long: {duration:?} for 100 nodes"
    );

    // Should have created some edges
    assert!(
        total_edges > 0,
        "Should have created at least some edges, got {total_edges}"
    );

    // Verify edges were actually added to graph
    let final_edge_count = graph.edge_count();
    assert!(
        final_edge_count >= total_edges,
        "Graph should have at least {total_edges} edges, got {final_edge_count}"
    );

    println!("Performance test: Discovered {total_edges} edges for 100 nodes in {duration:?}");
}
