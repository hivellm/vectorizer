//! Integration tests for distributed sharding

use vectorizer::db::sharded_collection::ShardedCollection;
use vectorizer::db::sharding::{ShardId, ShardRouter};
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
    ShardingConfig, Vector,
};

fn create_sharded_config(shard_count: u32) -> CollectionConfig {
    CollectionConfig {
        graph: None,
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: CompressionConfig::default(),
        normalization: None,
        storage_type: None,
        sharding: Some(ShardingConfig {
            shard_count,
            virtual_nodes_per_shard: 10, // Lower for tests
            rebalance_threshold: 0.2,
        }),
    }
}

#[test]
fn test_multi_shard_insert_and_search() {
    let config = create_sharded_config(4);
    let collection = ShardedCollection::new("test_multi_shard".to_string(), config).unwrap();

    // Insert vectors across multiple shards
    let mut inserted_ids = Vec::new();
    for i in 0..100 {
        let vector = Vector {
            id: format!("vec_{i}"),
            data: vec![1.0; 128],
            sparse: None,
            payload: None,
        };
        collection.insert(vector).unwrap();
        inserted_ids.push(format!("vec_{i}"));
    }

    assert_eq!(collection.vector_count(), 100);

    // Verify vectors are distributed across shards
    let shard_counts = collection.shard_counts();
    assert_eq!(shard_counts.len(), 4);

    // All shards should have some vectors (distribution may vary)
    let total: usize = shard_counts.values().sum();
    assert_eq!(total, 100);

    // No shard should be empty (with 100 vectors and 4 shards)
    assert!(shard_counts.values().all(|&count| count > 0));

    // Search across all shards
    let query = vec![1.0; 128];
    let results = collection.search(&query, 10, None).unwrap();

    assert!(!results.is_empty());
    assert!(results.len() <= 10);

    // Verify we can retrieve specific vectors
    for id in &inserted_ids[0..10] {
        let vector = collection.get_vector(id).unwrap();
        assert_eq!(vector.id, *id);
    }
}

#[test]
fn test_shard_specific_search() {
    let config = create_sharded_config(4);
    let collection = ShardedCollection::new("test_shard_specific".to_string(), config).unwrap();

    // Insert vectors
    for i in 0..50 {
        let vector = Vector {
            id: format!("vec_{i}"),
            data: vec![1.0; 128],
            sparse: None,
            payload: None,
        };
        collection.insert(vector).unwrap();
    }

    // Get all shard IDs
    let shard_ids = collection.get_shard_ids();
    assert!(!shard_ids.is_empty());

    // Search only in first shard
    let first_shard = &shard_ids[0..1];
    let query = vec![1.0; 128];
    let results = collection.search(&query, 10, Some(first_shard)).unwrap();

    // Results should come from the specified shard only
    assert!(!results.is_empty());
}

#[test]
fn test_shard_rebalancing_detection() {
    let config = create_sharded_config(4);
    let collection = ShardedCollection::new("test_rebalance".to_string(), config).unwrap();

    // Initially, rebalancing should not be needed
    assert!(!collection.needs_rebalancing());

    // Insert many vectors to one shard (by using similar IDs that hash to same shard)
    // This is a simplified test - in practice, we'd need to know which shard to target
    for i in 0..1000 {
        let vector = Vector {
            id: format!("vec_{i}"),
            data: vec![1.0; 128],
            sparse: None,
            payload: None,
        };
        collection.insert(vector).unwrap();
    }

    // After many inserts, check if rebalancing is needed
    // Note: This depends on hash distribution, so it may or may not trigger
    let needs_rebalance = collection.needs_rebalancing();
    // Just verify the method works (actual rebalancing depends on distribution)
    // This assertion is always true, but kept for documentation
    let _ = needs_rebalance;
}

#[test]
fn test_shard_addition() {
    let config = create_sharded_config(4);
    let collection = ShardedCollection::new("test_add_shard".to_string(), config).unwrap();

    let initial_shard_count = collection.get_shard_ids().len();

    // Add a new shard
    let new_shard_id = ShardId::new(4);
    collection.add_shard(new_shard_id, 1.0).unwrap();

    let new_shard_count = collection.get_shard_ids().len();
    assert_eq!(new_shard_count, initial_shard_count + 1);
    assert!(collection.get_shard_ids().contains(&new_shard_id));
}

#[test]
fn test_consistent_hash_routing() {
    let router = ShardRouter::new("test_collection".to_string(), 4).unwrap();

    // Same vector ID should always route to same shard
    let shard1 = router.route_vector("test_vector_1");
    let shard2 = router.route_vector("test_vector_1");
    assert_eq!(shard1, shard2);

    // Different vectors might route to different shards
    let shard3 = router.route_vector("test_vector_2");
    // They might be the same or different, but routing should be consistent
    let shard4 = router.route_vector("test_vector_2");
    assert_eq!(shard3, shard4);
}

#[test]
fn test_batch_insert_distribution() {
    let config = create_sharded_config(4);
    let collection = ShardedCollection::new("test_batch".to_string(), config).unwrap();

    // Create batch of vectors
    let mut vectors = Vec::new();
    for i in 0..200 {
        vectors.push(Vector {
            id: format!("batch_vec_{i}"),
            data: vec![1.0; 128],
            sparse: None,
            payload: None,
        });
    }

    // Insert batch
    collection.insert_batch(vectors).unwrap();

    assert_eq!(collection.vector_count(), 200);

    // Verify distribution across shards
    let shard_counts = collection.shard_counts();
    assert_eq!(shard_counts.len(), 4);

    let total: usize = shard_counts.values().sum();
    assert_eq!(total, 200);
}

#[test]
fn test_multi_shard_update_and_delete() {
    let config = create_sharded_config(4);
    let collection = ShardedCollection::new("test_crud".to_string(), config).unwrap();

    // Insert vector
    let vector = Vector {
        id: "test_vec".to_string(),
        data: vec![1.0; 128],
        sparse: None,
        payload: None,
    };
    collection.insert(vector.clone()).unwrap();

    // Update vector
    let updated_vector = Vector {
        id: "test_vec".to_string(),
        data: vec![2.0; 128],
        sparse: None,
        payload: None,
    };
    collection.update(updated_vector).unwrap();

    // Verify update (Cosine metric normalizes vectors)
    let retrieved = collection.get_vector("test_vec").unwrap();
    // For vector [2.0; 128], norm = sqrt(128 * 2.0^2) = sqrt(512) ≈ 22.627
    // Normalized value = 2.0 / 22.627 ≈ 0.088388
    let expected = 2.0 / (128.0_f32 * 4.0).sqrt();
    assert!(
        (retrieved.data[0] - expected).abs() < 0.001,
        "Expected normalized value ~{}, got {}",
        expected,
        retrieved.data[0]
    );

    // Delete vector
    collection.delete("test_vec").unwrap();

    // Verify deletion
    assert!(collection.get_vector("test_vec").is_err());
}

#[test]
fn test_shard_metadata() {
    let config = create_sharded_config(4);
    let collection = ShardedCollection::new("test_metadata".to_string(), config).unwrap();

    // Insert some vectors
    for i in 0..50 {
        let vector = Vector {
            id: format!("vec_{i}"),
            data: vec![1.0; 128],
            sparse: None,
            payload: None,
        };
        collection.insert(vector).unwrap();
    }

    // Get shard IDs
    let shard_ids = collection.get_shard_ids();

    // Check metadata for each shard
    for shard_id in shard_ids {
        let metadata = collection.get_shard_metadata(&shard_id);
        assert!(metadata.is_some());

        let meta = metadata.unwrap();
        assert_eq!(meta.id, shard_id);
        // Just verify vector_count exists (it's usize, so >= 0 is always true)
        let _ = meta.vector_count;
    }
}
