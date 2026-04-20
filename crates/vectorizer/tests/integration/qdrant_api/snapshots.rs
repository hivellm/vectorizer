#![allow(clippy::unwrap_used, clippy::expect_used)]

mod snapshot_model_tests {
    use vectorizer::models::qdrant::snapshot::{
        QdrantCreateSnapshotRequest, QdrantListSnapshotsResponse, QdrantRecoverSnapshotRequest,
        QdrantSnapshotDescription,
    };

    #[test]
    fn test_snapshot_description_serialization() {
        let snapshot = QdrantSnapshotDescription {
            name: "snapshot_20240101_120000".to_string(),
            creation_time: Some("2024-01-01T12:00:00Z".to_string()),
            size: 1024 * 1024, // 1MB
            checksum: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("snapshot_20240101_120000"));
        assert!(json.contains("2024-01-01T12:00:00Z"));
        assert!(json.contains("1048576"));
    }

    #[test]
    fn test_snapshot_description_without_optional() {
        let snapshot = QdrantSnapshotDescription {
            name: "minimal_snapshot".to_string(),
            creation_time: None,
            size: 512,
            checksum: None,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("minimal_snapshot"));
        assert!(json.contains("512"));
    }

    #[test]
    fn test_list_snapshots_response() {
        let response = QdrantListSnapshotsResponse {
            result: vec![
                QdrantSnapshotDescription {
                    name: "snap1".to_string(),
                    creation_time: None,
                    size: 100,
                    checksum: None,
                },
                QdrantSnapshotDescription {
                    name: "snap2".to_string(),
                    creation_time: None,
                    size: 200,
                    checksum: None,
                },
            ],
            status: "ok".to_string(),
            time: 0.001,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("snap1"));
        assert!(json.contains("snap2"));
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_create_snapshot_request() {
        let request = QdrantCreateSnapshotRequest { wait: Some(true) };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"wait\":true"));
    }

    #[test]
    fn test_recover_snapshot_request() {
        let request = QdrantRecoverSnapshotRequest {
            location: "s3://bucket/snapshot.tar".to_string(),
            priority: Some("high".to_string()),
            checksum: Some("sha256:abc123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("s3://bucket/snapshot.tar"));
        assert!(json.contains("high"));
        assert!(json.contains("sha256:abc123"));
    }
}

// ============================================================================
// Unit Tests for Sharding Models
// ============================================================================

#[cfg(test)]
mod snapshot_api_tests {
    use vectorizer::models::qdrant::snapshot::{
        QdrantCreateSnapshotResponse, QdrantDeleteSnapshotResponse, QdrantListSnapshotsResponse,
        QdrantRecoverSnapshotRequest, QdrantRecoverSnapshotResponse, QdrantSnapshotDescription,
    };

    #[test]
    fn test_create_snapshot_response() {
        let response = QdrantCreateSnapshotResponse {
            result: QdrantSnapshotDescription {
                name: "snapshot_2024_01_15_120000".to_string(),
                creation_time: Some("2024-01-15T12:00:00Z".to_string()),
                size: 1024 * 1024 * 100, // 100MB
                checksum: Some("sha256:abc123def456".to_string()),
            },
            status: "ok".to_string(),
            time: 5.25,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("snapshot_2024_01_15_120000"));
        assert!(json.contains("2024-01-15T12:00:00Z"));
        assert!(json.contains("sha256:abc123def456"));
    }

    #[test]
    fn test_delete_snapshot_response() {
        let response = QdrantDeleteSnapshotResponse {
            result: true,
            status: "ok".to_string(),
            time: 0.5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
    }

    #[test]
    fn test_recover_snapshot_request_s3() {
        let request = QdrantRecoverSnapshotRequest {
            location: "s3://my-bucket/backups/collection_snapshot.tar".to_string(),
            priority: Some("snapshot".to_string()),
            checksum: Some("sha256:xyz789".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("s3://my-bucket/backups/collection_snapshot.tar"));
        assert!(json.contains("snapshot"));
        assert!(json.contains("sha256:xyz789"));
    }

    #[test]
    fn test_recover_snapshot_request_local() {
        let request = QdrantRecoverSnapshotRequest {
            location: "file:///snapshots/backup.tar".to_string(),
            priority: None,
            checksum: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("file:///snapshots/backup.tar"));
    }

    #[test]
    fn test_recover_snapshot_response() {
        let response = QdrantRecoverSnapshotResponse {
            result: true,
            status: "ok".to_string(),
            time: 30.5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":true"));
        assert!(json.contains("30.5"));
    }

    #[test]
    fn test_list_snapshots_empty() {
        let response = QdrantListSnapshotsResponse {
            result: vec![],
            status: "ok".to_string(),
            time: 0.001,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":[]"));
    }

    #[test]
    fn test_list_snapshots_multiple() {
        let response = QdrantListSnapshotsResponse {
            result: vec![
                QdrantSnapshotDescription {
                    name: "snap1".to_string(),
                    creation_time: Some("2024-01-01T00:00:00Z".to_string()),
                    size: 1000,
                    checksum: None,
                },
                QdrantSnapshotDescription {
                    name: "snap2".to_string(),
                    creation_time: Some("2024-01-02T00:00:00Z".to_string()),
                    size: 2000,
                    checksum: Some("checksum2".to_string()),
                },
            ],
            status: "ok".to_string(),
            time: 0.01,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("snap1"));
        assert!(json.contains("snap2"));
        assert!(json.contains("checksum2"));
    }
}
