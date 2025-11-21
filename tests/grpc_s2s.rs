//! Server-to-Server (S2S) integration tests for gRPC API
//!
//! These tests connect to a REAL running Vectorizer server instance.
//! The server should be running on the configured address.
//!
//! Usage:
//!   VECTORIZER_GRPC_HOST=127.0.0.1 VECTORIZER_GRPC_PORT=15003 cargo test --test grpc_s2s
//!
//! Default: http://127.0.0.1:15003

use std::collections::HashMap;
use std::env;
use std::time::Duration;

use tokio::time::timeout;
use tonic::transport::Channel;
use vectorizer::grpc::vectorizer::vectorizer_service_client::VectorizerServiceClient;
use vectorizer::grpc::vectorizer::*;

// Import protobuf types
use vectorizer::grpc::vectorizer::{
    CollectionConfig as ProtoCollectionConfig,
    DistanceMetric as ProtoDistanceMetric,
    HnswConfig as ProtoHnswConfig,
    StorageType as ProtoStorageType,
};

/// Get gRPC server address from environment or use default
fn get_grpc_address() -> String {
    let host = env::var("VECTORIZER_GRPC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("VECTORIZER_GRPC_PORT")
        .unwrap_or_else(|_| "15003".to_string())
        .parse::<u16>()
        .unwrap_or(15003);
    format!("http://{}:{}", host, port)
}

/// Helper to create a test gRPC client connected to real server
async fn create_real_client() -> Result<VectorizerServiceClient<Channel>, Box<dyn std::error::Error>> {
    let addr = get_grpc_address();
    println!("🔌 Connecting to gRPC server at: {}", addr);
    let client = VectorizerServiceClient::connect(addr).await?;
    Ok(client)
}

/// Helper to create a test vector with correct dimension
fn create_test_vector(id: &str, seed: usize, dimension: usize) -> Vec<f32> {
    (0..dimension)
        .map(|i| ((seed * dimension + i) % 100) as f32 / 100.0)
        .collect()
}

/// Helper to generate unique collection name
fn unique_collection_name(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}_{}", prefix, timestamp)
}

/// Test 1: Health Check on Real Server
#[tokio::test]
async fn test_real_server_health_check() {
    let mut client = create_real_client().await.unwrap();

    let request = tonic::Request::new(HealthCheckRequest {});
    let response = timeout(Duration::from_secs(10), client.health_check(request))
        .await
        .expect("Health check timed out")
        .unwrap();

    let health = response.into_inner();
    println!("✅ Server Health: {}", health.status);
    println!("   Version: {}", health.version);
    println!("   Timestamp: {}", health.timestamp);
    
    assert_eq!(health.status, "healthy");
    assert!(!health.version.is_empty());
    assert!(health.timestamp > 0);
}

/// Test 2: Get Stats from Real Server
#[tokio::test]
async fn test_real_server_stats() {
    let mut client = create_real_client().await.unwrap();

    let request = tonic::Request::new(GetStatsRequest {});
    let response = timeout(Duration::from_secs(10), client.get_stats(request))
        .await
        .expect("Get stats timed out")
        .unwrap();

    let stats = response.into_inner();
    println!("✅ Server Stats:");
    println!("   Collections: {}", stats.collections_count);
    println!("   Total Vectors: {}", stats.total_vectors);
    println!("   Uptime: {}s", stats.uptime_seconds);
    println!("   Version: {}", stats.version);

    assert!(!stats.version.is_empty());
    assert!(stats.uptime_seconds >= 0);
}

/// Test 3: List Collections on Real Server
#[tokio::test]
async fn test_real_server_list_collections() {
    let mut client = create_real_client().await.unwrap();

    let request = tonic::Request::new(ListCollectionsRequest {});
    let response = timeout(Duration::from_secs(10), client.list_collections(request))
        .await
        .expect("List collections timed out")
        .unwrap();

    let collections = response.into_inner().collection_names;
    println!("✅ Found {} collections on server", collections.len());
    for (i, name) in collections.iter().take(10).enumerate() {
        println!("   {}. {}", i + 1, name);
    }
    if collections.len() > 10 {
        println!("   ... and {} more", collections.len() - 10);
    }
}

/// Test 4: Create Collection on Real Server
#[tokio::test]
async fn test_real_server_create_collection() {
    let mut client = create_real_client().await.unwrap();

    let collection_name = unique_collection_name("s2s_test");
    println!("📝 Creating collection: {}", collection_name);

    let request = tonic::Request::new(CreateCollectionRequest {
        name: collection_name.clone(),
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

    let response = timeout(Duration::from_secs(10), client.create_collection(request))
        .await
        .expect("Create collection timed out")
        .unwrap();

    let result = response.into_inner();
    println!("✅ Create Collection Result: {}", result.message);
    assert!(result.success);

    // Verify it exists
    let list_request = tonic::Request::new(ListCollectionsRequest {});
    let list_response = timeout(Duration::from_secs(10), client.list_collections(list_request))
        .await
        .unwrap()
        .unwrap();
    let collections = list_response.into_inner().collection_names;
    assert!(collections.contains(&collection_name));
}

/// Test 5: Insert and Get Vector on Real Server
#[tokio::test]
async fn test_real_server_insert_and_get() {
    let mut client = create_real_client().await.unwrap();

    let collection_name = unique_collection_name("s2s_insert");
    println!("📝 Testing insert/get on collection: {}", collection_name);

    // Create collection
    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: collection_name.clone(),
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
    timeout(Duration::from_secs(10), client.create_collection(create_request))
        .await
        .unwrap()
        .unwrap();

    // Insert vector
    let vector_data = create_test_vector("vec1", 1, 128);
    let mut payload = HashMap::new();
    payload.insert("test".to_string(), "s2s".to_string());
    payload.insert("timestamp".to_string(), format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));

    let insert_request = tonic::Request::new(InsertVectorRequest {
        collection_name: collection_name.clone(),
        vector_id: "vec1".to_string(),
        data: vector_data.clone(),
        payload: payload.clone(),
    });

    let insert_response = timeout(Duration::from_secs(10), client.insert_vector(insert_request))
        .await
        .expect("Insert timed out")
        .unwrap();
    assert!(insert_response.into_inner().success);
    println!("✅ Vector inserted successfully");

    // Get vector
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: collection_name.clone(),
        vector_id: "vec1".to_string(),
    });

    let get_response = timeout(Duration::from_secs(10), client.get_vector(get_request))
        .await
        .expect("Get timed out")
        .unwrap();

    let vector = get_response.into_inner();
    println!("✅ Vector retrieved:");
    println!("   ID: {}", vector.vector_id);
    println!("   Dimension: {}", vector.data.len());
    println!("   Payload keys: {}", vector.payload.len());

    assert_eq!(vector.vector_id, "vec1");
    assert_eq!(vector.data.len(), 128);
    assert!(!vector.payload.is_empty());
}

/// Test 6: Search on Real Server
#[tokio::test]
async fn test_real_server_search() {
    let mut client = create_real_client().await.unwrap();

    let collection_name = unique_collection_name("s2s_search");
    println!("📝 Testing search on collection: {}", collection_name);

    // Create collection
    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: collection_name.clone(),
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
    timeout(Duration::from_secs(10), client.create_collection(create_request))
        .await
        .unwrap()
        .unwrap();

    // Insert multiple vectors
    for i in 0..5 {
        let vector_data = create_test_vector(&format!("vec{}", i), i, 128);
        let insert_request = tonic::Request::new(InsertVectorRequest {
            collection_name: collection_name.clone(),
            vector_id: format!("vec{}", i),
            data: vector_data,
            payload: HashMap::new(),
        });
        timeout(Duration::from_secs(10), client.insert_vector(insert_request))
            .await
            .unwrap()
            .unwrap();
    }
    println!("✅ Inserted 5 vectors");

    // Search
    let query = create_test_vector("query", 1, 128);
    let search_request = tonic::Request::new(SearchRequest {
        collection_name: collection_name.clone(),
        query_vector: query,
        limit: 3,
        threshold: 0.0,
        filter: HashMap::new(),
    });

    let search_response = timeout(Duration::from_secs(10), client.search(search_request))
        .await
        .expect("Search timed out")
        .unwrap();

    let results = search_response.into_inner().results;
    println!("✅ Search returned {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("   {}. {} (score: {:.4})", i + 1, result.id, result.score);
    }

    assert!(!results.is_empty());
    assert!(results.len() <= 3);
}

/// Test 7: Streaming Bulk Insert on Real Server
#[tokio::test]
async fn test_real_server_streaming_bulk_insert() {
    let mut client = create_real_client().await.unwrap();

    let collection_name = unique_collection_name("s2s_bulk");
    println!("📝 Testing bulk insert on collection: {}", collection_name);

    // Create collection
    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: collection_name.clone(),
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
    timeout(Duration::from_secs(10), client.create_collection(create_request))
        .await
        .unwrap()
        .unwrap();

    // Create streaming request
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Send 20 vectors
    for i in 0..20 {
        let request = InsertVectorRequest {
            collection_name: collection_name.clone(),
            vector_id: format!("vec{}", i),
            data: create_test_vector(&format!("vec{}", i), i, 128),
            payload: HashMap::new(),
        };
        tx.send(request).await.unwrap();
    }
    drop(tx);

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let request = tonic::Request::new(stream);

    let response = timeout(Duration::from_secs(30), client.insert_vectors(request))
        .await
        .expect("Bulk insert timed out")
        .unwrap();

    let result = response.into_inner();
    println!("✅ Bulk Insert Result:");
    println!("   Inserted: {}", result.inserted_count);
    println!("   Failed: {}", result.failed_count);
    if !result.errors.is_empty() {
        println!("   Errors: {:?}", result.errors);
    }

    assert_eq!(result.inserted_count, 20);
    assert_eq!(result.failed_count, 0);
}

/// Test 8: Batch Search on Real Server
#[tokio::test]
async fn test_real_server_batch_search() {
    let mut client = create_real_client().await.unwrap();

    let collection_name = unique_collection_name("s2s_batch");
    println!("📝 Testing batch search on collection: {}", collection_name);

    // Create collection and insert vectors
    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: collection_name.clone(),
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
    timeout(Duration::from_secs(10), client.create_collection(create_request))
        .await
        .unwrap()
        .unwrap();

    // Insert vectors
    for i in 0..10 {
        let vector_data = create_test_vector(&format!("vec{}", i), i, 128);
        let insert_request = tonic::Request::new(InsertVectorRequest {
            collection_name: collection_name.clone(),
            vector_id: format!("vec{}", i),
            data: vector_data,
            payload: HashMap::new(),
        });
        timeout(Duration::from_secs(10), client.insert_vector(insert_request))
            .await
            .unwrap()
            .unwrap();
    }

    // Batch search
    let batch_queries = vec![
        SearchRequest {
            collection_name: collection_name.clone(),
            query_vector: create_test_vector("query1", 1, 128),
            limit: 3,
            threshold: 0.0,
            filter: HashMap::new(),
        },
        SearchRequest {
            collection_name: collection_name.clone(),
            query_vector: create_test_vector("query2", 2, 128),
            limit: 3,
            threshold: 0.0,
            filter: HashMap::new(),
        },
    ];

    let batch_request = tonic::Request::new(BatchSearchRequest {
        collection_name: collection_name.clone(),
        queries: batch_queries,
    });

    let batch_response = timeout(Duration::from_secs(10), client.batch_search(batch_request))
        .await
        .expect("Batch search timed out")
        .unwrap();

    let batch_results = batch_response.into_inner().results;
    println!("✅ Batch Search returned {} result sets", batch_results.len());
    for (i, result_set) in batch_results.iter().enumerate() {
        println!("   Query {}: {} results", i + 1, result_set.results.len());
    }

    assert_eq!(batch_results.len(), 2);
    assert!(!batch_results[0].results.is_empty());
    assert!(!batch_results[1].results.is_empty());
}

/// Test 9: Update and Delete on Real Server
#[tokio::test]
async fn test_real_server_update_and_delete() {
    let mut client = create_real_client().await.unwrap();

    let collection_name = unique_collection_name("s2s_update");
    println!("📝 Testing update/delete on collection: {}", collection_name);

    // Create collection
    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: collection_name.clone(),
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
    timeout(Duration::from_secs(10), client.create_collection(create_request))
        .await
        .unwrap()
        .unwrap();

    // Insert
    let original_data = create_test_vector("vec1", 1, 128);
    let insert_request = tonic::Request::new(InsertVectorRequest {
        collection_name: collection_name.clone(),
        vector_id: "vec1".to_string(),
        data: original_data.clone(),
        payload: HashMap::new(),
    });
    timeout(Duration::from_secs(10), client.insert_vector(insert_request))
        .await
        .unwrap()
        .unwrap();
    println!("✅ Vector inserted");

    // Update
    let updated_data = create_test_vector("vec1", 100, 128);
    let mut updated_payload = HashMap::new();
    updated_payload.insert("updated".to_string(), "true".to_string());

    let update_request = tonic::Request::new(UpdateVectorRequest {
        collection_name: collection_name.clone(),
        vector_id: "vec1".to_string(),
        data: updated_data,
        payload: updated_payload.clone(),
    });
    let update_response = timeout(Duration::from_secs(10), client.update_vector(update_request))
        .await
        .expect("Update timed out")
        .unwrap();
    assert!(update_response.into_inner().success);
    println!("✅ Vector updated");

    // Verify update
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: collection_name.clone(),
        vector_id: "vec1".to_string(),
    });
    let get_response = timeout(Duration::from_secs(10), client.get_vector(get_request))
        .await
        .unwrap()
        .unwrap();
    let vector = get_response.into_inner();
    assert!(vector.payload.contains_key("updated"));
    println!("✅ Update verified");

    // Delete
    let delete_request = tonic::Request::new(DeleteVectorRequest {
        collection_name: collection_name.clone(),
        vector_id: "vec1".to_string(),
    });
    let delete_response = timeout(Duration::from_secs(10), client.delete_vector(delete_request))
        .await
        .expect("Delete timed out")
        .unwrap();
    assert!(delete_response.into_inner().success);
    println!("✅ Vector deleted");

    // Verify deletion
    let get_request = tonic::Request::new(GetVectorRequest {
        collection_name: collection_name.clone(),
        vector_id: "vec1".to_string(),
    });
    let get_response = timeout(Duration::from_secs(10), client.get_vector(get_request)).await;
    assert!(get_response.is_err() || get_response.unwrap().is_err());
    println!("✅ Deletion verified");
}

/// Test 10: Get Collection Info on Real Server
#[tokio::test]
async fn test_real_server_get_collection_info() {
    let mut client = create_real_client().await.unwrap();

    let collection_name = unique_collection_name("s2s_info");
    println!("📝 Testing collection info on: {}", collection_name);

    // Create collection
    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: collection_name.clone(),
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
    timeout(Duration::from_secs(10), client.create_collection(create_request))
        .await
        .unwrap()
        .unwrap();

    // Insert some vectors
    for i in 0..3 {
        let vector_data = create_test_vector(&format!("vec{}", i), i, 128);
        let insert_request = tonic::Request::new(InsertVectorRequest {
            collection_name: collection_name.clone(),
            vector_id: format!("vec{}", i),
            data: vector_data,
            payload: HashMap::new(),
        });
        timeout(Duration::from_secs(10), client.insert_vector(insert_request))
            .await
            .unwrap()
            .unwrap();
    }

    // Get collection info
    let info_request = tonic::Request::new(GetCollectionInfoRequest {
        collection_name: collection_name.clone(),
    });
    let info_response = timeout(Duration::from_secs(10), client.get_collection_info(info_request))
        .await
        .expect("Get collection info timed out")
        .unwrap();

    let info = info_response.into_inner().info.unwrap();
    println!("✅ Collection Info:");
    println!("   Name: {}", info.name);
    println!("   Vector Count: {}", info.vector_count);
    println!("   Dimension: {}", info.config.as_ref().unwrap().dimension);
    println!("   Created: {}", info.created_at);
    println!("   Updated: {}", info.updated_at);

    assert_eq!(info.name, collection_name);
    assert_eq!(info.vector_count, 3);
    assert_eq!(info.config.as_ref().unwrap().dimension, 128);
}

