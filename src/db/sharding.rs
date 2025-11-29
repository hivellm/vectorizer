//! Distributed sharding implementation for Vectorizer
//!
//! This module provides consistent hash sharding for distributing vectors
//! across multiple shards, enabling horizontal scaling of collections.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{Result, VectorizerError};
use crate::models::{SearchResult, Vector};

/// Shard identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShardId(pub u32);

impl ShardId {
    /// Create a new shard ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the numeric shard ID
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for ShardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "shard_{}", self.0)
    }
}

/// Shard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMetadata {
    /// Shard ID
    pub id: ShardId,
    /// Shard name (for display/logging)
    pub name: String,
    /// Number of vectors in this shard
    pub vector_count: usize,
    /// Whether this shard is active
    pub active: bool,
    /// Shard weight (for load balancing)
    pub weight: f32,
}

/// Consistent hash ring for shard routing
#[derive(Debug, Clone)]
pub struct ConsistentHashRing {
    /// Virtual nodes per shard (for better distribution)
    virtual_nodes_per_shard: usize,
    /// Hash ring: hash -> shard_id
    ring: BTreeMap<u64, ShardId>,
    /// Shard metadata
    shards: HashMap<ShardId, ShardMetadata>,
}

impl ConsistentHashRing {
    /// Create a new consistent hash ring
    ///
    /// # Arguments
    /// * `shard_count` - Number of shards
    /// * `virtual_nodes_per_shard` - Number of virtual nodes per shard (default: 100)
    pub fn new(shard_count: u32, virtual_nodes_per_shard: usize) -> Result<Self> {
        if shard_count == 0 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Shard count must be greater than 0".to_string(),
            });
        }

        let mut ring = BTreeMap::new();
        let mut shards = HashMap::new();

        // Create shards and virtual nodes
        for shard_id in 0..shard_count {
            let shard = ShardId::new(shard_id);
            let metadata = ShardMetadata {
                id: shard,
                name: format!("shard_{}", shard_id),
                vector_count: 0,
                active: true,
                weight: 1.0,
            };
            shards.insert(shard, metadata);

            // Add virtual nodes to the ring
            for vnode in 0..virtual_nodes_per_shard {
                let hash = Self::hash_vnode(&shard, vnode);
                ring.insert(hash, shard);
            }
        }

        info!(
            "Created consistent hash ring with {} shards and {} virtual nodes",
            shard_count,
            shard_count as usize * virtual_nodes_per_shard
        );

        Ok(Self {
            virtual_nodes_per_shard,
            ring,
            shards,
        })
    }

    /// Hash a virtual node position
    fn hash_vnode(shard: &ShardId, vnode: usize) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        shard.hash(&mut hasher);
        vnode.hash(&mut hasher);
        hasher.finish()
    }

    /// Hash a vector ID to determine which shard it belongs to
    fn hash_vector_id(vector_id: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        vector_id.hash(&mut hasher);
        hasher.finish()
    }

    /// Find the shard for a given vector ID
    pub fn get_shard_for_vector(&self, vector_id: &str) -> ShardId {
        let hash = Self::hash_vector_id(vector_id);

        // Find the first shard with hash >= vector hash (circular)
        match self.ring.range(hash..).next() {
            Some((_, shard_id)) => *shard_id,
            None => {
                // Wrap around to the beginning of the ring
                self.ring
                    .iter()
                    .next()
                    .map(|(_, shard_id)| *shard_id)
                    .unwrap_or_else(|| ShardId::new(0))
            }
        }
    }

    /// Get all shards that should be queried for a search operation
    pub fn get_shards_for_search(&self, shard_keys: Option<&[ShardId]>) -> Vec<ShardId> {
        if let Some(keys) = shard_keys {
            // Filter to only active shards
            keys.iter()
                .filter(|id| self.shards.get(id).map(|s| s.active).unwrap_or(false))
                .copied()
                .collect()
        } else {
            // Return all active shards
            self.shards
                .values()
                .filter(|s| s.active)
                .map(|s| s.id)
                .collect()
        }
    }

    /// Add a new shard to the ring
    pub fn add_shard(&mut self, shard_id: ShardId, weight: f32) -> Result<()> {
        if self.shards.contains_key(&shard_id) {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!("Shard {} already exists", shard_id),
            });
        }

        let metadata = ShardMetadata {
            id: shard_id,
            name: format!("shard_{}", shard_id.as_u32()),
            vector_count: 0,
            active: true,
            weight,
        };
        self.shards.insert(shard_id, metadata);

        // Add virtual nodes
        for vnode in 0..self.virtual_nodes_per_shard {
            let hash = Self::hash_vnode(&shard_id, vnode);
            self.ring.insert(hash, shard_id);
        }

        info!("Added shard {} to ring", shard_id);
        Ok(())
    }

    /// Remove a shard from the ring
    pub fn remove_shard(&mut self, shard_id: ShardId) -> Result<()> {
        if !self.shards.contains_key(&shard_id) {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!("Shard {} does not exist", shard_id),
            });
        }

        // Remove virtual nodes
        let mut to_remove = Vec::new();
        for (hash, id) in &self.ring {
            if *id == shard_id {
                to_remove.push(*hash);
            }
        }
        for hash in to_remove {
            self.ring.remove(&hash);
        }

        self.shards.remove(&shard_id);
        info!("Removed shard {} from ring", shard_id);
        Ok(())
    }

    /// Get all shard IDs
    pub fn get_shard_ids(&self) -> Vec<ShardId> {
        self.shards.keys().copied().collect()
    }

    /// Get shard metadata
    pub fn get_shard_metadata(&self, shard_id: &ShardId) -> Option<&ShardMetadata> {
        self.shards.get(shard_id)
    }

    /// Update shard vector count
    pub fn update_shard_count(&mut self, shard_id: &ShardId, count: usize) {
        if let Some(metadata) = self.shards.get_mut(shard_id) {
            metadata.vector_count = count;
        }
    }

    /// Get total number of shards
    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }
}

/// Shard router for routing operations to appropriate shards
#[derive(Debug, Clone)]
pub struct ShardRouter {
    /// Consistent hash ring
    ring: Arc<RwLock<ConsistentHashRing>>,
    /// Collection name this router is for
    collection_name: String,
}

impl ShardRouter {
    /// Create a new shard router
    pub fn new(collection_name: String, shard_count: u32) -> Result<Self> {
        let ring = ConsistentHashRing::new(shard_count, 100)?;
        Ok(Self {
            ring: Arc::new(RwLock::new(ring)),
            collection_name,
        })
    }

    /// Get the shard for a vector ID
    pub fn route_vector(&self, vector_id: &str) -> ShardId {
        let ring = self.ring.read();
        ring.get_shard_for_vector(vector_id)
    }

    /// Get all shards for a search operation
    pub fn route_search(&self, shard_keys: Option<&[ShardId]>) -> Vec<ShardId> {
        let ring = self.ring.read();
        ring.get_shards_for_search(shard_keys)
    }

    /// Add a new shard
    pub fn add_shard(&self, shard_id: ShardId, weight: f32) -> Result<()> {
        let mut ring = self.ring.write();
        ring.add_shard(shard_id, weight)
    }

    /// Remove a shard
    pub fn remove_shard(&self, shard_id: ShardId) -> Result<()> {
        let mut ring = self.ring.write();
        ring.remove_shard(shard_id)
    }

    /// Get all shard IDs
    pub fn get_shard_ids(&self) -> Vec<ShardId> {
        let ring = self.ring.read();
        ring.get_shard_ids()
    }

    /// Get shard metadata
    pub fn get_shard_metadata(&self, shard_id: &ShardId) -> Option<ShardMetadata> {
        let ring = self.ring.read();
        ring.get_shard_metadata(shard_id).cloned()
    }

    /// Update shard vector count
    pub fn update_shard_count(&self, shard_id: &ShardId, count: usize) {
        let mut ring = self.ring.write();
        ring.update_shard_count(shard_id, count);
    }

    /// Get collection name
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }
}

/// Shard rebalancer for redistributing vectors when shards are added/removed
#[derive(Debug)]
pub struct ShardRebalancer {
    /// Router for the collection
    router: Arc<ShardRouter>,
    /// Rebalancing threshold (percentage of vectors that need to move)
    threshold: f32,
}

impl ShardRebalancer {
    /// Create a new shard rebalancer
    pub fn new(router: Arc<ShardRouter>, threshold: f32) -> Self {
        Self { router, threshold }
    }

    /// Calculate which vectors need to be moved when a shard is added
    pub fn calculate_moves_for_add(
        &self,
        vectors: &[(String, Vec<f32>)],
        new_shard_id: ShardId,
    ) -> Vec<(String, Vec<f32>, ShardId)> {
        let mut moves = Vec::new();

        for (vector_id, vector_data) in vectors {
            let old_shard = self.router.route_vector(vector_id);
            let new_shard = self.router.route_vector(vector_id);

            // If the shard changed, this vector needs to move
            if old_shard != new_shard {
                moves.push((vector_id.clone(), vector_data.clone(), new_shard));
            }
        }

        debug!(
            "Calculated {} vector moves for adding shard {}",
            moves.len(),
            new_shard_id
        );
        moves
    }

    /// Calculate which vectors need to be moved when a shard is removed
    pub fn calculate_moves_for_remove(
        &self,
        vectors: &[(String, Vec<f32>)],
        removed_shard_id: ShardId,
    ) -> Vec<(String, Vec<f32>, ShardId)> {
        let mut moves = Vec::new();

        for (vector_id, vector_data) in vectors {
            let current_shard = self.router.route_vector(vector_id);

            // If this vector is on the removed shard, it needs to move
            if current_shard == removed_shard_id {
                // Find new shard (will be recalculated after removal)
                let new_shard = self.router.route_vector(vector_id);
                moves.push((vector_id.clone(), vector_data.clone(), new_shard));
            }
        }

        debug!(
            "Calculated {} vector moves for removing shard {}",
            moves.len(),
            removed_shard_id
        );
        moves
    }

    /// Check if rebalancing is needed based on shard sizes
    pub fn needs_rebalancing(&self, shard_counts: &HashMap<ShardId, usize>) -> bool {
        if shard_counts.is_empty() {
            return false;
        }

        let total: usize = shard_counts.values().sum();
        if total == 0 {
            return false;
        }

        let avg = total as f32 / shard_counts.len() as f32;
        let max = *shard_counts.values().max().unwrap_or(&0) as f32;
        let min = *shard_counts.values().min().unwrap_or(&0) as f32;

        // Check if any shard deviates significantly from average
        let max_deviation = (max - avg) / avg;
        let min_deviation = (avg - min) / avg;

        max_deviation > self.threshold || min_deviation > self.threshold
    }

    /// Calculate rebalancing moves to balance shard sizes
    pub fn calculate_balance_moves(
        &self,
        vectors: &[(String, Vec<f32>)],
        shard_counts: &HashMap<ShardId, usize>,
    ) -> Vec<(String, Vec<f32>, ShardId, ShardId)> {
        let mut moves = Vec::new();

        if shard_counts.is_empty() {
            return moves;
        }

        let total: usize = shard_counts.values().sum();
        if total == 0 {
            return moves;
        }

        let target_per_shard = total as f32 / shard_counts.len() as f32;
        let threshold = target_per_shard * self.threshold;

        // Find overloaded and underloaded shards
        let mut overloaded: Vec<(ShardId, usize)> = shard_counts
            .iter()
            .filter(|(_, count)| **count as f32 > target_per_shard + threshold)
            .map(|(id, count)| (*id, *count))
            .collect();
        overloaded.sort_by(|a, b| b.1.cmp(&a.1));

        let mut underloaded: Vec<(ShardId, usize)> = shard_counts
            .iter()
            .filter(|(_, count)| (**count as f32) < (target_per_shard - threshold))
            .map(|(id, count)| (*id, *count))
            .collect();
        underloaded.sort_by(|a, b| a.1.cmp(&b.1));

        // Move vectors from overloaded to underloaded shards
        let mut overloaded_idx = 0;
        let mut underloaded_idx = 0;

        for (vector_id, vector_data) in vectors {
            let current_shard = self.router.route_vector(vector_id);

            // Check if this vector is on an overloaded shard
            if let Some((shard_id, _)) = overloaded.get(overloaded_idx) {
                if *shard_id == current_shard {
                    // Find an underloaded shard to move to
                    if let Some((target_shard, _)) = underloaded.get(underloaded_idx) {
                        moves.push((
                            vector_id.clone(),
                            vector_data.clone(),
                            current_shard,
                            *target_shard,
                        ));

                        // Update counts (simplified - in real implementation would track more carefully)
                        underloaded_idx = (underloaded_idx + 1) % underloaded.len();
                    }
                }
            }
        }

        debug!("Calculated {} rebalancing moves", moves.len());
        moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_hash_ring_creation() {
        let ring = ConsistentHashRing::new(4, 100).unwrap();
        assert_eq!(ring.shard_count(), 4);
        assert_eq!(ring.ring.len(), 400); // 4 shards * 100 virtual nodes
    }

    #[test]
    fn test_vector_routing() {
        let ring = ConsistentHashRing::new(4, 100).unwrap();
        let shard1 = ring.get_shard_for_vector("vector1");
        let shard2 = ring.get_shard_for_vector("vector2");
        let shard1_again = ring.get_shard_for_vector("vector1");

        // Same vector should route to same shard
        assert_eq!(shard1, shard1_again);
        // Different vectors might route to different shards
        // (not guaranteed, but likely)
    }

    #[test]
    fn test_shard_addition() {
        let mut ring = ConsistentHashRing::new(4, 100).unwrap();
        let new_shard = ShardId::new(4);

        ring.add_shard(new_shard, 1.0).unwrap();
        assert_eq!(ring.shard_count(), 5);
        assert!(ring.get_shard_metadata(&new_shard).is_some());
    }

    #[test]
    fn test_shard_removal() {
        let mut ring = ConsistentHashRing::new(4, 100).unwrap();
        let shard_to_remove = ShardId::new(2);

        ring.remove_shard(shard_to_remove).unwrap();
        assert_eq!(ring.shard_count(), 3);
        assert!(ring.get_shard_metadata(&shard_to_remove).is_none());
    }

    #[test]
    fn test_shard_router() {
        let router = ShardRouter::new("test_collection".to_string(), 4).unwrap();
        let shard = router.route_vector("test_vector");
        assert!(shard.as_u32() < 4);
    }

    #[test]
    fn test_rebalancer_needs_rebalancing() {
        let router = Arc::new(ShardRouter::new("test".to_string(), 4).unwrap());
        let rebalancer = ShardRebalancer::new(router, 0.2);

        // Balanced shards
        let mut balanced = HashMap::new();
        balanced.insert(ShardId::new(0), 100);
        balanced.insert(ShardId::new(1), 100);
        balanced.insert(ShardId::new(2), 100);
        balanced.insert(ShardId::new(3), 100);
        assert!(!rebalancer.needs_rebalancing(&balanced));

        // Unbalanced shards
        let mut unbalanced = HashMap::new();
        unbalanced.insert(ShardId::new(0), 200);
        unbalanced.insert(ShardId::new(1), 50);
        unbalanced.insert(ShardId::new(2), 50);
        unbalanced.insert(ShardId::new(3), 50);
        assert!(rebalancer.needs_rebalancing(&unbalanced));
    }
}
