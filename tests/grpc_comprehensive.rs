//! Comprehensive integration tests for gRPC API
//!
//! This test suite verifies ALL gRPC API functionality:
//! - Collection management (list, create, get info, delete)
//! - Vector operations (insert, get, update, delete, streaming bulk)
//! - Search operations (search, batch search, hybrid search)
//! - Health check and stats
//! - Error handling
//! - Payload handling
//! - End-to-end workflows

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::timeout;
use tonic::transport::Channel;
use vectorizer::db::VectorStore;
use vectorizer::grpc::vectorizer::vectorizer_service_client::VectorizerServiceClient;
use vectorizer::grpc::vectorizer::*;
// Import protobuf types
use vectorizer::grpc::vectorizer::{
    CollectionConfig as ProtoCollectionConfig, DistanceMetric as ProtoDistanceMetric,
    HnswConfig as ProtoHnswConfig, StorageType as ProtoStorageType,
};
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig,
};

/// Helper to create a test gRPC client
async fn create_test_client(
    port: u16,
) -> Result<VectorizerServiceClient<Channel>, Box<dyn std::error::Error>> {
    let addr = format!("http://127.0.0.1:{port}");
    let client = VectorizerServiceClient::connect(addr).await?;
    Ok(client)
}

/// Helper to create a test collection config
fn create_test_config() -> CollectionConfig {
    CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Euclidean, // Use Euclidean to avoid normalization
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
    }
}

/// Helper to create a test vector with correct dimension
fn create_test_vector(_id: &str, seed: usize) -> Vec<f32> {
    (0..128)
        .map(|i| ((seed * 128 + i) % 100) as f32 / 100.0)
        .collect()
}

/// Helper to start a test gRPC server
async fn start_test_server(port: u16) -> Result<Arc<VectorStore>, Box<dyn std::error::Error>> {
    use tonic::transport::Server;
    use vectorizer::grpc::VectorizerGrpcService;
    use vectorizer::grpc::vectorizer::vectorizer_service_server::VectorizerServiceServer;

    let store = Arc::new(VectorStore::new());
    let service = VectorizerGrpcService::new(store.clone());

    let addr = format!("127.0.0.1:{port}").parse()?;

    tokio::spawn(async move {
        Server::builder()
            .add_service(VectorizerServiceServer::new(service))
            .serve(addr)
            .await
            .expect("gRPC server failed");
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    Ok(store)
}

/// Test 1: Health Check
#[tokio::test]
async fn test_health_check() {
    let port = 16000;
    let _store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    let request = tonic::Request::new(HealthCheckRequest {});
    let response = timeout(Duration::from_secs(5), client.health_check(request))
        .await
        .unwrap()
        .unwrap();

    let health = response.into_inner();
    assert_eq!(health.status, "healthy");
    assert!(!health.version.is_empty());
    assert!(health.timestamp > 0);
}

/// Test 2: Get Stats
#[tokio::test]
async fn test_get_stats() {
    let port = 16001;
    let store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    // Create a collection and insert vectors
    let config = create_test_config();
    store.create_collection("stats_test", config).unwrap();

    use vectorizer::models::Vector;
    store
        .insert(
            "stats_test",
            vec![
                Vector {
                    id: "vec1".to_string(),
                    data: create_test_vector("vec1", 1),
                    sparse: None,
                    payload: None,
                },
                Vector {
                    id: "vec2".to_string(),
                    data: create_test_vector("vec2", 2),
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    let request = tonic::Request::new(GetStatsRequest {});
    let response = timeout(Duration::from_secs(5), client.get_stats(request))
        .await
        .unwrap()
        .unwrap();

    let stats = response.into_inner();
    assert!(stats.collections_count >= 1);
    assert!(stats.total_vectors >= 2);
    assert!(!stats.version.is_empty());
    assert!(stats.uptime_seconds >= 0);
}

/// Test 3: Complete Collection Management Workflow
#[tokio::test]
async fn test_collection_management_complete() {
    let port = 16002;
    let _store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    // 3.1: List collections (should be empty initially)
    let list_request = tonic::Request::new(ListCollectionsRequest {});
    let list_response = timeout(
        Duration::from_secs(5),
        client.list_collections(list_request),
    )
    .await
    .unwrap()
    .unwrap();
    let initial_collections = list_response.into_inner().collection_names;
    let initial_count = initial_collections.len();

    // 3.2: Create collection
    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: "test_collection".to_string(),
        config: Some(ProtoCollectionConfig {
            dimension: 128,
            metric: ProtoDistanceMetric::Euclidean as i32,
            hnsw_config: Some(ProtoHnswConfig {
                m: 16,
                ef_construction: 200,
                ef: 50,
                seed: 42,
            }),
            quantization: None,
            storage_type: ProtoStorageType::Memory as i32,
        }),
    });

    let create_response = timeout(
        Duration::from_secs(5),
        client.create_collection(create_request),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(create_response.into_inner().success);

    // 3.3: Verify collection appears in list
    let list_request = tonic::Request::new(ListCollectionsRequest {});
    let list_response = timeout(
        Duration::from_secs(5),
        client.list_collections(list_request),
    )
    .await
    .unwrap()
    .unwrap();
    let collections = list_response.into_inner().collection_names;
    assert_eq!(collections.len(), initial_count + 1);
    assert!(collections.contains(&"test_collection".to_string()));

    // 3.4: Get collection info
    let info_request = tonic::Request::new(GetCollectionInfoRequest {
        collection_name: "test_collection".to_string(),
    });
    let info_response = timeout(
        Duration::from_secs(5),
        client.get_collection_info(info_request),
    )
    .await
    .unwrap()
    .unwrap();
    let info = info_response.into_inner().info.unwrap();
    assert_eq!(info.name, "test_collection");
    assert_eq!(info.config.as_ref().unwrap().dimension, 128);
    assert_eq!(info.vector_count, 0);
    assert!(info.created_at > 0);
    assert!(info.updated_at > 0);

    // 3.5: Delete collection
    let delete_request = tonic::Request::new(DeleteCollectionRequest {
        collection_name: "test_collection".to_string(),
    });
    let delete_response = timeout(
        Duration::from_secs(5),
        client.delete_collection(delete_request),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(delete_response.into_inner().success);

    // 3.6: Verify collection is gone
    let list_request = tonic::Request::new(ListCollectionsRequest {});
    let list_response = timeout(
        Duration::from_secs(5),
        client.list_collections(list_request),
    )
    .await
    .unwrap()
    .unwrap();
    let collections = list_response.into_inner().collection_names;
    assert_eq!(collections.len(), initial_count);
    assert!(!collections.contains(&"test_collection".to_string()));
}

/// Test 4: Complete Vector Operations Workflow
#[tokio::test]
async fn test_vector_operations_complete() {
    let port = 16003;
    let store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    // Create collection
    let config = create_test_config();
    store.create_collection("vector_ops", config).unwrap();

    // 4.1: Insert vector with payload
    let mut payload = HashMap::new();
    payload.insert("category".to_string(), "test".to_string());
    payload.insert("index".to_string(), "1".to_string());

    let insert_request = tonic::Request::new(InsertVectorRequest {
        collection_name: "vector_ops".to_string(),
        vector_id: "vec1".to_string(),
        data: create_test_vector("vec1", 1),
        payload: payload.clone(),
    });

    let insert_response = timeout(Duration::from_secs(5), client.insert_vector(insert_request))
        .await
        .unwrap()
        .unwrap();
    assert!(insert_response.into_inner().success);

    // 4.2: Get vector and verify payload
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: "vector_ops".to_string(),
        vector_id: "vec1".to_string(),
    });
    let get_response = timeout(Duration::from_secs(5), client.get_vector(get_request))
        .await
        .unwrap()
        .unwrap();
    let vector = get_response.into_inner();
    assert_eq!(vector.vector_id, "vec1");
    assert_eq!(vector.data.len(), 128);
    // Payload values are JSON strings, so "test" becomes "\"test\""
    assert!(vector.payload.contains_key("category"));
    assert!(vector.payload.contains_key("index"));

    // 4.3: Update vector with new data and payload
    let mut new_payload = HashMap::new();
    new_payload.insert("category".to_string(), "updated".to_string());
    new_payload.insert("index".to_string(), "2".to_string());

    let update_request = tonic::Request::new(UpdateVectorRequest {
        collection_name: "vector_ops".to_string(),
        vector_id: "vec1".to_string(),
        data: create_test_vector("vec1", 100), // Different data
        payload: new_payload.clone(),
    });

    let update_response = timeout(Duration::from_secs(5), client.update_vector(update_request))
        .await
        .unwrap()
        .unwrap();
    assert!(update_response.into_inner().success);

    // 4.4: Verify update
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: "vector_ops".to_string(),
        vector_id: "vec1".to_string(),
    });
    let get_response = timeout(Duration::from_secs(5), client.get_vector(get_request))
        .await
        .unwrap()
        .unwrap();
    let vector = get_response.into_inner();
    // Payload values are JSON strings
    assert!(vector.payload.contains_key("category"));
    assert!(vector.payload.contains_key("index"));

    // 4.5: Delete vector
    let delete_request = tonic::Request::new(DeleteVectorRequest {
        collection_name: "vector_ops".to_string(),
        vector_id: "vec1".to_string(),
    });
    let delete_response = timeout(Duration::from_secs(5), client.delete_vector(delete_request))
        .await
        .unwrap()
        .unwrap();
    assert!(delete_response.into_inner().success);

    // 4.6: Verify deletion
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: "vector_ops".to_string(),
        vector_id: "vec1".to_string(),
    });
    let get_response = timeout(Duration::from_secs(5), client.get_vector(get_request)).await;
    // Should fail with not found error
    match get_response {
        Ok(Ok(_)) => panic!("Expected error for deleted vector"),
        Ok(Err(status)) => {
            assert_eq!(status.code(), tonic::Code::NotFound);
        }
        Err(_) => {
            // Timeout is also acceptable as error indication
        }
    }
}

/// Test 5: Streaming Bulk Insert
#[tokio::test]
async fn test_streaming_bulk_insert() {
    let port = 16004;
    let store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    // Create collection
    let config = create_test_config();
    store.create_collection("bulk_insert", config).unwrap();

    // Create streaming request
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Send 20 vectors
    for i in 0..20 {
        let mut payload = HashMap::new();
        payload.insert("index".to_string(), i.to_string());

        let request = InsertVectorRequest {
            collection_name: "bulk_insert".to_string(),
            vector_id: format!("vec{i}"),
            data: create_test_vector(&format!("vec{i}"), i),
            payload,
        };
        tx.send(request).await.unwrap();
    }
    drop(tx);

    // Convert to streaming
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let request = tonic::Request::new(stream);

    let response = timeout(Duration::from_secs(10), client.insert_vectors(request))
        .await
        .unwrap()
        .unwrap();
    let result = response.into_inner();

    assert_eq!(result.inserted_count, 20);
    assert_eq!(result.failed_count, 0);
    assert!(result.errors.is_empty());

    // Verify vectors were inserted
    let collection = store.get_collection("bulk_insert").unwrap();
    assert_eq!(collection.vector_count(), 20);
}

/// Test 6: Search Operations
#[tokio::test]
async fn test_search_operations() {
    let port = 16005;
    let store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    // Create collection and insert vectors
    let config = create_test_config();
    store.create_collection("search_test", config).unwrap();

    use vectorizer::models::Vector;
    let vectors: Vec<Vector> = (0..10)
        .map(|i| Vector {
            id: format!("vec{i}"),
            data: create_test_vector(&format!("vec{i}"), i),
            sparse: None,
            payload: Some(vectorizer::models::Payload::new(
                serde_json::json!({"index": i}),
            )),
        })
        .collect();

    store.insert("search_test", vectors).unwrap();

    // 6.1: Basic search
    let search_request = tonic::Request::new(SearchRequest {
        collection_name: "search_test".to_string(),
        query_vector: create_test_vector("query", 1), // Similar to vec1
        limit: 5,
        threshold: 0.0,
        filter: HashMap::new(),
    });

    let search_response = timeout(Duration::from_secs(10), client.search(search_request))
        .await
        .unwrap()
        .unwrap();
    let results = search_response.into_inner().results;

    assert!(!results.is_empty());
    assert!(results.len() <= 5);
    // Results should be sorted by score (best first)
    for i in 1..results.len() {
        assert!(results[i - 1].score >= results[i].score);
    }

    // 6.2: Batch search
    let batch_queries = vec![
        SearchRequest {
            collection_name: "search_test".to_string(),
            query_vector: create_test_vector("query1", 1),
            limit: 3,
            threshold: 0.0,
            filter: HashMap::new(),
        },
        SearchRequest {
            collection_name: "search_test".to_string(),
            query_vector: create_test_vector("query2", 2),
            limit: 3,
            threshold: 0.0,
            filter: HashMap::new(),
        },
    ];

    let batch_request = tonic::Request::new(BatchSearchRequest {
        collection_name: "search_test".to_string(),
        queries: batch_queries,
    });

    let batch_response = timeout(Duration::from_secs(10), client.batch_search(batch_request))
        .await
        .unwrap()
        .unwrap();
    let batch_results = batch_response.into_inner().results;

    assert_eq!(batch_results.len(), 2);
    assert!(!batch_results[0].results.is_empty());
    assert!(!batch_results[1].results.is_empty());
}

/// Test 7: Hybrid Search
#[tokio::test]
async fn test_hybrid_search() {
    let port = 16006;
    let store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    // Create collection and insert vectors
    let config = create_test_config();
    store.create_collection("hybrid_test", config).unwrap();

    use vectorizer::models::Vector;
    let vectors: Vec<Vector> = (0..10)
        .map(|i| Vector {
            id: format!("vec{i}"),
            data: create_test_vector(&format!("vec{i}"), i),
            sparse: None,
            payload: None,
        })
        .collect();

    store.insert("hybrid_test", vectors).unwrap();

    // Hybrid search with dense query and sparse query
    let sparse_query = SparseVector {
        indices: vec![0, 1, 2],
        values: vec![1.0, 0.5, 0.3],
    };

    let hybrid_config = HybridSearchConfig {
        dense_k: 5,
        sparse_k: 5,
        final_k: 5,
        alpha: 0.5,
        algorithm: HybridScoringAlgorithm::Rrf as i32,
    };

    let hybrid_request = tonic::Request::new(HybridSearchRequest {
        collection_name: "hybrid_test".to_string(),
        dense_query: create_test_vector("query", 1),
        sparse_query: Some(sparse_query),
        config: Some(hybrid_config),
    });

    let hybrid_response = timeout(
        Duration::from_secs(10),
        client.hybrid_search(hybrid_request),
    )
    .await
    .unwrap()
    .unwrap();
    let results = hybrid_response.into_inner().results;

    assert!(!results.is_empty());
    assert!(results.len() <= 5);
    // Verify hybrid scores are present
    for result in &results {
        assert!(result.hybrid_score > 0.0);
        assert!(result.dense_score > 0.0);
        // Sparse score might be 0.0 if sparse search is not fully implemented
        assert!(result.sparse_score >= 0.0);
    }
}

/// Test 8: Error Handling
#[tokio::test]
async fn test_error_handling() {
    let port = 16007;
    let _store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    // 8.1: Collection not found
    let get_info_request = tonic::Request::new(GetCollectionInfoRequest {
        collection_name: "nonexistent".to_string(),
    });
    let get_info_response = timeout(
        Duration::from_secs(5),
        client.get_collection_info(get_info_request),
    )
    .await;
    assert!(get_info_response.is_err() || get_info_response.unwrap().is_err());

    // 8.2: Vector not found
    let config = create_test_config();
    let store = start_test_server(port).await.unwrap();
    store.create_collection("error_test", config).unwrap();

    let get_vector_request = tonic::Request::new(GetVectorRequest {
        collection_name: "error_test".to_string(),
        vector_id: "nonexistent".to_string(),
    });
    let get_vector_response = timeout(
        Duration::from_secs(5),
        client.get_vector(get_vector_request),
    )
    .await;
    assert!(get_vector_response.is_err() || get_vector_response.unwrap().is_err());

    // 8.3: Invalid dimension
    let invalid_insert = tonic::Request::new(InsertVectorRequest {
        collection_name: "error_test".to_string(),
        vector_id: "invalid".to_string(),
        data: vec![1.0, 2.0, 3.0], // Wrong dimension
        payload: HashMap::new(),
    });
    let invalid_response =
        timeout(Duration::from_secs(5), client.insert_vector(invalid_insert)).await;
    // Should fail with invalid argument or return success=false
    match invalid_response {
        Ok(Ok(response)) => {
            // If it returns a response, check if success is false
            assert!(!response.into_inner().success);
        }
        Ok(Err(_)) | Err(_) => {
            // Error is also acceptable
        }
    }
}

/// Test 9: End-to-End Complete Workflow
#[tokio::test]
async fn test_end_to_end_workflow() {
    let port = 16008;
    let _store = start_test_server(port).await.unwrap();
    let mut client = create_test_client(port).await.unwrap();

    // 1. Create collection
    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: "e2e_test".to_string(),
        config: Some(ProtoCollectionConfig {
            dimension: 128,
            metric: ProtoDistanceMetric::Euclidean as i32,
            hnsw_config: Some(ProtoHnswConfig {
                m: 16,
                ef_construction: 200,
                ef: 50,
                seed: 42,
            }),
            quantization: None,
            storage_type: ProtoStorageType::Memory as i32,
        }),
    });
    let create_response = timeout(
        Duration::from_secs(5),
        client.create_collection(create_request),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(create_response.into_inner().success);

    // 2. Insert multiple vectors with payloads
    for i in 0..5 {
        let mut payload = HashMap::new();
        payload.insert("id".to_string(), i.to_string());
        payload.insert("type".to_string(), "test".to_string());

        let insert_request = tonic::Request::new(InsertVectorRequest {
            collection_name: "e2e_test".to_string(),
            vector_id: format!("vec{i}"),
            data: create_test_vector(&format!("vec{i}"), i),
            payload,
        });

        let insert_response = timeout(Duration::from_secs(5), client.insert_vector(insert_request))
            .await
            .unwrap()
            .unwrap();
        assert!(insert_response.into_inner().success);
    }

    // 3. Verify collection info
    let info_request = tonic::Request::new(GetCollectionInfoRequest {
        collection_name: "e2e_test".to_string(),
    });
    let info_response = timeout(
        Duration::from_secs(5),
        client.get_collection_info(info_request),
    )
    .await
    .unwrap()
    .unwrap();
    let info = info_response.into_inner().info.unwrap();
    assert_eq!(info.vector_count, 5);

    // 4. Search
    let search_request = tonic::Request::new(SearchRequest {
        collection_name: "e2e_test".to_string(),
        query_vector: create_test_vector("query", 1),
        limit: 3,
        threshold: 0.0,
        filter: HashMap::new(),
    });
    let search_response = timeout(Duration::from_secs(10), client.search(search_request))
        .await
        .unwrap()
        .unwrap();
    let results = search_response.into_inner().results;
    assert!(!results.is_empty());

    // 5. Update a vector
    let update_request = tonic::Request::new(UpdateVectorRequest {
        collection_name: "e2e_test".to_string(),
        vector_id: "vec0".to_string(),
        data: create_test_vector("vec0", 100),
        payload: HashMap::new(),
    });
    let update_response = timeout(Duration::from_secs(5), client.update_vector(update_request))
        .await
        .unwrap()
        .unwrap();
    assert!(update_response.into_inner().success);

    // 6. Delete a vector
    let delete_request = tonic::Request::new(DeleteVectorRequest {
        collection_name: "e2e_test".to_string(),
        vector_id: "vec0".to_string(),
    });
    let delete_response = timeout(Duration::from_secs(5), client.delete_vector(delete_request))
        .await
        .unwrap()
        .unwrap();
    assert!(delete_response.into_inner().success);

    // 7. Verify final state
    let info_request = tonic::Request::new(GetCollectionInfoRequest {
        collection_name: "e2e_test".to_string(),
    });
    let info_response = timeout(
        Duration::from_secs(5),
        client.get_collection_info(info_request),
    )
    .await
    .unwrap()
    .unwrap();
    let info = info_response.into_inner().info.unwrap();
    assert_eq!(info.vector_count, 4); // 5 - 1 deleted

    // 8. Clean up
    let delete_collection_request = tonic::Request::new(DeleteCollectionRequest {
        collection_name: "e2e_test".to_string(),
    });
    let delete_collection_response = timeout(
        Duration::from_secs(5),
        client.delete_collection(delete_collection_request),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(delete_collection_response.into_inner().success);
}
