//! gRPC server bootstrap. Runs on `port + 1` alongside the HTTP server
//! and serves the native Vectorizer gRPC API plus the Qdrant-compatible
//! services (Collections / Points / Snapshots) and the cluster service.

use std::sync::Arc;

use tracing::info;
use vectorizer::VectorStore;

use crate::server::VectorizerServer;

impl VectorizerServer {
    /// Start gRPC server
    pub(super) async fn start_grpc_server(
        host: &str,
        port: u16,
        store: Arc<VectorStore>,
        cluster_manager: Option<Arc<vectorizer::cluster::ClusterManager>>,
        snapshot_manager: Option<Arc<vectorizer::storage::SnapshotManager>>,
        raft_manager: Option<Arc<vectorizer::cluster::raft_node::RaftManager>>,
    ) -> anyhow::Result<()> {
        use tonic::transport::Server;

        use crate::grpc::VectorizerGrpcService;
        use crate::grpc::vectorizer::vectorizer_service_server::VectorizerServiceServer;

        let addr = format!("{}:{}", host, port).parse()?;
        let service = VectorizerGrpcService::new(store.clone());

        info!("🚀 Starting gRPC server on {}", addr);

        let mut server_builder =
            Server::builder().add_service(VectorizerServiceServer::new(service));

        // Add ClusterService if cluster is enabled
        if let Some(cluster_mgr) = cluster_manager {
            use vectorizer::cluster::ClusterGrpcService;

            use crate::grpc::cluster::cluster_service_server::ClusterServiceServer;

            info!("🔗 Adding Cluster gRPC service");
            let cluster_service =
                ClusterGrpcService::new(store.clone(), cluster_mgr, raft_manager.clone());
            server_builder = server_builder.add_service(ClusterServiceServer::new(cluster_service));
        }

        // Add Qdrant-compatible gRPC services
        {
            use crate::grpc::QdrantGrpcService;
            use crate::grpc::qdrant_proto::collections_server::CollectionsServer;
            use crate::grpc::qdrant_proto::points_server::PointsServer;
            use crate::grpc::qdrant_proto::snapshots_server::SnapshotsServer;

            info!("🔗 Adding Qdrant-compatible gRPC services (Collections, Points, Snapshots)");
            let qdrant_service = if let Some(sm) = snapshot_manager {
                QdrantGrpcService::with_snapshot_manager(store.clone(), sm)
            } else {
                QdrantGrpcService::new(store.clone())
            };

            // Add all Qdrant services using the same service instance (it implements all traits)
            server_builder = server_builder
                .add_service(CollectionsServer::new(qdrant_service.clone()))
                .add_service(PointsServer::new(qdrant_service.clone()))
                .add_service(SnapshotsServer::new(qdrant_service));
        }

        server_builder.serve(addr).await?;

        Ok(())
    }
}
