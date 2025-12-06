//! Backup tests for HiveHub Cloud integration
//!
//! Tests for the user-scoped backup system.

use std::collections::HashMap;
use std::path::PathBuf;

use uuid::Uuid;
use vectorizer::hub::backup::{
    BackupCollectionData, BackupConfig, BackupVector, UserBackupData, UserBackupInfo,
};

#[test]
fn test_backup_config_default() {
    let config = BackupConfig::default();

    assert_eq!(config.backup_dir, PathBuf::from("./data/hub_backups"));
    assert_eq!(config.max_backups_per_user, 10);
    assert_eq!(config.max_backup_age_hours, 0); // unlimited
    assert!(config.compression_enabled);
    assert_eq!(config.compression_level, 6);
}

#[test]
fn test_backup_config_serialization() {
    let config = BackupConfig {
        backup_dir: PathBuf::from("/custom/backup/path"),
        max_backups_per_user: 5,
        max_backup_age_hours: 24 * 7, // 1 week
        compression_enabled: false,
        compression_level: 9,
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: BackupConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.backup_dir, PathBuf::from("/custom/backup/path"));
    assert_eq!(parsed.max_backups_per_user, 5);
    assert_eq!(parsed.max_backup_age_hours, 24 * 7);
    assert!(!parsed.compression_enabled);
    assert_eq!(parsed.compression_level, 9);
}

#[test]
fn test_user_backup_info_creation() {
    let user_id = Uuid::new_v4();
    let backup_id = Uuid::new_v4();

    let info = UserBackupInfo {
        id: backup_id,
        user_id,
        name: "daily_backup".to_string(),
        description: Some("Automated daily backup".to_string()),
        created_at: chrono::Utc::now(),
        collections: vec!["documents".to_string(), "embeddings".to_string()],
        vector_count: 10000,
        size_bytes: 5_000_000, // 5MB
        format_version: 1,
        checksum: Some("abc123def456".to_string()),
        compressed: true,
    };

    assert_eq!(info.id, backup_id);
    assert_eq!(info.user_id, user_id);
    assert_eq!(info.collections.len(), 2);
    assert_eq!(info.vector_count, 10000);
    assert!(info.compressed);
}

#[test]
fn test_user_backup_info_serialization() {
    let info = UserBackupInfo {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        user_id: Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap(),
        name: "test_backup".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        collections: vec!["col1".to_string()],
        vector_count: 100,
        size_bytes: 1024,
        format_version: 1,
        checksum: None,
        compressed: true,
    };

    let json = serde_json::to_string(&info).unwrap();
    let parsed: UserBackupInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, "test_backup");
    assert_eq!(parsed.collections.len(), 1);
    assert_eq!(parsed.vector_count, 100);
}

#[test]
fn test_user_backup_info_without_optional_fields() {
    let json = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "user_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "name": "minimal_backup",
        "created_at": "2024-01-15T10:30:00Z",
        "collections": [],
        "vector_count": 0,
        "size_bytes": 0,
        "format_version": 1
    }"#;

    let parsed: UserBackupInfo = serde_json::from_str(json).unwrap();

    assert_eq!(parsed.name, "minimal_backup");
    assert!(parsed.description.is_none());
    assert!(parsed.checksum.is_none());
    // compressed should default to true
    assert!(parsed.compressed);
}

#[test]
fn test_backup_vector_creation() {
    let vector = BackupVector {
        id: "vec_001".to_string(),
        data: vec![0.1, 0.2, 0.3, 0.4],
        sparse: None,
        payload: Some(serde_json::json!({
            "text": "Hello world",
            "category": "greeting"
        })),
    };

    assert_eq!(vector.id, "vec_001");
    assert_eq!(vector.data.len(), 4);
    assert!(vector.sparse.is_none());
    assert!(vector.payload.is_some());
}

#[test]
fn test_backup_vector_serialization() {
    let vector = BackupVector {
        id: "test_vec".to_string(),
        data: vec![1.0, 2.0, 3.0],
        sparse: None,
        payload: None,
    };

    let json = serde_json::to_string(&vector).unwrap();

    // sparse and payload should be omitted when None
    assert!(!json.contains("sparse"));
    assert!(!json.contains("payload"));

    let parsed: BackupVector = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.id, "test_vec");
    assert_eq!(parsed.data, vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_backup_vector_with_payload() {
    let vector = BackupVector {
        id: "vec_with_meta".to_string(),
        data: vec![0.5, 0.5],
        sparse: None,
        payload: Some(serde_json::json!({
            "title": "Document Title",
            "score": 0.95,
            "tags": ["important", "review"]
        })),
    };

    let json = serde_json::to_string(&vector).unwrap();
    let parsed: BackupVector = serde_json::from_str(&json).unwrap();

    let payload = parsed.payload.unwrap();
    assert_eq!(payload["title"], "Document Title");
    assert_eq!(payload["score"], 0.95);
}

#[test]
fn test_backup_collection_data() {
    let collection = BackupCollectionData {
        name: "documents".to_string(),
        full_name: "user_123:documents".to_string(),
        dimension: 384,
        metric: "cosine".to_string(),
        vectors: vec![
            BackupVector {
                id: "doc_1".to_string(),
                data: vec![0.1; 384],
                sparse: None,
                payload: None,
            },
            BackupVector {
                id: "doc_2".to_string(),
                data: vec![0.2; 384],
                sparse: None,
                payload: None,
            },
        ],
        metadata: HashMap::new(),
    };

    assert_eq!(collection.name, "documents");
    assert_eq!(collection.full_name, "user_123:documents");
    assert_eq!(collection.dimension, 384);
    assert_eq!(collection.metric, "cosine");
    assert_eq!(collection.vectors.len(), 2);
}

#[test]
fn test_backup_collection_data_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert(
        "created_by".to_string(),
        serde_json::json!("user@example.com"),
    );
    metadata.insert("version".to_string(), serde_json::json!(1));

    let collection = BackupCollectionData {
        name: "embeddings".to_string(),
        full_name: "user_456:embeddings".to_string(),
        dimension: 768,
        metric: "euclidean".to_string(),
        vectors: vec![],
        metadata,
    };

    let json = serde_json::to_string(&collection).unwrap();
    let parsed: BackupCollectionData = serde_json::from_str(&json).unwrap();

    assert_eq!(
        parsed.metadata.get("created_by"),
        Some(&serde_json::json!("user@example.com"))
    );
    assert_eq!(parsed.metadata.get("version"), Some(&serde_json::json!(1)));
}

#[test]
fn test_user_backup_data_full() {
    let user_id = Uuid::new_v4();
    let backup_id = Uuid::new_v4();

    let info = UserBackupInfo {
        id: backup_id,
        user_id,
        name: "full_backup".to_string(),
        description: Some("Complete backup".to_string()),
        created_at: chrono::Utc::now(),
        collections: vec!["col1".to_string(), "col2".to_string()],
        vector_count: 500,
        size_bytes: 100000,
        format_version: 1,
        checksum: Some("checksum123".to_string()),
        compressed: true,
    };

    let collections = vec![
        BackupCollectionData {
            name: "col1".to_string(),
            full_name: format!("user_{user_id}:col1"),
            dimension: 128,
            metric: "cosine".to_string(),
            vectors: vec![BackupVector {
                id: "v1".to_string(),
                data: vec![0.1; 128],
                sparse: None,
                payload: None,
            }],
            metadata: HashMap::new(),
        },
        BackupCollectionData {
            name: "col2".to_string(),
            full_name: format!("user_{user_id}:col2"),
            dimension: 256,
            metric: "dot".to_string(),
            vectors: vec![],
            metadata: HashMap::new(),
        },
    ];

    let backup_data = UserBackupData { info, collections };

    let json = serde_json::to_string(&backup_data).unwrap();
    let parsed: UserBackupData = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.info.name, "full_backup");
    assert_eq!(parsed.collections.len(), 2);
    assert_eq!(parsed.collections[0].dimension, 128);
    assert_eq!(parsed.collections[1].dimension, 256);
}

#[test]
fn test_backup_config_compression_levels() {
    // Test various compression levels
    for level in 1..=9 {
        let config = BackupConfig {
            compression_level: level,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: BackupConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.compression_level, level);
    }
}

#[test]
fn test_backup_info_format_version() {
    let info = UserBackupInfo {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        name: "versioned".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        collections: vec![],
        vector_count: 0,
        size_bytes: 0,
        format_version: 2, // Future version
        checksum: None,
        compressed: false,
    };

    let json = serde_json::to_string(&info).unwrap();
    let parsed: UserBackupInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.format_version, 2);
}

#[test]
fn test_backup_large_vector_count() {
    let info = UserBackupInfo {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        name: "large_backup".to_string(),
        description: Some("Millions of vectors".to_string()),
        created_at: chrono::Utc::now(),
        collections: vec!["massive_collection".to_string()],
        vector_count: 10_000_000,   // 10 million vectors
        size_bytes: 50_000_000_000, // 50GB
        format_version: 1,
        checksum: Some("sha256:abc...".to_string()),
        compressed: true,
    };

    assert_eq!(info.vector_count, 10_000_000);
    assert_eq!(info.size_bytes, 50_000_000_000);
}

#[test]
fn test_empty_backup() {
    let info = UserBackupInfo {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        name: "empty_backup".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        collections: vec![],
        vector_count: 0,
        size_bytes: 0,
        format_version: 1,
        checksum: None,
        compressed: true,
    };

    let backup_data = UserBackupData {
        info,
        collections: vec![],
    };

    let json = serde_json::to_string(&backup_data).unwrap();
    let parsed: UserBackupData = serde_json::from_str(&json).unwrap();

    assert!(parsed.collections.is_empty());
    assert_eq!(parsed.info.vector_count, 0);
}
