//! Vector Operations Tests
//!
//! Tests for vector CRUD operations:
//! - Insert vector
//! - Get vector
//! - Update vector
//! - Delete vector
//! - Streaming bulk insert
//! - Payload handling

use std::collections::HashMap;

use vectorizer::grpc::vectorizer::*;

use crate::grpc::helpers::*;

#[tokio::test]
async fn test_insert_and_get_vector() {
    let port = 15030;
    let store = start_test_server(port).await.unwrap();

    // Create collection
    let config = create_test_config();
    store.create_collection("test_insert", config).unwrap();

    let mut client = create_test_client(port).await.unwrap();

    // Insert vector
    let test_vector = create_test_vector("vec1", 1, 128);
    let insert_request = tonic::Request::new(InsertVectorRequest {
        collection_name: "test_insert".to_string(),
        vector_id: "vec1".to_string(),
        data: test_vector.clone(),
        payload: HashMap::new(),
    });

    let insert_response = client.insert_vector(insert_request).await.unwrap();
    assert!(insert_response.into_inner().success);

    // Get vector
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: "test_insert".to_string(),
        vector_id: "vec1".to_string(),
    });

    let get_response = client.get_vector(get_request).await.unwrap();
    let vector = get_response.into_inner();

    assert_eq!(vector.vector_id, "vec1");
    assert_eq!(vector.data.len(), test_vector.len());
}

#[tokio::test]
async fn test_insert_with_payload() {
    let port = 15031;
    let store = start_test_server(port).await.unwrap();

    let config = create_test_config();
    store.create_collection("payload_test", config).unwrap();

    let mut client = create_test_client(port).await.unwrap();

    let mut payload = HashMap::new();
    payload.insert("category".to_string(), "test".to_string());
    payload.insert("index".to_string(), "1".to_string());

    let insert_request = tonic::Request::new(InsertVectorRequest {
        collection_name: "payload_test".to_string(),
        vector_id: "vec1".to_string(),
        data: create_test_vector("vec1", 1, 128),
        payload: payload.clone(),
    });

    let insert_response = client.insert_vector(insert_request).await.unwrap();
    assert!(insert_response.into_inner().success);

    // Get and verify payload
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: "payload_test".to_string(),
        vector_id: "vec1".to_string(),
    });
    let get_response = client.get_vector(get_request).await.unwrap();
    let vector = get_response.into_inner();

    assert!(vector.payload.contains_key("category"));
    assert!(vector.payload.contains_key("index"));
}

#[tokio::test]
async fn test_update_vector() {
    let port = 15032;
    let store = start_test_server(port).await.unwrap();

    // Create collection and insert vector
    let config = create_test_config();
    store.create_collection("test_update", config).unwrap();

    use vectorizer::models::Vector;
    let original_data = create_test_vector("vec1", 1, 128);
    store
        .insert(
            "test_update",
            vec![Vector {
                id: "vec1".to_string(),
                data: original_data.clone(),
                sparse: None,
                payload: None,
            }],
        )
        .unwrap();

    let mut client = create_test_client(port).await.unwrap();

    // Update vector
    let updated_data = create_test_vector("vec1", 100, 128);
    let update_request = tonic::Request::new(UpdateVectorRequest {
        collection_name: "test_update".to_string(),
        vector_id: "vec1".to_string(),
        data: updated_data.clone(),
        payload: HashMap::new(),
    });

    let update_response = client.update_vector(update_request).await.unwrap();
    assert!(update_response.into_inner().success);

    // Verify update
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: "test_update".to_string(),
        vector_id: "vec1".to_string(),
    });
    let get_response = client.get_vector(get_request).await.unwrap();
    let vector = get_response.into_inner();

    assert_eq!(vector.data.len(), updated_data.len());
}

#[tokio::test]
async fn test_delete_vector() {
    let port = 15033;
    let store = start_test_server(port).await.unwrap();

    // Create collection and insert vector
    let config = create_test_config();
    store.create_collection("test_delete", config).unwrap();

    use vectorizer::models::Vector;
    let test_vector = create_test_vector("vec1", 1, 128);
    store
        .insert(
            "test_delete",
            vec![Vector {
                id: "vec1".to_string(),
                data: test_vector,
                sparse: None,
                payload: None,
            }],
        )
        .unwrap();

    let mut client = create_test_client(port).await.unwrap();

    // Delete vector
    let delete_request = tonic::Request::new(DeleteVectorRequest {
        collection_name: "test_delete".to_string(),
        vector_id: "vec1".to_string(),
    });

    let delete_response = client.delete_vector(delete_request).await.unwrap();
    assert!(delete_response.into_inner().success);

    // Verify deletion
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: "test_delete".to_string(),
        vector_id: "vec1".to_string(),
    });

    let get_response = client.get_vector(get_request).await;
    assert!(get_response.is_err());
}

#[tokio::test]
async fn test_streaming_bulk_insert() {
    let port = 15034;
    let store = start_test_server(port).await.unwrap();

    // Create collection
    let config = create_test_config();
    store.create_collection("test_streaming", config).unwrap();

    let mut client = create_test_client(port).await.unwrap();

    // Create streaming request
    let (tx, rx) = tokio::sync::mpsc::channel(10);

    // Send multiple vectors
    for i in 0..5 {
        let vector_data = create_test_vector(&format!("vec{i}"), i, 128);
        let request = InsertVectorRequest {
            collection_name: "test_streaming".to_string(),
            vector_id: format!("vec{i}"),
            data: vector_data,
            payload: HashMap::new(),
        };
        tx.send(request).await.unwrap();
    }
    drop(tx);

    // Convert to streaming
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let request = tonic::Request::new(stream);

    let response = client.insert_vectors(request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.inserted_count, 5);
    assert_eq!(result.failed_count, 0);

    // Verify vectors were inserted
    let collection = store.get_collection("test_streaming").unwrap();
    assert_eq!(collection.vector_count(), 5);
}
