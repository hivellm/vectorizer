//! Sharded collection implementation for distributed vector storage
//!
//! This module provides a sharded collection wrapper that distributes vectors
//! across multiple shards and handles multi-shard queries.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

use super::collection::Collection;
use super::sharding::{ShardId, ShardRebalancer, ShardRouter};
use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, SearchResult, Vector};

/// A sharded collection that distributes vectors across multiple shards
#[derive(Clone, Debug)]
pub struct ShardedCollection {
    /// Collection name
    name: String,
    /// Base collection configuration
    config: CollectionConfig,
    /// Owner ID (tenant/user ID for multi-tenancy in HiveHub cluster mode)
    owner_id: Option<uuid::Uuid>,
    /// Shard router for routing vectors to shards
    router: Arc<ShardRouter>,
    /// Individual shard collections (shard_id -> Collection)
    shards: Arc<DashMap<ShardId, Collection>>,
    /// Rebalancer for redistributing vectors
    rebalancer: Arc<ShardRebalancer>,
}

impl ShardedCollection {
    /// Create a new sharded collection
    ///
    /// # Arguments
    /// * `name` - Collection name
    /// * `config` - Collection configuration (must have sharding enabled)
    pub fn new(name: String, config: CollectionConfig) -> Result<Self> {
        let sharding_config =
            config
                .sharding
                .as_ref()
                .ok_or_else(|| VectorizerError::InvalidConfiguration {
                    message: "Collection config must have sharding enabled".to_string(),
                })?;

        // Create router
        let router = Arc::new(ShardRouter::new(name.clone(), sharding_config.shard_count)?);

        // Create rebalancer
        let rebalancer = Arc::new(ShardRebalancer::new(
            router.clone(),
            sharding_config.rebalance_threshold,
        ));

        // Create shard collections
        let shards = Arc::new(DashMap::new());
        let shard_ids = router.get_shard_ids();

        for shard_id in shard_ids {
            // Create a collection for each shard with the same config (but no sharding)
            let mut shard_config = config.clone();
            shard_config.sharding = None; // Shards themselves are not sharded

            let shard_name = format!("{}_{}", name, shard_id);
            let shard_collection = Collection::new(shard_name, shard_config);
            shards.insert(shard_id, shard_collection);
        }

        info!(
            "Created sharded collection '{}' with {} shards",
            name,
            shards.len()
        );

        Ok(Self {
            name,
            config,
            owner_id: None,
            router,
            shards,
            rebalancer,
        })
    }

    /// Get collection name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get collection configuration
    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }

    /// Get the owner ID (tenant/user ID for multi-tenancy)
    pub fn owner_id(&self) -> Option<uuid::Uuid> {
        self.owner_id
    }

    /// Set the owner ID (used when loading from persistence or updating ownership)
    pub fn set_owner_id(&mut self, owner_id: Option<uuid::Uuid>) {
        self.owner_id = owner_id;
    }

    /// Check if this collection belongs to a specific owner
    pub fn belongs_to(&self, owner_id: &uuid::Uuid) -> bool {
        self.owner_id.map(|id| &id == owner_id).unwrap_or(false)
    }

    /// Insert a vector into the appropriate shard
    pub fn insert(&self, vector: Vector) -> Result<()> {
        let shard_id = self.router.route_vector(&vector.id);
        let shard = self
            .shards
            .get(&shard_id)
            .ok_or_else(|| VectorizerError::Storage(format!("Shard {} not found", shard_id)))?;

        let vector_id = vector.id.clone();
        shard.insert(vector)?;

        // Update shard count in router
        let count = shard.vector_count();
        self.router.update_shard_count(&shard_id, count);

        debug!(
            "Inserted vector '{}' into shard {} of collection '{}'",
            vector_id, shard_id, self.name
        );

        Ok(())
    }

    /// Insert multiple vectors (batch operation)
    pub fn insert_batch(&self, vectors: Vec<Vector>) -> Result<()> {
        // Group vectors by shard
        let mut shard_vectors: HashMap<ShardId, Vec<Vector>> = HashMap::new();

        for vector in vectors {
            let shard_id = self.router.route_vector(&vector.id);
            shard_vectors
                .entry(shard_id)
                .or_insert_with(Vec::new)
                .push(vector);
        }

        let shard_count = shard_vectors.len();

        // Insert into each shard
        for (shard_id, vectors) in shard_vectors {
            let shard = self
                .shards
                .get(&shard_id)
                .ok_or_else(|| VectorizerError::Storage(format!("Shard {} not found", shard_id)))?;

            for vector in vectors {
                shard.insert(vector)?;
            }

            // Update shard count
            let count = shard.vector_count();
            self.router.update_shard_count(&shard_id, count);
        }

        debug!(
            "Inserted batch of vectors into {} shards of collection '{}'",
            shard_count, self.name
        );

        Ok(())
    }

    /// Update a vector (must be in the same shard)
    pub fn update(&self, vector: Vector) -> Result<()> {
        let shard_id = self.router.route_vector(&vector.id);
        let shard = self
            .shards
            .get(&shard_id)
            .ok_or_else(|| VectorizerError::Storage(format!("Shard {} not found", shard_id)))?;

        shard.update(vector)?;
        Ok(())
    }

    /// Delete a vector from the appropriate shard
    pub fn delete(&self, vector_id: &str) -> Result<()> {
        let shard_id = self.router.route_vector(vector_id);
        let shard = self
            .shards
            .get(&shard_id)
            .ok_or_else(|| VectorizerError::Storage(format!("Shard {} not found", shard_id)))?;

        shard.delete(vector_id)?;

        // Update shard count
        let count = shard.vector_count();
        self.router.update_shard_count(&shard_id, count);

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        let shard_id = self.router.route_vector(vector_id);
        let shard = self
            .shards
            .get(&shard_id)
            .ok_or_else(|| VectorizerError::Storage(format!("Shard {} not found", shard_id)))?;

        shard.get_vector(vector_id)
    }

    /// Search across all shards and merge results
    ///
    /// # Arguments
    /// * `query_vector` - Query vector
    /// * `k` - Number of results to return
    /// * `shard_keys` - Optional list of specific shards to search (if None, searches all)
    pub fn search(
        &self,
        query_vector: &[f32],
        k: usize,
        shard_keys: Option<&[ShardId]>,
    ) -> Result<Vec<SearchResult>> {
        // Get shards to search
        let shard_ids = self.router.route_search(shard_keys);

        if shard_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Search each shard
        let mut all_results = Vec::new();
        let shard_count = shard_ids.len();

        for shard_id in shard_ids {
            if let Some(shard) = self.shards.get(&shard_id) {
                match shard.search(query_vector, k) {
                    Ok(results) => {
                        all_results.extend(results);
                    }
                    Err(e) => {
                        warn!("Error searching shard {}: {}", shard_id, e);
                        // Continue with other shards
                    }
                }
            }
        }

        // Merge and sort results by score (higher is better for similarity)
        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top k results
        all_results.truncate(k);

        debug!(
            "Multi-shard search in collection '{}' returned {} results from {} shards",
            self.name,
            all_results.len(),
            shard_count
        );

        Ok(all_results)
    }

    /// Get total vector count across all shards
    pub fn vector_count(&self) -> usize {
        self.shards
            .iter()
            .map(|shard| shard.value().vector_count())
            .sum()
    }

    /// Get vector count per shard
    pub fn shard_counts(&self) -> HashMap<ShardId, usize> {
        self.shards
            .iter()
            .map(|entry| (*entry.key(), entry.value().vector_count()))
            .collect()
    }

    /// Check if rebalancing is needed
    pub fn needs_rebalancing(&self) -> bool {
        let counts = self.shard_counts();
        self.rebalancer.needs_rebalancing(&counts)
    }

    /// Add a new shard to the collection
    pub fn add_shard(&self, shard_id: ShardId, weight: f32) -> Result<()> {
        // Add to router
        self.router.add_shard(shard_id, weight)?;

        // Create new shard collection
        let mut shard_config = self.config.clone();
        shard_config.sharding = None;

        let shard_name = format!("{}_{}", self.name, shard_id);
        let shard_collection = Collection::new(shard_name, shard_config);
        self.shards.insert(shard_id, shard_collection);

        info!("Added shard {} to collection '{}'", shard_id, self.name);
        Ok(())
    }

    /// Remove a shard from the collection
    ///
    /// # Warning
    /// This will delete all vectors in the shard. Consider rebalancing first.
    pub fn remove_shard(&self, shard_id: ShardId) -> Result<()> {
        // Remove from router
        self.router.remove_shard(shard_id)?;

        // Remove shard collection
        self.shards.remove(&shard_id);

        info!("Removed shard {} from collection '{}'", shard_id, self.name);
        Ok(())
    }

    /// Get all shard IDs
    pub fn get_shard_ids(&self) -> Vec<ShardId> {
        self.router.get_shard_ids()
    }

    /// Get shard metadata
    pub fn get_shard_metadata(&self, shard_id: &ShardId) -> Option<super::sharding::ShardMetadata> {
        self.router.get_shard_metadata(shard_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DistanceMetric, HnswConfig, QuantizationConfig};

    fn create_test_config() -> CollectionConfig {
        CollectionConfig {
            graph: None,
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::None,
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
            storage_type: None,
            sharding: Some(crate::models::ShardingConfig {
                shard_count: 4,
                virtual_nodes_per_shard: 10, // Lower for tests
                rebalance_threshold: 0.2,
            }),
        }
    }

    #[test]
    fn test_sharded_collection_creation() {
        let config = create_test_config();
        let collection = ShardedCollection::new("test".to_string(), config).unwrap();
        assert_eq!(collection.name(), "test");
        assert_eq!(collection.get_shard_ids().len(), 4);
    }

    #[test]
    fn test_sharded_insert_and_search() {
        let config = create_test_config();
        let collection = ShardedCollection::new("test".to_string(), config).unwrap();

        // Insert test vectors
        for i in 0..10 {
            let vector = Vector {
                id: format!("vec_{}", i),
                data: vec![1.0; 128],
                sparse: None,
                payload: None,
            };
            collection.insert(vector).unwrap();
        }

        assert_eq!(collection.vector_count(), 10);

        // Search
        let query = vec![1.0; 128];
        let results = collection.search(&query, 5, None).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_shard_routing() {
        let config = create_test_config();
        let collection = ShardedCollection::new("test".to_string(), config).unwrap();

        // Insert vectors with different IDs
        let vec1 = Vector {
            id: "vector_1".to_string(),
            data: vec![1.0; 128],
            sparse: None,
            payload: None,
        };
        let vec2 = Vector {
            id: "vector_2".to_string(),
            data: vec![1.0; 128],
            sparse: None,
            payload: None,
        };

        collection.insert(vec1.clone()).unwrap();
        collection.insert(vec2.clone()).unwrap();

        // Both should be retrievable
        assert!(collection.get_vector("vector_1").is_ok());
        assert!(collection.get_vector("vector_2").is_ok());
    }
}
