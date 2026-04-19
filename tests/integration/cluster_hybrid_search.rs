//! End-to-end tests for the cluster `RemoteHybridSearch` RPC introduced in
//! phase4_add-hybrid-search-to-distributed-grpc-client.
//!
//! Two scenarios:
//!
//! 1. `RemoteHybridSearch` against a single-node `ClusterGrpcService` — the
//!    response must contain hybrid results scored against both the dense and
//!    sparse query, exercising the same code path that distributed shards
//!    would hit.
//! 2. The same client request against a server that does **not** register
//!    `ClusterServiceServer` — tonic returns `Unimplemented`, and the
//!    `ClusterClient` wrapper must surface that as
//!    `VectorizerError::Unimplemented` so the distributed collection can
//!    fall back to dense-only search.
//!
//! These tests intentionally avoid spinning up a real multi-node cluster:
//! they exercise the wire surface, request/response conversions, and the
//! compatibility-fallback semantics without depending on physical nodes.

use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpListener;
use tonic::transport::Server;
use vectorizer::cluster::{ClusterGrpcService, ClusterManager};
use vectorizer::db::{HybridScoringAlgorithm, HybridSearchConfig, VectorStore};
use vectorizer::error::VectorizerError;
use vectorizer::grpc::VectorizerGrpcService;
use vectorizer::grpc::cluster::cluster_service_server::ClusterServiceServer;
use vectorizer::grpc::vectorizer::vectorizer_service_server::VectorizerServiceServer;
use vectorizer::models::{CollectionConfig, DistanceMetric, SparseVector, Vector};

/// Build a `VectorStore` with one collection populated for hybrid scoring.
fn populated_store() -> Arc<VectorStore> {
    let store = Arc::new(VectorStore::new());
    let cfg = CollectionConfig {
        dimension: 16,
        metric: DistanceMetric::Euclidean,
        quantization: vectorizer::models::QuantizationConfig::None,
        encryption: None,
        ..Default::default()
    };
    store.create_collection("hybrid_rpc_test", cfg).unwrap();

    let v1 = Vector::with_sparse(
        "match".to_string(),
        SparseVector::new(vec![0, 1, 2], vec![1.0, 1.0, 1.0]).unwrap(),
        16,
    );
    let v2 = Vector::with_sparse(
        "near".to_string(),
        SparseVector::new(vec![0, 1, 3], vec![1.0, 1.0, 1.0]).unwrap(),
        16,
    );
    let v3 = Vector::new("dense_only".to_string(), vec![0.5; 16]);
    store
        .insert("hybrid_rpc_test", vec![v1, v2, v3])
        .expect("insert vectors");

    store
}

/// Bind `127.0.0.1` to a free port, then drop the listener so tonic can take
/// over. Returns the bare `host:port` string (no scheme — `ClusterClient::new`
/// adds its own `http://`) plus the parsed `SocketAddr` for `Server::serve`.
async fn bind_free_port() -> (String, std::net::SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);
    (addr.to_string(), addr)
}

#[tokio::test]
async fn remote_hybrid_search_returns_fused_results() {
    use vectorizer::grpc::cluster::cluster_service_client::ClusterServiceClient;
    use vectorizer::grpc::cluster::{
        HybridScoringAlgorithm as ProtoAlgo, HybridSearchConfig as ProtoConfig,
        RemoteHybridSearchRequest, SparseVector as ProtoSparse,
    };

    let store = populated_store();
    let cluster_mgr = Arc::new(
        ClusterManager::new(vectorizer::cluster::ClusterConfig {
            enabled: true,
            node_id: Some("node-a".to_string()),
            ..Default::default()
        })
        .expect("cluster manager"),
    );
    let svc = ClusterGrpcService::new(store, cluster_mgr, None);

    let (endpoint, addr) = bind_free_port().await;
    tokio::spawn(async move {
        Server::builder()
            .add_service(ClusterServiceServer::new(svc))
            .serve(addr)
            .await
            .expect("cluster server");
    });
    tokio::time::sleep(Duration::from_millis(150)).await;

    let mut client = ClusterServiceClient::connect(format!("http://{endpoint}"))
        .await
        .expect("connect cluster client");

    let req = RemoteHybridSearchRequest {
        collection_name: "hybrid_rpc_test".to_string(),
        dense_query: vec![1.0; 16],
        sparse_query: Some(ProtoSparse {
            indices: vec![0, 1],
            values: vec![1.0, 1.0],
        }),
        config: Some(ProtoConfig {
            dense_k: 10,
            sparse_k: 10,
            final_k: 5,
            alpha: 0.5,
            algorithm: ProtoAlgo::HybridScoringRrf as i32,
        }),
        shard_ids: vec![],
        tenant: None,
    };

    let resp = client
        .remote_hybrid_search(req)
        .await
        .expect("hybrid search rpc")
        .into_inner();

    assert!(resp.success, "rpc reported failure: {}", resp.message);
    assert!(!resp.results.is_empty(), "hybrid search returned 0 results");

    let ids: Vec<_> = resp.results.iter().map(|r| r.id.as_str()).collect();
    assert!(
        ids.contains(&"match") || ids.contains(&"near"),
        "expected sparse-overlapping vectors in top results, got {ids:?}"
    );
}

#[tokio::test]
async fn cluster_client_falls_back_when_server_lacks_rpc() {
    use vectorizer::cluster::ClusterClient;
    use vectorizer::cluster::NodeId;
    use vectorizer::db::sharding::ShardId;

    // Server has VectorizerService but NO ClusterService — every cluster RPC
    // (including RemoteHybridSearch) should answer `Unimplemented`.
    let inner_store = Arc::new(VectorStore::new());
    let dummy_service = VectorizerGrpcService::new(inner_store);

    let (endpoint, addr) = bind_free_port().await;
    tokio::spawn(async move {
        Server::builder()
            .add_service(VectorizerServiceServer::new(dummy_service))
            .serve(addr)
            .await
            .expect("dummy server");
    });
    tokio::time::sleep(Duration::from_millis(150)).await;

    let client = ClusterClient::new(
        &endpoint,
        NodeId::new("legacy-node".to_string()),
        Duration::from_secs(2),
    )
    .await
    .expect("connect cluster client");

    let cfg = HybridSearchConfig {
        alpha: 0.7,
        dense_k: 10,
        sparse_k: 10,
        final_k: 5,
        algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
    };

    let shard_ids = [ShardId::new(0)];
    let result = client
        .hybrid_search("anything", &[0.1; 16], None, &cfg, Some(&shard_ids), None)
        .await;

    match result {
        Err(VectorizerError::Unimplemented(msg)) => {
            assert!(
                msg.contains("legacy-node") || msg.to_lowercase().contains("hybrid"),
                "Unimplemented message should identify the gap: {msg}"
            );
        }
        other => panic!("expected Unimplemented, got {other:?}"),
    }
}
