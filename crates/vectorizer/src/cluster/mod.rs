//! Distributed cluster management for horizontal sharding
//!
//! This module provides cluster membership management, server discovery,
//! and distributed shard routing across multiple Vectorizer server instances.

pub mod collection_sync;
pub mod dns_discovery;
mod grpc_service;
pub mod ha_manager;
pub mod leader_router;
mod manager;
mod node;
pub mod raft_node;
pub mod raft_watcher;
pub mod rebalance;
mod server_client;
pub mod shard_migrator;
mod shard_router;
mod state_sync;
mod topology;
pub mod validator;

use std::sync::Arc;

pub use collection_sync::{CollectionSynchronizer, QuorumError, QuorumResult, SyncReport};
pub use dns_discovery::DnsDiscovery;
pub use grpc_service::ClusterGrpcService;
pub use ha_manager::HaManager;
pub use leader_router::{LeaderInfo, LeaderRouter, NodeRole as HaNodeRole};
pub use manager::ClusterManager;
pub use node::{ClusterNode, NodeId, NodeStatus};
use parking_lot::RwLock;
pub use raft_node::{
    ClusterCommand, ClusterResponse, ClusterStateMachine, RaftManager, TypeConfig,
};
pub use raft_watcher::RaftWatcher;
pub use rebalance::{PeerInfo, PeerRole, RebalanceJob, RebalanceStatus};
pub use server_client::{ClusterClient, ClusterClientPool};
pub use shard_router::DistributedShardRouter;
pub use state_sync::ClusterStateSynchronizer;
pub use topology::ClusterShardTopology;
use tracing::{error, info, warn};
pub use validator::{
    ClusterConfigValidator, ClusterValidationError, ClusterValidationResult,
    ClusterValidationWarning,
};

// `ClusterConfig` and its sub-structs are plain serde data types owned
// by `config` (phase41_architecture-decoupling §2: config must not
// depend on cluster, so the dependency now points the other way).
// Re-exported here under the historical `crate::cluster::*` paths so
// every existing call site keeps compiling.
pub use crate::config::sections::cluster::{
    ClusterConfig, ClusterMemoryConfig, DiscoveryMethod, ServerConfig,
};
