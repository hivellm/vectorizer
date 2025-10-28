//! Integration tests for Qdrant filter functionality

use serde_json::json;
use vectorizer::db::VectorStore;
use vectorizer::models::qdrant::{
    FilterProcessor, QdrantCondition, QdrantFilter, QdrantFilterBuilder, QdrantGeoPoint,
    QdrantRange, QdrantTextMatchType, QdrantValuesCount,
};
use vectorizer::models::{CollectionConfig, DistanceMetric, Payload, Vector};

fn create_test_store() -> VectorStore {
    VectorStore::new_cpu_only()
}

fn create_payload(data: serde_json::Value) -> Payload {
    Payload::new(data)
}

#[tokio::test]
async fn test_range_filter_integration() {
    let store = create_test_store();

    // Create collection
    let config = CollectionConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        index_type: Default::default(),
        quantization: None,
        replication_factor: 1,
        write_consistency_factor: 1,
        on_disk_payload: false,
    };

    store
        .create_collection("test_range", config)
        .await
        .unwrap();

    let collection = store.get_collection("test_range").unwrap();

    // Add test vectors with price metadata
    let vectors = vec![
        Vector::new("1".to_string(), vec![1.0, 0.0, 0.0, 0.0])
            .with_payload(create_payload(json!({
                "product": "Item A",
                "price": 10.0
            }))),
        Vector::new("2".to_string(), vec![0.0, 1.0, 0.0, 0.0])
            .with_payload(create_payload(json!({
                "product": "Item B",
                "price": 50.0
            }))),
        Vector::new("3".to_string(), vec![0.0, 0.0, 1.0, 0.0])
            .with_payload(create_payload(json!({
                "product": "Item C",
                "price": 100.0
            }))),
    ];

    for vector in vectors {
        let id = vector.id.clone();
        collection.add_vector(id, vector).await.unwrap();
    }

    // Test gt (greater than)
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::range("price", QdrantRange::gt(40.0)))
        .build();

    let payload_b = create_payload(json!({"product": "Item B", "price": 50.0}));
    let payload_c = create_payload(json!({"product": "Item C", "price": 100.0}));
    let payload_a = create_payload(json!({"product": "Item A", "price": 10.0}));

    assert!(FilterProcessor::apply_filter(&filter, &payload_b));
    assert!(FilterProcessor::apply_filter(&filter, &payload_c));
    assert!(!FilterProcessor::apply_filter(&filter, &payload_a));

    // Test between
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::range(
            "price",
            QdrantRange::between(30.0, 80.0),
        ))
        .build();

    assert!(FilterProcessor::apply_filter(&filter, &payload_b));
    assert!(!FilterProcessor::apply_filter(&filter, &payload_c));
    assert!(!FilterProcessor::apply_filter(&filter, &payload_a));
}

#[tokio::test]
async fn test_geo_filters_integration() {
    let store = create_test_store();

    // Create collection
    let config = CollectionConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        index_type: Default::default(),
        quantization: None,
        replication_factor: 1,
        write_consistency_factor: 1,
        on_disk_payload: false,
    };

    store
        .create_collection("test_geo", config)
        .await
        .unwrap();

    let collection = store.get_collection("test_geo").unwrap();

    // Add test vectors with location metadata
    let vectors = vec![
        Vector::new("1".to_string(), vec![1.0, 0.0, 0.0, 0.0])
            .with_payload(create_payload(json!({
                "name": "New York",
                "location": {
                    "lat": 40.7128,
                    "lon": -74.0060
                }
            }))),
        Vector::new("2".to_string(), vec![0.0, 1.0, 0.0, 0.0])
            .with_payload(create_payload(json!({
                "name": "Los Angeles",
                "location": {
                    "lat": 34.0522,
                    "lon": -118.2437
                }
            }))),
        Vector::new("3".to_string(), vec![0.0, 0.0, 1.0, 0.0])
            .with_payload(create_payload(json!({
                "name": "London",
                "location": {
                    "lat": 51.5074,
                    "lon": -0.1278
                }
            }))),
    ];

    for vector in vectors {
        let id = vector.id.clone();
        collection.add_vector(id, vector).await.unwrap();
    }

    // Test GeoBoundingBox - USA only
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::geo_bounding_box(
            "location",
            QdrantGeoPoint::new(50.0, -60.0),  // top-right
            QdrantGeoPoint::new(25.0, -130.0), // bottom-left
        ))
        .build();

    let ny_payload = create_payload(json!({
        "name": "New York",
        "location": {"lat": 40.7128, "lon": -74.0060}
    }));
    let la_payload = create_payload(json!({
        "name": "Los Angeles",
        "location": {"lat": 34.0522, "lon": -118.2437}
    }));
    let london_payload = create_payload(json!({
        "name": "London",
        "location": {"lat": 51.5074, "lon": -0.1278}
    }));

    assert!(FilterProcessor::apply_filter(&filter, &ny_payload));
    assert!(FilterProcessor::apply_filter(&filter, &la_payload));
    assert!(!FilterProcessor::apply_filter(&filter, &london_payload));

    // Test GeoRadius - within 1000km of New York
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::geo_radius(
            "location",
            QdrantGeoPoint::new(40.7128, -74.0060),
            1_000_000.0, // 1000km in meters
        ))
        .build();

    assert!(FilterProcessor::apply_filter(&filter, &ny_payload));
    assert!(!FilterProcessor::apply_filter(&filter, &la_payload));
    assert!(!FilterProcessor::apply_filter(&filter, &london_payload));
}

#[tokio::test]
async fn test_values_count_filter() {
    let store = create_test_store();

    // Create collection
    let config = CollectionConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        index_type: Default::default(),
        quantization: None,
        replication_factor: 1,
        write_consistency_factor: 1,
        on_disk_payload: false,
    };

    store
        .create_collection("test_values_count", config)
        .await
        .unwrap();

    let collection = store.get_collection("test_values_count").unwrap();

    // Add test vectors with tags metadata
    let vectors = vec![
        Vector::new("1".to_string(), vec![1.0, 0.0, 0.0, 0.0])
            .with_payload(create_payload(json!({
                "name": "Product A",
                "tags": ["rust", "fast"]
            }))),
        Vector::new("2".to_string(), vec![0.0, 1.0, 0.0, 0.0])
            .with_payload(create_payload(json!({
                "name": "Product B",
                "tags": ["rust", "fast", "safe", "modern"]
            }))),
        Vector::new("3".to_string(), vec![0.0, 0.0, 1.0, 0.0])
            .with_payload(create_payload(json!({
                "name": "Product C",
                "tags": ["rust"]
            }))),
    ];

    for vector in vectors {
        let id = vector.id.clone();
        collection.add_vector(id, vector).await.unwrap();
    }

    // Test gte (greater than or equal) - at least 3 tags
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::values_count(
            "tags",
            QdrantValuesCount::gte(3),
        ))
        .build();

    let payload_a = create_payload(json!({"name": "Product A", "tags": ["rust", "fast"]}));
    let payload_b = create_payload(json!({
        "name": "Product B",
        "tags": ["rust", "fast", "safe", "modern"]
    }));
    let payload_c = create_payload(json!({"name": "Product C", "tags": ["rust"]}));

    assert!(!FilterProcessor::apply_filter(&filter, &payload_a));
    assert!(FilterProcessor::apply_filter(&filter, &payload_b));
    assert!(!FilterProcessor::apply_filter(&filter, &payload_c));

    // Test between - 2 to 3 tags
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::values_count(
            "tags",
            QdrantValuesCount::between(2, 4),
        ))
        .build();

    assert!(FilterProcessor::apply_filter(&filter, &payload_a));
    assert!(!FilterProcessor::apply_filter(&filter, &payload_b));
    assert!(!FilterProcessor::apply_filter(&filter, &payload_c));
}

#[tokio::test]
async fn test_combined_filters() {
    let payload = create_payload(json!({
        "name": "Premium Product",
        "price": 75.0,
        "category": "electronics",
        "tags": ["sale", "featured", "premium"],
        "in_stock": true,
        "location": {
            "lat": 40.7128,
            "lon": -74.0060
        }
    }));

    // Test complex filter: price 50-100, electronics, at least 2 tags, in stock, in US
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::range(
            "price",
            QdrantRange::between(50.0, 100.0),
        ))
        .must(QdrantCondition::match_string("category", "electronics"))
        .must(QdrantCondition::values_count(
            "tags",
            QdrantValuesCount::gte(2),
        ))
        .must(QdrantCondition::match_bool("in_stock", true))
        .must(QdrantCondition::geo_bounding_box(
            "location",
            QdrantGeoPoint::new(50.0, -60.0),
            QdrantGeoPoint::new(25.0, -130.0),
        ))
        .build();

    assert!(FilterProcessor::apply_filter(&filter, &payload));

    // Change one condition to fail
    let payload_fail = create_payload(json!({
        "name": "Premium Product",
        "price": 75.0,
        "category": "electronics",
        "tags": ["sale", "featured", "premium"],
        "in_stock": false, // Changed to false
        "location": {
            "lat": 40.7128,
            "lon": -74.0060
        }
    }));

    assert!(!FilterProcessor::apply_filter(&filter, &payload_fail));
}

#[tokio::test]
async fn test_must_not_filters() {
    let payload_electronics = create_payload(json!({
        "name": "Product",
        "category": "electronics",
        "price": 50.0
    }));

    let payload_books = create_payload(json!({
        "name": "Product",
        "category": "books",
        "price": 50.0
    }));

    // Filter: price 30-70 AND NOT electronics
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::range(
            "price",
            QdrantRange::between(30.0, 70.0),
        ))
        .must_not(QdrantCondition::match_string("category", "electronics"))
        .build();

    assert!(!FilterProcessor::apply_filter(&filter, &payload_electronics));
    assert!(FilterProcessor::apply_filter(&filter, &payload_books));
}

#[tokio::test]
async fn test_should_filters() {
    let payload_sale = create_payload(json!({
        "name": "Product",
        "on_sale": true,
        "featured": false
    }));

    let payload_featured = create_payload(json!({
        "name": "Product",
        "on_sale": false,
        "featured": true
    }));

    let payload_neither = create_payload(json!({
        "name": "Product",
        "on_sale": false,
        "featured": false
    }));

    // Filter: on_sale OR featured
    let filter = QdrantFilterBuilder::new()
        .should(QdrantCondition::match_bool("on_sale", true))
        .should(QdrantCondition::match_bool("featured", true))
        .build();

    assert!(FilterProcessor::apply_filter(&filter, &payload_sale));
    assert!(FilterProcessor::apply_filter(&filter, &payload_featured));
    assert!(!FilterProcessor::apply_filter(&filter, &payload_neither));
}

#[tokio::test]
async fn test_text_match_filters() {
    let payload = create_payload(json!({
        "description": "Rust programming language is fast and safe"
    }));

    // Test contains
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::match_text(
            "description",
            "Rust",
            QdrantTextMatchType::Contains,
        ))
        .build();
    assert!(FilterProcessor::apply_filter(&filter, &payload));

    // Test prefix
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::match_text(
            "description",
            "Rust prog",
            QdrantTextMatchType::Prefix,
        ))
        .build();
    assert!(FilterProcessor::apply_filter(&filter, &payload));

    // Test suffix
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::match_text(
            "description",
            "safe",
            QdrantTextMatchType::Suffix,
        ))
        .build();
    assert!(FilterProcessor::apply_filter(&filter, &payload));

    // Test exact (should fail as it's not exact match)
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::match_text(
            "description",
            "Rust",
            QdrantTextMatchType::Exact,
        ))
        .build();
    assert!(!FilterProcessor::apply_filter(&filter, &payload));
}

#[tokio::test]
async fn test_nested_keys() {
    let payload = create_payload(json!({
        "user": {
            "profile": {
                "age": 30,
                "location": {
                    "city": "New York"
                }
            }
        }
    }));

    // Test deep nesting
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::match_integer("user.profile.age", 30))
        .must(QdrantCondition::match_string(
            "user.profile.location.city",
            "New York",
        ))
        .build();

    assert!(FilterProcessor::apply_filter(&filter, &payload));

    // Test wrong value
    let filter = QdrantFilterBuilder::new()
        .must(QdrantCondition::match_string(
            "user.profile.location.city",
            "Los Angeles",
        ))
        .build();

    assert!(!FilterProcessor::apply_filter(&filter, &payload));
}

