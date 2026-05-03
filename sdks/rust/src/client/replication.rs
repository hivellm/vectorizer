//! Replication surface.
//!
//! Covers the `/replication/*` REST endpoints (status, configuration,
//! statistics, replica listing) and the `/cluster/*` admin endpoints
//! added in phase15 (failover, resync, add-peer, rebalance).

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::{
    AddPeerRequest, FailoverReport, PeerInfo, RebalanceJob, ReplicaInfo, ReplicationConfig,
    ReplicationStats, ReplicationStatus, ResyncJob,
};

impl VectorizerClient {
    /// Get the current replication status and role of this node.
    ///
    /// Calls `GET /replication/status`.
    pub async fn get_replication_status(&self) -> Result<ReplicationStatus> {
        let response = self
            .make_request("GET", "/replication/status", None)
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse get_replication_status response: {e}"
            ))
        })
    }

    /// Configure this node's replication role and parameters.
    ///
    /// Calls `POST /replication/configure` with the full
    /// [`ReplicationConfig`]. A server restart is required for the
    /// new config to take effect.
    pub async fn configure_replication(&self, config: ReplicationConfig) -> Result<()> {
        let payload = serde_json::to_value(&config).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to serialize configure_replication request: {e}"
            ))
        })?;
        self.make_request("POST", "/replication/configure", Some(payload))
            .await?;
        Ok(())
    }

    /// Get raw replication statistics for the active replication node.
    ///
    /// Calls `GET /replication/stats`. Returns an error when
    /// replication is not enabled on this node.
    pub async fn get_replication_stats(&self) -> Result<ReplicationStats> {
        let response = self.make_request("GET", "/replication/stats", None).await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse get_replication_stats response: {e}"
            ))
        })
    }

    /// List the replica nodes connected to this master.
    ///
    /// Calls `GET /replication/replicas`. Only available on master
    /// nodes; returns an error otherwise.
    pub async fn list_replicas(&self) -> Result<Vec<ReplicaInfo>> {
        let response = self
            .make_request("GET", "/replication/replicas", None)
            .await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list_replicas response: {e}"))
        })?;
        let arr = val
            .get("replicas")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse replica entry: {e}"))
                })
            })
            .collect()
    }

    /// Trigger a failover — promote a replica to primary.
    ///
    /// Calls `POST /cluster/failover` with `{replica_id}`.
    /// Returns 409 from the server when the replica's WAL lag exceeds the
    /// configured threshold.
    pub async fn cluster_failover(&self, replica_id: &str) -> Result<FailoverReport> {
        let payload = serde_json::json!({ "replica_id": replica_id });
        let response = self
            .make_request("POST", "/cluster/failover", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse cluster_failover response: {e}"))
        })
    }

    /// Force a full resync on a replica.
    ///
    /// Calls `POST /cluster/replicas/{id}/resync` with an empty body.
    pub async fn cluster_resync_replica(&self, replica_id: &str) -> Result<ResyncJob> {
        let response = self
            .make_request(
                "POST",
                &format!("/cluster/replicas/{replica_id}/resync"),
                Some(serde_json::json!({})),
            )
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse cluster_resync_replica response: {e}"
            ))
        })
    }

    /// Add a peer to the cluster.
    ///
    /// Calls `POST /cluster/peers` with `{address, role}`.
    pub async fn cluster_add_peer(&self, request: AddPeerRequest) -> Result<PeerInfo> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::server(format!("Failed to serialize cluster_add_peer request: {e}"))
        })?;
        let response = self
            .make_request("POST", "/cluster/peers", Some(payload))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse cluster_add_peer response: {e}"))
        })
    }

    /// Trigger a shard rebalance across all active cluster nodes.
    ///
    /// Calls `POST /cluster/rebalance` with an empty body.
    /// Returns 400 when fewer than 2 active nodes are present, or 400 when
    /// a rebalance is already in progress.
    pub async fn cluster_rebalance(&self) -> Result<RebalanceJob> {
        let response = self
            .make_request("POST", "/cluster/rebalance", Some(serde_json::json!({})))
            .await?;
        serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse cluster_rebalance response: {e}"))
        })
    }

    /// Return progress of the active (or last completed) rebalance job.
    ///
    /// Calls `GET /cluster/rebalance/status`.
    /// Returns `None` when no rebalance has been triggered on this node.
    pub async fn cluster_rebalance_status(&self) -> Result<Option<RebalanceJob>> {
        let response = self
            .make_request("GET", "/cluster/rebalance/status", None)
            .await?;
        let val: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse cluster_rebalance_status response: {e}"
            ))
        })?;
        // Server returns {status: "idle"} when no rebalance has been triggered.
        if val.get("status").and_then(|s| s.as_str()) == Some("idle") {
            return Ok(None);
        }
        serde_json::from_value(val).map(Some).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to deserialize cluster_rebalance_status: {e}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use crate::models::{ReplicationConfig, ReplicationStats, ReplicationStatus};

    #[test]
    fn replication_status_deserializes_standalone() {
        let raw = json!({
            "role": "Standalone",
            "enabled": false
        });
        let rs: ReplicationStatus = serde_json::from_value(raw).unwrap();
        assert_eq!(rs.role, "Standalone");
        assert!(!rs.enabled);
        assert!(rs.stats.is_none());
        assert!(rs.replicas.is_none());
    }

    #[test]
    fn replication_status_deserializes_master() {
        let raw = json!({
            "role": "Master",
            "enabled": true,
            "stats": {
                "master_offset": 100,
                "replica_offset": 95,
                "lag_operations": 5,
                "total_replicated": 500
            },
            "replicas": []
        });
        let rs: ReplicationStatus = serde_json::from_value(raw).unwrap();
        assert_eq!(rs.role, "Master");
        assert!(rs.enabled);
        assert!(rs.stats.is_some());
    }

    #[test]
    fn replication_config_serializes_master() {
        let cfg = ReplicationConfig {
            role: "master".into(),
            bind_address: Some("0.0.0.0:15010".into()),
            master_address: None,
            heartbeat_interval: Some(1000),
            log_size: None,
        };
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["role"], "master");
        assert_eq!(v["bind_address"], "0.0.0.0:15010");
        assert_eq!(v["heartbeat_interval"], 1000);
        assert!(v.get("master_address").is_none());
    }

    #[test]
    fn replication_config_serializes_replica() {
        let cfg = ReplicationConfig {
            role: "replica".into(),
            bind_address: None,
            master_address: Some("master.host:15010".into()),
            heartbeat_interval: None,
            log_size: None,
        };
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["role"], "replica");
        assert_eq!(v["master_address"], "master.host:15010");
    }

    #[test]
    fn replication_stats_round_trip() {
        let raw = json!({
            "master_offset": 200,
            "replica_offset": 190,
            "lag_operations": 10,
            "total_replicated": 1000,
            "bytes_sent": 4096u64,
            "connected_replicas": 2
        });
        let stats: ReplicationStats = serde_json::from_value(raw).unwrap();
        assert_eq!(stats.master_offset, 200);
        assert_eq!(stats.lag_operations, 10);
        assert_eq!(stats.bytes_sent, Some(4096));
        assert_eq!(stats.connected_replicas, Some(2));
    }

    // ── phase15 cluster admin ─────────────────────────────────────────────────

    use crate::models::{AddPeerRequest, FailoverReport, PeerInfo, RebalanceJob, ResyncJob};

    #[test]
    fn failover_report_round_trip() {
        let raw = json!({
            "promoted_replica_id": "replica-1",
            "master_offset_at_promotion": 1000u64,
            "replica_offset_at_promotion": 999u64,
            "residual_lag_operations": 1u64
        });
        let r: FailoverReport = serde_json::from_value(raw).unwrap();
        assert_eq!(r.promoted_replica_id, "replica-1");
        assert_eq!(r.master_offset_at_promotion, 1000);
        assert_eq!(r.residual_lag_operations, 1);
    }

    #[test]
    fn resync_job_round_trip() {
        let raw = json!({
            "replica_id": "replica-2",
            "snapshot_offset": 5000u64,
            "full_snapshot": true
        });
        let j: ResyncJob = serde_json::from_value(raw).unwrap();
        assert_eq!(j.replica_id, "replica-2");
        assert_eq!(j.snapshot_offset, 5000);
        assert!(j.full_snapshot);
    }

    #[test]
    fn peer_info_round_trip() {
        let raw = json!({
            "node_id": "peer-abc",
            "address": "10.0.0.2:15003",
            "role": "member"
        });
        let p: PeerInfo = serde_json::from_value(raw).unwrap();
        assert_eq!(p.node_id, "peer-abc");
        assert_eq!(p.role, "member");
    }

    #[test]
    fn add_peer_request_serializes() {
        let req = AddPeerRequest {
            address: "10.0.0.3:15003".into(),
            role: "observer".into(),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["address"], "10.0.0.3:15003");
        assert_eq!(v["role"], "observer");
    }

    #[test]
    fn rebalance_job_round_trip() {
        let raw = json!({
            "job_id": "job-xyz",
            "status": "running",
            "shards_to_move": 4usize,
            "shards_moved": 1usize,
            "message": "Rebalance started"
        });
        let j: RebalanceJob = serde_json::from_value(raw).unwrap();
        assert_eq!(j.job_id, "job-xyz");
        assert_eq!(j.status, "running");
        assert_eq!(j.shards_to_move, 4);
        assert!(j.last_checkpoint_node.is_none());
    }
}
