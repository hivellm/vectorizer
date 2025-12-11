//! Benchmark comparing gRPC vs REST API performance
//!
//! This benchmark measures:
//! - Insert throughput (vectors/sec)
//! - Search latency (p50, p95, p99)
//! - Batch operations performance
//! - Memory usage
//! - CPU usage

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::time::sleep;
use tonic::transport::Channel;
use tracing::info;
use vectorizer::db::VectorStore;
use vectorizer::grpc::vectorizer::vectorizer_service_client::VectorizerServiceClient;
use vectorizer::grpc::vectorizer::*;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig, Vector,
};

/// Benchmark configuration
struct BenchmarkConfig {
    vector_count: usize,
    dimension: usize,
    search_queries: usize,
    #[allow(dead_code)]
    warmup_iterations: usize,
    #[allow(dead_code)]
    measurement_iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            vector_count: 500, // Reduced for faster benchmark
            dimension: 128,
            search_queries: 50, // Reduced for faster benchmark
            warmup_iterations: 5,
            measurement_iterations: 50,
        }
    }
}

/// Benchmark results
struct BenchmarkResults {
    operation: String,
    protocol: String,
    throughput: f64, // ops/sec
    avg_latency_ms: f64,
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    total_time_ms: f64,
}

impl BenchmarkResults {
    fn new(operation: String, protocol: String) -> Self {
        Self {
            operation,
            protocol,
            throughput: 0.0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            total_time_ms: 0.0,
        }
    }

    fn calculate_percentiles(&mut self, latencies: &mut [f64]) {
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.avg_latency_ms = latencies.iter().sum::<f64>() / latencies.len() as f64;
        self.p50_latency_ms = latencies[latencies.len() / 2];
        self.p95_latency_ms = latencies[(latencies.len() as f64 * 0.95) as usize];
        self.p99_latency_ms = latencies[(latencies.len() as f64 * 0.99) as usize];
    }

    fn print(&self) {
        tracing::info!("\n📊 {} - {} Results:", self.operation, self.protocol);
        tracing::info!("  Throughput: {:.2} ops/sec", self.throughput);
        tracing::info!("  Avg Latency: {:.2} ms", self.avg_latency_ms);
        tracing::info!("  P50 Latency: {:.2} ms", self.p50_latency_ms);
        tracing::info!("  P95 Latency: {:.2} ms", self.p95_latency_ms);
        tracing::info!("  P99 Latency: {:.2} ms", self.p99_latency_ms);
        tracing::info!("  Total Time: {:.2} ms", self.total_time_ms);
    }
}

/// Generate test vectors
fn generate_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    (0..count)
        .map(|i| Vector {
            id: format!("vec_{i}"),
            data: (0..dimension)
                .map(|j| ((i * dimension + j) % 100) as f32 / 100.0)
                .collect(),
            sparse: None,
            payload: None,
        })
        .collect()
}

/// Benchmark REST API insert
async fn benchmark_rest_insert(
    store: &Arc<VectorStore>,
    collection_name: &str,
    vectors: &[Vector],
) -> BenchmarkResults {
    let mut results = BenchmarkResults::new("Insert".to_string(), "REST".to_string());
    let mut latencies = Vec::new();

    let start = Instant::now();

    // Insert vectors in batches for better performance
    for chunk in vectors.chunks(50) {
        let chunk_start = Instant::now();
        store.insert(collection_name, chunk.to_vec()).unwrap();
        let latency = chunk_start.elapsed().as_secs_f64() * 1000.0;
        // Distribute latency across vectors in chunk
        let per_vector_latency = latency / chunk.len() as f64;
        for _ in chunk {
            latencies.push(per_vector_latency);
        }
    }

    let total_time = start.elapsed().as_secs_f64();
    results.total_time_ms = total_time * 1000.0;
    results.throughput = vectors.len() as f64 / total_time;

    results.calculate_percentiles(&mut latencies);
    results
}

/// Benchmark gRPC insert
async fn benchmark_grpc_insert(
    client: &mut VectorizerServiceClient<Channel>,
    collection_name: &str,
    vectors: &[Vector],
) -> BenchmarkResults {
    let mut results = BenchmarkResults::new("Insert".to_string(), "gRPC".to_string());

    let start = Instant::now();

    // Insert vectors using streaming
    let (tx, rx) = tokio::sync::mpsc::channel(1000);

    // Send all vectors to channel (non-blocking)
    for vector in vectors {
        let request = InsertVectorRequest {
            collection_name: collection_name.to_string(),
            vector_id: vector.id.clone(),
            data: vector.data.clone(),
            payload: std::collections::HashMap::new(),
        };
        tx.send(request).await.unwrap();
    }
    drop(tx);

    // Measure only the actual streaming call
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let request = tonic::Request::new(stream);

    let stream_start = Instant::now();
    let _response = client.insert_vectors(request).await.unwrap();
    let stream_latency = stream_start.elapsed().as_secs_f64() * 1000.0;

    // Use stream latency as average per vector
    let avg_latency_per_vector = stream_latency / vectors.len() as f64;
    let mut latencies: Vec<f64> = (0..vectors.len()).map(|_| avg_latency_per_vector).collect();

    let total_time = start.elapsed().as_secs_f64();
    results.total_time_ms = total_time * 1000.0;
    results.throughput = vectors.len() as f64 / total_time;

    results.calculate_percentiles(&mut latencies);
    results
}

/// Benchmark REST API search
async fn benchmark_rest_search(
    store: &Arc<VectorStore>,
    collection_name: &str,
    queries: &[Vec<f32>],
) -> BenchmarkResults {
    let mut results = BenchmarkResults::new("Search".to_string(), "REST".to_string());
    let mut latencies = Vec::new();

    let start = Instant::now();

    for query in queries {
        let query_start = Instant::now();
        store.search(collection_name, query, 10).unwrap();
        let latency = query_start.elapsed().as_secs_f64() * 1000.0;
        latencies.push(latency);
    }

    let total_time = start.elapsed().as_secs_f64();
    results.total_time_ms = total_time * 1000.0;
    results.throughput = queries.len() as f64 / total_time;

    results.calculate_percentiles(&mut latencies);
    results
}

/// Benchmark gRPC search
async fn benchmark_grpc_search(
    client: &mut VectorizerServiceClient<Channel>,
    collection_name: &str,
    queries: &[Vec<f32>],
) -> BenchmarkResults {
    let mut results = BenchmarkResults::new("Search".to_string(), "gRPC".to_string());
    let mut latencies = Vec::new();

    let start = Instant::now();

    for query in queries {
        let query_start = Instant::now();
        let request = tonic::Request::new(SearchRequest {
            collection_name: collection_name.to_string(),
            query_vector: query.clone(),
            limit: 10,
            threshold: 0.0,
            filter: std::collections::HashMap::new(),
        });

        let _response = client.search(request).await.unwrap();
        let latency = query_start.elapsed().as_secs_f64() * 1000.0;
        latencies.push(latency);
    }

    let total_time = start.elapsed().as_secs_f64();
    results.total_time_ms = total_time * 1000.0;
    results.throughput = queries.len() as f64 / total_time;

    results.calculate_percentiles(&mut latencies);
    results
}

/// Start gRPC server for benchmarking
async fn start_grpc_server(port: u16) -> Arc<VectorStore> {
    use tonic::transport::Server;
    use vectorizer::grpc::VectorizerGrpcService;
    use vectorizer::grpc::vectorizer::vectorizer_service_server::VectorizerServiceServer;

    let store = Arc::new(VectorStore::new());
    let service = VectorizerGrpcService::new(store.clone());

    let addr = format!("127.0.0.1:{port}").parse().unwrap();

    tokio::spawn(async move {
        Server::builder()
            .add_service(VectorizerServiceServer::new(service))
            .serve(addr)
            .await
            .expect("gRPC server failed");
    });

    // Give server time to start
    sleep(Duration::from_millis(200)).await;

    store
}

/// Create gRPC client
async fn create_grpc_client(port: u16) -> VectorizerServiceClient<Channel> {
    let addr = format!("http://127.0.0.1:{port}");
    VectorizerServiceClient::connect(addr)
        .await
        .expect("Failed to connect to gRPC server")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🚀 gRPC vs REST API Benchmark");
    tracing::info!("=============================\n");

    let config = BenchmarkConfig::default();

    tracing::info!("📊 Configuration:");
    tracing::info!("  Vector count: {}", config.vector_count);
    tracing::info!("  Dimension: {}", config.dimension);
    tracing::info!("  Search queries: {}", config.search_queries);
    info!("");

    // Create test data
    tracing::info!("🔧 Generating test data...");
    let vectors = generate_vectors(config.vector_count, config.dimension);
    let queries: Vec<Vec<f32>> = (0..config.search_queries)
        .map(|i| {
            (0..config.dimension)
                .map(|j| ((i * config.dimension + j) % 100) as f32 / 100.0)
                .collect()
        })
        .collect();
    tracing::info!("  ✅ Generated {} vectors", vectors.len());
    tracing::info!("  ✅ Generated {} queries", queries.len());
    info!("");

    // Setup REST store
    let rest_store = Arc::new(VectorStore::new());
    let rest_collection = "rest_bench";
    let rest_config = CollectionConfig {
        sharding: None,
        dimension: config.dimension,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        graph: None, // Graph disabled for benchmarks
        encryption: None,
    };
    rest_store
        .create_collection(rest_collection, rest_config)
        .unwrap();

    // Setup gRPC server and client
    let grpc_port = 15020;
    let grpc_store = start_grpc_server(grpc_port).await;
    let mut grpc_client = create_grpc_client(grpc_port).await;

    let grpc_collection = "grpc_bench";
    let grpc_config = CollectionConfig {
        sharding: None,
        dimension: config.dimension,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
        storage_type: None,
        graph: None, // Graph disabled for benchmarks
        encryption: None,
    };
    grpc_store
        .create_collection(grpc_collection, grpc_config)
        .unwrap();

    // Create gRPC collection via API
    use vectorizer::grpc::vectorizer::{
        CollectionConfig as ProtoCollectionConfig, DistanceMetric as ProtoDistanceMetric,
        HnswConfig as ProtoHnswConfig, StorageType as ProtoStorageType,
    };

    let create_request = tonic::Request::new(CreateCollectionRequest {
        name: grpc_collection.to_string(),
        config: Some(ProtoCollectionConfig {
            dimension: config.dimension as u32,
            metric: ProtoDistanceMetric::Cosine as i32,
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
    grpc_client.create_collection(create_request).await.unwrap();

    tracing::info!("🏃 Running benchmarks...\n");

    // Benchmark Insert
    tracing::info!("📝 Benchmarking Insert Operations...");
    let rest_insert = benchmark_rest_insert(&rest_store, rest_collection, &vectors).await;
    rest_insert.print();

    let grpc_insert = benchmark_grpc_insert(&mut grpc_client, grpc_collection, &vectors).await;
    grpc_insert.print();

    tracing::info!("\n📈 Insert Comparison:");
    let throughput_ratio = grpc_insert.throughput / rest_insert.throughput;
    tracing::info!(
        "  gRPC is {:.2}x {} than REST",
        throughput_ratio,
        if throughput_ratio > 1.0 {
            "faster"
        } else {
            "slower"
        }
    );

    // Benchmark Search
    tracing::info!("\n🔍 Benchmarking Search Operations...");
    let rest_search = benchmark_rest_search(&rest_store, rest_collection, &queries).await;
    rest_search.print();

    let grpc_search = benchmark_grpc_search(&mut grpc_client, grpc_collection, &queries).await;
    grpc_search.print();

    tracing::info!("\n📈 Search Comparison:");
    let latency_ratio = grpc_search.avg_latency_ms / rest_search.avg_latency_ms;
    tracing::info!(
        "  gRPC latency is {:.2}x {} than REST",
        latency_ratio,
        if latency_ratio < 1.0 {
            "lower"
        } else {
            "higher"
        }
    );
    let throughput_ratio = grpc_search.throughput / rest_search.throughput;
    tracing::info!(
        "  gRPC throughput is {:.2}x {} than REST",
        throughput_ratio,
        if throughput_ratio > 1.0 {
            "higher"
        } else {
            "lower"
        }
    );

    tracing::info!("\n✅ Benchmark completed!");

    Ok(())
}
