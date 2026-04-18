mod collection_status_tests {
    use vectorizer::models::qdrant::collection::QdrantCollectionStatus;

    #[test]
    fn test_green_status_serialization() {
        let status = QdrantCollectionStatus::Green;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""green""#);
    }

    #[test]
    fn test_yellow_status_serialization() {
        let status = QdrantCollectionStatus::Yellow;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""yellow""#);
    }

    #[test]
    fn test_red_status_serialization() {
        let status = QdrantCollectionStatus::Red;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""red""#);
    }

    #[test]
    fn test_status_deserialization() {
        let green: QdrantCollectionStatus = serde_json::from_str(r#""green""#).unwrap();
        assert!(matches!(green, QdrantCollectionStatus::Green));

        let yellow: QdrantCollectionStatus = serde_json::from_str(r#""yellow""#).unwrap();
        assert!(matches!(yellow, QdrantCollectionStatus::Yellow));

        let red: QdrantCollectionStatus = serde_json::from_str(r#""red""#).unwrap();
        assert!(matches!(red, QdrantCollectionStatus::Red));
    }
}

// ============================================================================
// Unit Tests for Optimizer Status
// ============================================================================

#[cfg(test)]
mod optimizer_status_tests {
    use vectorizer::models::qdrant::collection::QdrantOptimizerStatus;

    #[test]
    fn test_optimizer_ok_status() {
        let status = QdrantOptimizerStatus {
            ok: true,
            error: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"ok\":true"));
    }

    #[test]
    fn test_optimizer_error_status() {
        let status = QdrantOptimizerStatus {
            ok: false,
            error: Some("Optimization failed: disk full".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"ok\":false"));
        assert!(json.contains("disk full"));
    }
}
