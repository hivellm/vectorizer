#![allow(clippy::unwrap_used, clippy::expect_used)]

mod distance_metric_tests {
    use vectorizer::models::qdrant::collection::QdrantDistance;

    #[test]
    fn test_cosine_distance_serialization() {
        let distance = QdrantDistance::Cosine;
        let json = serde_json::to_string(&distance).unwrap();
        assert_eq!(json, r#""Cosine""#);
    }

    #[test]
    fn test_euclid_distance_serialization() {
        let distance = QdrantDistance::Euclid;
        let json = serde_json::to_string(&distance).unwrap();
        assert_eq!(json, r#""Euclid""#);
    }

    #[test]
    fn test_dot_distance_serialization() {
        let distance = QdrantDistance::Dot;
        let json = serde_json::to_string(&distance).unwrap();
        assert_eq!(json, r#""Dot""#);
    }

    #[test]
    fn test_distance_deserialization() {
        let cosine: QdrantDistance = serde_json::from_str(r#""Cosine""#).unwrap();
        assert!(matches!(cosine, QdrantDistance::Cosine));

        let euclid: QdrantDistance = serde_json::from_str(r#""Euclid""#).unwrap();
        assert!(matches!(euclid, QdrantDistance::Euclid));

        let dot: QdrantDistance = serde_json::from_str(r#""Dot""#).unwrap();
        assert!(matches!(dot, QdrantDistance::Dot));
    }
}
