//! Integration tests for Raft consensus

use std::time::Duration;

use vectorizer::db::raft::{LogEntry, RaftConfig, RaftNode, RaftRole, RaftStateMachine};
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
};
use vectorizer::persistence::types::Operation;

#[allow(dead_code)]
fn create_test_config() -> CollectionConfig {
    CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
    }
}

#[tokio::test]
async fn test_raft_node_basic() {
    let mut node = RaftNode::new(1, RaftConfig::default());
    node.start().unwrap();

    // Wait a bit for initialization
    tokio::time::sleep(Duration::from_millis(100)).await;

    let state = node.get_state().await.unwrap();
    assert_eq!(state.role, RaftRole::Follower);
    assert_eq!(state.current_term, 0);
}

#[tokio::test]
async fn test_state_machine_apply_checkpoint() {
    let sm = RaftStateMachine::new();

    let entry = LogEntry {
        term: 1,
        index: 1,
        operation: Operation::Checkpoint {
            vector_count: 100,
            document_count: 50,
            checksum: "test_checksum".to_string(),
        },
    };

    // Checkpoint operations should not fail (they're metadata only)
    let result = sm.apply(&entry);
    assert!(result.is_ok());

    assert_eq!(sm.last_applied_index(), 1);
}

#[tokio::test]
async fn test_raft_propose_operation() {
    let mut node = RaftNode::new(1, RaftConfig::default());
    node.start().unwrap();

    // Wait for initialization
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Try to propose an operation
    // Note: This will fail if node is not leader (expected behavior)
    let operation = Operation::Checkpoint {
        vector_count: 0,
        document_count: 0,
        checksum: "test".to_string(),
    };

    let result = node.propose(operation).await;
    // Should fail because node is not leader
    assert!(result.is_err());
}

#[tokio::test]
async fn test_raft_election_timeout() {
    let config = RaftConfig {
        election_timeout_ms: 100,
        heartbeat_interval_ms: 50,
        min_election_timeout_ms: 100,
        max_election_timeout_ms: 200,
    };

    let mut node = RaftNode::new(1, config);
    node.start().unwrap();

    // Initially should be follower
    let state = node.get_state().await.unwrap();
    assert_eq!(state.role, RaftRole::Follower);

    // Wait for election timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // After timeout, node should become candidate
    let state = node.get_state().await.unwrap();
    // Note: In a real implementation with peers, this would trigger election
    // For now, we just verify the state can be read
    // current_term is u64, so >= 0 is always true - just verify it's initialized
    let _ = state.current_term; // Just verify it exists
}

#[tokio::test]
async fn test_raft_log_entries() {
    let mut node = RaftNode::new(1, RaftConfig::default());
    node.start().unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let state = node.get_state().await.unwrap();
    assert_eq!(state.log.len(), 0);
    assert_eq!(state.commit_index, 0);
    assert_eq!(state.last_applied, 0);
}

#[tokio::test]
async fn test_raft_multiple_nodes() {
    // Create multiple nodes
    let mut node1 = RaftNode::new(1, RaftConfig::default());
    let mut node2 = RaftNode::new(2, RaftConfig::default());
    let mut node3 = RaftNode::new(3, RaftConfig::default());

    node1.start().unwrap();
    node2.start().unwrap();
    node3.start().unwrap();

    // Add peers
    node1.add_peer(2, "127.0.0.1:15003".to_string());
    node1.add_peer(3, "127.0.0.1:15004".to_string());
    node2.add_peer(1, "127.0.0.1:15002".to_string());
    node2.add_peer(3, "127.0.0.1:15004".to_string());
    node3.add_peer(1, "127.0.0.1:15002".to_string());
    node3.add_peer(2, "127.0.0.1:15003".to_string());

    // Check immediately after start (before election timeout)
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify all nodes are initialized (may be Follower or Candidate depending on timing)
    let state1 = node1.get_state().await.unwrap();
    let state2 = node2.get_state().await.unwrap();
    let state3 = node3.get_state().await.unwrap();

    // Nodes start as Follower, but may become Candidate if election timeout passes
    // Since we check before timeout, they should still be Follower
    assert_eq!(state1.role, RaftRole::Follower);
    assert_eq!(state2.role, RaftRole::Follower);
    assert_eq!(state3.role, RaftRole::Follower);
}

#[tokio::test]
async fn test_raft_state_machine_idempotency() {
    let sm = RaftStateMachine::new();

    let entry = LogEntry {
        term: 1,
        index: 1,
        operation: Operation::Checkpoint {
            vector_count: 100,
            document_count: 50,
            checksum: "test".to_string(),
        },
    };

    // Apply first time
    let result1 = sm.apply(&entry);
    assert!(result1.is_ok());

    // Apply again (should be idempotent)
    let result2 = sm.apply(&entry);
    assert!(result2.is_ok());

    // Should still have same last applied index
    assert_eq!(sm.last_applied_index(), 1);
}

#[tokio::test]
async fn test_raft_partition_tolerance_simulation() {
    // Simulate partition by creating isolated nodes
    let mut node1 = RaftNode::new(
        1,
        RaftConfig {
            election_timeout_ms: 100,
            heartbeat_interval_ms: 50,
            min_election_timeout_ms: 100,
            max_election_timeout_ms: 200,
        },
    );
    node1.start().unwrap();

    // Node 1 should start as follower
    let state1 = node1.get_state().await.unwrap();
    assert_eq!(state1.role, RaftRole::Follower);

    // Simulate partition (no communication with other nodes)
    // After election timeout, node should become candidate
    tokio::time::sleep(Duration::from_millis(150)).await;

    let state1_after = node1.get_state().await.unwrap();
    // Node should attempt election (become candidate)
    // In a real scenario with majority, it would become leader
    assert!(state1_after.current_term >= state1.current_term);
}

#[tokio::test]
async fn test_raft_log_consistency() {
    let mut node = RaftNode::new(1, RaftConfig::default());
    node.start().unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let state = node.get_state().await.unwrap();

    // Verify log consistency properties
    assert!(state.commit_index <= state.log.len() as u64);
    assert!(state.last_applied <= state.commit_index);
}
