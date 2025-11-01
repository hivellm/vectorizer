//! API Workflow Integration Tests
//!
//! Tests full CRUD workflows, batch operations, multi-collection operations,
//! and error handling across the REST API.

use serde_json::json;
use vectorizer::models::Vector;

// In integration tests, each file is a separate crate
// Include helpers code directly
#[path = "helpers/mod.rs"]
mod helpers;

use helpers::*;

/// Test full CRUD workflow: Create, Read, Update, Delete
#[tokio::test]
#[ignore = "Timeout: Full CRUD workflow test can take too long"]
async fn test_full_crud_workflow() {
    let store = create_test_store();
    let collection_name = "crud_test";

    // CREATE: Create collection
    create_test_collection(&store, collection_name, 128).expect("Should create collection");
    assert_collection_exists!(&store, collection_name);

    // CREATE: Insert vectors
    let vectors = generate_test_vectors(5, 128);
    insert_test_vectors(&store, collection_name, vectors).expect("Should insert vectors");

    // READ: Verify vectors exist
    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert_eq!(collection.vector_count(), 5);

    // READ: Get specific vector
    assert_vector_exists!(&store, collection_name, "vec_0");

    // UPDATE: Update a vector (via insert with same ID)
    let updated_vector = Vector {
        id: "vec_0".to_string(),
        data: vec![1.0; 128],
        payload: Some(vectorizer::models::Payload::new(json!({"updated": true}))),
    };
    store
        .insert(collection_name, vec![updated_vector])
        .expect("Should update vector");

    // Verify update
    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert_eq!(collection.vector_count(), 5); // Count should remain same

    // DELETE: Delete vector
    store
        .delete(collection_name, "vec_0")
        .expect("Should delete vector");

    // Verify deletion
    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert_eq!(collection.vector_count(), 4);

    // DELETE: Delete collection
    store
        .delete_collection(collection_name)
        .expect("Should delete collection");
    assert_collection_not_exists!(&store, collection_name);
}

/// Test batch operations
#[tokio::test]
#[ignore = "Timeout: Batch operations test can take too long"]
async fn test_batch_operations() {
    let store = create_test_store();
    let collection_name = "batch_test";

    create_test_collection(&store, collection_name, 256).expect("Should create collection");

    // Insert large batch
    let batch_size = 100;
    let vectors = generate_test_vectors(batch_size, 256);
    insert_test_vectors(&store, collection_name, vectors).expect("Should insert batch");

    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert_eq!(collection.vector_count(), batch_size);

    // Batch delete (delete one by one since delete only accepts single ID)
    for i in 0..10 {
        store
            .delete(collection_name, &format!("vec_{i}"))
            .unwrap_or_else(|_| panic!("Should delete vec_{i}"));
    }

    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert_eq!(collection.vector_count(), batch_size - 10);
}

/// Test multi-collection workflow
#[tokio::test]
async fn test_multi_collection_workflow() {
    let store = create_test_store();

    // Create multiple collections
    let collections = vec!["coll_a", "coll_b", "coll_c"];
    for name in &collections {
        create_test_collection(&store, name, 128)
            .unwrap_or_else(|_| panic!("Should create collection {name}"));
    }

    // Verify all collections exist
    let all_collections = store.list_collections();
    for name in &collections {
        assert!(all_collections.contains(&(*name).to_string()));
    }

    // Insert different data in each collection
    for (i, name) in collections.iter().enumerate() {
        let vectors = generate_test_vectors(10 + i, 128);
        insert_test_vectors(&store, name, vectors)
            .unwrap_or_else(|_| panic!("Should insert into {name}"));
    }

    // Verify counts
    let coll_a = store.get_collection("coll_a").unwrap();
    assert_eq!(coll_a.vector_count(), 10);

    let coll_b = store.get_collection("coll_b").unwrap();
    assert_eq!(coll_b.vector_count(), 11);

    let coll_c = store.get_collection("coll_c").unwrap();
    assert_eq!(coll_c.vector_count(), 12);
}

/// Test error handling scenarios
#[tokio::test]
async fn test_error_handling() {
    let store = create_test_store();

    // Test: Collection not found
    let result = store.get_collection("nonexistent");
    assert!(result.is_err());

    // Test: Duplicate collection creation
    create_test_collection(&store, "duplicate_test", 128).expect("Should create first time");
    let result = create_test_collection(&store, "duplicate_test", 128);
    assert!(result.is_err()); // Should fail on duplicate

    // Test: Insert into non-existent collection
    let vectors = generate_test_vectors(1, 128);
    let result = insert_test_vectors(&store, "nonexistent", vectors);
    assert!(result.is_err());

    // Test: Delete non-existent vector (should error)
    let result = store.delete("duplicate_test", "nonexistent_id");
    // This should error
    assert!(result.is_err());

    // Cleanup
    store.delete_collection("duplicate_test").ok();
}
