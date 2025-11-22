//! Shared helpers for gRPC tests

use std::sync::Arc;
use std::time::Duration;

use tonic::transport::Channel;
use vectorizer::db::VectorStore;
use vectorizer::grpc::vectorizer::vectorizer_service_client::VectorizerServiceClient;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};

/// Helper to create a test gRPC client
pub async fn create_test_client(
    port: u16,
) -> Result<VectorizerServiceClient<Channel>, Box<dyn std::error::Error>> {
    let addr = format!("http://127.0.0.1:{port}");
    let client = VectorizerServiceClient::connect(addr).await?;
    Ok(client)
}

/// Helper to create a test collection config
/// Uses Euclidean metric to avoid automatic normalization
pub fn create_test_config() -> CollectionConfig {
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
pub fn create_test_vector(_id: &str, seed: usize, dimension: usize) -> Vec<f32> {
    (0..dimension)
        .map(|i| ((seed * dimension + i) % 100) as f32 / 100.0)
        .collect()
}

/// Helper to start a test gRPC server
pub async fn start_test_server(port: u16) -> Result<Arc<VectorStore>, Box<dyn std::error::Error>> {
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

/// Helper to generate unique collection name
#[allow(dead_code)]
pub fn unique_collection_name(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}_{timestamp}")
}
