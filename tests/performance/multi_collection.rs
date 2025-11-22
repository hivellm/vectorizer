//! Multi-Collection Integration Tests
//!
//! Tests scenarios with 100+ collections, cross-collection searches,
//! and memory scaling verification.

#[path = "../helpers/mod.rs"]
mod helpers;
use helpers::*;

/// Test creating and managing 100+ collections
#[tokio::test]
#[ignore = "Timeout: Creates 100+ collections which can take too long"]
async fn test_100_plus_collections() {
    let store = create_test_store();

    // Create 100 collections
    let collection_count = 100;
    for i in 0..collection_count {
        let collection_name = format!("collection_{i}");
        create_test_collection(&store, &collection_name, 128)
            .unwrap_or_else(|_| panic!("Should create collection {i}"));
    }

    // Verify all collections exist
    let all_collections = store.list_collections();
    assert_eq!(
        all_collections.len(),
        collection_count,
        "Should have {collection_count} collections"
    );

    // Insert different amounts of data in each collection
    for i in 0..collection_count {
        let collection_name = format!("collection_{i}");
        let vector_count = 10 + (i % 20); // Vary between 10-29 vectors

        let vectors = generate_test_vectors(vector_count, 128);
        insert_test_vectors(&store, &collection_name, vectors)
            .unwrap_or_else(|_| panic!("Should insert into collection {i}"));
    }

    // Verify all collections have data
    for i in 0..collection_count {
        let collection_name = format!("collection_{i}");
        let collection = store
            .get_collection(&collection_name)
            .unwrap_or_else(|_| panic!("Collection {i} should exist"));

        let expected_count = 10 + (i % 20);
        assert_eq!(
            collection.vector_count(),
            expected_count,
            "Collection {i} should have {expected_count} vectors"
        );
    }
}

/// Test cross-collection searches
#[tokio::test]
async fn test_cross_collection_searches() {
    let store = create_test_store();

    // Create multiple collections with different data
    let collections = vec!["tech_docs", "research_papers", "code_repos", "blog_posts"];

    for name in collections.iter() {
        create_test_collection(&store, name, 256)
            .unwrap_or_else(|_| panic!("Should create collection {name}"));

        // Insert themed vectors
        let vectors = generate_test_vectors(50, 256);
        insert_test_vectors(&store, name, vectors)
            .unwrap_or_else(|_| panic!("Should insert into {name}"));
    }

    // Create a query vector
    let query_vector = vec![0.5; 256];

    // Search across all collections
    for name in &collections {
        let result = store.search(name, &query_vector, 10);
        assert!(result.is_ok(), "Should be able to search in {name}");

        let search_results = result.unwrap();
        assert!(
            !search_results.is_empty(),
            "Search in {name} should return results"
        );
    }
}

/// Test memory scaling with many collections
#[tokio::test]
#[ignore = "Timeout: Memory scaling test with large collections"]
async fn test_memory_scaling() {
    let store = create_test_store();

    // Create collections with increasing sizes
    let collection_sizes = [10, 50, 100, 200, 500];

    for (idx, size) in collection_sizes.iter().enumerate() {
        let collection_name = format!("scale_test_{idx}");

        create_test_collection(&store, &collection_name, 512)
            .unwrap_or_else(|_| panic!("Should create collection {collection_name}"));

        let vectors = generate_test_vectors(*size, 512);
        insert_test_vectors(&store, &collection_name, vectors)
            .unwrap_or_else(|_| panic!("Should insert {size} vectors"));

        // Verify collection created and accessible
        let collection = store
            .get_collection(&collection_name)
            .unwrap_or_else(|_| panic!("Collection {collection_name} should exist"));
        assert_eq!(
            collection.vector_count(),
            *size,
            "Collection {collection_name} should have {size} vectors"
        );
    }

    // Verify all collections still accessible
    let all_collections = store.list_collections();
    assert!(
        all_collections.len() >= collection_sizes.len(),
        "Should have all collections"
    );

    // Perform operations on all collections to verify memory is manageable
    for idx in 0..collection_sizes.len() {
        let collection_name = format!("scale_test_{idx}");
        let query_vector = vec![0.3; 512];

        let result = store.search(&collection_name, &query_vector, 5);
        assert!(
            result.is_ok(),
            "Should be able to search in {collection_name}"
        );
    }
}

/// Test collection lifecycle with many collections
#[tokio::test]
async fn test_collection_lifecycle_many() {
    let store = create_test_store();

    // Get initial count (may have collections from other parallel tests)
    let initial_existing = store.list_collections().len();

    // Create many collections
    let initial_count = 50;
    for i in 0..initial_count {
        let name = format!("lifecycle_{i}");
        create_test_collection(&store, &name, 64)
            .unwrap_or_else(|_| panic!("Should create {name}"));
    }

    assert_eq!(
        store.list_collections().len(),
        initial_existing + initial_count,
        "Should have {} collections (initial {} + created {})",
        initial_existing + initial_count,
        initial_existing,
        initial_count
    );

    // Delete some collections
    let delete_count = 20;
    for i in 0..delete_count {
        let name = format!("lifecycle_{i}");
        store
            .delete_collection(&name)
            .unwrap_or_else(|_| panic!("Should delete {name}"));
    }

    assert_eq!(
        store.list_collections().len(),
        initial_existing + initial_count - delete_count,
        "Should have {} collections after deletion (initial {} + created {} - deleted {})",
        initial_existing + initial_count - delete_count,
        initial_existing,
        initial_count,
        delete_count
    );

    // Create new collections
    let new_count = 15;
    for i in initial_count..initial_count + new_count {
        let name = format!("lifecycle_{i}");
        create_test_collection(&store, &name, 64)
            .unwrap_or_else(|_| panic!("Should create new {name}"));
    }

    assert_eq!(
        store.list_collections().len(),
        initial_existing + initial_count - delete_count + new_count,
        "Should have {} collections after recreation (initial {} + created {} - deleted {} + new {})",
        initial_existing + initial_count - delete_count + new_count,
        initial_existing,
        initial_count,
        delete_count,
        new_count
    );
}

/// Test concurrent operations on multiple collections
#[tokio::test]
#[ignore = "Timeout: Concurrent operations on multiple collections"]
async fn test_concurrent_multi_collection() {
    let store = create_test_store();

    // Create multiple collections
    let collection_names: Vec<String> = (0..10).map(|i| format!("concurrent_coll_{i}")).collect();

    for name in &collection_names {
        create_test_collection(&store, name, 128)
            .unwrap_or_else(|_| panic!("Should create {name}"));

        let vectors = generate_test_vectors(20, 128);
        insert_test_vectors(&store, name, vectors)
            .unwrap_or_else(|_| panic!("Should insert into {name}"));
    }

    // Perform concurrent searches across collections
    let mut handles = Vec::new();
    for name in &collection_names {
        let store_clone = store.clone();
        let collection = name.clone();
        let query = vec![0.4; 128];

        handles.push(tokio::spawn(async move {
            for _ in 0..5 {
                let _ = store_clone.search(&collection, &query, 10);
            }
        }));
    }

    // Wait for all searches
    for handle in handles {
        handle.await.expect("Search task should complete");
    }

    // Verify all collections still accessible
    for name in &collection_names {
        let collection = store
            .get_collection(name)
            .unwrap_or_else(|_| panic!("Collection {name} should exist"));
        assert_eq!(collection.vector_count(), 20);
    }
}
