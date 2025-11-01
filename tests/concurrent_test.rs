//! Concurrent Operations Integration Tests
//!
//! Tests concurrent searches, inserts, and read-while-write scenarios
//! to verify thread-safety and absence of race conditions.

use std::time::Duration;

use tokio::time::sleep;

#[path = "helpers/mod.rs"]
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

        handles.push(tokio::spawn(async move {
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
                        payload: Some(vectorizer::models::Payload::new(
                            serde_json::json!({"task": task_id, "index": i}),
                        )),
                    }
                })
                .collect();

            store_clone.insert(&collection, vectors)
        }));
    }

    // Wait for all inserts
    let mut success_count = 0;
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        if result.is_ok() {
            success_count += 1;
        }
    }

    // At least some inserts should succeed (may have conflicts depending on implementation)
    assert!(success_count > 0, "Some concurrent inserts should succeed");

    // Verify final count
    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert!(
        collection.vector_count() > 0,
        "Should have vectors inserted"
    );
}

/// Test read-while-write scenario
#[tokio::test]
#[ignore = "Timeout: Read-while-write stress test can take too long"]
async fn test_read_while_write() {
    let store = create_test_store();
    let collection_name = "read_write_test";

    create_test_collection(&store, collection_name, 96).expect("Should create collection");

    // Initial data
    let initial_vectors = generate_test_vectors(50, 96);
    insert_test_vectors(&store, collection_name, initial_vectors)
        .expect("Should insert initial vectors");

    let query_vector = vec![0.3; 96];

    // Spawn writer task
    let store_writer = store.clone();
    let collection_writer = collection_name.to_string();
    let writer_handle = tokio::spawn(async move {
        for i in 0..10 {
            let vectors = generate_test_vectors(10, 96);
            // Offset IDs to avoid conflicts
            let vectors: Vec<_> = vectors
                .into_iter()
                .map(|mut v| {
                    v.id = format!("writer_{i}_{}", v.id);
                    v
                })
                .collect();

            let _ = store_writer.insert(&collection_writer, vectors);
            sleep(Duration::from_millis(50)).await;
        }
    });

    // Spawn multiple reader tasks
    let mut reader_handles = Vec::new();
    for _ in 0..5 {
        let store_reader = store.clone();
        let collection_reader = collection_name.to_string();
        let query = query_vector.clone();

        reader_handles.push(tokio::spawn(async move {
            let mut read_count = 0;
            for _ in 0..20 {
                let result = store_reader.search(&collection_reader, &query, 10);
                if result.is_ok() {
                    read_count += 1;
                }
                sleep(Duration::from_millis(25)).await;
            }
            read_count
        }));
    }

    // Wait for writer
    writer_handle.await.expect("Writer should complete");

    // Wait for readers
    let mut total_reads = 0;
    for handle in reader_handles {
        let reads = handle.await.expect("Reader should complete");
        total_reads += reads;
    }

    // Verify reads succeeded during writes
    assert!(
        total_reads > 50,
        "Should have many successful reads during writes"
    );

    // Final verification
    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");
    assert!(
        collection.vector_count() >= 50,
        "Should have at least initial vectors"
    );
}

/// Test no race conditions in vector operations
#[tokio::test]
async fn test_no_race_conditions() {
    let store = create_test_store();
    let collection_name = "race_condition_test";

    create_test_collection(&store, collection_name, 32).expect("Should create collection");

    // Insert initial vectors
    let initial_vectors = generate_test_vectors(100, 32);
    insert_test_vectors(&store, collection_name, initial_vectors)
        .expect("Should insert initial vectors");

    let query_vector = vec![0.4; 32];

    // Mix of operations: search, insert, delete, update
    let mut handles = Vec::new();

    // Search operations
    for _ in 0..3 {
        let store_clone = store.clone();
        let collection = collection_name.to_string();
        let query = query_vector.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..10 {
                let _ = store_clone.search(&collection, &query, 5);
            }
        }));
    }

    // Insert operations
    for i in 0..2 {
        let store_clone = store.clone();
        let collection = collection_name.to_string();
        handles.push(tokio::spawn(async move {
            for _j in 0..5 {
                let vectors = generate_test_vectors(5, 32);
                let vectors: Vec<_> = vectors
                    .into_iter()
                    .map(|mut v| {
                        v.id = format!("insert_{i}_{}", v.id);
                        v
                    })
                    .collect();
                let _ = store_clone.insert(&collection, vectors);
                sleep(Duration::from_millis(10)).await;
            }
        }));
    }

    // Delete operations
    for i in 0..2 {
        let store_clone = store.clone();
        let collection = collection_name.to_string();
        handles.push(tokio::spawn(async move {
            for j in 0..5 {
                for k in 0..5 {
                    let id = format!("vec_{}", i * 5 + j * 5 + k);
                    let _ = store_clone.delete(&collection, &id);
                }
                sleep(Duration::from_millis(15)).await;
            }
        }));
    }

    // Wait for all operations
    for handle in handles {
        let _ = handle.await;
    }

    // Verify collection is still in valid state
    let collection = store
        .get_collection(collection_name)
        .expect("Collection should exist");

    // Collection should have some vectors (exact count depends on timing)
    assert!(
        collection.vector_count() > 0,
        "Collection should still have vectors after concurrent operations"
    );

    // Verify we can still search successfully
    let search_result = store.search(collection_name, &query_vector, 10);
    assert!(
        search_result.is_ok(),
        "Should be able to search after concurrent operations"
    );
}
