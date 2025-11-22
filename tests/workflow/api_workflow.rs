//! API Workflow Integration Tests
//!
//! Tests full CRUD workflows, batch operations, multi-collection operations,
//! and error handling across the REST API.

use vectorizer::models::Vector;

// In integration tests, each file is a separate crate
// Include helpers code directly
#[macro_use]
#[path = "../helpers/mod.rs"]
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
        ..Default::default()
    };
    insert_test_vectors(&store, collection_name, vec![updated_vector])
        .expect("Should update vector");

    // DELETE: Delete a vector
    store
        .delete(collection_name, "vec_0")
        .expect("Should delete vector");
    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert_eq!(collection.vector_count(), 4);
}
