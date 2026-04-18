mod point_id_tests {
    use serde_json::json;
    use vectorizer::models::qdrant::point::QdrantPointId;

    #[test]
    fn test_numeric_point_id() {
        let id: QdrantPointId = serde_json::from_value(json!(123)).unwrap();
        match id {
            QdrantPointId::Numeric(n) => assert_eq!(n, 123),
            QdrantPointId::Uuid(_) => panic!("Expected numeric ID"),
        }
    }

    #[test]
    fn test_string_point_id() {
        let id: QdrantPointId = serde_json::from_value(json!("abc-123")).unwrap();
        match id {
            QdrantPointId::Uuid(s) => assert_eq!(s, "abc-123"),
            QdrantPointId::Numeric(_) => panic!("Expected string ID"),
        }
    }

    #[test]
    fn test_point_id_serialization() {
        let num_id = QdrantPointId::Numeric(456);
        let json = serde_json::to_value(&num_id).unwrap();
        assert_eq!(json, json!(456));

        let str_id = QdrantPointId::Uuid("def-789".to_string());
        let json = serde_json::to_value(&str_id).unwrap();
        assert_eq!(json, json!("def-789"));
    }

    #[test]
    fn test_uuid_point_id() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let id: QdrantPointId = serde_json::from_value(json!(uuid)).unwrap();
        match id {
            QdrantPointId::Uuid(s) => assert_eq!(s, uuid),
            QdrantPointId::Numeric(_) => panic!("Expected UUID string ID"),
        }
    }

    #[test]
    fn test_large_numeric_point_id() {
        let id: QdrantPointId = serde_json::from_value(json!(9999999999i64)).unwrap();
        match id {
            QdrantPointId::Numeric(n) => assert_eq!(n, 9999999999),
            QdrantPointId::Uuid(_) => panic!("Expected large numeric ID"),
        }
    }
}
