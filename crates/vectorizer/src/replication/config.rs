//! Replication configuration

use std::net::SocketAddr;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Replication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    /// Node role (master, replica, standalone)
    pub role: super::types::NodeRole,

    /// Master address (for replicas to connect to).
    ///
    /// Stored as the original string (`host:port`) so that DNS hostnames
    /// (e.g. K8s headless service names) are re-resolved on each connection
    /// attempt instead of being pinned to a single IP at config-load time.
    pub master_address: Option<SocketAddr>,

    /// The raw master address string before DNS resolution.
    /// When set, the replica will re-resolve this on every reconnect
    /// instead of using the cached `master_address` SocketAddr.
    #[serde(default)]
    pub master_address_raw: Option<String>,

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

    /// Enable WAL for durable replication (default: true)
    #[serde(default = "default_wal_enabled")]
    pub wal_enabled: bool,

    /// WAL directory path (default: data_dir/replication-wal)
    #[serde(default)]
    pub wal_dir: Option<String>,
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

fn default_wal_enabled() -> bool {
    true
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            role: super::types::NodeRole::Standalone,
            master_address: None,
            master_address_raw: None,
            bind_address: None,
            heartbeat_interval: default_heartbeat_interval(),
            replica_timeout: default_replica_timeout(),
            log_size: default_log_size(),
            reconnect_interval: default_reconnect_interval(),
            wal_enabled: default_wal_enabled(),
            wal_dir: None,
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

    /// Resolve the master address, re-resolving DNS if a raw hostname was stored.
    ///
    /// In Kubernetes, pod IPs change on restart. By re-resolving the DNS name
    /// on each reconnect attempt, replicas follow the master to its new IP.
    pub async fn resolve_master_address(&self) -> Option<SocketAddr> {
        // If we have the raw hostname, always re-resolve it
        if let Some(ref raw) = self.master_address_raw {
            // Try direct parse first (IP:port needs no resolution)
            if let Ok(sock) = raw.parse::<SocketAddr>() {
                return Some(sock);
            }

            // Resolve DNS asynchronously
            match tokio::net::lookup_host(raw).await {
                Ok(mut addrs) => {
                    let resolved = addrs.next();
                    if resolved.is_some() {
                        return resolved;
                    }
                    tracing::warn!("DNS resolution for '{}' returned no addresses", raw);
                    None
                }
                Err(e) => {
                    tracing::warn!("DNS resolution for '{}' failed: {}", raw, e);
                    // Fall back to cached address if DNS fails
                    self.master_address
                }
            }
        } else {
            // No raw address — use the pre-resolved one
            self.master_address
        }
    }
}
