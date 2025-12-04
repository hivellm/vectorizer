//! Migration tests for HiveHub Cloud integration
//!
//! Tests for the hub migration module that helps migrate
//! standalone instances to multi-tenant mode.

use uuid::Uuid;
use vectorizer::migration::{
    CollectionMapper, CollectionMigrationRecord, MigrationPlan, MigrationResult, MigrationStatus,
};

#[test]
fn test_migration_status_variants() {
    // Test all status variants serialize correctly
    let statuses = vec![
        (MigrationStatus::Pending, "\"pending\""),
        (MigrationStatus::InProgress, "\"in_progress\""),
        (MigrationStatus::Completed, "\"completed\""),
        (MigrationStatus::Failed, "\"failed\""),
        (MigrationStatus::RolledBack, "\"rolled_back\""),
        (MigrationStatus::Skipped, "\"skipped\""),
    ];

    for (status, expected_json) in statuses {
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(
            json, expected_json,
            "Status {status:?} should serialize to {expected_json}"
        );
    }
}

#[test]
fn test_migration_status_deserialize() {
    let test_cases = vec![
        ("\"pending\"", MigrationStatus::Pending),
        ("\"in_progress\"", MigrationStatus::InProgress),
        ("\"completed\"", MigrationStatus::Completed),
        ("\"failed\"", MigrationStatus::Failed),
        ("\"rolled_back\"", MigrationStatus::RolledBack),
        ("\"skipped\"", MigrationStatus::Skipped),
    ];

    for (json, expected_status) in test_cases {
        let status: MigrationStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, expected_status);
    }
}

#[test]
fn test_collection_mapper_new() {
    let mapper = CollectionMapper::new();
    let mappings = mapper.get_mappings();
    assert!(mappings.is_empty());
}

#[test]
fn test_collection_mapper_default() {
    let mapper = CollectionMapper::default();
    let mappings = mapper.get_mappings();
    assert!(mappings.is_empty());
}

#[test]
fn test_collection_mapper_map_single() {
    let mut mapper = CollectionMapper::new();
    let owner = Uuid::new_v4();

    mapper.map("collection1", owner);

    assert!(mapper.is_mapped("collection1"));
    assert!(!mapper.is_mapped("collection2"));

    let mappings = mapper.get_mappings();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings.get("collection1"), Some(&owner));
}

#[test]
fn test_collection_mapper_map_multiple() {
    let mut mapper = CollectionMapper::new();
    let owner1 = Uuid::new_v4();
    let owner2 = Uuid::new_v4();
    let owner3 = Uuid::new_v4();

    mapper.map("collection1", owner1);
    mapper.map("collection2", owner2);
    mapper.map("collection3", owner3);

    assert!(mapper.is_mapped("collection1"));
    assert!(mapper.is_mapped("collection2"));
    assert!(mapper.is_mapped("collection3"));
    assert!(!mapper.is_mapped("collection4"));

    let mappings = mapper.get_mappings();
    assert_eq!(mappings.len(), 3);
}

#[test]
fn test_collection_mapper_overwrite() {
    let mut mapper = CollectionMapper::new();
    let owner1 = Uuid::new_v4();
    let owner2 = Uuid::new_v4();

    mapper.map("collection1", owner1);
    assert_eq!(mapper.get_mappings().get("collection1"), Some(&owner1));

    // Overwrite with different owner
    mapper.map("collection1", owner2);
    assert_eq!(mapper.get_mappings().get("collection1"), Some(&owner2));
}

#[test]
fn test_collection_migration_record_serialization() {
    let owner_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let record = CollectionMigrationRecord {
        original_name: "my_collection".to_string(),
        new_name: Some("user_550e8400-e29b-41d4-a716-446655440000:my_collection".to_string()),
        owner_id: Some(owner_id),
        status: MigrationStatus::Pending,
        error: None,
        vector_count: 1000,
        migrated_at: None,
    };

    let json = serde_json::to_string(&record).unwrap();
    let parsed: CollectionMigrationRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.original_name, record.original_name);
    assert_eq!(parsed.new_name, record.new_name);
    assert_eq!(parsed.owner_id, record.owner_id);
    assert_eq!(parsed.status, record.status);
    assert_eq!(parsed.vector_count, record.vector_count);
}

#[test]
fn test_collection_migration_record_with_error() {
    let record = CollectionMigrationRecord {
        original_name: "failed_collection".to_string(),
        new_name: None,
        owner_id: None,
        status: MigrationStatus::Failed,
        error: Some("Connection timeout".to_string()),
        vector_count: 500,
        migrated_at: None,
    };

    let json = serde_json::to_string(&record).unwrap();
    let parsed: CollectionMigrationRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.status, MigrationStatus::Failed);
    assert_eq!(parsed.error, Some("Connection timeout".to_string()));
}

#[test]
fn test_migration_plan_serialization() {
    let plan = MigrationPlan {
        id: Uuid::new_v4(),
        created_at: chrono::Utc::now(),
        collections: vec![CollectionMigrationRecord {
            original_name: "col1".to_string(),
            new_name: None,
            owner_id: None,
            status: MigrationStatus::Pending,
            error: None,
            vector_count: 100,
            migrated_at: None,
        }],
        default_owner: Some(Uuid::new_v4()),
        backup_id: None,
        dry_run: true,
    };

    let json = serde_json::to_string(&plan).unwrap();
    let parsed: MigrationPlan = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.id, plan.id);
    assert_eq!(parsed.collections.len(), 1);
    assert!(parsed.dry_run);
}

#[test]
fn test_migration_plan_with_backup() {
    let plan = MigrationPlan {
        id: Uuid::new_v4(),
        created_at: chrono::Utc::now(),
        collections: vec![],
        default_owner: None,
        backup_id: Some("hub_migration_20241204_120000".to_string()),
        dry_run: false,
    };

    let json = serde_json::to_string(&plan).unwrap();
    let parsed: MigrationPlan = serde_json::from_str(&json).unwrap();

    assert_eq!(
        parsed.backup_id,
        Some("hub_migration_20241204_120000".to_string())
    );
}

#[test]
fn test_migration_result_serialization() {
    let result = MigrationResult {
        plan_id: Uuid::new_v4(),
        total: 10,
        succeeded: 8,
        failed: 1,
        skipped: 1,
        details: vec![],
        started_at: chrono::Utc::now(),
        completed_at: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&result).unwrap();
    let parsed: MigrationResult = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.total, 10);
    assert_eq!(parsed.succeeded, 8);
    assert_eq!(parsed.failed, 1);
    assert_eq!(parsed.skipped, 1);
}

#[test]
fn test_migration_result_with_details() {
    let result = MigrationResult {
        plan_id: Uuid::new_v4(),
        total: 2,
        succeeded: 1,
        failed: 1,
        skipped: 0,
        details: vec![
            CollectionMigrationRecord {
                original_name: "success_col".to_string(),
                new_name: Some("user_xxx:success_col".to_string()),
                owner_id: Some(Uuid::new_v4()),
                status: MigrationStatus::Completed,
                error: None,
                vector_count: 100,
                migrated_at: Some(chrono::Utc::now()),
            },
            CollectionMigrationRecord {
                original_name: "failed_col".to_string(),
                new_name: None,
                owner_id: None,
                status: MigrationStatus::Failed,
                error: Some("No owner assigned".to_string()),
                vector_count: 50,
                migrated_at: None,
            },
        ],
        started_at: chrono::Utc::now(),
        completed_at: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&result).unwrap();
    let parsed: MigrationResult = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.details.len(), 2);
    assert_eq!(parsed.details[0].status, MigrationStatus::Completed);
    assert_eq!(parsed.details[1].status, MigrationStatus::Failed);
}

#[test]
fn test_migration_status_equality() {
    assert_eq!(MigrationStatus::Pending, MigrationStatus::Pending);
    assert_ne!(MigrationStatus::Pending, MigrationStatus::Completed);
    assert_ne!(MigrationStatus::InProgress, MigrationStatus::Failed);
}

#[test]
fn test_collection_mapper_same_collection_different_owner() {
    let mut mapper = CollectionMapper::new();

    // Simulate scenario where a collection needs reassignment
    let original_owner = Uuid::new_v4();
    let new_owner = Uuid::new_v4();

    mapper.map("shared_data", original_owner);
    assert_eq!(
        mapper.get_mappings().get("shared_data"),
        Some(&original_owner)
    );

    // Reassign to new owner
    mapper.map("shared_data", new_owner);
    assert_eq!(mapper.get_mappings().get("shared_data"), Some(&new_owner));

    // Should only have one entry
    assert_eq!(mapper.get_mappings().len(), 1);
}

#[test]
fn test_empty_migration_result() {
    let result = MigrationResult {
        plan_id: Uuid::new_v4(),
        total: 0,
        succeeded: 0,
        failed: 0,
        skipped: 0,
        details: vec![],
        started_at: chrono::Utc::now(),
        completed_at: chrono::Utc::now(),
    };

    assert_eq!(result.total, 0);
    assert!(result.details.is_empty());
}
