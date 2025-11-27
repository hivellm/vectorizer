//! Qdrant-compatible gRPC API Tests
//!
//! Tests for Qdrant-compatible gRPC services:
//! - Collections service
//! - Points service
//! - Snapshots service

#![allow(deprecated)]

use std::sync::Arc;
use std::time::Duration;

use tonic::transport::Channel;
use vectorizer::db::VectorStore;
use vectorizer::grpc::qdrant_proto::collections_client::CollectionsClient;
use vectorizer::grpc::qdrant_proto::points_client::PointsClient;
use vectorizer::grpc::qdrant_proto::*;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};

/// Helper to start a Qdrant gRPC test server
async fn start_qdrant_test_server(
    port: u16,
) -> Result<Arc<VectorStore>, Box<dyn std::error::Error>> {
    use tonic::transport::Server;
    use vectorizer::grpc::QdrantGrpcService;
    use vectorizer::grpc::qdrant_proto::collections_server::CollectionsServer;
    use vectorizer::grpc::qdrant_proto::points_server::PointsServer;

    let store = Arc::new(VectorStore::new());
    let service = QdrantGrpcService::new(store.clone());

    let addr = format!("127.0.0.1:{port}").parse()?;

    tokio::spawn(async move {
        Server::builder()
            .add_service(CollectionsServer::new(service.clone()))
            .add_service(PointsServer::new(service))
            .serve(addr)
            .await
            .expect("Qdrant gRPC server failed");
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    Ok(store)
}

/// Helper to create a Qdrant Collections gRPC client
async fn create_collections_client(
    port: u16,
) -> Result<CollectionsClient<Channel>, Box<dyn std::error::Error>> {
    let addr = format!("http://127.0.0.1:{port}");
    let client = CollectionsClient::connect(addr).await?;
    Ok(client)
}

/// Helper to create a Qdrant Points gRPC client
async fn create_points_client(
    port: u16,
) -> Result<PointsClient<Channel>, Box<dyn std::error::Error>> {
    let addr = format!("http://127.0.0.1:{port}");
    let client = PointsClient::connect(addr).await?;
    Ok(client)
}

/// Helper to create a test collection config
fn create_test_config() -> CollectionConfig {
    CollectionConfig {
        dimension: 128,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        sharding: None,
        graph: None,
    }
}

// ============================================================================
// Collections Service Tests
// ============================================================================

#[tokio::test]
async fn test_qdrant_list_collections() {
    let port = 16020;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create a test collection
    let config = create_test_config();
    store.create_collection("qdrant_test", config).unwrap();

    let mut client = create_collections_client(port).await.unwrap();

    let request = tonic::Request::new(ListCollectionsRequest {});
    let response = client.list(request).await.unwrap();

    let collections = response.into_inner().collections;
    assert!(!collections.is_empty());
    assert!(collections.iter().any(|c| c.name == "qdrant_test"));
}

#[tokio::test]
async fn test_qdrant_create_collection() {
    let port = 16021;
    let _store = start_qdrant_test_server(port).await.unwrap();

    let mut client = create_collections_client(port).await.unwrap();

    let request = tonic::Request::new(CreateCollection {
        collection_name: "new_qdrant_collection".to_string(),
        vectors_config: Some(VectorsConfig {
            config: Some(vectors_config::Config::Params(VectorParams {
                size: 256,
                distance: Distance::Cosine as i32,
                hnsw_config: None,
                quantization_config: None,
                on_disk: None,
                datatype: None,
                multivector_config: None,
            })),
        }),
        hnsw_config: None,
        wal_config: None,
        optimizers_config: None,
        shard_number: None,
        on_disk_payload: None,
        timeout: None,
        replication_factor: None,
        write_consistency_factor: None,
        quantization_config: None,
        sharding_method: None,
        sparse_vectors_config: None,
        strict_mode_config: None,
        metadata: std::collections::HashMap::new(),
    });

    let response = client.create(request).await.unwrap();
    assert!(response.into_inner().result);
}

#[tokio::test]
async fn test_qdrant_get_collection() {
    let port = 16022;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection with vectors
    let config = create_test_config();
    store.create_collection("get_test", config).unwrap();

    // Add some vectors
    use vectorizer::models::Vector;
    store
        .insert(
            "get_test",
            vec![
                Vector {
                    id: "v1".to_string(),
                    data: vec![0.1; 128],
                    sparse: None,
                    payload: None,
                },
                Vector {
                    id: "v2".to_string(),
                    data: vec![0.2; 128],
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    let mut client = create_collections_client(port).await.unwrap();

    let request = tonic::Request::new(GetCollectionInfoRequest {
        collection_name: "get_test".to_string(),
    });
    let response = client.get(request).await.unwrap();

    let info = response.into_inner().result.unwrap();
    assert_eq!(info.points_count, Some(2));
}

#[tokio::test]
async fn test_qdrant_delete_collection() {
    let port = 16023;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection
    let config = create_test_config();
    store.create_collection("delete_test", config).unwrap();

    let mut client = create_collections_client(port).await.unwrap();

    // Delete collection
    let request = tonic::Request::new(DeleteCollection {
        collection_name: "delete_test".to_string(),
        timeout: None,
    });
    let response = client.delete(request).await.unwrap();
    assert!(response.into_inner().result);

    // Verify deletion
    let list_request = tonic::Request::new(ListCollectionsRequest {});
    let list_response = client.list(list_request).await.unwrap();
    let collections = list_response.into_inner().collections;
    assert!(!collections.iter().any(|c| c.name == "delete_test"));
}

#[tokio::test]
async fn test_qdrant_collection_exists() {
    let port = 16024;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection
    let config = create_test_config();
    store.create_collection("exists_test", config).unwrap();

    let mut client = create_collections_client(port).await.unwrap();

    // Check exists
    let request = tonic::Request::new(CollectionExistsRequest {
        collection_name: "exists_test".to_string(),
    });
    let response = client.collection_exists(request).await.unwrap();
    assert!(response.into_inner().result.unwrap().exists);

    // Check non-existent
    let request = tonic::Request::new(CollectionExistsRequest {
        collection_name: "nonexistent".to_string(),
    });
    let response = client.collection_exists(request).await.unwrap();
    assert!(!response.into_inner().result.unwrap().exists);
}

// ============================================================================
// Points Service Tests
// ============================================================================

#[tokio::test]
async fn test_qdrant_upsert_points() {
    let port = 16030;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection
    let config = create_test_config();
    store.create_collection("upsert_test", config).unwrap();

    let mut client = create_points_client(port).await.unwrap();

    // Create vectors using the deprecated data field for compatibility
    let request = tonic::Request::new(UpsertPoints {
        collection_name: "upsert_test".to_string(),
        wait: Some(true),
        points: vec![
            PointStruct {
                id: Some(PointId {
                    point_id_options: Some(point_id::PointIdOptions::Uuid("point1".to_string())),
                }),
                payload: std::collections::HashMap::new(),
                vectors: Some(Vectors {
                    vectors_options: Some(vectors::VectorsOptions::Vector(Vector {
                        data: vec![0.1; 128],
                        indices: None,
                        vectors_count: None,
                        vector: Some(vector::Vector::Dense(DenseVector {
                            data: vec![0.1; 128],
                        })),
                    })),
                }),
            },
            PointStruct {
                id: Some(PointId {
                    point_id_options: Some(point_id::PointIdOptions::Uuid("point2".to_string())),
                }),
                payload: std::collections::HashMap::new(),
                vectors: Some(Vectors {
                    vectors_options: Some(vectors::VectorsOptions::Vector(Vector {
                        data: vec![0.2; 128],
                        indices: None,
                        vectors_count: None,
                        vector: Some(vector::Vector::Dense(DenseVector {
                            data: vec![0.2; 128],
                        })),
                    })),
                }),
            },
        ],
        ordering: None,
        shard_key_selector: None,
        update_filter: None,
    });

    let response = client.upsert(request).await.unwrap();
    let result = response.into_inner().result.unwrap();
    assert_eq!(result.status, UpdateStatus::Completed as i32);
}

#[tokio::test]
async fn test_qdrant_get_points() {
    let port = 16031;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection and add vectors
    let config = create_test_config();
    store.create_collection("get_points_test", config).unwrap();

    use vectorizer::models::Vector as VecModel;
    store
        .insert(
            "get_points_test",
            vec![
                VecModel {
                    id: "get1".to_string(),
                    data: vec![0.1; 128],
                    sparse: None,
                    payload: None,
                },
                VecModel {
                    id: "get2".to_string(),
                    data: vec![0.2; 128],
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    let mut client = create_points_client(port).await.unwrap();

    let request = tonic::Request::new(GetPoints {
        collection_name: "get_points_test".to_string(),
        ids: vec![
            PointId {
                point_id_options: Some(point_id::PointIdOptions::Uuid("get1".to_string())),
            },
            PointId {
                point_id_options: Some(point_id::PointIdOptions::Uuid("get2".to_string())),
            },
        ],
        with_payload: None,
        with_vectors: None,
        read_consistency: None,
        shard_key_selector: None,
        timeout: None,
    });

    let response = client.get(request).await.unwrap();
    let points = response.into_inner().result;
    assert_eq!(points.len(), 2);
}

#[tokio::test]
async fn test_qdrant_search_points() {
    let port = 16032;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection and add vectors
    let config = create_test_config();
    store.create_collection("search_test", config).unwrap();

    use vectorizer::models::Vector as VecModel;
    store
        .insert(
            "search_test",
            vec![
                VecModel {
                    id: "search1".to_string(),
                    data: vec![0.1; 128],
                    sparse: None,
                    payload: None,
                },
                VecModel {
                    id: "search2".to_string(),
                    data: vec![0.9; 128],
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    let mut client = create_points_client(port).await.unwrap();

    let request = tonic::Request::new(SearchPoints {
        collection_name: "search_test".to_string(),
        vector: vec![0.1; 128],
        limit: 5,
        filter: None,
        with_payload: None,
        with_vectors: None,
        params: None,
        score_threshold: None,
        offset: None,
        vector_name: None,
        read_consistency: None,
        timeout: None,
        shard_key_selector: None,
        sparse_indices: None,
    });

    let response = client.search(request).await.unwrap();
    let results = response.into_inner().result;
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_qdrant_count_points() {
    let port = 16033;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection and add vectors
    let config = create_test_config();
    store.create_collection("count_test", config).unwrap();

    use vectorizer::models::Vector as VecModel;
    store
        .insert(
            "count_test",
            vec![
                VecModel {
                    id: "c1".to_string(),
                    data: vec![0.1; 128],
                    sparse: None,
                    payload: None,
                },
                VecModel {
                    id: "c2".to_string(),
                    data: vec![0.2; 128],
                    sparse: None,
                    payload: None,
                },
                VecModel {
                    id: "c3".to_string(),
                    data: vec![0.3; 128],
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    let mut client = create_points_client(port).await.unwrap();

    let request = tonic::Request::new(CountPoints {
        collection_name: "count_test".to_string(),
        filter: None,
        exact: None,
        read_consistency: None,
        shard_key_selector: None,
        timeout: None,
    });

    let response = client.count(request).await.unwrap();
    let count = response.into_inner().result.unwrap().count;
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_qdrant_delete_points() {
    let port = 16034;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection and add vectors
    let config = create_test_config();
    store
        .create_collection("delete_points_test", config)
        .unwrap();

    use vectorizer::models::Vector as VecModel;
    store
        .insert(
            "delete_points_test",
            vec![
                VecModel {
                    id: "del1".to_string(),
                    data: vec![0.1; 128],
                    sparse: None,
                    payload: None,
                },
                VecModel {
                    id: "del2".to_string(),
                    data: vec![0.2; 128],
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    let mut client = create_points_client(port).await.unwrap();

    let request = tonic::Request::new(DeletePoints {
        collection_name: "delete_points_test".to_string(),
        wait: Some(true),
        points: Some(PointsSelector {
            points_selector_one_of: Some(points_selector::PointsSelectorOneOf::Points(
                PointsIdsList {
                    ids: vec![PointId {
                        point_id_options: Some(point_id::PointIdOptions::Uuid("del1".to_string())),
                    }],
                },
            )),
        }),
        ordering: None,
        shard_key_selector: None,
    });

    let response = client.delete(request).await.unwrap();
    let result = response.into_inner().result.unwrap();
    assert_eq!(result.status, UpdateStatus::Completed as i32);

    // Verify count
    let count_request = tonic::Request::new(CountPoints {
        collection_name: "delete_points_test".to_string(),
        filter: None,
        exact: None,
        read_consistency: None,
        shard_key_selector: None,
        timeout: None,
    });

    let count_response = client.count(count_request).await.unwrap();
    let count = count_response.into_inner().result.unwrap().count;
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_qdrant_scroll_points() {
    let port = 16035;
    let store = start_qdrant_test_server(port).await.unwrap();

    // Create collection and add vectors
    let config = create_test_config();
    store.create_collection("scroll_test", config).unwrap();

    use vectorizer::models::Vector as VecModel;
    for i in 0..20 {
        store
            .insert(
                "scroll_test",
                vec![VecModel {
                    id: format!("scroll_{i}"),
                    data: vec![i as f32 / 20.0; 128],
                    sparse: None,
                    payload: None,
                }],
            )
            .unwrap();
    }

    let mut client = create_points_client(port).await.unwrap();

    let request = tonic::Request::new(ScrollPoints {
        collection_name: "scroll_test".to_string(),
        limit: Some(10),
        offset: None,
        filter: None,
        with_payload: None,
        with_vectors: None,
        read_consistency: None,
        shard_key_selector: None,
        order_by: None,
        timeout: None,
    });

    let response = client.scroll(request).await.unwrap();
    let result = response.into_inner();
    assert_eq!(result.result.len(), 10);
}
