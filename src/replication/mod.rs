//! Replication module - Master-Replica architecture for high availability
//!
//! Design inspired by Redis and Synap Replication:
//! - 1 Master node (accepts writes)
//! - N Replica nodes (read-only)
//! - Async replication (non-blocking)
//! - Manual failover (promote replica to master)
//!
//! Features:
//! - Full sync on replica connect (snapshot + incremental)
//! - Partial resync on reconnect (from last offset)
//! - Lag monitoring and metrics
//! - Configurable replication modes

pub mod config;
pub mod master;
pub mod replica;
pub mod replication_log;
pub mod sync;
pub mod types;

#[cfg(test)]
mod tests;

pub use config::ReplicationConfig;
pub use master::MasterNode;
pub use replica::ReplicaNode;
pub use replication_log::ReplicationLog;
pub use types::{
    CollectionConfigData, NodeRole, ReplicaInfo, ReplicationCommand, ReplicationError,
    ReplicationOperation, ReplicationResult, ReplicationStats, VectorOperation,
};

