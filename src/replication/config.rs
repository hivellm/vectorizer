//! Replication configuration

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

/// Replication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    /// Node role (master, replica, standalone)
    pub role: super::types::NodeRole,

    /// Master address (for replicas to connect to)
    pub master_address: Option<SocketAddr>,

    /// Replication bind address (for master to listen on)
    pub bind_address: Option<SocketAddr>,

    /// Heartbeat interval in seconds
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval: u64,

    /// Replica timeout in seconds
    #[serde(default = "default_replica_timeout")]
    pub replica_timeout: u64,

    /// Replication log size (number of operations)
    #[serde(default = "default_log_size")]
    pub log_size: usize,

    /// Auto-reconnect interval in seconds
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval: u64,
}

fn default_heartbeat_interval() -> u64 {
    5
}

fn default_replica_timeout() -> u64 {
    30
}

fn default_log_size() -> usize {
    1_000_000 // 1M operations like Redis
}

fn default_reconnect_interval() -> u64 {
    5
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            role: super::types::NodeRole::Standalone,
            master_address: None,
            bind_address: None,
            heartbeat_interval: default_heartbeat_interval(),
            replica_timeout: default_replica_timeout(),
            log_size: default_log_size(),
            reconnect_interval: default_reconnect_interval(),
        }
    }
}

impl ReplicationConfig {
    /// Create a master configuration
    pub fn master(bind_address: SocketAddr) -> Self {
        Self {
            role: super::types::NodeRole::Master,
            bind_address: Some(bind_address),
            ..Default::default()
        }
    }

    /// Create a replica configuration
    pub fn replica(master_address: SocketAddr) -> Self {
        Self {
            role: super::types::NodeRole::Replica,
            master_address: Some(master_address),
            ..Default::default()
        }
    }

    /// Get heartbeat interval as Duration
    pub fn heartbeat_duration(&self) -> Duration {
        Duration::from_secs(self.heartbeat_interval)
    }

    /// Get replica timeout as Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.replica_timeout)
    }

    /// Get reconnect interval as Duration
    pub fn reconnect_duration(&self) -> Duration {
        Duration::from_secs(self.reconnect_interval)
    }
}

