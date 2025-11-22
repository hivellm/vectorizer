//! Collection Management Tests
//!
//! Tests for collection operations:
//! - List collections
//! - Create collection
//! - Get collection info
//! - Delete collection
//! - Multiple collections

use vectorizer::grpc::vectorizer::*;

use crate::grpc::helpers::*;

#[tokio::test]
async fn test_list_collections() {
    let port = 15020;
    let _store = start_test_server(port).await.unwrap();

    // Create a collection via gRPC
    let mut client = create_test_client(port).await.unwrap();

    use vectorizer::grpc::vectorizer::{
        CollectionConfig as ProtoCollectionConfig, DistanceMetric as ProtoDistanceMetric,
        HnswConfig as ProtoHnswConfig, StorageType as ProtoStorageType,
    };

    let config = ProtoCollectionConfig {
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
    };

    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: "test_collection".to_string(),
        config: Some(config),
    });
    client.create_collection(create_request).await.unwrap();

    let request = tonic::Request::new(ListCollectionsRequest {});
    let response = client.list_collections(request).await.unwrap();

    let collections = response.into_inner().collection_names;
    assert!(collections.contains(&"test_collection".to_string()));
}

#[tokio::test]
async fn test_create_collection() {
    let port = 15021;
    let _store = start_test_server(port).await.unwrap();

    let mut client = create_test_client(port).await.unwrap();

    use vectorizer::grpc::vectorizer::{
        CollectionConfig as ProtoCollectionConfig, DistanceMetric as ProtoDistanceMetric,
        HnswConfig as ProtoHnswConfig, StorageType as ProtoStorageType,
    };

    let config = ProtoCollectionConfig {
        dimension: 128,
        metric: ProtoDistanceMetric::Cosine as i32,
        hnsw_config: Some(ProtoHnswConfig {
            m: 16,
            ef_construction: 200,
            ef: 50,
            seed: 42,
        }),
        quantization: None,
        storage_type: ProtoStorageType::Memory as i32,
    };

    let request = tonic::Request::new(CreateCollectionRequest {
        name: "grpc_test_collection".to_string(),
        config: Some(config),
    });

    let response = client.create_collection(request).await.unwrap();
    let result = response.into_inner();

    assert!(result.success);
    assert!(result.message.contains("created successfully"));
}

#[tokio::test]
async fn test_get_collection_info() {
    let port = 15022;
    let store = start_test_server(port).await.unwrap();

    // Create collection and insert vectors
    let config = create_test_config();
    store.create_collection("info_test", config).unwrap();

    use vectorizer::models::Vector;
    store
        .insert(
            "info_test",
            vec![
                Vector {
                    id: "vec1".to_string(),
                    data: create_test_vector("vec1", 1, 128),
                    sparse: None,
                    payload: None,
                },
                Vector {
                    id: "vec2".to_string(),
                    data: create_test_vector("vec2", 2, 128),
                    sparse: None,
                    payload: None,
                },
            ],
        )
        .unwrap();

    let mut client = create_test_client(port).await.unwrap();

    let info_request = tonic::Request::new(GetCollectionInfoRequest {
        collection_name: "info_test".to_string(),
    });
    let info_response = client.get_collection_info(info_request).await.unwrap();
    let info = info_response.into_inner().info.unwrap();

    assert_eq!(info.name, "info_test");
    assert_eq!(info.vector_count, 2);
    assert_eq!(info.config.as_ref().unwrap().dimension, 128);
}

#[tokio::test]
async fn test_delete_collection() {
    let port = 15023;
    let store = start_test_server(port).await.unwrap();

    // Create collection
    let config = create_test_config();
    store.create_collection("delete_test", config).unwrap();

    let mut client = create_test_client(port).await.unwrap();

    // Delete collection
    let delete_request = tonic::Request::new(DeleteCollectionRequest {
        collection_name: "delete_test".to_string(),
    });
    let delete_response = client.delete_collection(delete_request).await.unwrap();
    assert!(delete_response.into_inner().success);

    // Verify deletion
    let list_request = tonic::Request::new(ListCollectionsRequest {});
    let list_response = client.list_collections(list_request).await.unwrap();
    let collections = list_response.into_inner().collection_names;
    assert!(!collections.contains(&"delete_test".to_string()));
}

#[tokio::test]
async fn test_multiple_collections() {
    let port = 15024;
    let store = start_test_server(port).await.unwrap();

    // Create multiple collections
    for i in 0..5 {
        let config = create_test_config();
        store
            .create_collection(&format!("multi_{i}"), config)
            .unwrap();
    }

    let mut client = create_test_client(port).await.unwrap();

    // Verify all collections exist
    let list_request = tonic::Request::new(ListCollectionsRequest {});
    let list_response = client.list_collections(list_request).await.unwrap();
    let collections = list_response.into_inner().collection_names;

    for i in 0..5 {
        assert!(collections.contains(&format!("multi_{i}")));
    }
}
