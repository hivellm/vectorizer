//! gRPC service implementation for cluster inter-server communication

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

use super::manager::ClusterManager;
use super::node::{ClusterNode as LocalClusterNode, NodeId, NodeStatus};
use crate::db::VectorStore;
use crate::error::VectorizerError;
// Use the generated cluster proto code from grpc module
use crate::grpc::cluster::cluster_service_server::ClusterService as ClusterServiceTrait;
use crate::grpc::cluster::*;

/// Cluster gRPC service implementation
#[derive(Clone)]
pub struct ClusterGrpcService {
    /// Vector store
    store: Arc<VectorStore>,
    /// Cluster manager
    cluster_manager: Arc<ClusterManager>,
}

impl ClusterGrpcService {
    /// Create a new cluster gRPC service
    pub fn new(store: Arc<VectorStore>, cluster_manager: Arc<ClusterManager>) -> Self {
        Self {
            store,
            cluster_manager,
        }
    }
}

#[tonic::async_trait]
impl ClusterServiceTrait for ClusterGrpcService {
    /// Get cluster state from a node
    async fn get_cluster_state(
        &self,
        request: Request<GetClusterStateRequest>,
    ) -> Result<Response<GetClusterStateResponse>, Status> {
        debug!("gRPC: GetClusterState request from node {:?}", request.get_ref().node_id);

        let nodes = self.cluster_manager.get_nodes();
        let shard_router = self.cluster_manager.shard_router();

        // Convert local nodes to proto nodes
        let mut proto_nodes = Vec::new();
        for node in nodes {
            let mut proto_node = ClusterNode {
                id: node.id.as_str().to_string(),
                address: node.address.clone(),
                grpc_port: node.grpc_port as u32,
                status: match node.status {
                    NodeStatus::Active => crate::grpc::cluster::NodeStatus::Active as i32,
                    NodeStatus::Joining => crate::grpc::cluster::NodeStatus::Joining as i32,
                    NodeStatus::Leaving => crate::grpc::cluster::NodeStatus::Leaving as i32,
                    NodeStatus::Unavailable => crate::grpc::cluster::NodeStatus::Unavailable as i32,
                },
                shards: node.shards.iter().map(|s| s.as_u32()).collect(),
                metadata: {
                    // Add basic metadata for each node
                    Some(NodeMetadata {
                        version: Some(env!("CARGO_PKG_VERSION").to_string()),
                        capabilities: vec!["vector_search".to_string(), "sharding".to_string()],
                        vector_count: 0, // Would need to query each node for actual count
                        memory_usage: 0,  // Would need to query each node for actual usage
                        cpu_usage: 0.0,  // Would need system monitoring
                    })
                },
            };
            proto_nodes.push(proto_node);
        }

        // Build shard to node mapping
        let mut shard_to_node = std::collections::HashMap::new();
        // Get all shards from router
        let all_nodes = self.cluster_manager.get_nodes();
        for node in all_nodes {
            let shards = shard_router.get_shards_for_node(&node.id);
            for shard_id in shards {
                shard_to_node.insert(shard_id.as_u32(), node.id.as_str().to_string());
            }
        }

        let response = GetClusterStateResponse {
            nodes: proto_nodes,
            shard_to_node,
        };

        Ok(Response::new(response))
    }

    /// Update cluster state (heartbeat, membership changes)
    async fn update_cluster_state(
        &self,
        request: Request<UpdateClusterStateRequest>,
    ) -> Result<Response<UpdateClusterStateResponse>, Status> {
        debug!("gRPC: UpdateClusterState request");

        let req = request.into_inner();

        // Update node if provided
        if let Some(proto_node) = req.node {
            let node_id = NodeId::new(proto_node.id.clone());
            let mut local_node = LocalClusterNode::new(
                node_id.clone(),
                proto_node.address.clone(),
                proto_node.grpc_port as u16,
            );
            
            // Update status
            match proto_node.status {
                x if x == crate::grpc::cluster::NodeStatus::Active as i32 => local_node.mark_active(),
                x if x == crate::grpc::cluster::NodeStatus::Joining as i32 => {
                    local_node.status = NodeStatus::Joining;
                }
                x if x == crate::grpc::cluster::NodeStatus::Leaving as i32 => {
                    local_node.status = NodeStatus::Leaving;
                }
                _ => local_node.mark_unavailable(),
            }

            self.cluster_manager.add_node(local_node);
            self.cluster_manager.update_node_heartbeat(&node_id);
        }

        // Update shard assignments if provided
        for assignment in req.shard_assignments {
            let shard_id = crate::db::sharding::ShardId::new(assignment.shard_id);
            let node_id = NodeId::new(assignment.node_id);
            // Note: Shard router will be updated during rebalancing
            // This is just for state synchronization
        }

        let response = UpdateClusterStateResponse {
            success: true,
            message: "Cluster state updated".to_string(),
        };

        Ok(Response::new(response))
    }

    /// Remote insert vector
    async fn remote_insert_vector(
        &self,
        request: Request<RemoteInsertVectorRequest>,
    ) -> Result<Response<RemoteInsertVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: RemoteInsertVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(e.to_string()))?;

        let payload_obj = req.payload_json
            .as_ref()
            .map(|json_str| {
                serde_json::from_str(json_str)
                    .map_err(|e| Status::invalid_argument(format!("Invalid payload JSON: {}", e)))
            })
            .transpose()?;

        let vector_obj = crate::models::Vector {
            id: req.vector_id.clone(),
            data: req.vector.clone(),
            sparse: None,
            payload: payload_obj,
        };

        {
            let mut collection = self.store
                .get_collection_mut(&req.collection_name)
                .map_err(|e: crate::error::VectorizerError| Status::not_found(e.to_string()))?;
            collection.add_vector(req.vector_id.clone(), vector_obj)
                .map_err(|e: crate::error::VectorizerError| Status::internal(e.to_string()))?;
        }

        let response = RemoteInsertVectorResponse {
            success: true,
            message: format!("Vector {} inserted successfully", req.vector_id),
        };

        Ok(Response::new(response))
    }

    /// Remote update vector
    async fn remote_update_vector(
        &self,
        request: Request<RemoteUpdateVectorRequest>,
    ) -> Result<Response<RemoteUpdateVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: RemoteUpdateVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(e.to_string()))?;

        let payload_obj = req.payload_json
            .as_ref()
            .map(|json_str| {
                serde_json::from_str(json_str)
                    .map_err(|e| Status::invalid_argument(format!("Invalid payload JSON: {}", e)))
            })
            .transpose()?;

        let vector_obj = crate::models::Vector {
            id: req.vector_id.clone(),
            data: req.vector.clone(),
            sparse: None,
            payload: payload_obj,
        };

        let mut collection = self.store
            .get_collection_mut(&req.collection_name)
            .map_err(|e: crate::error::VectorizerError| Status::not_found(e.to_string()))?;
        collection.update_vector(vector_obj)
            .map_err(|e: crate::error::VectorizerError| Status::internal(e.to_string()))?;

        let response = RemoteUpdateVectorResponse {
            success: true,
            message: format!("Vector {} updated successfully", req.vector_id),
        };

        Ok(Response::new(response))
    }

    /// Remote delete vector
    async fn remote_delete_vector(
        &self,
        request: Request<RemoteDeleteVectorRequest>,
    ) -> Result<Response<RemoteDeleteVectorResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: RemoteDeleteVector request for collection '{}', vector '{}'",
            req.collection_name, req.vector_id
        );

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(e.to_string()))?;

        {
            let mut collection = self.store
                .get_collection_mut(&req.collection_name)
                .map_err(|e: crate::error::VectorizerError| Status::not_found(e.to_string()))?;
            collection.delete_vector(&req.vector_id)
                .map_err(|e: crate::error::VectorizerError| Status::internal(e.to_string()))?;
        }

        let response = RemoteDeleteVectorResponse {
            success: true,
            message: format!("Vector {} deleted successfully", req.vector_id),
        };

        Ok(Response::new(response))
    }

    /// Remote search vectors
    async fn remote_search_vectors(
        &self,
        request: Request<RemoteSearchVectorsRequest>,
    ) -> Result<Response<RemoteSearchVectorsResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: RemoteSearchVectors request for collection '{}' with limit {}",
            req.collection_name, req.limit
        );

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(e.to_string()))?;

        let shard_ids = if req.shard_ids.is_empty() {
            None
        } else {
            Some(
                req.shard_ids
                    .iter()
                    .map(|&id| crate::db::sharding::ShardId::new(id))
                    .collect::<Vec<_>>(),
            )
        };

        let results = collection
            .search(
                &req.query_vector,
                req.limit as usize,
            )
            .map_err(|e| Status::internal(e.to_string()))?;

        // Convert SearchResult to proto SearchResult
        let proto_results: Vec<SearchResult> = results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                score: r.score,
                vector: r.vector.unwrap_or_default(),
                payload_json: r.payload.as_ref().map(|p| serde_json::to_string(p).unwrap_or_default()),
            })
            .collect();

        let response = RemoteSearchVectorsResponse {
            results: proto_results,
            success: true,
            message: "Search completed successfully".to_string(),
        };

        Ok(Response::new(response))
    }

    /// Remote create collection
    async fn remote_create_collection(
        &self,
        request: Request<RemoteCreateCollectionRequest>,
    ) -> Result<Response<RemoteCreateCollectionResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: RemoteCreateCollection request for collection '{}'",
            req.collection_name
        );

        // Note: Collection creation should be coordinated, not done via remote call
        // This is a placeholder for future implementation
        warn!("Remote collection creation not fully implemented");

        let response = RemoteCreateCollectionResponse {
            success: false,
            message: "Remote collection creation not yet supported".to_string(),
        };

        Ok(Response::new(response))
    }

    /// Remote get collection info
    async fn remote_get_collection_info(
        &self,
        request: Request<RemoteGetCollectionInfoRequest>,
    ) -> Result<Response<RemoteGetCollectionInfoResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: RemoteGetCollectionInfo request for collection '{}'",
            req.collection_name
        );

        match self.store.get_collection(&req.collection_name) {
            Ok(collection) => {
                let info = CollectionInfo {
                    name: req.collection_name.clone(),
                    vector_count: collection.vector_count() as u64,
                    document_count: 0, // TODO: Add document count if available
                };

                let response = RemoteGetCollectionInfoResponse {
                    info: Some(info),
                    success: true,
                    message: "Collection info retrieved successfully".to_string(),
                };

                Ok(Response::new(response))
            }
            Err(e) => {
                let response = RemoteGetCollectionInfoResponse {
                    info: None,
                    success: false,
                    message: e.to_string(),
                };

                Ok(Response::new(response))
            }
        }
    }

    /// Remote delete collection
    async fn remote_delete_collection(
        &self,
        request: Request<RemoteDeleteCollectionRequest>,
    ) -> Result<Response<RemoteDeleteCollectionResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: RemoteDeleteCollection request for collection '{}'",
            req.collection_name
        );

        // Note: Collection deletion should be coordinated, not done via remote call
        warn!("Remote collection deletion not fully implemented");

        let response = RemoteDeleteCollectionResponse {
            success: false,
            message: "Remote collection deletion not yet supported".to_string(),
        };

        Ok(Response::new(response))
    }

    /// Health check
    async fn health_check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let req = request.into_inner();
        debug!("gRPC: HealthCheck request from node '{}'", req.node_id);

        // Update heartbeat for the requesting node
        let node_id = NodeId::new(req.node_id.clone());
        self.cluster_manager.update_node_heartbeat(&node_id);

        // Get actual vector count from store
        let stats = self.store.stats();
        let vector_count = stats.total_vectors as u64;
        let memory_usage = stats.total_memory_bytes as u64;
        
        // CPU usage would require system monitoring, skip for now
        let cpu_usage = 0.0;

        let metadata = NodeMetadata {
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            capabilities: vec!["vector_search".to_string(), "sharding".to_string(), "cluster".to_string()],
            vector_count,
            memory_usage,
            cpu_usage,
        };

        let response = HealthCheckResponse {
            healthy: true,
            message: "Node is healthy".to_string(),
            metadata: Some(metadata),
        };

        Ok(Response::new(response))
    }
}
