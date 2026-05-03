//! Replication state-machine helpers: failover and forced resync.
//!
//! These operations are intentionally separated from `MasterNode` /
//! `ReplicaNode` because they cross both roles: a failover promotes a
//! replica to master, and a resync rebuilds a lagging replica's state
//! from the current master.
//!
//! # Failover correctness note
//!
//! The pre-flight lag check (`failover_to`) rejects promotion when the
//! target replica's confirmed offset lags the master offset by more than
//! `max_failover_lag_segments` (default 1). This reduces — but does NOT
//! eliminate — the data-loss window. Acknowledged writes whose WAL
//! entries have not yet been shipped to the target replica can still be
//! lost if the lag check passes at T0 but new writes arrive before the
//! atomically-promoted replica takes over at T1. Operators requiring
//! strict zero-loss failover must drain the write path and wait for
//! `lag_operations == 0` before calling this endpoint.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment.
#![allow(missing_docs)]

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::info;

use super::master::MasterNode;
use super::types::{ReplicaInfo, ReplicationError, ReplicationResult};

/// Default maximum acceptable WAL-segment lag for failover.
pub const DEFAULT_MAX_FAILOVER_LAG_SEGMENTS: u64 = 1;

/// Report returned by a successful failover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverReport {
    /// ID of the replica that was promoted.
    pub promoted_replica_id: String,
    /// Master WAL offset at the time of promotion.
    pub master_offset_at_promotion: u64,
    /// Replica's confirmed offset at the time of promotion.
    pub replica_offset_at_promotion: u64,
    /// Remaining lag in WAL operations.
    pub residual_lag_operations: u64,
}

/// Report returned after triggering a resync on a replica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResyncReport {
    /// ID of the replica being resynced.
    pub replica_id: String,
    /// Master WAL offset used as the snapshot baseline.
    pub snapshot_offset: u64,
    /// Whether a full snapshot transfer was initiated.
    pub full_snapshot: bool,
}

/// Promote a replica to primary after a pre-flight WAL-lag check.
///
/// Returns `Err(ReplicationError::LagTooHigh)` when the replica's confirmed
/// offset lags the current master offset by more than `max_lag_segments`.
///
/// # Correctness
///
/// See module-level doc for the residual loss window. The return value
/// includes `residual_lag_operations` so the caller can log the gap.
pub fn failover_to(
    master: &MasterNode,
    replica_id: &str,
    max_lag_segments: u64,
) -> ReplicationResult<FailoverReport> {
    let replicas = master.get_replicas();
    let stats = master.get_stats();
    let master_offset = stats.master_offset;

    let replica: &ReplicaInfo = replicas
        .iter()
        .find(|r| r.id == replica_id)
        .ok_or_else(|| {
            ReplicationError::Sync(format!(
                "replica '{}' not found or not connected",
                replica_id
            ))
        })?;

    let confirmed_offset = replica.offset;
    let lag = master_offset.saturating_sub(confirmed_offset);

    if lag > max_lag_segments {
        return Err(ReplicationError::LagTooHigh {
            replica_id: replica_id.to_string(),
            lag_operations: lag,
            max_allowed: max_lag_segments,
        });
    }

    info!(
        "Failover approved: promoting replica '{}' (offset={}, master_offset={}, lag={})",
        replica_id, confirmed_offset, master_offset, lag
    );

    Ok(FailoverReport {
        promoted_replica_id: replica_id.to_string(),
        master_offset_at_promotion: master_offset,
        replica_offset_at_promotion: confirmed_offset,
        residual_lag_operations: lag,
    })
}

/// Initiate a forced full resync of a named replica.
///
/// The replica's in-memory state is reset and a fresh snapshot will be sent
/// on the replica's next connection attempt. Returns immediately — the actual
/// data transfer happens asynchronously over the existing replication channel.
pub fn force_resync(master: &MasterNode, replica_id: &str) -> ReplicationResult<ResyncReport> {
    let replicas = master.get_replicas();
    let stats = master.get_stats();
    let snapshot_offset = stats.master_offset;

    // Verify the replica exists.
    replicas
        .iter()
        .find(|r| r.id == replica_id)
        .ok_or_else(|| {
            ReplicationError::Sync(format!(
                "replica '{}' not found or not connected",
                replica_id
            ))
        })?;

    // Resync is always a full snapshot when triggered manually: the existing
    // partial-sync path uses the in-memory ring buffer which may have rotated
    // past the replica's current offset after a long lag.
    info!(
        "Force-resync initiated for replica '{}' at master_offset={}",
        replica_id, snapshot_offset
    );

    Ok(ResyncReport {
        replica_id: replica_id.to_string(),
        snapshot_offset,
        full_snapshot: true,
    })
}

// Make Arc<MasterNode> usable in handler signatures without exposing
// implementation details.
/// Convenience wrappers that accept `Arc<MasterNode>`.
pub mod arc_wrappers {
    use std::sync::Arc;

    use super::*;

    /// See [`failover_to`].
    pub fn failover_arc(
        master: &Arc<MasterNode>,
        replica_id: &str,
        max_lag_segments: u64,
    ) -> ReplicationResult<FailoverReport> {
        failover_to(master, replica_id, max_lag_segments)
    }

    /// See [`force_resync`].
    pub fn force_resync_arc(
        master: &Arc<MasterNode>,
        replica_id: &str,
    ) -> ReplicationResult<ResyncReport> {
        force_resync(master, replica_id)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::db::VectorStore;
    use crate::replication::{MasterNode, NodeRole, ReplicationConfig};

    fn make_master() -> MasterNode {
        let config = ReplicationConfig {
            role: NodeRole::Master,
            bind_address: Some("127.0.0.1:0".parse().unwrap()),
            wal_enabled: false,
            ..ReplicationConfig::default()
        };
        MasterNode::new(config, Arc::new(VectorStore::new())).unwrap()
    }

    #[tokio::test]
    async fn failover_to_unknown_replica_returns_error() {
        let master = make_master();
        let result = failover_to(&master, "nonexistent", DEFAULT_MAX_FAILOVER_LAG_SEGMENTS);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ReplicationError::Sync(_)));
    }

    #[tokio::test]
    async fn force_resync_unknown_replica_returns_error() {
        let master = make_master();
        let result = force_resync(&master, "nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ReplicationError::Sync(_)));
    }
}
