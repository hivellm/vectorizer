//! Quota management tests for HiveHub integration

use vectorizer::hub::quota::{
    CollectionQuota, QuotaInfo, QuotaType, RateLimitQuota, StorageQuota, VectorQuota,
};

#[test]
fn test_storage_quota_can_use() {
    let quota = StorageQuota {
        limit: 1_000_000, // 1MB
        used: 500_000,    // 500KB
        can_allocate: true,
    };

    // Within remaining capacity
    assert!(quota.can_use(499_999));
    assert!(quota.can_use(500_000)); // Exactly at limit

    // Exceeds capacity
    assert!(!quota.can_use(500_001));
}

#[test]
fn test_storage_quota_disabled() {
    let quota = StorageQuota {
        limit: 1_000_000,
        used: 100_000,
        can_allocate: false, // Allocation disabled
    };

    // Even small amounts should fail
    assert!(!quota.can_use(1));
    assert!(!quota.can_use(100));
}

#[test]
fn test_storage_quota_remaining() {
    let quota = StorageQuota {
        limit: 1_000_000,
        used: 300_000,
        can_allocate: true,
    };

    assert_eq!(quota.remaining(), 700_000);
}

#[test]
fn test_storage_quota_remaining_over_limit() {
    // Edge case: used > limit
    let quota = StorageQuota {
        limit: 1_000_000,
        used: 1_500_000, // Over limit
        can_allocate: false,
    };

    assert_eq!(quota.remaining(), 0); // Should not go negative
}

#[test]
fn test_storage_quota_usage_percent() {
    let quota = StorageQuota {
        limit: 1_000_000,
        used: 250_000,
        can_allocate: true,
    };

    let percent = quota.usage_percent();
    assert!((percent - 25.0).abs() < 0.01);
}

#[test]
fn test_storage_quota_usage_percent_empty_limit() {
    let quota = StorageQuota {
        limit: 0,
        used: 0,
        can_allocate: false,
    };

    // Should handle division by zero gracefully
    assert!((quota.usage_percent() - 100.0).abs() < 0.01);
}

#[test]
fn test_vector_quota_can_use() {
    let quota = VectorQuota {
        limit: 10_000,
        used: 5_000,
        can_insert: true,
    };

    assert!(quota.can_use(4_999));
    assert!(quota.can_use(5_000)); // Exactly at limit
    assert!(!quota.can_use(5_001)); // Over limit
}

#[test]
fn test_vector_quota_disabled() {
    let quota = VectorQuota {
        limit: 10_000,
        used: 1_000,
        can_insert: false,
    };

    assert!(!quota.can_use(1));
}

#[test]
fn test_vector_quota_remaining() {
    let quota = VectorQuota {
        limit: 10_000,
        used: 7_500,
        can_insert: true,
    };

    assert_eq!(quota.remaining(), 2_500);
}

#[test]
fn test_collection_quota_can_use() {
    let quota = CollectionQuota {
        limit: 10,
        used: 9,
        can_create: true,
    };

    assert!(quota.can_use()); // Can create one more

    let full_quota = CollectionQuota {
        limit: 10,
        used: 10,
        can_create: true,
    };

    assert!(!full_quota.can_use()); // At limit
}

#[test]
fn test_collection_quota_disabled() {
    let quota = CollectionQuota {
        limit: 10,
        used: 5,
        can_create: false,
    };

    assert!(!quota.can_use());
}

#[test]
fn test_collection_quota_remaining() {
    let quota = CollectionQuota {
        limit: 10,
        used: 3,
        can_create: true,
    };

    assert_eq!(quota.remaining(), 7);
}

#[test]
fn test_rate_limit_quota_structure() {
    let quota = RateLimitQuota {
        requests_per_minute: 100,
        requests_per_hour: 1000,
        requests_per_day: 10000,
    };

    assert_eq!(quota.requests_per_minute, 100);
    assert_eq!(quota.requests_per_hour, 1000);
    assert_eq!(quota.requests_per_day, 10000);
}

#[test]
fn test_quota_type_display() {
    assert_eq!(QuotaType::Storage.to_string(), "storage");
    assert_eq!(QuotaType::VectorCount.to_string(), "vector_count");
    assert_eq!(QuotaType::CollectionCount.to_string(), "collection_count");
    assert_eq!(
        QuotaType::RequestsPerMinute.to_string(),
        "requests_per_minute"
    );
    assert_eq!(QuotaType::RequestsPerHour.to_string(), "requests_per_hour");
    assert_eq!(QuotaType::RequestsPerDay.to_string(), "requests_per_day");
}

#[test]
fn test_quota_info_structure() {
    let info = QuotaInfo {
        tenant_id: "tenant_123".to_string(),
        storage: StorageQuota {
            limit: 1_000_000,
            used: 500_000,
            can_allocate: true,
        },
        vectors: VectorQuota {
            limit: 10_000,
            used: 5_000,
            can_insert: true,
        },
        collections: CollectionQuota {
            limit: 10,
            used: 3,
            can_create: true,
        },
        rate_limits: RateLimitQuota {
            requests_per_minute: 100,
            requests_per_hour: 1000,
            requests_per_day: 10000,
        },
        updated_at: chrono::Utc::now(),
    };

    assert_eq!(info.tenant_id, "tenant_123");
    assert!(info.storage.can_use(100_000));
    assert!(info.vectors.can_use(1_000));
    assert!(info.collections.can_use());
}

// ============================================================================
// QuotaManager Integration Tests
// ============================================================================

#[cfg(test)]
mod quota_manager_tests {
    use super::*;

    #[test]
    fn test_storage_quota_edge_cases() {
        // Test at exact limit
        let quota = StorageQuota {
            limit: 1000,
            used: 999,
            can_allocate: true,
        };
        assert!(quota.can_use(1)); // Exactly at limit
        assert!(!quota.can_use(2)); // Over by 1

        // Test when disabled
        let disabled = StorageQuota {
            limit: 1000,
            used: 0,
            can_allocate: false,
        };
        assert!(!disabled.can_use(1));
    }

    #[test]
    fn test_storage_quota_zero_limit() {
        let quota = StorageQuota {
            limit: 0,
            used: 0,
            can_allocate: true,
        };

        assert!(!quota.can_use(1));
        assert_eq!(quota.remaining(), 0);
        assert!((quota.usage_percent() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_storage_quota_over_limit() {
        // Already over limit
        let quota = StorageQuota {
            limit: 1000,
            used: 1500,
            can_allocate: true,
        };

        assert!(!quota.can_use(1));
        assert_eq!(quota.remaining(), 0);
        assert!(quota.usage_percent() > 100.0);
    }

    #[test]
    fn test_vector_quota_edge_cases() {
        // Test boundary conditions
        let quota = VectorQuota {
            limit: 100,
            used: 99,
            can_insert: true,
        };

        assert!(quota.can_use(0)); // No vectors
        assert!(quota.can_use(1)); // Exactly at limit
        assert!(!quota.can_use(2)); // Over limit
        assert_eq!(quota.remaining(), 1);

        // Test with high but reasonable values
        let high_quota = VectorQuota {
            limit: 1_000_000_000,
            used: 999_999_990,
            can_insert: true,
        };

        assert!(high_quota.can_use(10));
        assert!(!high_quota.can_use(11));
    }

    #[test]
    fn test_collection_quota_edge_cases() {
        // At exact limit
        let full = CollectionQuota {
            limit: 5,
            used: 5,
            can_create: true,
        };
        assert!(!full.can_use());
        assert_eq!(full.remaining(), 0);

        // One below limit
        let almost_full = CollectionQuota {
            limit: 5,
            used: 4,
            can_create: true,
        };
        assert!(almost_full.can_use());
        assert_eq!(almost_full.remaining(), 1);

        // Disabled
        let disabled = CollectionQuota {
            limit: 100,
            used: 0,
            can_create: false,
        };
        assert!(!disabled.can_use());
    }

    #[test]
    fn test_quota_type_serialization() {
        use serde_json;

        let types = vec![
            QuotaType::Storage,
            QuotaType::VectorCount,
            QuotaType::CollectionCount,
            QuotaType::RequestsPerMinute,
            QuotaType::RequestsPerHour,
            QuotaType::RequestsPerDay,
        ];

        for quota_type in types {
            let serialized = serde_json::to_string(&quota_type).unwrap();
            let deserialized: QuotaType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(quota_type, deserialized);
        }
    }

    #[test]
    fn test_quota_info_serialization() {
        use serde_json;

        let info = QuotaInfo {
            tenant_id: "test_tenant".to_string(),
            storage: StorageQuota {
                limit: 1_000_000,
                used: 500_000,
                can_allocate: true,
            },
            vectors: VectorQuota {
                limit: 10_000,
                used: 5_000,
                can_insert: true,
            },
            collections: CollectionQuota {
                limit: 10,
                used: 5,
                can_create: true,
            },
            rate_limits: RateLimitQuota {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                requests_per_day: 10000,
            },
            updated_at: chrono::Utc::now(),
        };

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: QuotaInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(info.tenant_id, deserialized.tenant_id);
        assert_eq!(info.storage.limit, deserialized.storage.limit);
        assert_eq!(info.vectors.limit, deserialized.vectors.limit);
        assert_eq!(info.collections.limit, deserialized.collections.limit);
    }

    #[test]
    fn test_multiple_quota_types_simultaneously() {
        let info = QuotaInfo {
            tenant_id: "multi_test".to_string(),
            storage: StorageQuota {
                limit: 1_000_000,
                used: 900_000, // 90% used
                can_allocate: true,
            },
            vectors: VectorQuota {
                limit: 1_000,
                used: 995, // 99.5% used
                can_insert: true,
            },
            collections: CollectionQuota {
                limit: 10,
                used: 9, // 90% used
                can_create: true,
            },
            rate_limits: RateLimitQuota {
                requests_per_minute: 100,
                requests_per_hour: 1000,
                requests_per_day: 10000,
            },
            updated_at: chrono::Utc::now(),
        };

        // Storage - can add 100KB, not 200KB
        assert!(info.storage.can_use(100_000));
        assert!(!info.storage.can_use(200_000));

        // Vectors - can add 5, not 10
        assert!(info.vectors.can_use(5));
        assert!(!info.vectors.can_use(10));

        // Collections - can add 1 more
        assert!(info.collections.can_use());
    }

    #[test]
    fn test_quota_remaining_calculations() {
        let storage = StorageQuota {
            limit: 1_000_000,
            used: 300_000,
            can_allocate: true,
        };
        assert_eq!(storage.remaining(), 700_000);
        assert!((storage.usage_percent() - 30.0).abs() < 0.01);

        let vectors = VectorQuota {
            limit: 10_000,
            used: 7_500,
            can_insert: true,
        };
        assert_eq!(vectors.remaining(), 2_500);

        let collections = CollectionQuota {
            limit: 10,
            used: 3,
            can_create: true,
        };
        assert_eq!(collections.remaining(), 7);
    }

    #[test]
    fn test_rate_limit_quota_structure() {
        let limits = RateLimitQuota {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
        };

        // Verify structure
        assert_eq!(limits.requests_per_minute, 60);
        assert_eq!(limits.requests_per_hour, 1000);
        assert_eq!(limits.requests_per_day, 10000);

        // Verify relationships make sense
        assert!(limits.requests_per_hour >= limits.requests_per_minute);
        assert!(limits.requests_per_day >= limits.requests_per_hour);
    }

    #[test]
    fn test_quota_with_zero_usage() {
        let storage = StorageQuota {
            limit: 1_000_000,
            used: 0,
            can_allocate: true,
        };
        assert_eq!(storage.remaining(), 1_000_000);
        assert!((storage.usage_percent() - 0.0).abs() < 0.01);
        assert!(storage.can_use(1_000_000));
        assert!(!storage.can_use(1_000_001));

        let vectors = VectorQuota {
            limit: 10_000,
            used: 0,
            can_insert: true,
        };
        assert_eq!(vectors.remaining(), 10_000);
        assert!(vectors.can_use(10_000));
        assert!(!vectors.can_use(10_001));
    }

    #[test]
    fn test_quota_info_updated_timestamp() {
        let now = chrono::Utc::now();
        let info = QuotaInfo {
            tenant_id: "time_test".to_string(),
            storage: StorageQuota {
                limit: 1_000_000,
                used: 0,
                can_allocate: true,
            },
            vectors: VectorQuota {
                limit: 10_000,
                used: 0,
                can_insert: true,
            },
            collections: CollectionQuota {
                limit: 10,
                used: 0,
                can_create: true,
            },
            rate_limits: RateLimitQuota {
                requests_per_minute: 100,
                requests_per_hour: 1000,
                requests_per_day: 10000,
            },
            updated_at: now,
        };

        assert_eq!(info.updated_at, now);
        assert!(info.updated_at <= chrono::Utc::now());
    }

    #[test]
    fn test_storage_quota_percentage_bounds() {
        // 0% usage
        let zero = StorageQuota {
            limit: 1000,
            used: 0,
            can_allocate: true,
        };
        assert!((zero.usage_percent() - 0.0).abs() < 0.01);

        // 50% usage
        let half = StorageQuota {
            limit: 1000,
            used: 500,
            can_allocate: true,
        };
        assert!((half.usage_percent() - 50.0).abs() < 0.01);

        // 100% usage
        let full = StorageQuota {
            limit: 1000,
            used: 1000,
            can_allocate: true,
        };
        assert!((full.usage_percent() - 100.0).abs() < 0.01);

        // Over 100% usage
        let over = StorageQuota {
            limit: 1000,
            used: 1500,
            can_allocate: false,
        };
        assert!((over.usage_percent() - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_quota_disabled_states() {
        let storage_disabled = StorageQuota {
            limit: 1_000_000,
            used: 0,
            can_allocate: false,
        };
        assert!(!storage_disabled.can_use(1));
        assert!(!storage_disabled.can_use(1000));
        assert!(!storage_disabled.can_use(999_999));

        let vectors_disabled = VectorQuota {
            limit: 10_000,
            used: 0,
            can_insert: false,
        };
        assert!(!vectors_disabled.can_use(1));
        assert!(!vectors_disabled.can_use(100));
        assert!(!vectors_disabled.can_use(9_999));

        let collections_disabled = CollectionQuota {
            limit: 10,
            used: 0,
            can_create: false,
        };
        assert!(!collections_disabled.can_use());
    }

    #[test]
    fn test_large_quota_values() {
        // Test with very large but reasonable values (1 PB storage, 1 trillion vectors)
        let large_storage = StorageQuota {
            limit: 1_000_000_000_000_000, // 1 PB
            used: 500_000_000_000_000,    // 500 TB
            can_allocate: true,
        };
        assert_eq!(large_storage.remaining(), 500_000_000_000_000);
        assert!(large_storage.can_use(1_000_000));

        let large_vectors = VectorQuota {
            limit: 1_000_000_000_000, // 1 trillion vectors
            used: 0,
            can_insert: true,
        };
        assert_eq!(large_vectors.remaining(), 1_000_000_000_000);
        assert!(large_vectors.can_use(1_000_000));
    }
}

// ============================================================================
// QuotaManager Integration Tests with SDK Mocks
// ============================================================================

#[cfg(test)]
mod quota_manager_integration_tests {
    use super::*;
    use crate::hub::mock_hub::MockHubApi;

    #[test]
    fn test_quota_enforcement_with_mock_api() {
        let mock = MockHubApi::new();

        // Create tenant with specific quotas
        let tenant = mock.create_test_user("quota_test");

        // Check initial quota status
        let quota_info = mock.get_quota_info(tenant.id).unwrap();
        assert_eq!(quota_info.collections_used, 0);
        assert_eq!(quota_info.vectors_used, 0);
        assert_eq!(quota_info.storage_used, 0);

        // Create collections within quota
        mock.create_collection(tenant.id, "col1").unwrap();
        mock.create_collection(tenant.id, "col2").unwrap();

        // Record usage
        mock.record_usage(tenant.id, 1000, 50000).unwrap();

        // Verify quota is updated
        let updated_quota = mock.get_quota_info(tenant.id).unwrap();
        assert_eq!(updated_quota.collections_used, 2);
        assert_eq!(updated_quota.vectors_used, 1000);
        assert_eq!(updated_quota.storage_used, 50000);
    }

    #[test]
    fn test_quota_exceeded_prevents_operations() {
        let mock = MockHubApi::new();

        // Create tenant with strict limits
        let tenant = crate::hub::mock_hub::MockUser::new("strict_user").with_quota(2, 100, 10000); // 2 collections, 100 vectors, 10KB storage

        mock.add_user(tenant.clone());

        // Fill up collection quota
        mock.create_collection(tenant.id, "col1").unwrap();
        mock.create_collection(tenant.id, "col2").unwrap();

        // Try to exceed collection quota
        let result = mock.create_collection(tenant.id, "col3");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("quota exceeded"));

        // Verify quota check returns false
        assert!(!mock.check_quota(tenant.id, "collections", 1).unwrap());
    }

    #[test]
    fn test_quota_check_multiple_types() {
        let mock = MockHubApi::new();

        let tenant =
            crate::hub::mock_hub::MockUser::new("multi_quota").with_quota(10, 10000, 1_000_000); // 10 collections, 10K vectors, 1MB

        mock.add_user(tenant.clone());

        // All quotas should be available initially
        assert!(mock.check_quota(tenant.id, "collections", 1).unwrap());
        assert!(mock.check_quota(tenant.id, "vectors", 5000).unwrap());
        assert!(mock.check_quota(tenant.id, "storage", 500_000).unwrap());

        // Use up some quota
        mock.create_collection(tenant.id, "col1").unwrap();
        mock.record_usage(tenant.id, 8000, 800_000).unwrap();

        // Check remaining capacity
        assert!(mock.check_quota(tenant.id, "collections", 9).unwrap());
        assert!(mock.check_quota(tenant.id, "vectors", 2000).unwrap());
        assert!(mock.check_quota(tenant.id, "storage", 200_000).unwrap());

        // Check over capacity
        assert!(!mock.check_quota(tenant.id, "collections", 10).unwrap());
        assert!(!mock.check_quota(tenant.id, "vectors", 2001).unwrap());
        assert!(!mock.check_quota(tenant.id, "storage", 200_001).unwrap());
    }

    #[test]
    fn test_quota_isolation_between_tenants() {
        let mock = MockHubApi::new();

        let tenant_a = crate::hub::mock_hub::MockUser::new("tenant_a").with_quota(5, 1000, 100_000);
        let tenant_b =
            crate::hub::mock_hub::MockUser::new("tenant_b").with_quota(10, 5000, 500_000);

        mock.add_user(tenant_a.clone());
        mock.add_user(tenant_b.clone());

        // Tenant A uses quota
        mock.create_collection(tenant_a.id, "a_col1").unwrap();
        mock.create_collection(tenant_a.id, "a_col2").unwrap();
        mock.record_usage(tenant_a.id, 500, 50_000).unwrap();

        // Tenant B uses different quota
        mock.create_collection(tenant_b.id, "b_col1").unwrap();
        mock.record_usage(tenant_b.id, 1000, 100_000).unwrap();

        // Verify quotas are independent
        let quota_a = mock.get_quota_info(tenant_a.id).unwrap();
        let quota_b = mock.get_quota_info(tenant_b.id).unwrap();

        assert_eq!(quota_a.collections_used, 2);
        assert_eq!(quota_a.vectors_used, 500);
        assert_eq!(quota_a.storage_used, 50_000);

        assert_eq!(quota_b.collections_used, 1);
        assert_eq!(quota_b.vectors_used, 1000);
        assert_eq!(quota_b.storage_used, 100_000);

        // Tenant A's usage doesn't affect tenant B's quota
        assert!(mock.check_quota(tenant_b.id, "collections", 9).unwrap());
        assert!(mock.check_quota(tenant_b.id, "vectors", 4000).unwrap());
    }

    #[test]
    fn test_quota_info_structure_completeness() {
        let now = chrono::Utc::now();
        let info = QuotaInfo {
            tenant_id: "test_123".to_string(),
            storage: StorageQuota {
                limit: 1_000_000,
                used: 500_000,
                can_allocate: true,
            },
            vectors: VectorQuota {
                limit: 10_000,
                used: 5_000,
                can_insert: true,
            },
            collections: CollectionQuota {
                limit: 10,
                used: 5,
                can_create: true,
            },
            rate_limits: RateLimitQuota {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                requests_per_day: 10000,
            },
            updated_at: now,
        };

        // Verify all fields are accessible
        assert_eq!(info.tenant_id, "test_123");
        assert_eq!(info.storage.limit, 1_000_000);
        assert_eq!(info.storage.used, 500_000);
        assert!(info.storage.can_allocate);
        assert_eq!(info.vectors.limit, 10_000);
        assert_eq!(info.vectors.used, 5_000);
        assert!(info.vectors.can_insert);
        assert_eq!(info.collections.limit, 10);
        assert_eq!(info.collections.used, 5);
        assert!(info.collections.can_create);
        assert_eq!(info.rate_limits.requests_per_minute, 60);
        assert_eq!(info.updated_at, now);
    }

    #[test]
    fn test_quota_soft_limits_and_hard_limits() {
        let mock = MockHubApi::new();

        let tenant =
            crate::hub::mock_hub::MockUser::new("limits_test").with_quota(5, 1000, 100_000);

        mock.add_user(tenant.clone());

        // Fill quota to just below limit (soft limit)
        for i in 0..4 {
            mock.create_collection(tenant.id, &format!("col{i}"))
                .unwrap();
        }
        mock.record_usage(tenant.id, 900, 90_000).unwrap();

        // Should still be able to operate (soft limit not hit)
        assert!(mock.check_quota(tenant.id, "collections", 1).unwrap());
        assert!(mock.check_quota(tenant.id, "vectors", 100).unwrap());
        assert!(mock.check_quota(tenant.id, "storage", 10_000).unwrap());

        // Create one more collection (reaches hard limit)
        mock.create_collection(tenant.id, "col4").unwrap();

        // Now collection quota is at hard limit
        assert!(!mock.check_quota(tenant.id, "collections", 1).unwrap());
    }

    #[test]
    fn test_concurrent_quota_checks() {
        use std::sync::Arc;
        use std::thread;

        let mock = Arc::new(MockHubApi::new());

        let tenant = crate::hub::mock_hub::MockUser::new("concurrent_test")
            .with_quota(100, 10_000, 1_000_000);

        mock.add_user(tenant.clone());

        // Spawn multiple threads checking quota
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let mock = Arc::clone(&mock);
                let tenant_id = tenant.id;

                thread::spawn(move || {
                    for _ in 0..10 {
                        // All should succeed as we're within quota
                        assert!(mock.check_quota(tenant_id, "vectors", 100).unwrap());
                        assert!(mock.check_quota(tenant_id, "storage", 10_000).unwrap());
                    }
                })
            })
            .collect();

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_quota_type_all_variants() {
        // Ensure all QuotaType variants are covered
        let types = vec![
            QuotaType::Storage,
            QuotaType::VectorCount,
            QuotaType::CollectionCount,
            QuotaType::RequestsPerMinute,
            QuotaType::RequestsPerHour,
            QuotaType::RequestsPerDay,
        ];

        for quota_type in types {
            let s = quota_type.to_string();
            assert!(!s.is_empty());
            assert!(s.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
        }
    }

    #[test]
    fn test_quota_usage_tracking_accuracy() {
        let mock = MockHubApi::new();

        let tenant = mock.create_test_user("usage_tracking");

        // Initial state
        let initial = mock.get_quota_info(tenant.id).unwrap();
        assert_eq!(initial.vectors_used, 0);
        assert_eq!(initial.storage_used, 0);

        // Record usage multiple times
        mock.record_usage(tenant.id, 100, 1000).unwrap();
        mock.record_usage(tenant.id, 200, 2000).unwrap();
        mock.record_usage(tenant.id, 300, 3000).unwrap();

        // Verify cumulative usage
        let updated = mock.get_quota_info(tenant.id).unwrap();
        assert_eq!(updated.vectors_used, 600); // 100 + 200 + 300
        assert_eq!(updated.storage_used, 6000); // 1000 + 2000 + 3000
    }

    #[test]
    fn test_quota_reset_behavior() {
        let mock = MockHubApi::new();

        let tenant = mock.create_test_user("reset_test");

        // Use some quota
        mock.create_collection(tenant.id, "col1").unwrap();
        mock.record_usage(tenant.id, 100, 1000).unwrap();

        // Verify usage
        let before_reset = mock.get_quota_info(tenant.id).unwrap();
        assert!(before_reset.collections_used > 0);
        assert!(before_reset.vectors_used > 0);

        // Delete collection
        let collections = mock.get_user_collections(tenant.id);
        for col in collections {
            mock.delete_collection(col.id, tenant.id).unwrap();
        }

        // Verify collection count is reset
        let after_delete = mock.get_quota_info(tenant.id).unwrap();
        assert_eq!(after_delete.collections_used, 0);
    }

    #[test]
    fn test_quota_boundary_conditions() {
        let mock = MockHubApi::new();

        let tenant = crate::hub::mock_hub::MockUser::new("boundary_test").with_quota(1, 1, 1); // Minimum quotas

        mock.add_user(tenant.clone());

        // Should be able to use exactly up to limit
        assert!(mock.check_quota(tenant.id, "collections", 1).unwrap());
        assert!(mock.check_quota(tenant.id, "vectors", 1).unwrap());
        assert!(mock.check_quota(tenant.id, "storage", 1).unwrap());

        // Should not be able to exceed by even 1
        assert!(!mock.check_quota(tenant.id, "collections", 2).unwrap());
        assert!(!mock.check_quota(tenant.id, "vectors", 2).unwrap());
        assert!(!mock.check_quota(tenant.id, "storage", 2).unwrap());

        // Create the one allowed collection
        mock.create_collection(tenant.id, "only_one").unwrap();

        // Now can't create any more
        assert!(!mock.check_quota(tenant.id, "collections", 1).unwrap());
        let result = mock.create_collection(tenant.id, "second");
        assert!(result.is_err());
    }
}
