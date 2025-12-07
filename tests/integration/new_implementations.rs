//! Integration tests for new implementations
//!
//! Tests for:
//! - Distributed batch insert
//! - Sharded hybrid search
//! - Document count tracking
//! - API request tracking
//! - Per-key rate limiting

// ============================================================================
// Document Count Tracking Tests
// ============================================================================

#[cfg(test)]
mod document_count_tests {
    use vectorizer::db::sharded_collection::ShardedCollection;
    use vectorizer::models::{CollectionConfig, ShardingConfig, Vector};

    fn create_sharding_config(shard_count: u32) -> ShardingConfig {
        ShardingConfig {
            shard_count,
            virtual_nodes_per_shard: 100,
            rebalance_threshold: 0.2,
        }
    }

    fn create_vector(id: &str, data: Vec<f32>) -> Vector {
        Vector {
            id: id.to_string(),
            data,
            sparse: None,
            payload: None,
        }
    }

    #[test]
    fn test_sharded_collection_document_count() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("test_sharded_doc_count".to_string(), config)
            .expect("Failed to create sharded collection");

        // Initially should be 0
        assert_eq!(collection.document_count(), 0);

        // Insert some vectors
        for i in 0..10 {
            let vector = create_vector(&format!("vec_{i}"), vec![i as f32, 0.0, 0.0, 0.0]);
            collection.insert(vector).unwrap();
        }

        // Vector count should be 10
        assert_eq!(collection.vector_count(), 10);
    }

    #[test]
    fn test_sharded_collection_document_count_aggregation() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(4)),
            ..Default::default()
        };

        let collection =
            ShardedCollection::new("test_doc_aggregation".to_string(), config).unwrap();

        // Insert vectors that will be distributed across shards
        for i in 0..100 {
            let vector = create_vector(&format!("vec_{i}"), vec![i as f32 / 100.0, 0.0, 0.0, 0.0]);
            collection.insert(vector).unwrap();
        }

        // Total vector count should be 100
        assert_eq!(collection.vector_count(), 100);

        // Shard counts should sum to total
        let shard_counts = collection.shard_counts();
        let sum: usize = shard_counts.values().sum();
        assert_eq!(sum, 100);
    }
}

// ============================================================================
// Sharded Hybrid Search Tests
// ============================================================================

#[cfg(test)]
mod sharded_hybrid_search_tests {
    use vectorizer::db::HybridSearchConfig;
    use vectorizer::db::sharded_collection::ShardedCollection;
    use vectorizer::models::{CollectionConfig, ShardingConfig, Vector};

    fn create_sharding_config(shard_count: u32) -> ShardingConfig {
        ShardingConfig {
            shard_count,
            virtual_nodes_per_shard: 100,
            rebalance_threshold: 0.2,
        }
    }

    fn create_vector(id: &str, data: Vec<f32>) -> Vector {
        Vector {
            id: id.to_string(),
            data,
            sparse: None,
            payload: None,
        }
    }

    #[test]
    fn test_sharded_hybrid_search_basic() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("test_hybrid_sharded".to_string(), config).unwrap();

        // Insert test vectors
        for i in 0..20 {
            let vector = create_vector(&format!("vec_{i}"), vec![i as f32 / 20.0, 0.5, 0.3, 0.1]);
            collection.insert(vector).unwrap();
        }

        // Perform hybrid search
        let query = vec![0.5, 0.5, 0.3, 0.1];
        let hybrid_config = HybridSearchConfig {
            dense_k: 10,
            sparse_k: 10,
            final_k: 5,
            alpha: 0.5,
            ..Default::default()
        };

        let results = collection.hybrid_search(&query, None, hybrid_config, None);

        // Should return results
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_sharded_hybrid_search_empty_collection() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("test_hybrid_empty".to_string(), config).unwrap();

        let query = vec![0.5, 0.5, 0.5, 0.5];
        let hybrid_config = HybridSearchConfig::default();

        let results = collection.hybrid_search(&query, None, hybrid_config, None);

        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[test]
    fn test_sharded_hybrid_search_result_ordering() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(4)),
            ..Default::default()
        };

        let collection =
            ShardedCollection::new("test_hybrid_ordering".to_string(), config).unwrap();

        // Insert vectors
        for i in 0..50 {
            let vector = create_vector(&format!("vec_{i}"), vec![i as f32 / 50.0, 0.2, 0.3, 0.4]);
            collection.insert(vector).unwrap();
        }

        let query = vec![0.5, 0.2, 0.3, 0.4];
        let hybrid_config = HybridSearchConfig {
            dense_k: 20,
            sparse_k: 20,
            final_k: 10,
            alpha: 0.7,
            ..Default::default()
        };

        let results = collection
            .hybrid_search(&query, None, hybrid_config, None)
            .unwrap();

        // Results should be sorted by score (descending)
        for i in 1..results.len() {
            assert!(
                results[i - 1].score >= results[i].score,
                "Results should be sorted by score descending"
            );
        }
    }
}

// ============================================================================
// Rate Limiting Tests
// ============================================================================

#[cfg(test)]
mod rate_limiting_tests {
    use vectorizer::security::rate_limit::{RateLimitConfig, RateLimiter};

    #[test]
    fn test_per_key_rate_limiter_creation() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 20,
        };
        let limiter = RateLimiter::new(config);

        // First request should pass
        assert!(limiter.check_key("api_key_1"));
    }

    #[test]
    fn test_per_key_rate_limiter_isolation() {
        let config = RateLimitConfig {
            requests_per_second: 5,
            burst_size: 5,
        };
        let limiter = RateLimiter::new(config);

        // Exhaust key1's limit
        for _ in 0..5 {
            limiter.check_key("key1");
        }

        // key2 should still work (isolated rate limiting)
        assert!(limiter.check_key("key2"));
    }

    #[test]
    fn test_combined_rate_limit_check() {
        let config = RateLimitConfig {
            requests_per_second: 100,
            burst_size: 200,
        };
        let limiter = RateLimiter::new(config);

        // Combined check with API key
        assert!(limiter.check(Some("test_api_key")));

        // Combined check without API key (global only)
        assert!(limiter.check(None));
    }

    #[test]
    fn test_rate_limiter_default_config() {
        let limiter = RateLimiter::default();

        // Default should allow requests
        assert!(limiter.check_global());
        assert!(limiter.check_key("any_key"));
    }

    #[test]
    fn test_rate_limiter_burst_capacity() {
        let config = RateLimitConfig {
            requests_per_second: 1,
            burst_size: 10,
        };
        let limiter = RateLimiter::new(config);

        // Should allow burst of 10 requests
        let mut allowed = 0;
        for _ in 0..15 {
            if limiter.check_key("burst_test_key") {
                allowed += 1;
            }
        }

        // Should have allowed at least the burst size
        assert!(allowed >= 10);
    }

    #[test]
    fn test_rate_limiter_multiple_keys() {
        let config = RateLimitConfig {
            requests_per_second: 100,
            burst_size: 100,
        };
        let limiter = RateLimiter::new(config);

        // Test multiple keys
        for i in 0..10 {
            let key = format!("key_{i}");
            assert!(limiter.check_key(&key));
        }
    }
}

// ============================================================================
// API Request Tracking Tests
// ============================================================================

#[cfg(test)]
mod api_request_tracking_tests {
    use vectorizer::monitoring::metrics::METRICS;

    #[test]
    fn test_tenant_api_request_recording() {
        let tenant_id = "test_tenant_unique_123";

        // Get initial count
        let initial_count = METRICS.get_tenant_api_requests(tenant_id);

        // Record some requests
        METRICS.record_tenant_api_request(tenant_id);
        METRICS.record_tenant_api_request(tenant_id);
        METRICS.record_tenant_api_request(tenant_id);

        // Verify count increased
        let new_count = METRICS.get_tenant_api_requests(tenant_id);
        assert_eq!(new_count, initial_count + 3);
    }

    #[test]
    fn test_tenant_api_request_isolation() {
        let tenant1 = "isolated_tenant_a";
        let tenant2 = "isolated_tenant_b";

        let initial1 = METRICS.get_tenant_api_requests(tenant1);
        let initial2 = METRICS.get_tenant_api_requests(tenant2);

        // Record requests for tenant1 only
        METRICS.record_tenant_api_request(tenant1);
        METRICS.record_tenant_api_request(tenant1);

        let final1 = METRICS.get_tenant_api_requests(tenant1);
        let final2 = METRICS.get_tenant_api_requests(tenant2);

        // tenant1 should have 2 more, tenant2 should be unchanged
        assert_eq!(final1, initial1 + 2);
        assert_eq!(final2, initial2);
    }

    #[test]
    fn test_tenant_api_request_nonexistent() {
        let nonexistent_tenant = "nonexistent_tenant_xyz_unique_12345";

        // Should return 0 for nonexistent tenant
        let count = METRICS.get_tenant_api_requests(nonexistent_tenant);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_tenant_api_request_concurrent() {
        use std::thread;

        let tenant_id = "concurrent_tenant_test";
        let initial = METRICS.get_tenant_api_requests(tenant_id);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let tid = tenant_id.to_string();
                thread::spawn(move || {
                    for _ in 0..100 {
                        METRICS.record_tenant_api_request(&tid);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let final_count = METRICS.get_tenant_api_requests(tenant_id);
        assert_eq!(final_count, initial + 1000);
    }
}

// ============================================================================
// Batch Insert Tests
// ============================================================================

#[cfg(test)]
mod batch_insert_tests {
    use vectorizer::db::sharded_collection::ShardedCollection;
    use vectorizer::models::{CollectionConfig, ShardingConfig, Vector};

    fn create_sharding_config(shard_count: u32) -> ShardingConfig {
        ShardingConfig {
            shard_count,
            virtual_nodes_per_shard: 100,
            rebalance_threshold: 0.2,
        }
    }

    fn create_vector(id: &str, data: Vec<f32>) -> Vector {
        Vector {
            id: id.to_string(),
            data,
            sparse: None,
            payload: None,
        }
    }

    #[test]
    fn test_sharded_batch_insert() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(4)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("test_batch_insert".to_string(), config).unwrap();

        // Create batch of vectors
        let vectors: Vec<Vector> = (0..100)
            .map(|i| {
                create_vector(
                    &format!("batch_vec_{i}"),
                    vec![i as f32 / 100.0, 0.5, 0.3, 0.1],
                )
            })
            .collect();

        // Batch insert
        let result = collection.insert_batch(vectors);
        assert!(result.is_ok());

        // Verify all vectors were inserted
        assert_eq!(collection.vector_count(), 100);
    }

    #[test]
    fn test_sharded_batch_insert_distribution() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(4)),
            ..Default::default()
        };

        let collection =
            ShardedCollection::new("test_batch_distribution".to_string(), config).unwrap();

        // Create batch
        let vectors: Vec<Vector> = (0..1000)
            .map(|i| {
                create_vector(
                    &format!("dist_vec_{i}"),
                    vec![i as f32 / 1000.0, 0.2, 0.3, 0.4],
                )
            })
            .collect();

        collection.insert_batch(vectors).unwrap();

        // Check distribution across shards
        let shard_counts = collection.shard_counts();
        assert_eq!(shard_counts.len(), 4);

        // Each shard should have some vectors (not all in one)
        for count in shard_counts.values() {
            assert!(*count > 0, "Each shard should have vectors");
        }

        // Total should be 1000
        let total: usize = shard_counts.values().sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn test_sharded_batch_insert_empty() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("test_batch_empty".to_string(), config).unwrap();

        // Empty batch insert
        let result = collection.insert_batch(vec![]);
        assert!(result.is_ok());
        assert_eq!(collection.vector_count(), 0);
    }

    #[test]
    fn test_sharded_batch_insert_single() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("test_batch_single".to_string(), config).unwrap();

        let vectors = vec![create_vector("single_vec", vec![1.0, 0.0, 0.0, 0.0])];

        let result = collection.insert_batch(vectors);
        assert!(result.is_ok());
        assert_eq!(collection.vector_count(), 1);
    }

    #[test]
    fn test_sharded_batch_insert_large() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(8)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("test_batch_large".to_string(), config).unwrap();

        // Insert 10000 vectors in batch
        let vectors: Vec<Vector> = (0..10000)
            .map(|i| {
                create_vector(
                    &format!("large_vec_{i}"),
                    vec![
                        (i % 100) as f32 / 100.0,
                        (i % 50) as f32 / 50.0,
                        (i % 25) as f32 / 25.0,
                        (i % 10) as f32 / 10.0,
                    ],
                )
            })
            .collect();

        let result = collection.insert_batch(vectors);
        assert!(result.is_ok());
        assert_eq!(collection.vector_count(), 10000);
    }
}

// ============================================================================
// Collection Metadata Tests
// ============================================================================

#[cfg(test)]
mod collection_metadata_tests {
    use vectorizer::db::sharded_collection::ShardedCollection;
    use vectorizer::models::{CollectionConfig, ShardingConfig};

    fn create_sharding_config(shard_count: u32) -> ShardingConfig {
        ShardingConfig {
            shard_count,
            virtual_nodes_per_shard: 100,
            rebalance_threshold: 0.2,
        }
    }

    #[test]
    fn test_sharded_collection_name() {
        let config = CollectionConfig {
            dimension: 8,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let collection =
            ShardedCollection::new("my_test_collection".to_string(), config.clone()).unwrap();

        assert_eq!(collection.name(), "my_test_collection");
        assert_eq!(collection.config().dimension, 8);
    }

    #[test]
    fn test_sharded_collection_config() {
        let config = CollectionConfig {
            dimension: 128,
            sharding: Some(ShardingConfig {
                shard_count: 8,
                virtual_nodes_per_shard: 150,
                rebalance_threshold: 0.3,
            }),
            ..Default::default()
        };

        let collection = ShardedCollection::new("config_test".to_string(), config.clone()).unwrap();

        let retrieved_config = collection.config();
        assert_eq!(retrieved_config.dimension, 128);
        assert!(retrieved_config.sharding.is_some());
        assert_eq!(retrieved_config.sharding.as_ref().unwrap().shard_count, 8);
    }

    #[test]
    fn test_sharded_collection_owner_id() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let mut collection = ShardedCollection::new("owner_test".to_string(), config).unwrap();

        // Initially no owner
        assert!(collection.owner_id().is_none());

        // Set owner
        let owner = uuid::Uuid::new_v4();
        collection.set_owner_id(Some(owner));

        assert_eq!(collection.owner_id(), Some(owner));
        assert!(collection.belongs_to(&owner));
    }
}

// ============================================================================
// Search Result Merging Tests
// ============================================================================

#[cfg(test)]
mod search_result_tests {
    use vectorizer::db::sharded_collection::ShardedCollection;
    use vectorizer::models::{CollectionConfig, ShardingConfig, Vector};

    fn create_sharding_config(shard_count: u32) -> ShardingConfig {
        ShardingConfig {
            shard_count,
            virtual_nodes_per_shard: 100,
            rebalance_threshold: 0.2,
        }
    }

    fn create_vector(id: &str, data: Vec<f32>) -> Vector {
        Vector {
            id: id.to_string(),
            data,
            sparse: None,
            payload: None,
        }
    }

    #[test]
    fn test_multi_shard_search_merging() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(4)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("merge_test".to_string(), config).unwrap();

        // Insert vectors
        for i in 0..100 {
            let vector = create_vector(
                &format!("merge_vec_{i}"),
                vec![i as f32 / 100.0, 0.5, 0.3, 0.2],
            );
            collection.insert(vector).unwrap();
        }

        // Search
        let query = vec![0.5, 0.5, 0.3, 0.2];
        let results = collection.search(&query, 10, None).unwrap();

        // Results should be sorted by score (descending)
        for i in 1..results.len() {
            assert!(
                results[i - 1].score >= results[i].score,
                "Results should be sorted by score descending"
            );
        }

        // Should have at most k results
        assert!(results.len() <= 10);
    }

    #[test]
    fn test_search_with_limit() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("limit_test".to_string(), config).unwrap();

        // Insert many vectors
        for i in 0..50 {
            let vector = create_vector(
                &format!("limit_vec_{i}"),
                vec![i as f32 / 50.0, 0.1, 0.2, 0.3],
            );
            collection.insert(vector).unwrap();
        }

        // Test different limits
        for limit in [1, 5, 10, 25, 50] {
            let results = collection
                .search(&[0.5, 0.1, 0.2, 0.3], limit, None)
                .unwrap();
            assert!(results.len() <= limit);
        }
    }

    #[test]
    fn test_search_empty_collection() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(create_sharding_config(2)),
            ..Default::default()
        };

        let collection = ShardedCollection::new("empty_search".to_string(), config).unwrap();

        let results = collection.search(&[0.5, 0.5, 0.5, 0.5], 10, None).unwrap();
        assert_eq!(results.len(), 0);
    }
}

// ============================================================================
// Rebalancing Tests
// ============================================================================

#[cfg(test)]
mod rebalancing_tests {
    use vectorizer::db::sharded_collection::ShardedCollection;
    use vectorizer::models::{CollectionConfig, ShardingConfig, Vector};

    fn create_vector(id: &str, data: Vec<f32>) -> Vector {
        Vector {
            id: id.to_string(),
            data,
            sparse: None,
            payload: None,
        }
    }

    #[test]
    fn test_needs_rebalancing_balanced() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(ShardingConfig {
                shard_count: 4,
                virtual_nodes_per_shard: 100,
                rebalance_threshold: 0.2,
            }),
            ..Default::default()
        };

        let collection = ShardedCollection::new("rebalance_test".to_string(), config).unwrap();

        // Empty collection shouldn't need rebalancing
        assert!(!collection.needs_rebalancing());
    }

    #[test]
    fn test_shard_counts() {
        let config = CollectionConfig {
            dimension: 4,
            sharding: Some(ShardingConfig {
                shard_count: 4,
                virtual_nodes_per_shard: 100,
                rebalance_threshold: 0.2,
            }),
            ..Default::default()
        };

        let collection = ShardedCollection::new("shard_counts_test".to_string(), config).unwrap();

        // Insert vectors
        for i in 0..100 {
            let vector = create_vector(&format!("sc_vec_{i}"), vec![i as f32, 0.0, 0.0, 0.0]);
            collection.insert(vector).unwrap();
        }

        let counts = collection.shard_counts();

        // Should have 4 shards
        assert_eq!(counts.len(), 4);

        // Sum should equal total
        let total: usize = counts.values().sum();
        assert_eq!(total, 100);
    }
}
