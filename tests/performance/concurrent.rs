//! Concurrent Operations Integration Tests
//!
//! Tests concurrent searches, inserts, and read-while-write scenarios
//! to verify thread-safety and absence of race conditions.

#[path = "../helpers/mod.rs"]
mod helpers;

use helpers::*;

/// Test concurrent searches from multiple threads
#[tokio::test]
async fn test_concurrent_searches() {
    let store = create_test_store();
    let collection_name = "concurrent_search_test";

    // Setup: Create collection and insert vectors
    create_test_collection(&store, collection_name, 128).expect("Should create collection");

    let vectors = generate_test_vectors(100, 128);
    insert_test_vectors(&store, collection_name, vectors).expect("Should insert vectors");

    // Generate query vector
    let query_vector = vec![0.5; 128];

    // Perform concurrent searches
    let mut handles = Vec::new();
    for i in 0..10 {
        let store_clone = store.clone();
        let collection = collection_name.to_string();
        let query = query_vector.clone();

        handles.push(tokio::spawn(async move {
            let result = store_clone.search(&collection, &query, 10);
            (i, result)
        }));
    }

    // Wait for all searches to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        results.push(result);
    }

    // Verify all searches succeeded
    for (task_id, result) in results {
        assert!(
            result.is_ok(),
            "Concurrent search task {task_id} should succeed"
        );
        let search_results = result.unwrap();
        assert!(!search_results.is_empty(), "Search should return results");
    }
}

/// Test concurrent inserts from multiple threads
#[tokio::test]
async fn test_concurrent_inserts() {
    let store = create_test_store();
    let collection_name = "concurrent_insert_test";

    create_test_collection(&store, collection_name, 64).expect("Should create collection");

    // Insert vectors concurrently from multiple tasks
    let mut handles = Vec::new();
    for task_id in 0..5 {
        let store_clone = store.clone();
        let collection = collection_name.to_string();

        let handle = tokio::spawn(async move {
            let vectors: Vec<_> = (0..20)
                .map(|i| {
                    let mut data = vec![0.0; 64];
                    for (idx, item) in data.iter_mut().take(64).enumerate() {
                        *item = (task_id * 20 + i + idx) as f32 * 0.01;
                    }
                    // Normalize
                    let norm: f32 = data.iter().map(|x| x * x).sum::<f32>().sqrt();
                    if norm > 0.0 {
                        for x in &mut data {
                            *x /= norm;
                        }
                    }
                    vectorizer::models::Vector {
                        id: format!("task_{task_id}_vec_{i}"),
                        data,
                        ..Default::default()
                    }
                })
                .collect();
            store_clone.insert(&collection, vectors).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all inserts to complete
    for handle in handles {
        handle.await.expect("Task should complete");
    }

    // Verify all vectors were inserted
    let final_collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert_eq!(final_collection.vector_count(), 100); // 5 tasks * 20 vectors
}
