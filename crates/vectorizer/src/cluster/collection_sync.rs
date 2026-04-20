//! Distributed collection consistency for cluster nodes
//!
//! Provides quorum-based collection creation and background sync to repair
//! collections that are missing from one or more nodes after a partial failure.

use std::collections::HashSet;
use std::sync::Arc;

use tracing::{debug, error, info, warn};

use super::manager::ClusterManager;
use super::node::NodeId;
use super::server_client::ClusterClientPool;
use crate::db::VectorStore;

/// Handles distributed collection consistency across cluster nodes
pub struct CollectionSynchronizer {
    manager: Arc<ClusterManager>,
    client_pool: Arc<ClusterClientPool>,
    store: Arc<VectorStore>,
}

impl CollectionSynchronizer {
    /// Create a new `CollectionSynchronizer`.
    pub fn new(
        manager: Arc<ClusterManager>,
        client_pool: Arc<ClusterClientPool>,
        store: Arc<VectorStore>,
    ) -> Self {
        Self {
            manager,
            client_pool,
            store,
        }
    }

    /// Create a collection with quorum consensus.
    ///
    /// Attempts to create the collection on the local node and all active remote
    /// nodes. Returns [`QuorumResult`] when a majority succeeds, or rolls back
    /// all successful creations and returns [`QuorumError::QuorumNotMet`] when
    /// fewer than half+1 nodes succeed.
    pub async fn create_collection_with_quorum(
        &self,
        name: &str,
        config: crate::models::CollectionConfig,
        owner_id: Option<uuid::Uuid>,
    ) -> Result<QuorumResult, QuorumError> {
        let nodes = self.manager.get_active_nodes();
        let total = nodes.len();
        let quorum = total / 2 + 1;

        let local_node_id = self.manager.local_node_id().clone();

        let mut successes: Vec<NodeId> = Vec::new();
        let mut failures: Vec<(NodeId, String)> = Vec::new();

        // Create locally first
        let local_result = if let Some(owner) = owner_id {
            self.store
                .create_collection_with_owner(name, config.clone(), owner)
        } else {
            self.store.create_collection(name, config.clone())
        };

        match local_result {
            Ok(_) => successes.push(local_node_id.clone()),
            Err(e) => failures.push((local_node_id.clone(), e.to_string())),
        }

        // Create on all remote nodes
        for node in &nodes {
            if node.id == local_node_id {
                continue;
            }

            match self
                .client_pool
                .get_client(&node.id, &node.grpc_address())
                .await
            {
                Ok(client) => {
                    match client
                        .remote_create_collection(name, &config, owner_id)
                        .await
                    {
                        Ok(resp) if resp.success => successes.push(node.id.clone()),
                        Ok(resp) => failures.push((node.id.clone(), resp.message)),
                        Err(e) => failures.push((node.id.clone(), e.to_string())),
                    }
                }
                Err(e) => failures.push((node.id.clone(), e.to_string())),
            }
        }

        if successes.len() >= quorum {
            if !failures.is_empty() {
                warn!(
                    "Collection '{}' created on {}/{} nodes (quorum met, {} failures)",
                    name,
                    successes.len(),
                    total,
                    failures.len()
                );
            } else {
                info!(
                    "Collection '{}' created on all {} nodes",
                    name,
                    successes.len()
                );
            }

            return Ok(QuorumResult {
                successful_nodes: successes,
                failed_nodes: failures,
                quorum_met: true,
            });
        }

        // Quorum not met – roll back every node that succeeded
        error!(
            "Quorum not met for collection '{}': required {}, achieved {}. Rolling back.",
            name,
            quorum,
            successes.len()
        );

        for node_id in &successes {
            if *node_id == local_node_id {
                if let Err(e) = self.store.delete_collection(name) {
                    error!("Rollback failed for local collection '{}': {}", name, e);
                }
            } else if let Some(node) = self.manager.get_node(node_id) {
                match self
                    .client_pool
                    .get_client(node_id, &node.grpc_address())
                    .await
                {
                    Ok(client) => {
                        if let Err(e) = client.remote_delete_collection(name).await {
                            error!(
                                "Rollback failed for collection '{}' on node {}: {}",
                                name, node_id, e
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            "Could not connect to node {} for rollback of '{}': {}",
                            node_id, name, e
                        );
                    }
                }
            }
        }

        Err(QuorumError::QuorumNotMet {
            required: quorum,
            achieved: successes.len(),
            failures,
        })
    }

    /// Background sync: detect and repair collections missing from remote nodes.
    ///
    /// Iterates over all active remote nodes and ensures every locally-known
    /// collection exists on each one. Missing collections are re-created using
    /// the local configuration as the source of truth.
    pub async fn sync_collections(
        &self,
    ) -> Result<SyncReport, Box<dyn std::error::Error + Send + Sync>> {
        let local_collections: HashSet<String> =
            self.store.list_collections().into_iter().collect();
        let nodes = self.manager.get_active_nodes();
        let local_node_id = self.manager.local_node_id().clone();
        let mut repaired: Vec<(String, NodeId)> = Vec::new();

        for node in &nodes {
            if node.id == local_node_id {
                continue;
            }

            match self
                .client_pool
                .get_client(&node.id, &node.grpc_address())
                .await
            {
                Ok(client) => {
                    for collection_name in &local_collections {
                        match client.remote_get_collection_info(collection_name).await {
                            // success == false means collection is absent on the remote node
                            Ok(resp) if !resp.success => {
                                warn!(
                                    "Collection '{}' missing on node {}, repairing",
                                    collection_name, node.id
                                );
                                if let Ok(col) = self.store.get_collection(collection_name) {
                                    let config = col.config().clone();
                                    match client
                                        .remote_create_collection(collection_name, &config, None)
                                        .await
                                    {
                                        Ok(create_resp) if create_resp.success => {
                                            info!(
                                                "Repaired collection '{}' on node {}",
                                                collection_name, node.id
                                            );
                                            repaired
                                                .push((collection_name.clone(), node.id.clone()));
                                        }
                                        Ok(create_resp) => {
                                            error!(
                                                "Failed to repair collection '{}' on node {}: {}",
                                                collection_name, node.id, create_resp.message
                                            );
                                        }
                                        Err(e) => {
                                            error!(
                                                "gRPC error repairing collection '{}' on node {}: {}",
                                                collection_name, node.id, e
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                debug!(
                                    "Could not check collection '{}' on node {}: {}",
                                    collection_name, node.id, e
                                );
                            }
                            // success == true: collection present, nothing to do
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Could not connect to node {} for collection sync: {}",
                        node.id, e
                    );
                }
            }
        }

        Ok(SyncReport {
            repaired_count: repaired.len(),
            repaired,
        })
    }
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Result of a quorum-based collection creation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QuorumResult {
    /// Node IDs on which the collection was created successfully.
    pub successful_nodes: Vec<NodeId>,
    /// Node IDs and error messages for nodes on which creation failed.
    pub failed_nodes: Vec<(NodeId, String)>,
    /// Whether the required quorum was reached.
    pub quorum_met: bool,
}

/// Error type for quorum-based collection operations.
#[derive(Debug, thiserror::Error)]
pub enum QuorumError {
    /// Fewer than half+1 nodes acknowledged the create; the operation was rolled back.
    #[error("Quorum not met: required {required}, achieved {achieved}")]
    QuorumNotMet {
        /// Minimum number of nodes that must succeed.
        required: usize,
        /// Actual number of nodes that succeeded before rollback.
        achieved: usize,
        /// Per-node failure descriptions.
        failures: Vec<(NodeId, String)>,
    },
}

/// Report produced by a background collection sync pass.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncReport {
    /// Total number of (collection, node) pairs that were repaired.
    pub repaired_count: usize,
    /// Each repaired pair as (collection_name, node_id).
    pub repaired: Vec<(String, NodeId)>,
}
