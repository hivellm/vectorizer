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
    /// Optional Raft consensus manager (present only when HA mode is active)
    raft: Option<Arc<crate::cluster::raft_node::RaftManager>>,
}

impl ClusterGrpcService {
    /// Create a new cluster gRPC service.
    ///
    /// Pass `raft = Some(...)` to enable the Raft RPC endpoints (vote,
    /// append-entries, snapshot).  Pass `None` when Raft HA is not enabled.
    pub fn new(
        store: Arc<VectorStore>,
        cluster_manager: Arc<ClusterManager>,
        raft: Option<Arc<crate::cluster::raft_node::RaftManager>>,
    ) -> Self {
        Self {
            store,
            cluster_manager,
            raft,
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
        debug!(
            "gRPC: GetClusterState request from node {:?}",
            request.get_ref().node_id
        );

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
                        memory_usage: 0, // Would need to query each node for actual usage
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

        // Include per-shard epochs and the global epoch in the response so
        // remote nodes can perform epoch-based conflict resolution.
        let shard_epochs: std::collections::HashMap<u32, u64> = shard_router
            .get_all_shard_epochs()
            .into_iter()
            .map(|(shard_id, epoch)| (shard_id.as_u32(), epoch))
            .collect();
        let current_epoch = shard_router.current_epoch();

        let response = GetClusterStateResponse {
            nodes: proto_nodes,
            shard_to_node,
            current_epoch,
            shard_epochs,
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
                x if x == crate::grpc::cluster::NodeStatus::Active as i32 => {
                    local_node.mark_active()
                }
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

        // Apply incoming shard assignments using epoch-based conflict resolution.
        // Each assignment carries a config_epoch; we only adopt it when its
        // epoch is strictly higher than our locally recorded epoch for that shard.
        if !req.shard_assignments.is_empty() {
            let shard_router = self.cluster_manager.shard_router();
            for assignment in req.shard_assignments {
                let shard_id = crate::db::sharding::ShardId::new(assignment.shard_id);
                let node_id = NodeId::new(assignment.node_id);
                let remote_epoch = assignment.config_epoch;

                if shard_router.apply_if_higher_epoch(shard_id, node_id.clone(), remote_epoch) {
                    debug!(
                        "UpdateClusterState: applied remote assignment shard {} -> {} at epoch {}",
                        assignment.shard_id, node_id, remote_epoch
                    );
                } else {
                    debug!(
                        "UpdateClusterState: skipped shard {} assignment (remote epoch {} \
                         not higher than local)",
                        assignment.shard_id, remote_epoch
                    );
                }
            }
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

        let payload_obj = req
            .payload_json
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
            let mut collection = self
                .store
                .get_collection_mut(&req.collection_name)
                .map_err(|e: crate::error::VectorizerError| Status::not_found(e.to_string()))?;
            collection
                .add_vector(req.vector_id.clone(), vector_obj)
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

        let payload_obj = req
            .payload_json
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

        let mut collection = self
            .store
            .get_collection_mut(&req.collection_name)
            .map_err(|e: crate::error::VectorizerError| Status::not_found(e.to_string()))?;
        collection
            .update_vector(vector_obj)
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
            let mut collection = self
                .store
                .get_collection_mut(&req.collection_name)
                .map_err(|e: crate::error::VectorizerError| Status::not_found(e.to_string()))?;
            collection
                .delete_vector(&req.vector_id)
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
            .search(&req.query_vector, req.limit as usize)
            .map_err(|e| Status::internal(e.to_string()))?;

        // Convert SearchResult to proto SearchResult
        let proto_results: Vec<SearchResult> = results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                score: r.score,
                vector: r.vector.unwrap_or_default(),
                payload_json: r
                    .payload
                    .as_ref()
                    .map(|p| serde_json::to_string(p).unwrap_or_default()),
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
        info!(
            "gRPC: RemoteCreateCollection request for collection '{}'",
            req.collection_name
        );

        // Extract config from request
        let config = match req.config {
            Some(cfg) => crate::models::CollectionConfig {
                dimension: cfg.dimension as usize,
                metric: match cfg.metric.as_str() {
                    "cosine" => crate::models::DistanceMetric::Cosine,
                    "euclidean" => crate::models::DistanceMetric::Euclidean,
                    "dot" => crate::models::DistanceMetric::DotProduct,
                    _ => crate::models::DistanceMetric::Cosine,
                },
                ..Default::default()
            },
            None => {
                // Use default config if not provided
                crate::models::CollectionConfig::default()
            }
        };

        // Extract owner_id from tenant context if provided (for multi-tenant isolation)
        let owner_id = req.tenant.and_then(|t| {
            use uuid::Uuid;
            Uuid::parse_str(&t.tenant_id).ok()
        });

        // Create the collection on this node
        let result = if let Some(owner) = owner_id {
            self.store
                .create_collection_with_owner(&req.collection_name, config, owner)
        } else {
            self.store.create_collection(&req.collection_name, config)
        };

        match result {
            Ok(_) => {
                info!(
                    "Successfully created collection '{}' on remote node",
                    req.collection_name
                );
                let response = RemoteCreateCollectionResponse {
                    success: true,
                    message: format!("Collection '{}' created successfully", req.collection_name),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!(
                    "Failed to create collection '{}' on remote node: {}",
                    req.collection_name, e
                );
                let response = RemoteCreateCollectionResponse {
                    success: false,
                    message: format!("Failed to create collection: {}", e),
                };
                Ok(Response::new(response))
            }
        }
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
                    document_count: collection.document_count() as u64,
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
        info!(
            "gRPC: RemoteDeleteCollection request for collection '{}'",
            req.collection_name
        );

        // Extract owner_id from tenant context if provided (for multi-tenant isolation)
        let owner_id = req.tenant.and_then(|t| {
            use uuid::Uuid;
            Uuid::parse_str(&t.tenant_id).ok()
        });

        // Verify ownership if owner_id is provided
        if let Some(owner) = owner_id {
            match self.store.get_collection(&req.collection_name) {
                Ok(collection) => {
                    // Check if collection belongs to this owner
                    if let Some(col_owner) = collection.owner_id() {
                        if col_owner != owner {
                            warn!(
                                "Attempted to delete collection '{}' by non-owner (owner: {}, requester: {})",
                                req.collection_name, col_owner, owner
                            );
                            let response = RemoteDeleteCollectionResponse {
                                success: false,
                                message: "Collection not owned by this tenant".to_string(),
                            };
                            return Ok(Response::new(response));
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to verify ownership for collection '{}': {}",
                        req.collection_name, e
                    );
                    let response = RemoteDeleteCollectionResponse {
                        success: false,
                        message: format!("Collection not found: {}", e),
                    };
                    return Ok(Response::new(response));
                }
            }
        }

        // Delete the collection on this node
        match self.store.delete_collection(&req.collection_name) {
            Ok(_) => {
                info!(
                    "Successfully deleted collection '{}' on remote node",
                    req.collection_name
                );
                let response = RemoteDeleteCollectionResponse {
                    success: true,
                    message: format!("Collection '{}' deleted successfully", req.collection_name),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                error!(
                    "Failed to delete collection '{}' on remote node: {}",
                    req.collection_name, e
                );
                let response = RemoteDeleteCollectionResponse {
                    success: false,
                    message: format!("Failed to delete collection: {}", e),
                };
                Ok(Response::new(response))
            }
        }
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
            capabilities: vec![
                "vector_search".to_string(),
                "sharding".to_string(),
                "cluster".to_string(),
            ],
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

    /// Fetch shard vectors in paginated batches for shard data migration
    async fn get_shard_vectors(
        &self,
        request: Request<GetShardVectorsRequest>,
    ) -> Result<Response<GetShardVectorsResponse>, Status> {
        let req = request.into_inner();
        debug!(
            "gRPC: GetShardVectors request for collection '{}', shard {}, offset={}, limit={}",
            req.collection_name, req.shard_id, req.offset, req.limit
        );

        let collection = self
            .store
            .get_collection(&req.collection_name)
            .map_err(|e| Status::not_found(e.to_string()))?;

        let all_vectors = collection.get_all_vectors();
        let total_count = all_vectors.len() as u32;

        let offset = req.offset as usize;
        let limit = if req.limit == 0 {
            500
        } else {
            req.limit as usize
        };

        let batch: Vec<VectorData> = all_vectors
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|v| {
                let payload_json = v
                    .payload
                    .as_ref()
                    .and_then(|p| serde_json::to_string(p).ok());
                VectorData {
                    id: v.id,
                    vector: v.data,
                    payload_json,
                }
            })
            .collect();

        let fetched = batch.len() as u32;
        let has_more = (offset as u32 + fetched) < total_count;

        debug!(
            "gRPC: GetShardVectors returning {} vectors (total={}, has_more={})",
            fetched, total_count, has_more
        );

        Ok(Response::new(GetShardVectorsResponse {
            vectors: batch,
            total_count,
            has_more,
        }))
    }

    /// Check quota across cluster
    ///
    /// This allows distributed quota checking by aggregating usage from all nodes.
    async fn check_quota(
        &self,
        request: Request<CheckQuotaRequest>,
    ) -> Result<Response<CheckQuotaResponse>, Status> {
        let req = request.into_inner();

        let tenant = req.tenant.as_ref().ok_or_else(|| {
            Status::invalid_argument("Tenant context is required for quota check")
        })?;

        debug!(
            "gRPC: CheckQuota request for tenant '{}', type {:?}",
            tenant.tenant_id, req.quota_type
        );

        // Get local usage for this tenant
        let (current_usage, limit) = match req.quota_type() {
            QuotaType::QuotaCollections => {
                // Count collections owned by this tenant
                let tenant_id = uuid::Uuid::parse_str(&tenant.tenant_id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid tenant ID: {}", e)))?;
                let collections = self.store.list_collections_for_owner(&tenant_id);
                (collections.len() as u64, 100u64) // Default limit, should come from HiveHub
            }
            QuotaType::QuotaVectors => {
                // Count vectors in tenant's collections
                let tenant_id = uuid::Uuid::parse_str(&tenant.tenant_id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid tenant ID: {}", e)))?;
                let collections = self.store.list_collections_for_owner(&tenant_id);
                let mut total_vectors = 0u64;
                for name in collections {
                    if let Ok(collection) = self.store.get_collection(&name) {
                        total_vectors += collection.vector_count() as u64;
                    }
                }
                (total_vectors, 1_000_000u64) // Default limit
            }
            QuotaType::QuotaStorage => {
                // Estimate storage usage for tenant's collections
                let tenant_id = uuid::Uuid::parse_str(&tenant.tenant_id)
                    .map_err(|e| Status::invalid_argument(format!("Invalid tenant ID: {}", e)))?;
                let collections = self.store.list_collections_for_owner(&tenant_id);
                let mut total_storage = 0u64;
                for name in collections {
                    if let Ok(collection) = self.store.get_collection(&name) {
                        // Rough estimate: vectors * dimension * 4 bytes per float
                        let config = collection.config();
                        total_storage +=
                            collection.vector_count() as u64 * config.dimension as u64 * 4;
                    }
                }
                (total_storage, 10_737_418_240u64) // 10GB default limit
            }
        };

        let remaining = limit.saturating_sub(current_usage);
        let allowed = current_usage + req.requested_amount <= limit;

        let response = CheckQuotaResponse {
            allowed,
            current_usage,
            limit,
            remaining,
            message: if allowed {
                "Quota check passed".to_string()
            } else {
                format!(
                    "Quota exceeded: current={}, requested={}, limit={}",
                    current_usage, req.requested_amount, limit
                )
            },
        };

        Ok(Response::new(response))
    }

    /// Raft vote RPC — forwards the request to the local Raft node.
    async fn raft_vote(
        &self,
        request: Request<RaftVoteRequest>,
    ) -> Result<Response<RaftVoteResponse>, Status> {
        let raft = self
            .raft
            .as_ref()
            .ok_or_else(|| Status::unavailable("Raft HA is not enabled on this node"))?;

        let data = request.into_inner().data;
        let vote_req: openraft::raft::VoteRequest<crate::cluster::raft_node::TypeConfig> =
            crate::codec::deserialize(&data)
                .map_err(|e| Status::invalid_argument(format!("deserialize vote: {}", e)))?;

        let resp = raft
            .raft
            .vote(vote_req)
            .await
            .map_err(|e| Status::internal(format!("raft vote: {}", e)))?;

        let resp_data = crate::codec::serialize(&resp)
            .map_err(|e| Status::internal(format!("serialize vote response: {}", e)))?;

        Ok(Response::new(RaftVoteResponse { data: resp_data }))
    }

    /// Raft append-entries RPC — forwards the request to the local Raft node.
    async fn raft_append_entries(
        &self,
        request: Request<RaftAppendEntriesRequest>,
    ) -> Result<Response<RaftAppendEntriesResponse>, Status> {
        let raft = self
            .raft
            .as_ref()
            .ok_or_else(|| Status::unavailable("Raft HA is not enabled on this node"))?;

        let data = request.into_inner().data;
        let append_req: openraft::raft::AppendEntriesRequest<
            crate::cluster::raft_node::TypeConfig,
        > = crate::codec::deserialize(&data)
            .map_err(|e| Status::invalid_argument(format!("deserialize append_entries: {}", e)))?;

        let resp = raft
            .raft
            .append_entries(append_req)
            .await
            .map_err(|e| Status::internal(format!("raft append_entries: {}", e)))?;

        let resp_data = crate::codec::serialize(&resp)
            .map_err(|e| Status::internal(format!("serialize append_entries response: {}", e)))?;

        Ok(Response::new(RaftAppendEntriesResponse { data: resp_data }))
    }

    /// Raft snapshot RPC — forwards the snapshot to the local Raft node.
    async fn raft_snapshot(
        &self,
        request: Request<RaftSnapshotRequest>,
    ) -> Result<Response<RaftSnapshotResponse>, Status> {
        use std::io::Cursor;

        let raft = self
            .raft
            .as_ref()
            .ok_or_else(|| Status::unavailable("Raft HA is not enabled on this node"))?;

        let inner = request.into_inner();

        let vote: openraft::alias::VoteOf<crate::cluster::raft_node::TypeConfig> =
            crate::codec::deserialize(&inner.vote_data)
                .map_err(|e| Status::invalid_argument(format!("deserialize vote: {}", e)))?;

        let meta: openraft::alias::SnapshotMetaOf<crate::cluster::raft_node::TypeConfig> =
            crate::codec::deserialize(&inner.snapshot_meta).map_err(|e| {
                Status::invalid_argument(format!("deserialize snapshot meta: {}", e))
            })?;

        let snapshot_cursor = Cursor::new(inner.snapshot_data);

        let snapshot = openraft::alias::SnapshotOf::<crate::cluster::raft_node::TypeConfig> {
            meta,
            snapshot: snapshot_cursor,
        };

        let resp = raft
            .raft
            .install_full_snapshot(vote, snapshot)
            .await
            .map_err(|e| Status::internal(format!("raft install_full_snapshot: {}", e)))?;

        let resp_data = crate::codec::serialize(&resp)
            .map_err(|e| Status::internal(format!("serialize snapshot response: {}", e)))?;

        Ok(Response::new(RaftSnapshotResponse { data: resp_data }))
    }
}
