//! Vectorizer client with transport abstraction

use std::sync::Arc;

use serde_json;

use crate::error::{Result, VectorizerError};
use crate::http_transport::HttpTransport;
use crate::models::hybrid_search::{
    HybridScoringAlgorithm, HybridSearchRequest, HybridSearchResponse,
};
use crate::models::*;
use crate::transport::{Protocol, Transport};
#[cfg(feature = "umicp")]
use crate::umicp_transport::UmicpTransport;

/// Configuration for VectorizerClient
#[derive(Clone)]
pub struct ClientConfig {
    /// Base URL for HTTP transport (single-node deployments)
    pub base_url: Option<String>,
    /// Connection string (supports http://, https://, umicp://)
    pub connection_string: Option<String>,
    /// Protocol to use
    pub protocol: Option<Protocol>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,
    /// UMICP configuration
    #[cfg(feature = "umicp")]
    pub umicp: Option<UmicpConfig>,
    /// Master/replica host configuration for read/write routing
    pub hosts: Option<HostConfig>,
    /// Default read preference for read operations
    pub read_preference: Option<ReadPreference>,
}

#[cfg(feature = "umicp")]
/// UMICP-specific configuration
#[derive(Clone)]
pub struct UmicpConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: Some("http://localhost:15002".to_string()),
            connection_string: None,
            protocol: None,
            api_key: None,
            timeout_secs: Some(30),
            #[cfg(feature = "umicp")]
            umicp: None,
            hosts: None,
            read_preference: None,
        }
    }
}

/// Vectorizer client with optional master/replica topology support
pub struct VectorizerClient {
    transport: Arc<dyn Transport>,
    protocol: Protocol,
    base_url: String,
    /// Master transport for write operations (if replica mode is enabled)
    #[allow(dead_code)]
    master_transport: Option<Arc<dyn Transport>>,
    /// Replica transports for read operations (if replica mode is enabled)
    #[allow(dead_code)]
    replica_transports: Vec<Arc<dyn Transport>>,
    /// Current replica index for round-robin selection
    #[allow(dead_code)]
    replica_index: std::sync::atomic::AtomicUsize,
    /// Default read preference
    #[allow(dead_code)]
    read_preference: ReadPreference,
    /// Whether replica mode is enabled
    #[allow(dead_code)]
    is_replica_mode: bool,
    /// Original config for creating child clients
    config: ClientConfig,
}

impl VectorizerClient {
    /// Get the base URL (for HTTP transport)
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Create a new client with configuration
    pub fn new(config: ClientConfig) -> Result<Self> {
        let timeout_secs = config.timeout_secs.unwrap_or(30);

        // Determine protocol and create transport
        let (transport, protocol, base_url): (Arc<dyn Transport>, Protocol, String) =
            if let Some(ref conn_str) = config.connection_string {
                // Use connection string
                #[allow(unused_variables)] // port is only used when umicp feature is enabled
                let (proto, host, port) = crate::transport::parse_connection_string(conn_str)?;

                match proto {
                    Protocol::Http => {
                        let transport =
                            HttpTransport::new(&host, config.api_key.as_deref(), timeout_secs)?;
                        (Arc::new(transport), Protocol::Http, host.clone())
                    }
                    #[cfg(feature = "umicp")]
                    Protocol::Umicp => {
                        let umicp_port = port.unwrap_or(15003);
                        let transport = UmicpTransport::new(
                            &host,
                            umicp_port,
                            config.api_key.as_deref(),
                            timeout_secs,
                        )?;
                        let base_url = format!("umicp://{host}:{umicp_port}");
                        (Arc::new(transport), Protocol::Umicp, base_url)
                    }
                }
            } else {
                // Use explicit configuration
                let proto = config.protocol.unwrap_or(Protocol::Http);

                match proto {
                    Protocol::Http => {
                        let base_url = config
                            .base_url
                            .clone()
                            .unwrap_or_else(|| "http://localhost:15002".to_string());
                        let transport =
                            HttpTransport::new(&base_url, config.api_key.as_deref(), timeout_secs)?;
                        (Arc::new(transport), Protocol::Http, base_url)
                    }
                    #[cfg(feature = "umicp")]
                    Protocol::Umicp => {
                        #[cfg(feature = "umicp")]
                        {
                            let umicp_config = config.umicp.clone().ok_or_else(|| {
                                VectorizerError::configuration(
                                    "UMICP configuration is required when using UMICP protocol",
                                )
                            })?;

                            let transport = UmicpTransport::new(
                                &umicp_config.host,
                                umicp_config.port,
                                config.api_key.as_deref(),
                                timeout_secs,
                            )?;
                            let base_url =
                                format!("umicp://{}:{}", umicp_config.host, umicp_config.port);
                            (Arc::new(transport), Protocol::Umicp, base_url)
                        }
                        #[cfg(not(feature = "umicp"))]
                        {
                            return Err(VectorizerError::configuration(
                                "UMICP feature is not enabled. Enable it with --features umicp",
                            ));
                        }
                    }
                }
            };

        // Initialize replica mode if hosts are configured
        let (master_transport, replica_transports, is_replica_mode) =
            if let Some(ref hosts) = config.hosts {
                let master =
                    HttpTransport::new(&hosts.master, config.api_key.as_deref(), timeout_secs)?;
                let replicas: Result<Vec<Arc<dyn Transport>>> = hosts
                    .replicas
                    .iter()
                    .map(|url| {
                        let t = HttpTransport::new(url, config.api_key.as_deref(), timeout_secs)?;
                        Ok(Arc::new(t) as Arc<dyn Transport>)
                    })
                    .collect();
                (
                    Some(Arc::new(master) as Arc<dyn Transport>),
                    replicas?,
                    true,
                )
            } else {
                (None, vec![], false)
            };

        let read_preference = config.read_preference.unwrap_or(ReadPreference::Replica);

        Ok(Self {
            transport,
            protocol,
            base_url,
            master_transport,
            replica_transports,
            replica_index: std::sync::atomic::AtomicUsize::new(0),
            read_preference,
            is_replica_mode,
            config,
        })
    }

    /// Create a new client with default configuration
    pub fn new_default() -> Result<Self> {
        Self::new(ClientConfig::default())
    }

    /// Create client with custom URL
    pub fn new_with_url(base_url: &str) -> Result<Self> {
        Self::new(ClientConfig {
            base_url: Some(base_url.to_string()),
            ..Default::default()
        })
    }

    /// Create client with API key
    pub fn new_with_api_key(base_url: &str, api_key: &str) -> Result<Self> {
        Self::new(ClientConfig {
            base_url: Some(base_url.to_string()),
            api_key: Some(api_key.to_string()),
            ..Default::default()
        })
    }

    /// Create client from connection string
    pub fn from_connection_string(connection_string: &str, api_key: Option<&str>) -> Result<Self> {
        Self::new(ClientConfig {
            connection_string: Some(connection_string.to_string()),
            api_key: api_key.map(|s| s.to_string()),
            ..Default::default()
        })
    }

    /// Get the current protocol being used
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// Get transport for write operations (always master)
    #[allow(dead_code)]
    fn get_write_transport(&self) -> &Arc<dyn Transport> {
        if self.is_replica_mode {
            self.master_transport.as_ref().unwrap_or(&self.transport)
        } else {
            &self.transport
        }
    }

    /// Get transport for read operations based on read preference
    #[allow(dead_code)]
    fn get_read_transport(&self, options: Option<&ReadOptions>) -> &Arc<dyn Transport> {
        if !self.is_replica_mode {
            return &self.transport;
        }

        let preference = options
            .and_then(|o| o.read_preference)
            .unwrap_or(self.read_preference);

        match preference {
            ReadPreference::Master => self.master_transport.as_ref().unwrap_or(&self.transport),
            ReadPreference::Replica | ReadPreference::Nearest => {
                if self.replica_transports.is_empty() {
                    return self.master_transport.as_ref().unwrap_or(&self.transport);
                }
                // Round-robin selection
                let idx = self
                    .replica_index
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    % self.replica_transports.len();
                &self.replica_transports[idx]
            }
        }
    }

    /// Execute a callback with master transport for read-your-writes scenarios.
    /// All operations within the callback will be routed to master.
    pub async fn with_master<F, Fut, T>(&self, callback: F) -> Result<T>
    where
        F: FnOnce(VectorizerClient) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut master_config = self.config.clone();
        master_config.read_preference = Some(ReadPreference::Master);
        let master_client = VectorizerClient::new(master_config)?;
        callback(master_client).await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.make_request("GET", "/health", None).await?;
        let health: HealthStatus = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse health check response: {e}"))
        })?;
        Ok(health)
    }

    /// List collections
    pub async fn list_collections(&self) -> Result<Vec<Collection>> {
        let response = self.make_request("GET", "/collections", None).await?;
        // Handle both array and {collections: [...]} response formats
        let collections: Vec<Collection> = if let Ok(wrapper) =
            serde_json::from_str::<serde_json::Value>(&response)
        {
            if let Some(arr) = wrapper.get("collections").and_then(|v| v.as_array()) {
                serde_json::from_value(serde_json::Value::Array(arr.clone())).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse collections array: {e}"))
                })?
            } else if wrapper.is_array() {
                serde_json::from_value(wrapper).map_err(|e| {
                    VectorizerError::server(format!("Failed to parse collections response: {e}"))
                })?
            } else {
                return Err(VectorizerError::server(
                    "Unexpected collections response format".to_string(),
                ));
            }
        } else {
            return Err(VectorizerError::server(
                "Failed to parse collections response".to_string(),
            ));
        };
        Ok(collections)
    }

    /// Search vectors
    pub async fn search_vectors(
        &self,
        collection: &str,
        query: &str,
        limit: Option<usize>,
        score_threshold: Option<f32>,
    ) -> Result<SearchResponse> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        payload.insert(
            "limit".to_string(),
            serde_json::Value::Number(limit.unwrap_or(10).into()),
        );

        if let Some(threshold) = score_threshold {
            payload.insert(
                "score_threshold".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(threshold as f64).unwrap()),
            );
        }

        let response = self
            .make_request(
                "POST",
                &format!("/collections/{}/search/text", collection),
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let search_response: SearchResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse search response: {e}"))
        })?;
        Ok(search_response)
    }

    // ===== INTELLIGENT SEARCH OPERATIONS =====

    /// Intelligent search with multi-query expansion and semantic reranking
    pub async fn intelligent_search(
        &self,
        request: IntelligentSearchRequest,
    ) -> Result<IntelligentSearchResponse> {
        let response = self
            .make_request(
                "POST",
                "/intelligent_search",
                Some(serde_json::to_value(request).unwrap()),
            )
            .await?;
        let search_response: IntelligentSearchResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!(
                    "Failed to parse intelligent search response: {}",
                    e
                ))
            })?;
        Ok(search_response)
    }

    /// Semantic search with advanced reranking and similarity thresholds
    pub async fn semantic_search(
        &self,
        request: SemanticSearchRequest,
    ) -> Result<SemanticSearchResponse> {
        let response = self
            .make_request(
                "POST",
                "/semantic_search",
                Some(serde_json::to_value(request).unwrap()),
            )
            .await?;
        let search_response: SemanticSearchResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!("Failed to parse semantic search response: {e}"))
            })?;
        Ok(search_response)
    }

    /// Context-aware search with metadata filtering and contextual reranking
    pub async fn contextual_search(
        &self,
        request: ContextualSearchRequest,
    ) -> Result<ContextualSearchResponse> {
        let response = self
            .make_request(
                "POST",
                "/contextual_search",
                Some(serde_json::to_value(request).unwrap()),
            )
            .await?;
        let search_response: ContextualSearchResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!(
                    "Failed to parse contextual search response: {}",
                    e
                ))
            })?;
        Ok(search_response)
    }

    /// Multi-collection search with cross-collection reranking and aggregation
    pub async fn multi_collection_search(
        &self,
        request: MultiCollectionSearchRequest,
    ) -> Result<MultiCollectionSearchResponse> {
        let response = self
            .make_request(
                "POST",
                "/multi_collection_search",
                Some(serde_json::to_value(request).unwrap()),
            )
            .await?;
        let search_response: MultiCollectionSearchResponse = serde_json::from_str(&response)
            .map_err(|e| {
                VectorizerError::server(format!(
                    "Failed to parse multi-collection search response: {}",
                    e
                ))
            })?;
        Ok(search_response)
    }

    /// Perform hybrid search combining dense and sparse vectors
    pub async fn hybrid_search(
        &self,
        request: HybridSearchRequest,
    ) -> Result<HybridSearchResponse> {
        let url = format!("/collections/{}/hybrid_search", request.collection);
        let payload = serde_json::json!({
            "query": request.query,
            "alpha": request.alpha,
            "algorithm": match request.algorithm {
                HybridScoringAlgorithm::ReciprocalRankFusion => "rrf",
                HybridScoringAlgorithm::WeightedCombination => "weighted",
                HybridScoringAlgorithm::AlphaBlending => "alpha",
            },
            "dense_k": request.dense_k,
            "sparse_k": request.sparse_k,
            "final_k": request.final_k,
            "query_sparse": request.query_sparse.as_ref().map(|sv| serde_json::json!({
                "indices": sv.indices,
                "values": sv.values,
            })),
        });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let search_response: HybridSearchResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!("Failed to parse hybrid search response: {e}"))
            })?;
        Ok(search_response)
    }

    // ===== QDRANT COMPATIBILITY METHODS =====

    /// List all collections (Qdrant-compatible API)
    pub async fn qdrant_list_collections(&self) -> Result<serde_json::Value> {
        let response = self
            .make_request("GET", "/qdrant/collections", None)
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant collections response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Get collection information (Qdrant-compatible API)
    pub async fn qdrant_get_collection(&self, name: &str) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{name}");
        let response = self.make_request("GET", &url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse Qdrant collection response: {e}"))
        })?;
        Ok(result)
    }

    /// Create collection (Qdrant-compatible API)
    pub async fn qdrant_create_collection(
        &self,
        name: &str,
        config: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{name}");
        let payload = serde_json::json!({ "config": config });
        let response = self.make_request("PUT", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant create collection response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Upsert points to collection (Qdrant-compatible API)
    pub async fn qdrant_upsert_points(
        &self,
        collection: &str,
        points: &serde_json::Value,
        wait: bool,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points");
        let payload = serde_json::json!({
            "points": points,
            "wait": wait,
        });
        let response = self.make_request("PUT", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant upsert points response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Search points in collection (Qdrant-compatible API)
    pub async fn qdrant_search_points(
        &self,
        collection: &str,
        vector: &[f32],
        limit: Option<usize>,
        filter: Option<&serde_json::Value>,
        with_payload: bool,
        with_vector: bool,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/search");
        let mut payload = serde_json::json!({
            "vector": vector,
            "limit": limit.unwrap_or(10),
            "with_payload": with_payload,
            "with_vector": with_vector,
        });
        if let Some(filter) = filter {
            payload["filter"] = filter.clone();
        }
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse Qdrant search response: {e}"))
        })?;
        Ok(result)
    }

    /// Delete points from collection (Qdrant-compatible API)
    pub async fn qdrant_delete_points(
        &self,
        collection: &str,
        point_ids: &[serde_json::Value],
        wait: bool,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/delete");
        let payload = serde_json::json!({
            "points": point_ids,
            "wait": wait,
        });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant delete points response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Retrieve points by IDs (Qdrant-compatible API)
    pub async fn qdrant_retrieve_points(
        &self,
        collection: &str,
        point_ids: &[serde_json::Value],
        with_payload: bool,
        with_vector: bool,
    ) -> Result<serde_json::Value> {
        let ids_str = point_ids
            .iter()
            .map(|id| match id {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => serde_json::to_string(id).unwrap_or_default(),
            })
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "/qdrant/collections/{}/points?ids={}&with_payload={}&with_vector={}",
            collection, ids_str, with_payload, with_vector
        );
        let response = self.make_request("GET", &url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant retrieve points response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Count points in collection (Qdrant-compatible API)
    pub async fn qdrant_count_points(
        &self,
        collection: &str,
        filter: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/count");
        let payload = if let Some(filter) = filter {
            serde_json::json!({ "filter": filter })
        } else {
            serde_json::json!({})
        };
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant count points response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    // ===== QDRANT ADVANCED FEATURES (1.14.x) =====

    /// List snapshots for a collection (Qdrant-compatible API)
    pub async fn qdrant_list_collection_snapshots(
        &self,
        collection: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/snapshots");
        let response = self.make_request("GET", &url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant list snapshots response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Create snapshot for a collection (Qdrant-compatible API)
    pub async fn qdrant_create_collection_snapshot(
        &self,
        collection: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/snapshots");
        let response = self.make_request("POST", &url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant create snapshot response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Delete snapshot (Qdrant-compatible API)
    pub async fn qdrant_delete_collection_snapshot(
        &self,
        collection: &str,
        snapshot_name: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/snapshots/{snapshot_name}");
        let response = self.make_request("DELETE", &url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant delete snapshot response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Recover collection from snapshot (Qdrant-compatible API)
    pub async fn qdrant_recover_collection_snapshot(
        &self,
        collection: &str,
        location: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/snapshots/recover");
        let payload = serde_json::json!({ "location": location });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant recover snapshot response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// List all snapshots (Qdrant-compatible API)
    pub async fn qdrant_list_all_snapshots(&self) -> Result<serde_json::Value> {
        let url = "/qdrant/snapshots";
        let response = self.make_request("GET", url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant list all snapshots response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Create full snapshot (Qdrant-compatible API)
    pub async fn qdrant_create_full_snapshot(&self) -> Result<serde_json::Value> {
        let url = "/qdrant/snapshots";
        let response = self.make_request("POST", url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant create full snapshot response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// List shard keys for a collection (Qdrant-compatible API)
    pub async fn qdrant_list_shard_keys(&self, collection: &str) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/shards");
        let response = self.make_request("GET", &url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant list shard keys response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Create shard key (Qdrant-compatible API)
    pub async fn qdrant_create_shard_key(
        &self,
        collection: &str,
        shard_key: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/shards");
        let payload = serde_json::json!({ "shard_key": shard_key });
        let response = self.make_request("PUT", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant create shard key response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Delete shard key (Qdrant-compatible API)
    pub async fn qdrant_delete_shard_key(
        &self,
        collection: &str,
        shard_key: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/shards/delete");
        let payload = serde_json::json!({ "shard_key": shard_key });
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant delete shard key response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Get cluster status (Qdrant-compatible API)
    pub async fn qdrant_get_cluster_status(&self) -> Result<serde_json::Value> {
        let url = "/qdrant/cluster";
        let response = self.make_request("GET", url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant cluster status response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Recover current peer (Qdrant-compatible API)
    pub async fn qdrant_cluster_recover(&self) -> Result<serde_json::Value> {
        let url = "/qdrant/cluster/recover";
        let response = self.make_request("POST", url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant cluster recover response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Remove peer from cluster (Qdrant-compatible API)
    pub async fn qdrant_remove_peer(&self, peer_id: &str) -> Result<serde_json::Value> {
        let url = format!("/qdrant/cluster/peer/{peer_id}");
        let response = self.make_request("DELETE", &url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant remove peer response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// List metadata keys (Qdrant-compatible API)
    pub async fn qdrant_list_metadata_keys(&self) -> Result<serde_json::Value> {
        let url = "/qdrant/cluster/metadata/keys";
        let response = self.make_request("GET", url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant list metadata keys response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Get metadata key (Qdrant-compatible API)
    pub async fn qdrant_get_metadata_key(&self, key: &str) -> Result<serde_json::Value> {
        let url = format!("/qdrant/cluster/metadata/keys/{key}");
        let response = self.make_request("GET", &url, None).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant get metadata key response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Update metadata key (Qdrant-compatible API)
    pub async fn qdrant_update_metadata_key(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/cluster/metadata/keys/{key}");
        let payload = serde_json::json!({ "value": value });
        let response = self.make_request("PUT", &url, Some(payload)).await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant update metadata key response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Query points (Qdrant 1.7+ Query API)
    pub async fn qdrant_query_points(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/query");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant query points response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Batch query points (Qdrant 1.7+ Query API)
    pub async fn qdrant_batch_query_points(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/query/batch");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant batch query points response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Query points with groups (Qdrant 1.7+ Query API)
    pub async fn qdrant_query_points_groups(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/query/groups");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant query points groups response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Search points with groups (Qdrant Search Groups API)
    pub async fn qdrant_search_points_groups(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/search/groups");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant search points groups response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Search matrix pairs (Qdrant Search Matrix API)
    pub async fn qdrant_search_matrix_pairs(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/search/matrix/pairs");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant search matrix pairs response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Search matrix offsets (Qdrant Search Matrix API)
    pub async fn qdrant_search_matrix_offsets(
        &self,
        collection: &str,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("/qdrant/collections/{collection}/points/search/matrix/offsets");
        let response = self
            .make_request("POST", &url, Some(request.clone()))
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!(
                "Failed to parse Qdrant search matrix offsets response: {}",
                e
            ))
        })?;
        Ok(result)
    }

    /// Create collection
    pub async fn create_collection(
        &self,
        name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<CollectionInfo> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "name".to_string(),
            serde_json::Value::String(name.to_string()),
        );
        payload.insert(
            "dimension".to_string(),
            serde_json::Value::Number(dimension.into()),
        );
        payload.insert(
            "metric".to_string(),
            serde_json::Value::String(format!("{:?}", metric.unwrap_or_default()).to_lowercase()),
        );

        let response = self
            .make_request(
                "POST",
                "/collections",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let create_response: CreateCollectionResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!(
                    "Failed to parse create collection response: {}",
                    e
                ))
            })?;

        // Create a basic CollectionInfo from the response
        let info = CollectionInfo {
            name: create_response.collection,
            dimension,
            metric: format!("{:?}", metric.unwrap_or_default()).to_lowercase(),
            vector_count: 0,
            document_count: 0,
            created_at: "".to_string(),
            updated_at: "".to_string(),
            indexing_status: Some(crate::models::IndexingStatus {
                status: "created".to_string(),
                progress: 0.0,
                total_documents: 0,
                processed_documents: 0,
                vector_count: 0,
                estimated_time_remaining: None,
                last_updated: "".to_string(),
            }),
        };
        Ok(info)
    }

    /// Insert texts
    pub async fn insert_texts(
        &self,
        collection: &str,
        texts: Vec<BatchTextRequest>,
    ) -> Result<BatchResponse> {
        let payload = serde_json::json!({
            "texts": texts
        });

        let response = self
            .make_request(
                "POST",
                &format!("/collections/{collection}/documents"),
                Some(serde_json::to_value(payload)?),
            )
            .await?;
        let batch_response: BatchResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse insert texts response: {e}"))
        })?;
        Ok(batch_response)
    }

    /// Delete collection
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        self.make_request("DELETE", &format!("/collections/{}", name), None)
            .await?;
        Ok(())
    }

    /// Get vector
    pub async fn get_vector(&self, collection: &str, vector_id: &str) -> Result<Vector> {
        let response = self
            .make_request(
                "GET",
                &format!("/collections/{collection}/vectors/{vector_id}"),
                None,
            )
            .await?;
        let vector: Vector = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse get vector response: {e}"))
        })?;
        Ok(vector)
    }

    /// Get collection info
    pub async fn get_collection_info(&self, collection: &str) -> Result<CollectionInfo> {
        let response = self
            .make_request("GET", &format!("/collections/{}", collection), None)
            .await?;
        let info: CollectionInfo = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse collection info: {e}"))
        })?;
        Ok(info)
    }

    /// Generate embeddings
    pub async fn embed_text(&self, text: &str, model: Option<&str>) -> Result<EmbeddingResponse> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "text".to_string(),
            serde_json::Value::String(text.to_string()),
        );

        if let Some(model) = model {
            payload.insert(
                "model".to_string(),
                serde_json::Value::String(model.to_string()),
            );
        }

        let response = self
            .make_request("POST", "/embed", Some(serde_json::Value::Object(payload)))
            .await?;
        let embedding_response: EmbeddingResponse =
            serde_json::from_str(&response).map_err(|e| {
                VectorizerError::server(format!("Failed to parse embedding response: {e}"))
            })?;
        Ok(embedding_response)
    }

    // =============================================================================
    // DISCOVERY OPERATIONS
    // =============================================================================

    /// Complete discovery pipeline with intelligent search and prompt generation
    pub async fn discover(
        &self,
        query: &str,
        include_collections: Option<Vec<String>>,
        exclude_collections: Option<Vec<String>>,
        max_bullets: Option<usize>,
        broad_k: Option<usize>,
        focus_k: Option<usize>,
    ) -> Result<serde_json::Value> {
        // Validate query
        if query.trim().is_empty() {
            return Err(VectorizerError::validation("Query cannot be empty"));
        }

        // Validate max_bullets
        if let Some(max) = max_bullets
            && max == 0
        {
            return Err(VectorizerError::validation(
                "max_bullets must be greater than 0",
            ));
        }

        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );

        if let Some(inc) = include_collections {
            payload.insert(
                "include_collections".to_string(),
                serde_json::to_value(inc).unwrap(),
            );
        }
        if let Some(exc) = exclude_collections {
            payload.insert(
                "exclude_collections".to_string(),
                serde_json::to_value(exc).unwrap(),
            );
        }
        if let Some(max) = max_bullets {
            payload.insert(
                "max_bullets".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }
        if let Some(k) = broad_k {
            payload.insert("broad_k".to_string(), serde_json::Value::Number(k.into()));
        }
        if let Some(k) = focus_k {
            payload.insert("focus_k".to_string(), serde_json::Value::Number(k.into()));
        }

        let response = self
            .make_request(
                "POST",
                "/discover",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse discover response: {e}"))
        })?;
        Ok(result)
    }

    /// Pre-filter collections by name patterns
    pub async fn filter_collections(
        &self,
        query: &str,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    ) -> Result<serde_json::Value> {
        // Validate query
        if query.trim().is_empty() {
            return Err(VectorizerError::validation("Query cannot be empty"));
        }

        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );

        if let Some(inc) = include {
            payload.insert("include".to_string(), serde_json::to_value(inc).unwrap());
        }
        if let Some(exc) = exclude {
            payload.insert("exclude".to_string(), serde_json::to_value(exc).unwrap());
        }

        let response = self
            .make_request(
                "POST",
                "/discovery/filter_collections",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse filter response: {e}"))
        })?;
        Ok(result)
    }

    /// Rank collections by relevance
    pub async fn score_collections(
        &self,
        query: &str,
        name_match_weight: Option<f32>,
        term_boost_weight: Option<f32>,
        signal_boost_weight: Option<f32>,
    ) -> Result<serde_json::Value> {
        // Validate weights (must be between 0.0 and 1.0)
        if let Some(w) = name_match_weight
            && !(0.0..=1.0).contains(&w)
        {
            return Err(VectorizerError::validation(
                "name_match_weight must be between 0.0 and 1.0",
            ));
        }
        if let Some(w) = term_boost_weight
            && !(0.0..=1.0).contains(&w)
        {
            return Err(VectorizerError::validation(
                "term_boost_weight must be between 0.0 and 1.0",
            ));
        }
        if let Some(w) = signal_boost_weight
            && !(0.0..=1.0).contains(&w)
        {
            return Err(VectorizerError::validation(
                "signal_boost_weight must be between 0.0 and 1.0",
            ));
        }

        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );

        if let Some(w) = name_match_weight {
            payload.insert("name_match_weight".to_string(), serde_json::json!(w));
        }
        if let Some(w) = term_boost_weight {
            payload.insert("term_boost_weight".to_string(), serde_json::json!(w));
        }
        if let Some(w) = signal_boost_weight {
            payload.insert("signal_boost_weight".to_string(), serde_json::json!(w));
        }

        let response = self
            .make_request(
                "POST",
                "/discovery/score_collections",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse score response: {e}")))?;
        Ok(result)
    }

    /// Generate query variations
    pub async fn expand_queries(
        &self,
        query: &str,
        max_expansions: Option<usize>,
        include_definition: Option<bool>,
        include_features: Option<bool>,
        include_architecture: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );

        if let Some(max) = max_expansions {
            payload.insert(
                "max_expansions".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }
        if let Some(def) = include_definition {
            payload.insert(
                "include_definition".to_string(),
                serde_json::Value::Bool(def),
            );
        }
        if let Some(feat) = include_features {
            payload.insert(
                "include_features".to_string(),
                serde_json::Value::Bool(feat),
            );
        }
        if let Some(arch) = include_architecture {
            payload.insert(
                "include_architecture".to_string(),
                serde_json::Value::Bool(arch),
            );
        }

        let response = self
            .make_request(
                "POST",
                "/discovery/expand_queries",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse expand response: {e}"))
        })?;
        Ok(result)
    }

    // =============================================================================
    // FILE OPERATIONS
    // =============================================================================

    /// Retrieve complete file content from a collection
    pub async fn get_file_content(
        &self,
        collection: &str,
        file_path: &str,
        max_size_kb: Option<usize>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "file_path".to_string(),
            serde_json::Value::String(file_path.to_string()),
        );

        if let Some(max) = max_size_kb {
            payload.insert(
                "max_size_kb".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }

        let response = self
            .make_request(
                "POST",
                "/file/content",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse file content response: {e}"))
        })?;
        Ok(result)
    }

    /// List all indexed files in a collection
    pub async fn list_files_in_collection(
        &self,
        collection: &str,
        filter_by_type: Option<Vec<String>>,
        min_chunks: Option<usize>,
        max_results: Option<usize>,
        sort_by: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );

        if let Some(types) = filter_by_type {
            payload.insert(
                "filter_by_type".to_string(),
                serde_json::to_value(types).unwrap(),
            );
        }
        if let Some(min) = min_chunks {
            payload.insert(
                "min_chunks".to_string(),
                serde_json::Value::Number(min.into()),
            );
        }
        if let Some(max) = max_results {
            payload.insert(
                "max_results".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }
        if let Some(sort) = sort_by {
            payload.insert(
                "sort_by".to_string(),
                serde_json::Value::String(sort.to_string()),
            );
        }

        let response = self
            .make_request(
                "POST",
                "/file/list",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list files response: {e}"))
        })?;
        Ok(result)
    }

    /// Get extractive or structural summary of an indexed file
    pub async fn get_file_summary(
        &self,
        collection: &str,
        file_path: &str,
        summary_type: Option<&str>,
        max_sentences: Option<usize>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "file_path".to_string(),
            serde_json::Value::String(file_path.to_string()),
        );

        if let Some(stype) = summary_type {
            payload.insert(
                "summary_type".to_string(),
                serde_json::Value::String(stype.to_string()),
            );
        }
        if let Some(max) = max_sentences {
            payload.insert(
                "max_sentences".to_string(),
                serde_json::Value::Number(max.into()),
            );
        }

        let response = self
            .make_request(
                "POST",
                "/file/summary",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse file summary response: {e}"))
        })?;
        Ok(result)
    }

    /// Retrieve chunks in original file order for progressive reading
    pub async fn get_file_chunks_ordered(
        &self,
        collection: &str,
        file_path: &str,
        start_chunk: Option<usize>,
        limit: Option<usize>,
        include_context: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "file_path".to_string(),
            serde_json::Value::String(file_path.to_string()),
        );

        if let Some(start) = start_chunk {
            payload.insert(
                "start_chunk".to_string(),
                serde_json::Value::Number(start.into()),
            );
        }
        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(ctx) = include_context {
            payload.insert("include_context".to_string(), serde_json::Value::Bool(ctx));
        }

        let response = self
            .make_request(
                "POST",
                "/file/chunks",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse chunks response: {e}"))
        })?;
        Ok(result)
    }

    /// Generate hierarchical project structure overview
    pub async fn get_project_outline(
        &self,
        collection: &str,
        max_depth: Option<usize>,
        include_summaries: Option<bool>,
        highlight_key_files: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );

        if let Some(depth) = max_depth {
            payload.insert(
                "max_depth".to_string(),
                serde_json::Value::Number(depth.into()),
            );
        }
        if let Some(summ) = include_summaries {
            payload.insert(
                "include_summaries".to_string(),
                serde_json::Value::Bool(summ),
            );
        }
        if let Some(highlight) = highlight_key_files {
            payload.insert(
                "highlight_key_files".to_string(),
                serde_json::Value::Bool(highlight),
            );
        }

        let response = self
            .make_request(
                "POST",
                "/file/outline",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse outline response: {e}"))
        })?;
        Ok(result)
    }

    /// Find semantically related files using vector similarity
    pub async fn get_related_files(
        &self,
        collection: &str,
        file_path: &str,
        limit: Option<usize>,
        similarity_threshold: Option<f32>,
        include_reason: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "file_path".to_string(),
            serde_json::Value::String(file_path.to_string()),
        );

        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(thresh) = similarity_threshold {
            payload.insert(
                "similarity_threshold".to_string(),
                serde_json::json!(thresh),
            );
        }
        if let Some(reason) = include_reason {
            payload.insert(
                "include_reason".to_string(),
                serde_json::Value::Bool(reason),
            );
        }

        let response = self
            .make_request(
                "POST",
                "/file/related",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse related files response: {e}"))
        })?;
        Ok(result)
    }

    /// Semantic search filtered by file type
    pub async fn search_by_file_type(
        &self,
        collection: &str,
        query: &str,
        file_types: Vec<String>,
        limit: Option<usize>,
        return_full_files: Option<bool>,
    ) -> Result<serde_json::Value> {
        // Validate file_types is not empty
        if file_types.is_empty() {
            return Err(VectorizerError::validation("file_types cannot be empty"));
        }

        let mut payload = serde_json::Map::new();
        payload.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
        payload.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        payload.insert(
            "file_types".to_string(),
            serde_json::to_value(file_types).unwrap(),
        );

        if let Some(lim) = limit {
            payload.insert("limit".to_string(), serde_json::Value::Number(lim.into()));
        }
        if let Some(full) = return_full_files {
            payload.insert(
                "return_full_files".to_string(),
                serde_json::Value::Bool(full),
            );
        }

        let response = self
            .make_request(
                "POST",
                "/file/search_by_type",
                Some(serde_json::Value::Object(payload)),
            )
            .await?;
        let result: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse search by type response: {e}"))
        })?;
        Ok(result)
    }

    /// Make HTTP request
    async fn make_request(
        &self,
        method: &str,
        endpoint: &str,
        payload: Option<serde_json::Value>,
    ) -> Result<String> {
        match method {
            "GET" => self.transport.get(endpoint).await,
            "POST" => self.transport.post(endpoint, payload.as_ref()).await,
            "PUT" => self.transport.put(endpoint, payload.as_ref()).await,
            "DELETE" => self.transport.delete(endpoint).await,
            _ => Err(VectorizerError::configuration(format!(
                "Unsupported method: {}",
                method
            ))),
        }
    }

    // ========== Graph Operations ==========

    /// List all nodes in a collection's graph
    pub async fn list_graph_nodes(&self, collection: &str) -> Result<ListNodesResponse> {
        let url = format!("/graph/nodes/{}", collection);
        let response = self.make_request("GET", &url, None).await?;
        let result: ListNodesResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list nodes response: {e}"))
        })?;
        Ok(result)
    }

    /// Get neighbors of a specific node
    pub async fn get_graph_neighbors(
        &self,
        collection: &str,
        node_id: &str,
    ) -> Result<GetNeighborsResponse> {
        let url = format!("/graph/nodes/{}/{}/neighbors", collection, node_id);
        let response = self.make_request("GET", &url, None).await?;
        let result: GetNeighborsResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse neighbors response: {e}"))
        })?;
        Ok(result)
    }

    /// Find related nodes within N hops
    pub async fn find_related_nodes(
        &self,
        collection: &str,
        node_id: &str,
        request: FindRelatedRequest,
    ) -> Result<FindRelatedResponse> {
        let url = format!("/graph/nodes/{}/{}/related", collection, node_id);
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let result: FindRelatedResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse related nodes response: {e}"))
        })?;
        Ok(result)
    }

    /// Find shortest path between two nodes
    pub async fn find_graph_path(&self, request: FindPathRequest) -> Result<FindPathResponse> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self
            .make_request("POST", "/graph/path", Some(payload))
            .await?;
        let result: FindPathResponse = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse path response: {e}")))?;
        Ok(result)
    }

    /// Create an explicit edge between two nodes
    pub async fn create_graph_edge(
        &self,
        request: CreateEdgeRequest,
    ) -> Result<CreateEdgeResponse> {
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self
            .make_request("POST", "/graph/edges", Some(payload))
            .await?;
        let result: CreateEdgeResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse create edge response: {e}"))
        })?;
        Ok(result)
    }

    /// Delete an edge by ID
    pub async fn delete_graph_edge(&self, edge_id: &str) -> Result<()> {
        let url = format!("/graph/edges/{}", edge_id);
        self.make_request("DELETE", &url, None).await?;
        Ok(())
    }

    /// List all edges in a collection
    pub async fn list_graph_edges(&self, collection: &str) -> Result<ListEdgesResponse> {
        let url = format!("/graph/collections/{}/edges", collection);
        let response = self.make_request("GET", &url, None).await?;
        let result: ListEdgesResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse list edges response: {}", e))
        })?;
        Ok(result)
    }

    /// Discover SIMILAR_TO edges for entire collection
    pub async fn discover_graph_edges(
        &self,
        collection: &str,
        request: DiscoverEdgesRequest,
    ) -> Result<DiscoverEdgesResponse> {
        let url = format!("/graph/discover/{}", collection);
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let result: DiscoverEdgesResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse discover edges response: {}", e))
        })?;
        Ok(result)
    }

    /// Discover SIMILAR_TO edges for a specific node
    pub async fn discover_graph_edges_for_node(
        &self,
        collection: &str,
        node_id: &str,
        request: DiscoverEdgesRequest,
    ) -> Result<DiscoverEdgesResponse> {
        let url = format!("/graph/discover/{collection}/{node_id}");
        let payload = serde_json::to_value(&request).map_err(|e| {
            VectorizerError::validation(format!("Failed to serialize request: {e}"))
        })?;
        let response = self.make_request("POST", &url, Some(payload)).await?;
        let result: DiscoverEdgesResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse discover edges response: {}", e))
        })?;
        Ok(result)
    }

    /// Get discovery status for a collection
    pub async fn get_graph_discovery_status(
        &self,
        collection: &str,
    ) -> Result<DiscoveryStatusResponse> {
        let url = format!("/graph/discover/{}/status", collection);
        let response = self.make_request("GET", &url, None).await?;
        let result: DiscoveryStatusResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse discovery status response: {}", e))
        })?;
        Ok(result)
    }

    // ============== FILE UPLOAD METHODS ==============

    /// Upload a file for automatic text extraction, chunking, and indexing.
    ///
    /// # Arguments
    /// * `file_bytes` - File content as bytes
    /// * `filename` - Name of the file (used for extension detection)
    /// * `collection_name` - Target collection name
    /// * `options` - Upload options (chunk size, overlap, metadata)
    ///
    /// # Example
    /// ```no_run
    /// use vectorizer_sdk::{VectorizerClient, ClientConfig, UploadFileOptions};
    /// use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = ClientConfig::default();
    ///     let client = VectorizerClient::new(config)?;
    ///
    ///     let file_bytes = std::fs::read("document.pdf")?;
    ///     let options = UploadFileOptions::default();
    ///
    ///     let response = client.upload_file(
    ///         file_bytes,
    ///         "document.pdf",
    ///         "my-docs",
    ///         options
    ///     ).await?;
    ///
    ///     println!("Uploaded: {} chunks created", response.chunks_created);
    ///     Ok(())
    /// }
    /// ```
    pub async fn upload_file(
        &self,
        file_bytes: Vec<u8>,
        filename: &str,
        collection_name: &str,
        options: UploadFileOptions,
    ) -> Result<FileUploadResponse> {
        let mut form_fields = std::collections::HashMap::new();
        form_fields.insert("collection_name".to_string(), collection_name.to_string());

        if let Some(chunk_size) = options.chunk_size {
            form_fields.insert("chunk_size".to_string(), chunk_size.to_string());
        }

        if let Some(chunk_overlap) = options.chunk_overlap {
            form_fields.insert("chunk_overlap".to_string(), chunk_overlap.to_string());
        }

        if let Some(metadata) = options.metadata {
            let metadata_json = serde_json::to_string(&metadata).map_err(|e| {
                VectorizerError::validation(format!("Failed to serialize metadata: {e}"))
            })?;
            form_fields.insert("metadata".to_string(), metadata_json);
        }

        // Use HttpTransport's multipart method
        let http_transport = crate::http_transport::HttpTransport::new(
            &self.base_url,
            self.config.api_key.as_deref(),
            self.config.timeout_secs.unwrap_or(30),
        )?;

        let response = http_transport
            .post_multipart("/files/upload", file_bytes, filename, form_fields)
            .await?;

        let result: FileUploadResponse = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse upload response: {e}"))
        })?;

        Ok(result)
    }

    /// Upload file content directly as a string.
    ///
    /// This is a convenience method that accepts text content directly instead of file bytes.
    ///
    /// # Arguments
    /// * `content` - File content as string
    /// * `filename` - Name of the file (used for extension detection)
    /// * `collection_name` - Target collection name
    /// * `options` - Upload options (chunk size, overlap, metadata)
    ///
    /// # Example
    /// ```no_run
    /// use vectorizer_sdk::{VectorizerClient, ClientConfig, UploadFileOptions};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = ClientConfig::default();
    ///     let client = VectorizerClient::new(config)?;
    ///
    ///     let code = r#"fn main() { println!("Hello!"); }"#;
    ///     let options = UploadFileOptions::default();
    ///
    ///     let response = client.upload_file_content(
    ///         code,
    ///         "main.rs",
    ///         "rust-code",
    ///         options
    ///     ).await?;
    ///
    ///     println!("Uploaded: {} vectors created", response.vectors_created);
    ///     Ok(())
    /// }
    /// ```
    pub async fn upload_file_content(
        &self,
        content: &str,
        filename: &str,
        collection_name: &str,
        options: UploadFileOptions,
    ) -> Result<FileUploadResponse> {
        let file_bytes = content.as_bytes().to_vec();
        self.upload_file(file_bytes, filename, collection_name, options)
            .await
    }

    /// Get file upload configuration from the server.
    ///
    /// Returns the maximum file size, allowed extensions, and default chunk settings.
    ///
    /// # Example
    /// ```no_run
    /// use vectorizer_sdk::{VectorizerClient, ClientConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = ClientConfig::default();
    ///     let client = VectorizerClient::new(config)?;
    ///
    ///     let upload_config = client.get_upload_config().await?;
    ///     println!("Max file size: {}MB", upload_config.max_file_size_mb);
    ///     println!("Allowed extensions: {:?}", upload_config.allowed_extensions);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_upload_config(&self) -> Result<FileUploadConfig> {
        let response = self.make_request("GET", "/files/config", None).await?;
        let result: FileUploadConfig = serde_json::from_str(&response)
            .map_err(|e| VectorizerError::server(format!("Failed to parse upload config: {e}")))?;
        Ok(result)
    }
}
