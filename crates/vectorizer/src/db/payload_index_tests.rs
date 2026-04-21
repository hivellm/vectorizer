//! Extracted unit tests (phase3 test-extraction).
//!
//! Wired from `src/db/payload_index.rs` via the `#[path]` attribute.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::json;

use super::*;
use crate::models::Payload;

#[test]
fn test_keyword_index() {
    let index = PayloadIndex::new();

    // Add index config
    let config = PayloadIndexConfig::new("status".to_string(), PayloadIndexType::Keyword);
    index.add_index_config(config);

    // Index vectors
    let payload1 = Payload {
        data: json!({"status": "active", "name": "test1"}),
    };
    index.index_vector("v1".to_string(), &payload1);

    let payload2 = Payload {
        data: json!({"status": "inactive", "name": "test2"}),
    };
    index.index_vector("v2".to_string(), &payload2);

    // Query
    let ids = index.get_ids_for_keyword("status", "active").unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains("v1"));

    // Remove vector
    index.remove_vector("v1");
    let ids = index.get_ids_for_keyword("status", "active");
    assert!(ids.is_none() || ids.unwrap().is_empty());
}

#[test]
fn test_integer_index() {
    let index = PayloadIndex::new();

    // Add index config
    let config = PayloadIndexConfig::new("age".to_string(), PayloadIndexType::Integer);
    index.add_index_config(config);

    // Index vectors
    let payload1 = Payload {
        data: json!({"age": 25, "name": "test1"}),
    };
    index.index_vector("v1".to_string(), &payload1);

    let payload2 = Payload {
        data: json!({"age": 30, "name": "test2"}),
    };
    index.index_vector("v2".to_string(), &payload2);

    let payload3 = Payload {
        data: json!({"age": 35, "name": "test3"}),
    };
    index.index_vector("v3".to_string(), &payload3);

    // Query range
    let ids = index.get_ids_in_range("age", Some(25), Some(30)).unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains("v1"));
    assert!(ids.contains("v2"));
}

#[test]
fn test_index_stats() {
    let index = PayloadIndex::new();

    let config = PayloadIndexConfig::new("status".to_string(), PayloadIndexType::Keyword);
    index.add_index_config(config);

    let payload = Payload {
        data: json!({"status": "active"}),
    };
    index.index_vector("v1".to_string(), &payload);

    let stats = index.get_stats();
    assert!(stats.contains_key("status"));
    assert_eq!(stats["status"].indexed_count, 1);
}

#[test]
fn test_float_index() {
    let index = PayloadIndex::new();

    let config = PayloadIndexConfig::new("price".to_string(), PayloadIndexType::Float);
    index.add_index_config(config);

    let payload1 = Payload {
        data: json!({"price": 19.99, "name": "item1"}),
    };
    index.index_vector("v1".to_string(), &payload1);

    let payload2 = Payload {
        data: json!({"price": 29.99, "name": "item2"}),
    };
    index.index_vector("v2".to_string(), &payload2);

    let payload3 = Payload {
        data: json!({"price": 39.99, "name": "item3"}),
    };
    index.index_vector("v3".to_string(), &payload3);

    // Query range
    let ids = index
        .get_ids_in_float_range("price", Some(20.0), Some(35.0))
        .unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains("v2"));
}

#[test]
fn test_text_index() {
    let index = PayloadIndex::new();

    let config = PayloadIndexConfig::new("description".to_string(), PayloadIndexType::Text);
    index.add_index_config(config);

    let payload1 = Payload {
        data: json!({"description": "rust programming language tutorial"}),
    };
    index.index_vector("v1".to_string(), &payload1);

    let payload2 = Payload {
        data: json!({"description": "python programming tutorial"}),
    };
    index.index_vector("v2".to_string(), &payload2);

    let payload3 = Payload {
        data: json!({"description": "rust and python comparison"}),
    };
    index.index_vector("v3".to_string(), &payload3);

    // Search for "rust"
    let ids = index.search_text("description", "rust").unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains("v1"));
    assert!(ids.contains("v3"));

    // Search for "rust tutorial"
    let ids = index.search_text("description", "rust tutorial").unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains("v1"));
}

#[test]
fn test_geo_index() {
    let index = PayloadIndex::new();

    let config = PayloadIndexConfig::new("location".to_string(), PayloadIndexType::Geo);
    index.add_index_config(config);

    // São Paulo, Brazil
    let payload1 = Payload {
        data: json!({"location": {"lat": -23.5505, "lon": -46.6333}}),
    };
    index.index_vector("v1".to_string(), &payload1);

    // Rio de Janeiro, Brazil
    let payload2 = Payload {
        data: json!({"location": {"lat": -22.9068, "lon": -43.1729}}),
    };
    index.index_vector("v2".to_string(), &payload2);

    // New York, USA
    let payload3 = Payload {
        data: json!({"location": [-74.0060, 40.7128]}), // Array format
    };
    index.index_vector("v3".to_string(), &payload3);

    // Query bounding box (Brazil region)
    let ids = index
        .get_ids_in_geo_bounding_box(
            "location", -25.0, // min_lat
            -20.0, // max_lat
            -50.0, // min_lon
            -40.0, // max_lon
        )
        .unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains("v1"));
    assert!(ids.contains("v2"));

    // Query radius (1000km from São Paulo)
    let ids = index
        .get_ids_in_geo_radius(
            "location", -23.5505, // center_lat (São Paulo)
            -46.6333, // center_lon
            1000.0,   // radius_km
        )
        .unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains("v1"));
    assert!(ids.contains("v2"));
}

#[test]
fn test_nested_field_indexing() {
    let index = PayloadIndex::new();

    // Index nested field "user.age"
    let config = PayloadIndexConfig::new("user.age".to_string(), PayloadIndexType::Integer);
    index.add_index_config(config);

    // Index nested field "metadata.price"
    let config = PayloadIndexConfig::new("metadata.price".to_string(), PayloadIndexType::Float);
    index.add_index_config(config);

    // Index nested field "user.name"
    let config = PayloadIndexConfig::new("user.name".to_string(), PayloadIndexType::Keyword);
    index.add_index_config(config);

    // Index vectors with nested payloads
    let payload1 = Payload {
        data: json!({
            "user": {
                "name": "Alice",
                "age": 25
            },
            "metadata": {
                "price": 19.99
            }
        }),
    };
    index.index_vector("v1".to_string(), &payload1);

    let payload2 = Payload {
        data: json!({
            "user": {
                "name": "Bob",
                "age": 30
            },
            "metadata": {
                "price": 29.99
            }
        }),
    };
    index.index_vector("v2".to_string(), &payload2);

    // Query nested integer field
    let ids = index
        .get_ids_in_range("user.age", Some(25), Some(30))
        .unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains("v1"));
    assert!(ids.contains("v2"));

    // Query nested float field
    let ids = index
        .get_ids_in_float_range("metadata.price", Some(20.0), Some(30.0))
        .unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains("v2"));

    // Query nested keyword field
    let ids = index.get_ids_for_keyword("user.name", "Alice").unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains("v1"));
}
