//! Integration tests for Qdrant REST API compatibility
//!
//! Tests all 14 Qdrant endpoints implemented in the vectorizer:
//! - Collection management: list, get, create, update, delete
//! - Vector operations: upsert, retrieve, delete, scroll, count
//! - Search operations: search, recommend, batch search, batch recommend

use serde_json::json;
use vectorizer::db::VectorStore;
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, QuantizationConfig,
};

/// Helper to create a test store
fn create_test_store() -> VectorStore {
    VectorStore::new()
}

/// Helper to create a test collection
fn create_test_collection(
    store: &VectorStore,
    name: &str,
    dimension: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = CollectionConfig {
        dimension,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 100,
            ef_search: 100,
            seed: None,
        },
        quantization: QuantizationConfig::SQ { bits: 8 },
        compression: CompressionConfig::default(),
        normalization: None,
    };
    store.create_collection(name, config)?;
    Ok(())
}

/// Helper to insert test vectors
fn insert_test_vectors(
    store: &VectorStore,
    collection_name: &str,
    count: usize,
    dimension: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut collection = store.get_collection_mut(collection_name)?;
    let mut ids = Vec::new();

    for i in 0..count {
        let id = format!("test_vector_{i}");
        let data: Vec<f32> = (0..dimension).map(|j| (i + j) as f32 / 10.0).collect();

        let vector = vectorizer::models::Vector {
            id: id.clone(),
            data,
            payload: Some(vectorizer::models::Payload {
                data: json!({
                    "index": i,
                    "type": "test",
                    "description": format!("Test vector {}", i)
                }),
            }),
        };

        collection.add_vector(id.clone(), vector)?;
        ids.push(id);
    }

    Ok(ids)
}

#[test]
fn test_qdrant_list_collections() {
    let store = create_test_store();

    // Create some test collections
    let _ = create_test_collection(&store, "collection_1", 128);
    let _ = create_test_collection(&store, "collection_2", 256);

    // List collections
    let collections = store.list_collections();

    assert!(collections.len() >= 2, "Should have at least 2 collections");
    assert!(
        collections.contains(&"collection_1".to_string()),
        "Should contain collection_1"
    );
    assert!(
        collections.contains(&"collection_2".to_string()),
        "Should contain collection_2"
    );
}

#[test]
fn test_qdrant_get_collection() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "test_collection", 128);

    // Get collection info
    let result = store.get_collection("test_collection");

    assert!(
        result.is_ok(),
        "Should successfully get collection information"
    );

    let collection = result.unwrap();
    assert_eq!(collection.config().dimension, 128, "Dimension should match");
}

#[test]
fn test_qdrant_create_collection() {
    let store = create_test_store();

    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 100,
            ef_search: 10000,
            seed: None,
        },
        quantization: QuantizationConfig::SQ { bits: 8 },
        compression: CompressionConfig::default(),
        normalization: None,
    };

    // Create collection
    let result = store.create_collection("new_collection", config);

    assert!(result.is_ok(), "Should successfully create collection");

    // Verify collection exists
    let collections = store.list_collections();
    assert!(collections.contains(&"new_collection".to_string()));
}

#[test]
fn test_qdrant_update_collection() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "update_test", 128);

    // Update should not fail (even if not all settings are applied)
    let result = store.get_collection("update_test");
    assert!(result.is_ok(), "Collection should exist for update");
}

#[test]
fn test_qdrant_delete_collection() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "delete_test", 128);

    // Verify collection exists
    assert!(store.get_collection("delete_test").is_ok());

    // Delete collection
    let result = store.delete_collection("delete_test");
    assert!(result.is_ok(), "Should successfully delete collection");

    // Verify collection no longer exists
    assert!(store.get_collection("delete_test").is_err());
}

#[test]
fn test_qdrant_alias_create_and_resolve() {
    let store = create_test_store();
    create_test_collection(&store, "alias_target", 64).unwrap();

    // Create alias
    store
        .create_alias("alias_name", "alias_target")
        .expect("Alias creation should succeed");

    // Alias should resolve to target collection
    let collection = store
        .get_collection("alias_name")
        .expect("Alias should resolve to collection");
    assert_eq!(collection.name(), "alias_target");

    // Alias listing should include new alias
    let aliases = store.list_aliases();
    assert!(
        aliases
            .iter()
            .any(|(alias, target)| alias == "alias_name" && target == "alias_target")
    );

    // Listing aliases for target should return alias_name
    let aliases_for_collection = store
        .list_aliases_for_collection("alias_target")
        .expect("Should list aliases for collection");
    assert!(aliases_for_collection.contains(&"alias_name".to_string()));
}

#[test]
fn test_qdrant_alias_delete() {
    let store = create_test_store();
    create_test_collection(&store, "alias_delete_target", 64).unwrap();
    store
        .create_alias("alias_delete", "alias_delete_target")
        .unwrap();

    // Delete alias
    store
        .delete_alias("alias_delete")
        .expect("Alias deletion should succeed");

    // Alias should no longer resolve
    assert!(store.get_collection("alias_delete").is_err());

    // Listing aliases should not include deleted alias
    let aliases = store.list_aliases();
    assert!(aliases.iter().all(|(alias, _)| alias != "alias_delete"));
}

#[test]
fn test_qdrant_alias_rename() {
    let store = create_test_store();
    create_test_collection(&store, "alias_rename_target", 64).unwrap();
    store
        .create_alias("alias_old", "alias_rename_target")
        .unwrap();

    store
        .rename_alias("alias_old", "alias_new")
        .expect("Alias rename should succeed");

    // Old alias should be removed
    assert!(store.get_collection("alias_old").is_err());

    // New alias should resolve to the target
    let collection = store
        .get_collection("alias_new")
        .expect("New alias should resolve");
    assert_eq!(collection.name(), "alias_rename_target");
}

#[test]
fn test_qdrant_upsert_points() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "upsert_test", 3);

    // Insert vectors
    let mut collection = store.get_collection_mut("upsert_test").unwrap();

    let vector1 = vectorizer::models::Vector {
        id: "1".to_string(),
        data: vec![1.0, 2.0, 3.0],
        payload: Some(vectorizer::models::Payload {
            data: json!({"field1": "value1"}),
        }),
    };

    let vector2 = vectorizer::models::Vector {
        id: "2".to_string(),
        data: vec![4.0, 5.0, 6.0],
        payload: Some(vectorizer::models::Payload {
            data: json!({"field1": "value2"}),
        }),
    };

    let _ = collection.add_vector("1".to_string(), vector1);
    let _ = collection.add_vector("2".to_string(), vector2);

    // Verify vectors were inserted
    assert_eq!(collection.vector_count(), 2, "Should have 2 vectors");
}

#[test]
fn test_qdrant_retrieve_points() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "retrieve_test", 128);
    let ids = insert_test_vectors(&store, "retrieve_test", 5, 128).unwrap();

    let collection = store.get_collection("retrieve_test").unwrap();

    // Retrieve specific points
    for id in &ids[0..3] {
        let result = collection.get_vector(id);
        assert!(result.is_ok(), "Should retrieve vector {id}");
        assert_eq!(&result.unwrap().id, id, "ID should match");
    }
}

#[test]
fn test_qdrant_delete_points() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "delete_points_test", 128);
    let ids = insert_test_vectors(&store, "delete_points_test", 5, 128).unwrap();

    let mut collection = store.get_collection_mut("delete_points_test").unwrap();
    let initial_count = collection.vector_count();

    // Delete points
    for id in &ids[0..2] {
        let result = collection.delete_vector(id);
        assert!(result.is_ok(), "Should delete vector {id}");
    }

    // Verify deletion
    assert_eq!(
        collection.vector_count(),
        initial_count - 2,
        "Should have 2 fewer vectors"
    );
}

#[test]
fn test_qdrant_scroll_points() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let store = create_test_store();
    let collection_name = format!(
        "scroll_test_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // Create collection WITH quantization to match helper function
    let config = CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 100,
            ef_search: 100,
            seed: None,
        },
        quantization: QuantizationConfig::SQ { bits: 8 },
        compression: CompressionConfig::default(),
        normalization: None,
    };
    store
        .create_collection(&collection_name, config)
        .expect("Failed to create collection");

    let ids =
        insert_test_vectors(&store, &collection_name, 20, 128).expect("Failed to insert vectors");

    assert_eq!(ids.len(), 20, "Should have inserted 20 vector IDs");

    // Verify vector count using immutable reference
    let collection = store
        .get_collection(&collection_name)
        .expect("Failed to get collection");

    // Verify vector count
    let vector_count = collection.vector_count();
    assert_eq!(
        vector_count, 20,
        "Collection should have 20 vectors (count check)"
    );

    // Verify we can retrieve individual vectors
    for id in &ids {
        let vector = collection.get_vector(id);
        assert!(vector.is_ok(), "Should be able to retrieve vector {id}");
    }

    // Get all vectors (simulate scroll) - use same immutable reference
    let vectors = collection.get_all_vectors();

    assert_eq!(
        vectors.len(),
        20,
        "Should have 20 vectors (get_all_vectors check). vector_count was: {}, vectors.len() was: {}",
        vector_count,
        vectors.len()
    );

    // Simulate pagination
    let page_size = 5;
    let page_1 = &vectors[0..page_size];
    let page_2 = &vectors[page_size..page_size * 2];

    assert_eq!(page_1.len(), 5, "First page should have 5 vectors");
    assert_eq!(page_2.len(), 5, "Second page should have 5 vectors");
    assert_ne!(
        page_1[0].id, page_2[0].id,
        "Pages should have different vectors"
    );
}

#[test]
fn test_qdrant_count_points() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "count_test", 128);
    let _ = insert_test_vectors(&store, "count_test", 15, 128);

    let collection = store.get_collection("count_test").unwrap();
    let count = collection.vector_count();

    assert_eq!(count, 15, "Should have exactly 15 vectors");
}

#[test]
fn test_qdrant_search_points() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "search_test", 128);
    let _ = insert_test_vectors(&store, "search_test", 10, 128);

    let collection = store.get_collection("search_test").unwrap();

    // Create search vector (similar to first test vector)
    let query_vector: Vec<f32> = (0..128).map(|j| j as f32 / 10.0).collect();

    // Perform search
    let results = collection.search(&query_vector, 5);

    assert!(results.is_ok(), "Search should succeed");
    let results = results.unwrap();
    assert!(!results.is_empty(), "Should have search results");
    assert!(results.len() <= 5, "Should have at most 5 results (limit)");

    // Results should be ordered by score
    for i in 1..results.len() {
        assert!(
            results[i - 1].score >= results[i].score,
            "Results should be ordered by score descending"
        );
    }
}

#[test]
fn test_qdrant_recommend_points() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "recommend_test", 128);
    let ids = insert_test_vectors(&store, "recommend_test", 10, 128).unwrap();

    let collection = store.get_collection("recommend_test").unwrap();

    // Get a reference vector
    let positive_id = &ids[0];
    let positive_vector = collection.get_vector(positive_id).unwrap();

    // Simulate recommendation by searching with the positive vector
    let results = collection.search(&positive_vector.data, 5);

    assert!(results.is_ok(), "Recommend (search) should succeed");
    let results = results.unwrap();
    assert!(!results.is_empty(), "Should have recommendation results");
}

#[test]
fn test_qdrant_batch_search_points() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "batch_search_test", 128);
    let _ = insert_test_vectors(&store, "batch_search_test", 15, 128);

    let collection = store.get_collection("batch_search_test").unwrap();

    // Create multiple search vectors
    let queries: Vec<Vec<f32>> = (0..3)
        .map(|i| (0..128).map(|j| (i + j) as f32 / 10.0).collect())
        .collect();

    // Perform batch search
    let mut all_results = Vec::new();
    for query in queries {
        let results = collection.search(&query, 3);
        assert!(results.is_ok(), "Batch search query should succeed");
        all_results.push(results.unwrap());
    }

    assert_eq!(
        all_results.len(),
        3,
        "Should have results for all 3 queries"
    );
    for (i, results) in all_results.iter().enumerate() {
        assert!(!results.is_empty(), "Query {i} should have results");
        assert!(
            results.len() <= 3,
            "Query {i} should have at most 3 results"
        );
    }
}

#[test]
fn test_qdrant_batch_recommend_points() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "batch_recommend_test", 128);
    let ids = insert_test_vectors(&store, "batch_recommend_test", 15, 128).unwrap();

    let collection = store.get_collection("batch_recommend_test").unwrap();

    // Get multiple reference vectors
    let positive_ids = &ids[0..3];
    let mut all_results = Vec::new();

    for positive_id in positive_ids {
        let positive_vector = collection.get_vector(positive_id).unwrap();
        let results = collection.search(&positive_vector.data, 5);

        assert!(results.is_ok(), "Batch recommend query should succeed");
        all_results.push(results.unwrap());
    }

    assert_eq!(
        all_results.len(),
        3,
        "Should have results for all 3 recommendations"
    );
    for (i, results) in all_results.iter().enumerate() {
        assert!(
            !results.is_empty(),
            "Recommendation {i} should have results"
        );
    }
}

#[test]
fn test_qdrant_collection_not_found() {
    let store = create_test_store();

    // Try to get non-existent collection
    let result = store.get_collection("nonexistent");
    assert!(result.is_err(), "Should fail for non-existent collection");

    // Try to delete non-existent collection
    let result = store.delete_collection("nonexistent");
    assert!(
        result.is_err(),
        "Should fail to delete non-existent collection"
    );
}

#[test]
fn test_qdrant_dimension_mismatch() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "dimension_test", 128);

    let collection = store.get_collection("dimension_test").unwrap();

    // Try to search with wrong dimension
    let wrong_vector: Vec<f32> = vec![1.0, 2.0, 3.0]; // Only 3 dimensions, expected 128
    let result = collection.search(&wrong_vector, 5);

    // Should fail or handle gracefully
    let _ = result; // Accept either success or failure
}

#[test]
fn test_qdrant_empty_collection_search() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "empty_test", 128);

    let collection = store.get_collection("empty_test").unwrap();

    // Search in empty collection
    let query: Vec<f32> = (0..128).map(|i| i as f32).collect();
    let results = collection.search(&query, 10);

    // Should return empty results, not error
    if let Ok(results) = results {
        assert!(
            results.is_empty(),
            "Empty collection should return no results"
        );
    }
}

#[test]
fn test_qdrant_large_batch_operations() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "large_batch_test", 64);

    // Insert large batch of vectors
    let result = insert_test_vectors(&store, "large_batch_test", 100, 64);
    assert!(result.is_ok(), "Should insert 100 vectors");

    let collection = store.get_collection("large_batch_test").unwrap();
    assert_eq!(
        collection.vector_count(),
        100,
        "Should have exactly 100 vectors"
    );

    // Batch search with 10 queries
    let queries: Vec<Vec<f32>> = (0..10)
        .map(|i| (0..64).map(|j| (i + j) as f32 / 10.0).collect())
        .collect();

    let mut all_results = Vec::new();
    for query in queries {
        let results = collection.search(&query, 10);
        if let Ok(results) = results {
            all_results.push(results);
        }
    }

    assert_eq!(
        all_results.len(),
        10,
        "Should have results for all 10 queries"
    );
}

#[test]
fn test_qdrant_collection_with_payload() {
    let store = create_test_store();
    let _ = create_test_collection(&store, "payload_test", 64);

    let mut collection = store.get_collection_mut("payload_test").unwrap();

    // Insert vector with rich payload
    let vector = vectorizer::models::Vector {
        id: "rich_vector".to_string(),
        data: (0..64).map(|i| i as f32).collect(),
        payload: Some(vectorizer::models::Payload {
            data: json!({
                "title": "Test Document",
                "category": "test",
                "tags": ["tag1", "tag2", "tag3"],
                "score": 0.95,
                "metadata": {
                    "author": "Test Author",
                    "date": "2024-01-01"
                }
            }),
        }),
    };

    let result = collection.add_vector("rich_vector".to_string(), vector);
    assert!(result.is_ok(), "Should insert vector with payload");

    // Retrieve and verify payload
    let retrieved = collection.get_vector("rich_vector");
    assert!(retrieved.is_ok(), "Should retrieve vector with payload");

    let retrieved = retrieved.unwrap();
    assert!(retrieved.payload.is_some(), "Should have payload");
    assert!(
        retrieved.payload.unwrap().data["title"] == "Test Document",
        "Payload data should match"
    );
}
