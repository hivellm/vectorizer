//! Distributed sharded collection implementation
//!
//! This module extends ShardedCollection to support distributed sharding
//! across multiple server instances in a cluster.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{debug, info, warn, error};

use super::collection::Collection;
use super::sharding::ShardId;
use crate::cluster::{ClusterManager, ClusterClientPool, DistributedShardRouter, NodeId};
use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, SearchResult, Vector};

/// A distributed sharded collection that distributes vectors across multiple servers
#[derive(Clone, Debug)]
pub struct DistributedShardedCollection {
    /// Collection name
    name: String,
    /// Base collection configuration
    config: CollectionConfig,
    /// Distributed shard router
    shard_router: Arc<DistributedShardRouter>,
    /// Cluster manager
    cluster_manager: Arc<ClusterManager>,
    /// Client pool for remote operations
    client_pool: Arc<ClusterClientPool>,
    /// Local shard collections (shard_id -> Collection) for shards on this node
    local_shards: Arc<parking_lot::RwLock<HashMap<ShardId, Collection>>>,
    /// Cache for vector count (count, timestamp)
    vector_count_cache: Arc<RwLock<Option<(usize, Instant)>>>,
    /// Cache TTL for vector count (default: 5 seconds)
    vector_count_cache_ttl: Duration,
}

impl DistributedShardedCollection {
    /// Create a new distributed sharded collection
    ///
    /// # Arguments
    /// * `name` - Collection name
    /// * `config` - Collection configuration (must have sharding enabled)
    /// * `cluster_manager` - Cluster manager instance
    /// * `client_pool` - Client pool for remote operations
    pub fn new(
        name: String,
        config: CollectionConfig,
        cluster_manager: Arc<ClusterManager>,
        client_pool: Arc<ClusterClientPool>,
    ) -> Result<Self> {
        let sharding_config = config
            .sharding
            .as_ref()
            .ok_or_else(|| VectorizerError::InvalidConfiguration {
                message: "Collection config must have sharding enabled".to_string(),
            })?;

        // Create distributed shard router
        let shard_router = Arc::new(DistributedShardRouter::new(
            sharding_config.virtual_nodes_per_shard,
        ));

        // Get active nodes
        let nodes = cluster_manager.get_active_nodes();
        if nodes.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "No active cluster nodes available".to_string(),
            });
        }

        // Assign shards to nodes using consistent hashing
        let shard_ids: Vec<ShardId> = (0..sharding_config.shard_count)
            .map(|i| ShardId::new(i))
            .collect();

        let node_ids: Vec<NodeId> = nodes.iter().map(|n| n.id.clone()).collect();
        shard_router.rebalance(&shard_ids, &node_ids);

        // Create local shards for shards assigned to this node
        let local_node_id = cluster_manager.local_node_id();
        let local_shards = Arc::new(parking_lot::RwLock::new(HashMap::new()));

        for shard_id in &shard_ids {
            if let Some(node_id) = shard_router.get_node_for_shard(shard_id) {
                if node_id == *local_node_id {
                    // This shard is on this node, create local collection
                    let mut shard_config = config.clone();
                    shard_config.sharding = None; // Shards themselves are not sharded

                    let shard_name = format!("{}_{}", name, shard_id);
                    let shard_collection = Collection::new(shard_name, shard_config);
                    local_shards.write().insert(*shard_id, shard_collection);
                }
            }
        }

        info!(
            "Created distributed sharded collection '{}' with {} shards across {} nodes",
            name,
            shard_ids.len(),
            nodes.len()
        );

        Ok(Self {
            name,
            config,
            shard_router,
            cluster_manager,
            client_pool,
            local_shards,
            vector_count_cache: Arc::new(RwLock::new(None)),
            vector_count_cache_ttl: Duration::from_secs(5),
        })
    }

    /// Get collection name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get collection config
    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }

    /// Insert a vector into the appropriate shard (local or remote)
    pub async fn insert(&self, vector: Vector) -> Result<()> {
        let shard_id = self.shard_router.get_shard_for_vector(&vector.id);
        
        if let Some(node_id) = self.shard_router.get_node_for_shard(&shard_id) {
            let local_node_id = self.cluster_manager.local_node_id();
            
            if node_id == *local_node_id {
                // Local shard
                let vector_id = vector.id.clone();
                let local_shards = self.local_shards.read();
                if let Some(shard) = local_shards.get(&shard_id) {
                    shard.insert(vector)?;
                    self.invalidate_vector_count_cache(); // Invalidate cache after insert
                    debug!(
                        "Inserted vector '{}' into local shard {} of collection '{}'",
                        vector_id, shard_id, self.name
                    );
                    return Ok(());
                }
            } else {
                // Remote shard - use gRPC client
                if let Some(node) = self.cluster_manager.get_node(&node_id) {
                    let client = self.client_pool
                        .get_client(&node_id, &node.grpc_address())
                        .await
                        .map_err(|e| VectorizerError::Storage(format!(
                            "Failed to get client for node {}: {}",
                            node_id, e
                        )))?;

                    let payload_json = vector.payload.as_ref()
                        .map(|p| serde_json::to_value(p).unwrap_or_default());

                    client.insert_vector(
                        &self.name,
                        &vector.id,
                        &vector.data,
                        payload_json.as_ref(),
                    ).await
                    .map_err(|e| VectorizerError::Storage(format!(
                        "Failed to insert vector on remote node {}: {}",
                        node_id, e
                    )))?;

                    debug!(
                        "Inserted vector '{}' into remote shard {} on node {} of collection '{}'",
                        vector.id, shard_id, node_id, self.name
                    );
                    return Ok(());
                }
            }
        }

        Err(VectorizerError::Storage(format!(
            "Shard {} not found or not assigned to any node",
            shard_id
        )))
    }

    /// Update a vector in the appropriate shard (local or remote)
    pub async fn update(&self, vector: Vector) -> Result<()> {
        let shard_id = self.shard_router.get_shard_for_vector(&vector.id);
        
        if let Some(node_id) = self.shard_router.get_node_for_shard(&shard_id) {
            let local_node_id = self.cluster_manager.local_node_id();
            
            if node_id == *local_node_id {
                // Local shard
                let vector_id = vector.id.clone();
                let local_shards = self.local_shards.read();
                if let Some(shard) = local_shards.get(&shard_id) {
                    shard.update(vector)?;
                    // Note: Update doesn't change count, so we don't invalidate cache
                    debug!(
                        "Updated vector '{}' in local shard {} of collection '{}'",
                        vector_id, shard_id, self.name
                    );
                    return Ok(());
                }
            } else {
                // Remote shard - use gRPC client
                if let Some(node) = self.cluster_manager.get_node(&node_id) {
                    let client = self.client_pool
                        .get_client(&node_id, &node.grpc_address())
                        .await
                        .map_err(|e| VectorizerError::Storage(format!(
                            "Failed to get client for node {}: {}",
                            node_id, e
                        )))?;

                    let payload_json = vector.payload.as_ref()
                        .map(|p| serde_json::to_value(p).unwrap_or_default());

                    client.update_vector(
                        &self.name,
                        &vector.id,
                        &vector.data,
                        payload_json.as_ref(),
                    ).await
                    .map_err(|e| VectorizerError::Storage(format!(
                        "Failed to update vector on remote node {}: {}",
                        node_id, e
                    )))?;

                    debug!(
                        "Updated vector '{}' in remote shard {} on node {} of collection '{}'",
                        vector.id, shard_id, node_id, self.name
                    );
                    return Ok(());
                }
            }
        }

        Err(VectorizerError::Storage(format!(
            "Shard {} not found or not assigned to any node",
            shard_id
        )))
    }

    /// Delete a vector from the appropriate shard (local or remote)
    pub async fn delete(&self, vector_id: &str) -> Result<()> {
        let shard_id = self.shard_router.get_shard_for_vector(vector_id);
        
        if let Some(node_id) = self.shard_router.get_node_for_shard(&shard_id) {
            let local_node_id = self.cluster_manager.local_node_id();
            
            if node_id == *local_node_id {
                // Local shard
                let local_shards = self.local_shards.read();
                if let Some(shard) = local_shards.get(&shard_id) {
                    shard.delete(vector_id)?;
                    self.invalidate_vector_count_cache(); // Invalidate cache after delete
                    debug!(
                        "Deleted vector '{}' from local shard {} of collection '{}'",
                        vector_id, shard_id, self.name
                    );
                    return Ok(());
                }
            } else {
                // Remote shard - use gRPC client
                if let Some(node) = self.cluster_manager.get_node(&node_id) {
                    let client = self.client_pool
                        .get_client(&node_id, &node.grpc_address())
                        .await
                        .map_err(|e| VectorizerError::Storage(format!(
                            "Failed to get client for node {}: {}",
                            node_id, e
                        )))?;

                    client.delete_vector(
                        &self.name,
                        vector_id,
                    ).await
                    .map_err(|e| VectorizerError::Storage(format!(
                        "Failed to delete vector on remote node {}: {}",
                        node_id, e
                    )))?;

                    self.invalidate_vector_count_cache(); // Invalidate cache after delete
                    debug!(
                        "Deleted vector '{}' from remote shard {} on node {} of collection '{}'",
                        vector_id, shard_id, node_id, self.name
                    );
                    return Ok(());
                }
            }
        }

        Err(VectorizerError::Storage(format!(
            "Shard {} not found or not assigned to any node",
            shard_id
        )))
    }

    /// Search across all shards (local and remote) and merge results
    pub async fn search(
        &self,
        query_vector: &[f32],
        k: usize,
        threshold: Option<f32>,
        shard_keys: Option<&[ShardId]>,
    ) -> Result<Vec<SearchResult>> {
        // Get all shards to search
        let shard_ids = if let Some(keys) = shard_keys {
            keys.to_vec()
        } else {
            // Get all shards - for now, get from all nodes
            // TODO: Get all shards from router when method is available
            let mut all_shards = Vec::new();
            for node in self.cluster_manager.get_active_nodes() {
                all_shards.extend(self.shard_router.get_shards_for_node(&node.id));
            }
            all_shards
        };

        if shard_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_results = Vec::new();
        let local_node_id = self.cluster_manager.local_node_id();

        // Group shards by node
        let mut node_shards: HashMap<NodeId, Vec<ShardId>> = HashMap::new();
        for shard_id in &shard_ids {
            if let Some(node_id) = self.shard_router.get_node_for_shard(shard_id) {
                node_shards.entry(node_id).or_insert_with(Vec::new).push(*shard_id);
            }
        }

        // Search local shards
        if let Some(local_shard_ids) = node_shards.get(local_node_id) {
            let local_shards = self.local_shards.read();
            for shard_id in local_shard_ids {
                if let Some(shard) = local_shards.get(shard_id) {
                    match shard.search(query_vector, k) {
                        Ok(results) => {
                            all_results.extend(results);
                        }
                        Err(e) => {
                            warn!("Error searching local shard {}: {}", shard_id, e);
                        }
                    }
                }
            }
        }

        // Search remote shards
        let node_count = node_shards.len();
        for (node_id, remote_shard_ids) in node_shards {
            if node_id == *local_node_id {
                continue; // Already handled
            }

            if let Some(node) = self.cluster_manager.get_node(&node_id) {
                match self.client_pool.get_client(&node_id, &node.grpc_address()).await {
                    Ok(client) => {
                        match client.search_vectors(
                            &self.name,
                            query_vector,
                            k,
                            threshold,
                            Some(&remote_shard_ids),
                        ).await {
                            Ok(results) => {
                                all_results.extend(results);
                            }
                            Err(e) => {
                                warn!("Error searching remote shards on node {}: {}", node_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get client for node {}: {}", node_id, e);
                    }
                }
            }
        }

        // Optimized merge: Use partial sort for top k instead of full sort
        // This is more efficient when k << total_results
        if all_results.len() > k {
            // Use select_nth_unstable for partial sort (O(n) instead of O(n log n))
            let (left, _middle, _right) = all_results.select_nth_unstable_by(k - 1, |a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            // Sort only the top k elements
            left.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            // Keep only top k
            all_results = left.to_vec();
        } else {
            // If we have fewer results than k, just sort all
            all_results.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        debug!(
            "Distributed multi-shard search in collection '{}' returned {} results from {} nodes",
            self.name,
            all_results.len(),
            node_count
        );

        Ok(all_results)
    }

    /// Get total vector count across all shards (with caching)
    pub async fn vector_count(&self) -> Result<usize> {
        // Check cache first
        {
            let cache = self.vector_count_cache.read();
            if let Some((cached_count, timestamp)) = *cache {
                if timestamp.elapsed() < self.vector_count_cache_ttl {
                    debug!("Using cached vector count: {}", cached_count);
                    return Ok(cached_count);
                }
            }
        }

        let mut total = 0;

        // Count local shards
        let local_shards = self.local_shards.read();
        for shard in local_shards.values() {
            total += shard.vector_count();
        }
        drop(local_shards);

        // Count remote shards via gRPC
        let local_node_id = self.cluster_manager.local_node_id();
        let all_nodes = self.cluster_manager.get_active_nodes();
        
        for node in all_nodes {
            if node.id == *local_node_id {
                continue; // Already counted
            }

            // Get shards for this node
            let node_shards = self.shard_router.get_shards_for_node(&node.id);
            if node_shards.is_empty() {
                continue;
            }

            // Get collection info from remote node
            if let Some(node) = self.cluster_manager.get_node(&node.id) {
                match self.client_pool.get_client(&node.id, &node.grpc_address()).await {
                    Ok(client) => {
                        // Get collection info for each shard on this node
                        // Since shards are separate collections, we need to query each shard
                        for shard_id in &node_shards {
                            let shard_collection_name = format!("{}_{}", self.name, shard_id);
                            
                            match client.get_collection_info(&shard_collection_name).await {
                                Ok(Some(info)) => {
                                    total += info.vector_count as usize;
                                    debug!(
                                        "Remote shard {} on node {} has {} vectors",
                                        shard_id, node.id, info.vector_count
                                    );
                                }
                                Ok(None) => {
                                    debug!("No collection info returned for shard {} on node {}", shard_id, node.id);
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to get collection info for shard {} on node {}: {}",
                                        shard_id, node.id, e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get client for node {} to count vectors: {}", node.id, e);
                    }
                }
            }
        }

        // Update cache
        {
            let mut cache = self.vector_count_cache.write();
            *cache = Some((total, Instant::now()));
        }

        Ok(total)
    }

    /// Invalidate vector count cache (call after insert/delete operations)
    fn invalidate_vector_count_cache(&self) {
        let mut cache = self.vector_count_cache.write();
        *cache = None;
    }
}

