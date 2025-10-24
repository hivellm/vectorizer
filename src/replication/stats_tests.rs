//! Tests for replication statistics and health tracking

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::replication::{NodeRole, ReplicaInfo, ReplicaStatus, ReplicationStats};

    #[test]
    fn test_replication_stats_default() {
        let stats = ReplicationStats::default();

        assert_eq!(stats.role, NodeRole::Standalone);
        assert_eq!(stats.lag_ms, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.operations_pending, 0);
        assert_eq!(stats.snapshot_size, 0);
        assert_eq!(stats.connected_replicas, None);
    }

    #[test]
    fn test_replication_stats_master() {
        let stats = ReplicationStats {
            role: NodeRole::Master,
            lag_ms: 0,
            bytes_sent: 1024000,
            bytes_received: 0,
            last_sync: SystemTime::now(),
            operations_pending: 5,
            snapshot_size: 524288,
            connected_replicas: Some(3),
            master_offset: 100,
            replica_offset: 0,
            lag_operations: 0,
            total_replicated: 95,
        };

        assert_eq!(stats.role, NodeRole::Master);
        assert_eq!(stats.bytes_sent, 1024000);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.connected_replicas, Some(3));
        assert_eq!(stats.operations_pending, 5);
        assert_eq!(stats.snapshot_size, 524288);
    }

    #[test]
    fn test_replication_stats_replica() {
        let stats = ReplicationStats {
            role: NodeRole::Replica,
            lag_ms: 15,
            bytes_sent: 0,
            bytes_received: 2048000,
            last_sync: SystemTime::now(),
            operations_pending: 0,
            snapshot_size: 0,
            connected_replicas: None,
            master_offset: 0,
            replica_offset: 85,
            lag_operations: 15,
            total_replicated: 85,
        };

        assert_eq!(stats.role, NodeRole::Replica);
        assert_eq!(stats.lag_ms, 15);
        assert_eq!(stats.bytes_received, 2048000);
        assert_eq!(stats.connected_replicas, None);
        assert_eq!(stats.replica_offset, 85);
    }

    #[test]
    fn test_replica_info_new() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6381);
        let info = ReplicaInfo::new("replica-1".to_string(), addr);

        assert_eq!(info.id, "replica-1");
        assert_eq!(info.host, "127.0.0.1");
        assert_eq!(info.port, 6381);
        assert_eq!(info.status, ReplicaStatus::Connected);
        assert_eq!(info.lag_ms, 0);
        assert_eq!(info.operations_synced, 0);
        assert_eq!(info.address, Some(addr));
    }

    #[test]
    fn test_replica_info_status_update_healthy() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6381);
        let mut info = ReplicaInfo::new("replica-1".to_string(), addr);

        // Set healthy state
        info.lag_ms = 10;
        info.offset = 100;
        info.last_heartbeat = SystemTime::now();

        info.update_status();

        assert_eq!(info.status, ReplicaStatus::Connected);
    }

    #[test]
    fn test_replica_info_status_update_lagging() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6381);
        let mut info = ReplicaInfo::new("replica-1".to_string(), addr);

        // Set lagging state
        info.lag_ms = 1500; // > 1000ms threshold
        info.offset = 100;
        info.last_heartbeat = SystemTime::now();

        info.update_status();

        assert_eq!(info.status, ReplicaStatus::Lagging);
    }

    #[test]
    fn test_replica_info_status_update_syncing() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6381);
        let mut info = ReplicaInfo::new("replica-1".to_string(), addr);

        // Set syncing state (offset = 0)
        info.lag_ms = 10;
        info.offset = 0;
        info.last_heartbeat = SystemTime::now();

        info.update_status();

        assert_eq!(info.status, ReplicaStatus::Syncing);
    }

    #[test]
    fn test_replica_info_status_update_disconnected() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        use std::time::Duration;

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6381);
        let mut info = ReplicaInfo::new("replica-1".to_string(), addr);

        // Set disconnected state (old heartbeat)
        info.lag_ms = 10;
        info.offset = 100;
        info.last_heartbeat = SystemTime::now() - Duration::from_secs(65); // > 60s threshold

        info.update_status();

        assert_eq!(info.status, ReplicaStatus::Disconnected);
    }

    #[test]
    fn test_replica_status_transitions() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        use std::time::Duration;

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6381);
        let mut info = ReplicaInfo::new("replica-1".to_string(), addr);

        // Initial: Connected
        assert_eq!(info.status, ReplicaStatus::Connected);

        // Transition to Syncing
        info.offset = 0;
        info.update_status();
        assert_eq!(info.status, ReplicaStatus::Syncing);

        // Transition to Connected
        info.offset = 50;
        info.lag_ms = 10;
        info.update_status();
        assert_eq!(info.status, ReplicaStatus::Connected);

        // Transition to Lagging
        info.lag_ms = 2000;
        info.update_status();
        assert_eq!(info.status, ReplicaStatus::Lagging);

        // Transition to Disconnected
        info.last_heartbeat = SystemTime::now() - Duration::from_secs(70);
        info.update_status();
        assert_eq!(info.status, ReplicaStatus::Disconnected);
    }

    #[test]
    fn test_replication_stats_serialization() {
        let stats = ReplicationStats {
            role: NodeRole::Master,
            lag_ms: 5,
            bytes_sent: 1024,
            bytes_received: 512,
            last_sync: UNIX_EPOCH + std::time::Duration::from_secs(1000),
            operations_pending: 10,
            snapshot_size: 2048,
            connected_replicas: Some(2),
            master_offset: 100,
            replica_offset: 95,
            lag_operations: 5,
            total_replicated: 95,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"role\":\"master\""));
        assert!(json.contains("\"lag_ms\":5"));
        assert!(json.contains("\"bytes_sent\":1024"));

        // Test deserialization
        let deserialized: ReplicationStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, NodeRole::Master);
        assert_eq!(deserialized.lag_ms, 5);
        assert_eq!(deserialized.bytes_sent, 1024);
    }

    #[test]
    fn test_replica_info_serialization() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 6381);
        let info = ReplicaInfo::new("test-replica".to_string(), addr);

        // Test JSON serialization
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"id\":\"test-replica\""));
        assert!(json.contains("\"host\":\"192.168.1.10\""));
        assert!(json.contains("\"port\":6381"));
        assert!(json.contains("\"status\":\"Connected\""));

        // Test deserialization
        let deserialized: ReplicaInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-replica");
        assert_eq!(deserialized.host, "192.168.1.10");
        assert_eq!(deserialized.port, 6381);
        assert_eq!(deserialized.status, ReplicaStatus::Connected);
    }

    #[test]
    fn test_replica_status_edge_cases() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};
        use std::time::Duration;

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6381);
        let mut info = ReplicaInfo::new("edge-test".to_string(), addr);

        // Exactly at threshold (1000ms)
        info.lag_ms = 1000;
        info.offset = 10;
        info.last_heartbeat = SystemTime::now();
        info.update_status();
        assert_eq!(info.status, ReplicaStatus::Connected); // Not lagging yet

        // Just over threshold (1001ms)
        info.lag_ms = 1001;
        info.update_status();
        assert_eq!(info.status, ReplicaStatus::Lagging);

        // Exactly at disconnect threshold (60s)
        info.lag_ms = 10;
        info.last_heartbeat = SystemTime::now() - Duration::from_secs(60);
        info.update_status();
        assert_eq!(info.status, ReplicaStatus::Connected); // Not disconnected yet

        // Just over disconnect threshold (61s)
        info.last_heartbeat = SystemTime::now() - Duration::from_secs(61);
        info.update_status();
        assert_eq!(info.status, ReplicaStatus::Disconnected);
    }
}
