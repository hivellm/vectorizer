//! gRPC client for inter-server cluster communication

use std::time::Duration;

use tonic::transport::Channel;
use tracing::{debug, error, info, warn};

use super::node::NodeId;
use crate::error::{Result, VectorizerError};
use crate::hub::TenantContext;

// Include generated cluster proto code
mod cluster_proto {
    include!("../grpc/vectorizer.cluster.rs");
}

use cluster_proto::cluster_service_client::ClusterServiceClient;

/// Convert hub TenantContext to proto TenantContext
fn tenant_to_proto(tenant: Option<&TenantContext>) -> Option<cluster_proto::TenantContext> {
    tenant.map(|t| cluster_proto::TenantContext {
        tenant_id: t.tenant_id.clone(),
        username: Some(t.tenant_name.clone()),
        permissions: t.permissions.iter().map(|p| format!("{:?}", p)).collect(),
        trace_id: None,
    })
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

/// Connection pool for cluster clients
#[derive(Debug, Clone)]
pub struct ClusterClientPool {
    /// Clients by node ID
    clients: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<NodeId, ClusterClient>>>,
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

    /// Get or create a client for a node
    pub async fn get_client(&self, node_id: &NodeId, address: &str) -> Result<ClusterClient> {
        // Check if client already exists
        {
            let client_opt = {
                let clients = self.clients.read();
                clients.get(node_id).cloned()
            };
            if let Some(client) = client_opt {
                // Check if connection is still healthy
                if client.health_check().await.unwrap_or(false) {
                    return Ok(client);
                }
            }
        }

        // Create new client
        let client = ClusterClient::new(address, node_id.clone(), self.timeout).await?;

        // Store in pool
        {
            let mut clients = self.clients.write();
            clients.insert(node_id.clone(), client.clone());
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
