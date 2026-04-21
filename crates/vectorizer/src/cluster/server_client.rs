//! gRPC client for inter-server cluster communication

use std::time::Duration;

use cluster_proto::cluster_service_client::ClusterServiceClient;
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};
// Cluster proto types live in the `vectorizer-protocol` crate after
// phase4_split-vectorizer-workspace sub-phase 2.
use vectorizer_protocol::grpc_gen::cluster as cluster_proto;

use super::node::NodeId;
use crate::error::{Result, VectorizerError};
use crate::hub::TenantContext;

/// Convert hub TenantContext to proto TenantContext
fn tenant_to_proto(tenant: Option<&TenantContext>) -> Option<cluster_proto::TenantContext> {
    tenant.map(|t| cluster_proto::TenantContext {
        tenant_id: t.tenant_id.clone(),
        username: Some(t.tenant_name.clone()),
        permissions: t.permissions.iter().map(|p| format!("{:?}", p)).collect(),
        trace_id: None,
    })
}

/// Plain data struct used by [`ClusterClient::broadcast_cluster_state`] to
/// avoid proto type mismatches across module boundaries.
#[derive(Debug, Clone)]
pub struct BroadcastNode {
    pub id: String,
    pub address: String,
    pub grpc_port: u32,
    pub status: i32,
    pub shards: Vec<u32>,
    pub version: Option<String>,
    pub capabilities: Vec<String>,
}

/// gRPC client for cluster inter-server communication
#[derive(Debug, Clone)]
pub struct ClusterClient {
    /// gRPC client to the remote server
    client: ClusterServiceClient<Channel>,
    /// Node ID this client connects to
    node_id: NodeId,
    /// Request timeout
    timeout: Duration,
}

impl ClusterClient {
    /// Create a new cluster client
    pub async fn new(address: &str, node_id: NodeId, timeout: Duration) -> Result<Self> {
        info!(
            "Creating cluster client for node {} at {}",
            node_id, address
        );

        let client = ClusterServiceClient::connect(format!("http://{}", address))
            .await
            .map_err(|e| VectorizerError::InvalidConfiguration {
                message: format!("Failed to connect to cluster server {}: {}", address, e),
            })?;

        Ok(Self {
            client,
            node_id,
            timeout,
        })
    }

    /// Get the node ID this client connects to
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Check if the connection is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(cluster_proto::HealthCheckRequest {
            node_id: self.node_id.as_str().to_string(),
        });

        match client.health_check(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                Ok(resp.healthy)
            }
            Err(e) => {
                warn!("Health check failed for node {}: {}", self.node_id, e);
                Ok(false)
            }
        }
    }

    /// Insert a vector on the remote server with retry and exponential backoff
    ///
    /// If `tenant` is provided, the operation is scoped to that tenant.
    pub async fn insert_vector(
        &self,
        collection_name: &str,
        vector_id: &str,
        vector: &[f32],
        payload: Option<&serde_json::Value>,
        tenant: Option<&TenantContext>,
    ) -> Result<()> {
        let payload_json = payload
            .map(|p| serde_json::to_string(p))
            .transpose()
            .map_err(|e| VectorizerError::InvalidConfiguration {
                message: format!("Failed to serialize payload: {}", e),
            })?;

        let request = cluster_proto::RemoteInsertVectorRequest {
            collection_name: collection_name.to_string(),
            vector_id: vector_id.to_string(),
            vector: vector.to_vec(),
            payload_json,
            tenant: tenant_to_proto(tenant),
        };

        // Retry with exponential backoff (3 retries)
        let mut last_error = None;
        for attempt in 0..3 {
            let mut client = self.client.clone();
            let request_clone = tonic::Request::new(request.clone());

            match client.remote_insert_vector(request_clone).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    if resp.success {
                        debug!(
                            "Successfully inserted vector {} in collection {} on node {}",
                            vector_id, collection_name, self.node_id
                        );
                        return Ok(());
                    } else {
                        last_error = Some(VectorizerError::Storage(resp.message));
                    }
                }
                Err(e) => {
                    last_error = Some(VectorizerError::Storage(format!("gRPC error: {}", e)));
                }
            }

            // Exponential backoff: 100ms, 200ms, 400ms
            if attempt < 2 {
                let delay = Duration::from_millis(100 * (1 << attempt));
                tokio::time::sleep(delay).await;
            }
        }

        error!(
            "Failed to insert vector {} in collection {} on node {} after 3 attempts",
            vector_id, collection_name, self.node_id
        );
        Err(last_error.unwrap_or_else(|| VectorizerError::Storage("Unknown error".to_string())))
    }

    /// Update a vector on the remote server
    ///
    /// If `tenant` is provided, the operation is scoped to that tenant.
    pub async fn update_vector(
        &self,
        collection_name: &str,
        vector_id: &str,
        vector: &[f32],
        payload: Option<&serde_json::Value>,
        tenant: Option<&TenantContext>,
    ) -> Result<()> {
        let mut client = self.client.clone();

        let payload_json = payload
            .map(|p| serde_json::to_string(p))
            .transpose()
            .map_err(|e| VectorizerError::InvalidConfiguration {
                message: format!("Failed to serialize payload: {}", e),
            })?;

        let request = tonic::Request::new(cluster_proto::RemoteUpdateVectorRequest {
            collection_name: collection_name.to_string(),
            vector_id: vector_id.to_string(),
            vector: vector.to_vec(),
            payload_json,
            tenant: tenant_to_proto(tenant),
        });

        match client.remote_update_vector(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                if resp.success {
                    debug!(
                        "Successfully updated vector {} in collection {} on node {}",
                        vector_id, collection_name, self.node_id
                    );
                    Ok(())
                } else {
                    Err(VectorizerError::Storage(resp.message))
                }
            }
            Err(e) => {
                error!(
                    "Failed to update vector {} in collection {} on node {}: {}",
                    vector_id, collection_name, self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Delete a vector on the remote server
    ///
    /// If `tenant` is provided, the operation is scoped to that tenant.
    pub async fn delete_vector(
        &self,
        collection_name: &str,
        vector_id: &str,
        tenant: Option<&TenantContext>,
    ) -> Result<()> {
        let mut client = self.client.clone();

        let request = tonic::Request::new(cluster_proto::RemoteDeleteVectorRequest {
            collection_name: collection_name.to_string(),
            vector_id: vector_id.to_string(),
            tenant: tenant_to_proto(tenant),
        });

        match client.remote_delete_vector(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                if resp.success {
                    debug!(
                        "Successfully deleted vector {} in collection {} on node {}",
                        vector_id, collection_name, self.node_id
                    );
                    Ok(())
                } else {
                    Err(VectorizerError::Storage(resp.message))
                }
            }
            Err(e) => {
                error!(
                    "Failed to delete vector {} in collection {} on node {}: {}",
                    vector_id, collection_name, self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Search vectors on the remote server with retry and exponential backoff
    ///
    /// If `tenant` is provided, the search is scoped to that tenant's collections.
    pub async fn search_vectors(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        limit: usize,
        threshold: Option<f32>,
        shard_ids: Option<&[crate::db::sharding::ShardId]>,
        tenant: Option<&TenantContext>,
    ) -> Result<Vec<crate::models::SearchResult>> {
        let shard_ids_vec = shard_ids
            .map(|ids| ids.iter().map(|id| id.as_u32()).collect())
            .unwrap_or_default();

        let request = cluster_proto::RemoteSearchVectorsRequest {
            collection_name: collection_name.to_string(),
            query_vector: query_vector.to_vec(),
            limit: limit as u32,
            threshold,
            shard_ids: shard_ids_vec,
            tenant: tenant_to_proto(tenant),
        };

        // Retry with exponential backoff (2 retries for search - faster failure)
        let mut last_error = None;
        for attempt in 0..2 {
            let mut client = self.client.clone();
            let request_clone = tonic::Request::new(request.clone());

            match client.remote_search_vectors(request_clone).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    if resp.success {
                        // Convert proto SearchResult to models::SearchResult
                        let results: Vec<crate::models::SearchResult> = resp
                            .results
                            .into_iter()
                            .map(|r| {
                                let payload = r
                                    .payload_json
                                    .as_ref()
                                    .and_then(|json| serde_json::from_str(json).ok());

                                crate::models::SearchResult {
                                    id: r.id,
                                    score: r.score,
                                    dense_score: None,
                                    sparse_score: None,
                                    vector: Some(r.vector),
                                    payload,
                                }
                            })
                            .collect();

                        debug!(
                            "Successfully searched collection {} on node {}, got {} results",
                            collection_name,
                            self.node_id,
                            results.len()
                        );
                        return Ok(results);
                    } else {
                        last_error = Some(VectorizerError::Storage(resp.message));
                    }
                }
                Err(e) => {
                    last_error = Some(VectorizerError::Storage(format!("gRPC error: {}", e)));
                }
            }

            // Exponential backoff: 50ms, 100ms (shorter for search)
            if attempt < 1 {
                let delay = Duration::from_millis(50 * (1 << attempt));
                tokio::time::sleep(delay).await;
            }
        }

        error!(
            "Failed to search collection {} on node {} after 2 attempts",
            collection_name, self.node_id
        );
        Err(last_error.unwrap_or_else(|| VectorizerError::Storage("Unknown error".to_string())))
    }

    /// Hybrid (dense + sparse) search on the remote node.
    ///
    /// Returns `Err(VectorizerError::Unimplemented(_))` when the remote
    /// server predates this RPC, so the caller can fall back to dense-only
    /// `search_vectors`. Other errors propagate normally.
    pub async fn hybrid_search(
        &self,
        collection_name: &str,
        dense_query: &[f32],
        sparse_query: Option<&crate::models::SparseVector>,
        config: &crate::db::HybridSearchConfig,
        shard_ids: Option<&[crate::db::sharding::ShardId]>,
        tenant: Option<&TenantContext>,
    ) -> Result<Vec<crate::models::SearchResult>> {
        let algorithm = match config.algorithm {
            crate::db::HybridScoringAlgorithm::ReciprocalRankFusion => {
                cluster_proto::HybridScoringAlgorithm::HybridScoringRrf
            }
            crate::db::HybridScoringAlgorithm::WeightedCombination => {
                cluster_proto::HybridScoringAlgorithm::HybridScoringWeighted
            }
            crate::db::HybridScoringAlgorithm::AlphaBlending => {
                cluster_proto::HybridScoringAlgorithm::HybridScoringAlphaBlend
            }
        };

        let proto_config = cluster_proto::HybridSearchConfig {
            dense_k: config.dense_k as u32,
            sparse_k: config.sparse_k as u32,
            final_k: config.final_k as u32,
            alpha: config.alpha as f64,
            algorithm: algorithm as i32,
        };

        let proto_sparse = sparse_query.map(|sv| cluster_proto::SparseVector {
            indices: sv.indices.iter().map(|&i| i as u32).collect(),
            values: sv.values.clone(),
        });

        let shard_ids_vec: Vec<u32> = shard_ids
            .map(|ids| ids.iter().map(|id| id.as_u32()).collect())
            .unwrap_or_default();

        let request = cluster_proto::RemoteHybridSearchRequest {
            collection_name: collection_name.to_string(),
            dense_query: dense_query.to_vec(),
            sparse_query: proto_sparse,
            config: Some(proto_config),
            shard_ids: shard_ids_vec,
            tenant: tenant_to_proto(tenant),
        };

        let mut last_error = None;
        for attempt in 0..2 {
            let mut client = self.client.clone();
            let request_clone = tonic::Request::new(request.clone());

            match client.remote_hybrid_search(request_clone).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    if resp.success {
                        let results: Vec<crate::models::SearchResult> = resp
                            .results
                            .into_iter()
                            .map(|r| {
                                let payload = r
                                    .payload_json
                                    .as_ref()
                                    .and_then(|json| serde_json::from_str(json).ok());
                                crate::models::SearchResult {
                                    id: r.id,
                                    score: r.hybrid_score,
                                    dense_score: r.dense_score,
                                    sparse_score: r.sparse_score,
                                    vector: Some(r.vector),
                                    payload,
                                }
                            })
                            .collect();
                        debug!(
                            "Hybrid search on node {} for '{}' returned {} results",
                            self.node_id,
                            collection_name,
                            results.len()
                        );
                        return Ok(results);
                    } else {
                        last_error = Some(VectorizerError::Storage(resp.message));
                    }
                }
                Err(status) if status.code() == tonic::Code::Unimplemented => {
                    return Err(VectorizerError::Unimplemented(format!(
                        "remote node {} does not implement RemoteHybridSearch: {}",
                        self.node_id,
                        status.message()
                    )));
                }
                Err(e) => {
                    last_error = Some(VectorizerError::Storage(format!("gRPC error: {}", e)));
                }
            }

            if attempt < 1 {
                let delay = Duration::from_millis(50 * (1 << attempt));
                tokio::time::sleep(delay).await;
            }
        }

        error!(
            "Failed hybrid search on collection {} at node {} after 2 attempts",
            collection_name, self.node_id
        );
        Err(last_error.unwrap_or_else(|| VectorizerError::Storage("Unknown error".to_string())))
    }

    /// Fetch a batch of vectors from a remote shard for migration purposes.
    ///
    /// Returns `(vectors, total_count, has_more)`.
    pub async fn get_shard_vectors(
        &self,
        collection_name: &str,
        shard_id: u32,
        offset: u32,
        limit: u32,
        tenant: Option<&crate::hub::TenantContext>,
    ) -> Result<(Vec<cluster_proto::VectorData>, u32, bool)> {
        let mut client = self.client.clone();

        let request = tonic::Request::new(cluster_proto::GetShardVectorsRequest {
            collection_name: collection_name.to_string(),
            shard_id,
            offset,
            limit,
            tenant: tenant_to_proto(tenant),
        });

        match client.get_shard_vectors(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!(
                    "GetShardVectors from node {}: got {} vectors (total={}, has_more={})",
                    self.node_id,
                    resp.vectors.len(),
                    resp.total_count,
                    resp.has_more,
                );
                Ok((resp.vectors, resp.total_count, resp.has_more))
            }
            Err(e) => {
                error!(
                    "Failed to get shard vectors from node {}: {}",
                    self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Create a collection on the remote node.
    ///
    /// Wraps the `RemoteCreateCollection` gRPC call.  `owner_id`, when
    /// provided, is forwarded as the tenant ID for multi-tenant isolation.
    pub async fn remote_create_collection(
        &self,
        collection_name: &str,
        config: &crate::models::CollectionConfig,
        owner_id: Option<uuid::Uuid>,
    ) -> Result<cluster_proto::RemoteCreateCollectionResponse> {
        let mut client = self.client.clone();

        let tenant = owner_id.map(|id| cluster_proto::TenantContext {
            tenant_id: id.to_string(),
            username: None,
            permissions: Vec::new(),
            trace_id: None,
        });

        let proto_config = cluster_proto::CollectionConfig {
            dimension: config.dimension as u32,
            metric: format!("{:?}", config.metric).to_lowercase(),
        };

        let request = tonic::Request::new(cluster_proto::RemoteCreateCollectionRequest {
            collection_name: collection_name.to_string(),
            config: Some(proto_config),
            tenant,
        });

        match client.remote_create_collection(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!(
                    "remote_create_collection '{}' on node {}: success={}",
                    collection_name, self.node_id, resp.success
                );
                Ok(resp)
            }
            Err(e) => {
                error!(
                    "remote_create_collection '{}' on node {} failed: {}",
                    collection_name, self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Delete a collection on the remote node.
    ///
    /// Wraps the `RemoteDeleteCollection` gRPC call without tenant scoping so
    /// that rollback operations during quorum failures can always proceed.
    pub async fn remote_delete_collection(
        &self,
        collection_name: &str,
    ) -> Result<cluster_proto::RemoteDeleteCollectionResponse> {
        let mut client = self.client.clone();

        let request = tonic::Request::new(cluster_proto::RemoteDeleteCollectionRequest {
            collection_name: collection_name.to_string(),
            tenant: None,
        });

        match client.remote_delete_collection(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!(
                    "remote_delete_collection '{}' on node {}: success={}",
                    collection_name, self.node_id, resp.success
                );
                Ok(resp)
            }
            Err(e) => {
                error!(
                    "remote_delete_collection '{}' on node {} failed: {}",
                    collection_name, self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Probe whether a collection exists on the remote node.
    ///
    /// Wraps the `RemoteGetCollectionInfo` gRPC call.  Returns the raw
    /// response so callers can inspect the `success` flag to distinguish
    /// "collection absent" from a hard transport error.
    pub async fn remote_get_collection_info(
        &self,
        collection_name: &str,
    ) -> Result<cluster_proto::RemoteGetCollectionInfoResponse> {
        let mut client = self.client.clone();

        let request = tonic::Request::new(cluster_proto::RemoteGetCollectionInfoRequest {
            collection_name: collection_name.to_string(),
            tenant: None,
        });

        match client.remote_get_collection_info(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!(
                    "remote_get_collection_info '{}' on node {}: success={}",
                    collection_name, self.node_id, resp.success
                );
                Ok(resp)
            }
            Err(e) => {
                error!(
                    "remote_get_collection_info '{}' on node {} failed: {}",
                    collection_name, self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Push local cluster state to a remote node.
    ///
    /// Wraps the `UpdateClusterState` gRPC call so that shard assignments and
    /// node information are propagated eagerly (in addition to the periodic
    /// pull-based sync).
    ///
    /// Accepts shard assignments as `(shard_id, node_id, epoch)` tuples and an
    /// optional local node description to avoid cross-module proto type mismatches.
    ///
    /// Returns `(success, message)` from the remote node.
    pub async fn broadcast_cluster_state(
        &self,
        local_node: Option<BroadcastNode>,
        shard_assignments: &[(u32, String, u64)],
    ) -> Result<(bool, String)> {
        let mut client = self.client.clone();

        let proto_node = local_node.map(|n| cluster_proto::ClusterNode {
            id: n.id,
            address: n.address,
            grpc_port: n.grpc_port,
            status: n.status,
            shards: n.shards,
            metadata: Some(cluster_proto::NodeMetadata {
                version: n.version,
                capabilities: n.capabilities,
                vector_count: 0,
                memory_usage: 0,
                cpu_usage: 0.0,
            }),
        });

        let proto_assignments: Vec<cluster_proto::ShardAssignment> = shard_assignments
            .iter()
            .map(
                |(shard_id, node_id, epoch)| cluster_proto::ShardAssignment {
                    shard_id: *shard_id,
                    node_id: node_id.clone(),
                    config_epoch: *epoch,
                },
            )
            .collect();

        let request = tonic::Request::new(cluster_proto::UpdateClusterStateRequest {
            node: proto_node,
            shard_assignments: proto_assignments,
        });

        match client.update_cluster_state(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                Ok((resp.success, resp.message))
            }
            Err(e) => {
                error!(
                    "Failed to update cluster state on node {}: {}",
                    self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Get cluster state from remote server
    pub async fn get_cluster_state(&self) -> Result<cluster_proto::GetClusterStateResponse> {
        let mut client = self.client.clone();

        let request = tonic::Request::new(cluster_proto::GetClusterStateRequest {
            node_id: self.node_id.as_str().to_string(),
        });

        match client.get_cluster_state(request).await {
            Ok(response) => Ok(response.into_inner()),
            Err(e) => {
                error!(
                    "Failed to get cluster state from node {}: {}",
                    self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Get collection info from remote server
    ///
    /// If `tenant` is provided, only returns info if the collection belongs to that tenant.
    pub async fn get_collection_info(
        &self,
        collection_name: &str,
        tenant: Option<&TenantContext>,
    ) -> Result<Option<cluster_proto::CollectionInfo>> {
        let mut client = self.client.clone();

        let request = tonic::Request::new(cluster_proto::RemoteGetCollectionInfoRequest {
            collection_name: collection_name.to_string(),
            tenant: tenant_to_proto(tenant),
        });

        match client.remote_get_collection_info(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                if resp.success {
                    Ok(resp.info)
                } else {
                    warn!(
                        "Failed to get collection info from node {}: {}",
                        self.node_id, resp.message
                    );
                    Ok(None)
                }
            }
            Err(e) => {
                error!(
                    "Failed to get collection info from node {}: {}",
                    self.node_id, e
                );
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }

    /// Check quota on remote server
    ///
    /// Returns (allowed, current_usage, limit, remaining)
    pub async fn check_quota(
        &self,
        tenant: &TenantContext,
        quota_type: cluster_proto::QuotaType,
        requested_amount: u64,
    ) -> Result<(bool, u64, u64, u64)> {
        let mut client = self.client.clone();

        let request = tonic::Request::new(cluster_proto::CheckQuotaRequest {
            tenant: Some(cluster_proto::TenantContext {
                tenant_id: tenant.tenant_id.clone(),
                username: Some(tenant.tenant_name.clone()),
                permissions: tenant
                    .permissions
                    .iter()
                    .map(|p| format!("{:?}", p))
                    .collect(),
                trace_id: None,
            }),
            quota_type: quota_type as i32,
            requested_amount,
        });

        match client.check_quota(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!(
                    "Quota check on node {}: allowed={}, current={}, limit={}, remaining={}",
                    self.node_id, resp.allowed, resp.current_usage, resp.limit, resp.remaining
                );
                Ok((resp.allowed, resp.current_usage, resp.limit, resp.remaining))
            }
            Err(e) => {
                error!("Failed to check quota on node {}: {}", self.node_id, e);
                Err(VectorizerError::Storage(format!("gRPC error: {}", e)))
            }
        }
    }
}

/// How long a cached client is considered healthy without re-checking.
const HEALTH_CHECK_GRACE_PERIOD: Duration = Duration::from_secs(30);

/// A cached cluster client with a timestamp of last successful use.
#[derive(Debug, Clone)]
struct PooledClient {
    client: ClusterClient,
    last_healthy: std::time::Instant,
}

/// Connection pool for cluster clients
#[derive(Debug, Clone)]
pub struct ClusterClientPool {
    /// Clients by node ID
    clients: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<NodeId, PooledClient>>>,
    /// Default timeout
    timeout: Duration,
}

impl ClusterClientPool {
    /// Create a new client pool
    pub fn new(timeout: Duration) -> Self {
        Self {
            clients: std::sync::Arc::new(
                parking_lot::RwLock::new(std::collections::HashMap::new()),
            ),
            timeout,
        }
    }

    /// Get or create a client for a node.
    ///
    /// Clients that were successfully used within [`HEALTH_CHECK_GRACE_PERIOD`]
    /// are returned immediately without a health check RPC.
    pub async fn get_client(&self, node_id: &NodeId, address: &str) -> Result<ClusterClient> {
        // Check if client already exists and is fresh
        {
            let pooled_opt = {
                let clients = self.clients.read();
                clients.get(node_id).cloned()
            };
            if let Some(pooled) = pooled_opt {
                if pooled.last_healthy.elapsed() < HEALTH_CHECK_GRACE_PERIOD {
                    return Ok(pooled.client);
                }
                // Grace period expired — verify health
                if pooled.client.health_check().await.unwrap_or(false) {
                    // Update timestamp
                    let mut clients = self.clients.write();
                    if let Some(entry) = clients.get_mut(node_id) {
                        entry.last_healthy = std::time::Instant::now();
                    }
                    return Ok(pooled.client);
                }
            }
        }

        // Create new client
        let client = ClusterClient::new(address, node_id.clone(), self.timeout).await?;

        // Store in pool
        {
            let mut clients = self.clients.write();
            clients.insert(
                node_id.clone(),
                PooledClient {
                    client: client.clone(),
                    last_healthy: std::time::Instant::now(),
                },
            );
        }

        Ok(client)
    }

    /// Remove a client from the pool
    pub fn remove_client(&self, node_id: &NodeId) {
        let mut clients = self.clients.write();
        clients.remove(node_id);
        debug!("Removed client for node {} from pool", node_id);
    }

    /// Clear all clients
    pub fn clear(&self) {
        let mut clients = self.clients.write();
        clients.clear();
        debug!("Cleared all clients from pool");
    }
}
