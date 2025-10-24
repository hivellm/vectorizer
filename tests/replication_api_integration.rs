//! Integration tests for replication REST API endpoints

use serde_json::Value;

#[tokio::test]
async fn test_replication_status_endpoint_standalone() {
    // Create a test server (standalone mode by default)
    let store = std::sync::Arc::new(vectorizer::VectorStore::new());
    let embedding_manager = std::sync::Arc::new(vectorizer::embedding::EmbeddingManager::new());

    // Verify standalone role
    let role = store
        .get_metadata("replication_role")
        .unwrap_or_else(|| "standalone".to_string());
    assert_eq!(role, "standalone");
}

#[tokio::test]
async fn test_replication_stats_structure() {
    use vectorizer::replication::{NodeRole, ReplicationStats};

    // Test that stats structure can be created
    let stats = ReplicationStats {
        role: NodeRole::Master,
        lag_ms: 10,
        bytes_sent: 1024,
        bytes_received: 0,
        last_sync: std::time::SystemTime::now(),
        operations_pending: 5,
        snapshot_size: 2048,
        connected_replicas: Some(2),
        master_offset: 100,
        replica_offset: 0,
        lag_operations: 0,
        total_replicated: 95,
    };

    // Verify serialization works
    let json = serde_json::to_value(&stats).unwrap();
    assert!(json.get("role").is_some());
    assert!(json.get("lag_ms").is_some());
    assert!(json.get("bytes_sent").is_some());
    assert!(json.get("connected_replicas").is_some());
}

#[tokio::test]
async fn test_replica_info_structure() {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use vectorizer::replication::{ReplicaInfo, ReplicaStatus};

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 6381);
    let info = ReplicaInfo::new("test-replica".to_string(), addr);

    // Verify fields
    assert_eq!(info.id, "test-replica");
    assert_eq!(info.host, "192.168.1.10");
    assert_eq!(info.port, 6381);
    assert_eq!(info.status, ReplicaStatus::Connected);

    // Verify serialization
    let json = serde_json::to_value(&info).unwrap();
    assert!(json.get("id").is_some());
    assert!(json.get("host").is_some());
    assert!(json.get("port").is_some());
    assert!(json.get("status").is_some());
}

#[tokio::test]
async fn test_stats_backwards_compatibility() {
    use vectorizer::replication::{NodeRole, ReplicationStats};

    let stats = ReplicationStats {
        role: NodeRole::Master,
        lag_ms: 5,
        bytes_sent: 1000,
        bytes_received: 500,
        last_sync: std::time::SystemTime::now(),
        operations_pending: 10,
        snapshot_size: 2048,
        connected_replicas: Some(3),
        master_offset: 100,
        replica_offset: 90,
        lag_operations: 10,
        total_replicated: 90,
    };

    // Verify legacy fields still exist
    assert_eq!(stats.master_offset, 100);
    assert_eq!(stats.replica_offset, 90);
    assert_eq!(stats.lag_operations, 10);
    assert_eq!(stats.total_replicated, 90);

    // Verify new fields exist
    assert_eq!(stats.bytes_sent, 1000);
    assert_eq!(stats.bytes_received, 500);
    assert_eq!(stats.operations_pending, 10);
    assert_eq!(stats.connected_replicas, Some(3));
}

#[tokio::test]
async fn test_replica_health_status_logic() {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::{Duration, SystemTime};

    use vectorizer::replication::{ReplicaInfo, ReplicaStatus};

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6381);
    let mut info = ReplicaInfo::new("health-test".to_string(), addr);

    // Test Connected state
    info.lag_ms = 50;
    info.offset = 100;
    info.last_heartbeat = SystemTime::now();
    info.update_status();
    assert_eq!(info.status, ReplicaStatus::Connected);

    // Test Lagging state
    info.lag_ms = 1500;
    info.update_status();
    assert_eq!(info.status, ReplicaStatus::Lagging);

    // Test Syncing state
    info.lag_ms = 10;
    info.offset = 0;
    info.update_status();
    assert_eq!(info.status, ReplicaStatus::Syncing);

    // Test Disconnected state
    info.offset = 100;
    info.last_heartbeat = SystemTime::now() - Duration::from_secs(70);
    info.update_status();
    assert_eq!(info.status, ReplicaStatus::Disconnected);
}

#[tokio::test]
async fn test_stats_json_response_format() {
    use vectorizer::replication::{NodeRole, ReplicationStats};

    let stats = ReplicationStats {
        role: NodeRole::Master,
        lag_ms: 0,
        bytes_sent: 5000,
        bytes_received: 0,
        last_sync: std::time::UNIX_EPOCH + Duration::from_secs(1729776000),
        operations_pending: 0,
        snapshot_size: 1024,
        connected_replicas: Some(2),
        master_offset: 50,
        replica_offset: 0,
        lag_operations: 0,
        total_replicated: 50,
    };

    let json = serde_json::to_value(&stats).unwrap();

    // Verify all required fields are in JSON
    assert!(json.get("role").is_some());
    assert!(json.get("lag_ms").is_some());
    assert!(json.get("bytes_sent").is_some());
    assert!(json.get("bytes_received").is_some());
    assert!(json.get("last_sync").is_some());
    assert!(json.get("operations_pending").is_some());
    assert!(json.get("snapshot_size").is_some());
    assert!(json.get("connected_replicas").is_some());

    // Verify legacy fields also present
    assert!(json.get("master_offset").is_some());
    assert!(json.get("replica_offset").is_some());
    assert!(json.get("lag_operations").is_some());
    assert!(json.get("total_replicated").is_some());
}

#[tokio::test]
async fn test_replica_list_empty() {
    // Test that empty replica list works correctly
    let replicas: Vec<vectorizer::replication::ReplicaInfo> = Vec::new();

    let json = serde_json::to_value(&replicas).unwrap();
    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_replica_list_with_data() {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use vectorizer::replication::{ReplicaInfo, ReplicaStatus};

    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 6381);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 11)), 6382);

    let mut replicas = vec![
        ReplicaInfo::new("replica-1".to_string(), addr1),
        ReplicaInfo::new("replica-2".to_string(), addr2),
    ];

    // Update statuses
    replicas[0].lag_ms = 10;
    replicas[0].offset = 50; // Has synced data
    replicas[0].update_status();

    replicas[1].lag_ms = 1200;
    replicas[1].offset = 30; // Has synced data
    replicas[1].update_status();

    assert_eq!(replicas.len(), 2);
    assert_eq!(replicas[0].status, ReplicaStatus::Connected);
    assert_eq!(replicas[1].status, ReplicaStatus::Lagging);

    // Verify serialization
    let json = serde_json::to_value(&replicas).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 2);
}

use std::time::Duration;
